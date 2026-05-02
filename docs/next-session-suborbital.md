# Handoff: реалистичное поведение при потере тяги / нехватке скорости

> **Базовый коммит:** `f3793ac` (HEAD на момент написания).
> **Контекст разговора:** обсуждали что нужно, чтобы при недоборе скорости/высоты или раннем MECO ракета **не прыгала сразу в сцену Orbit**, а либо упала по баллистике, либо вышла на нестабильную орбиту с последствиями для миссии.
> **Файл живёт в репозитории чтобы новая сессия Claude нашла его через Glob/Grep.**

---

## TL;DR — что делаем

Сейчас и `check_meco` (в `src/physics/rocket.rs`), и `handle_engine_cutoff` (в `src/input.rs`) вызывают `next_stage.set(MissionStage::Orbit)` без каких‑либо проверок. Меняем это на **арбитра** — система при гашении двигателей проверяет высоту и горизонтальную скорость и либо переходит в Orbit, либо оставляет ракету в Launch, где уже работающая физика (gravity → vertical_speed_ms → altitude_m) будет тянуть её вниз. На альт ≤ 1 км и vert_speed < −50 м/с — `MissionEvent::Abort("suborbital impact")`.

---

## Что есть в коде сейчас (важные точки)

| Файл | Строка | Что |
|---|---|---|
| `src/physics/rocket.rs` | 146 `fn tick_rocket_physics` | Считает гравитацию через `orbital::gravity_at_altitude`, интегрирует Эйлером altitude/speed. **Уже клампит altitude_m в 0** — это нужно убрать или дополнить event'ом. |
| `src/physics/rocket.rs` | 225 `fn check_meco` | По таймеру `t >= meco_target_t_plus_s` ставит `phase = Coast`, пишет `MissionEvent::Meco`, **сразу** `next_stage.set(Orbit)`. |
| `src/input.rs` | 87 `fn handle_engine_cutoff` | На `KeyX` тоже сразу `next_stage.set(Orbit)`. |
| `src/physics/orbital.rs` | 7 `gravity_at_altitude(altitude_m)` | Уже есть. Сюда же добавим `air_density(h_m)`. |
| `src/config.rs` | — | Здесь живут константы (`G0`, `WORLD_SCALE` и т.д.) — добавим `ORBITAL_INSERTION_*`. |
| `src/ui/hud.rs` | — | Сюда добавим строку «SUBORBITAL — ALT FALLING». |

`SlsParams` и `IcpsParams` отдельно — не трогаем.

---

## План реализации (по шагам, в порядке коммитов)

### Шаг 1 — критерий выхода на орбиту в `config.rs`

```rust
/// Минимальная высота для устойчивой LEO‑орбиты, м.
pub const ORBITAL_INSERTION_ALT_M:   f32 = 150_000.0;
/// Минимальная горизонтальная скорость для LEO, м/с (~v_circ для 200 км).
pub const ORBITAL_INSERTION_VEL_MS:  f32 = 7_600.0;
/// Высота «нестабильной орбиты» — между этой и стабильной игрок попадает в Orbit
/// с пометкой unstable (TLI стартует с штрафом точности).
pub const UNSTABLE_ORBIT_ALT_M:      f32 = 80_000.0;
pub const UNSTABLE_ORBIT_VEL_MS:     f32 = 6_500.0;
```

Опционально расширить `TliResult` или ввести новый ресурс:
```rust
#[derive(Resource, Default, Debug)]
pub struct OrbitInsertion {
    pub achieved: bool,
    pub unstable: bool,
    pub altitude_m: f32,
    pub horizontal_speed_ms: f32,
}
```

### Шаг 2 — арбитр MECO

Убрать `next_stage.set(Orbit)` из `check_meco` и `handle_engine_cutoff`. Эти системы только переводят в `Coast` (как сейчас) и эмитят `MissionEvent::Meco`. Новая система:

```rust
fn evaluate_meco_outcome(
    rockets: Query<&FlightDynamics, With<Rocket>>,
    mut meco_events: MessageReader<MissionEvent>,
    mut insertion: ResMut<OrbitInsertion>,
    mut next_stage: ResMut<NextState<MissionStage>>,
) {
    let mut meco_seen = false;
    for ev in meco_events.read() {
        if matches!(ev, MissionEvent::Meco) { meco_seen = true; }
    }
    if !meco_seen { return; }

    let Ok(dyn_) = rockets.single() else { return };
    insertion.altitude_m = dyn_.altitude_m;
    insertion.horizontal_speed_ms = dyn_.horizontal_speed_ms;

    if dyn_.altitude_m >= ORBITAL_INSERTION_ALT_M
        && dyn_.horizontal_speed_ms >= ORBITAL_INSERTION_VEL_MS
    {
        insertion.achieved = true;
        insertion.unstable = false;
        next_stage.set(MissionStage::Orbit);
    } else if dyn_.altitude_m >= UNSTABLE_ORBIT_ALT_M
        && dyn_.horizontal_speed_ms >= UNSTABLE_ORBIT_VEL_MS
    {
        insertion.achieved = true;
        insertion.unstable = true;
        next_stage.set(MissionStage::Orbit);
    }
    // Иначе ничего не делаем — рокета остаётся в Launch, физика её доуронит.
}
```

Добавить в `physics::plugin` или новый `stages/launch.rs`‑plugin. Запускать каждый кадр в Launch.

### Шаг 3 — детектор удара

Убрать клампинг `altitude_m.max(0.0)` из `tick_rocket_physics`, либо оставить и добавить отдельную систему:

```rust
fn check_impact(
    rockets: Query<(&Rocket, &FlightDynamics)>,
    mut events: MessageWriter<MissionEvent>,
) {
    let Ok((rocket, dyn_)) = rockets.single() else { return };
    if rocket.phase == FlightPhase::Coast
        && dyn_.altitude_m < 1.0
        && dyn_.vertical_speed_ms < -50.0
    {
        events.write(MissionEvent::Abort("suborbital impact".into()));
    }
}
```

`run_if(in_state(MissionStage::Launch))`. Чтобы не спамить (как было с pitch abort) — добавить `bool` поле в Rocket или ресурс‑флаг, либо проверять что `MissionFailed::reason.is_none()`.

### Шаг 4 — атмосферное сопротивление (улучшение, можно отложить)

В `src/physics/orbital.rs`:

```rust
/// Плотность воздуха по простой экспоненциальной модели, кг/м³.
pub fn air_density(h_m: f32) -> f32 {
    let h0 = 8_000.0;
    if h_m < 0.0 { 1.225 } else { 1.225 * (-h_m / h0).exp() }
}
```

В `tick_rocket_physics` после расчёта `a_thrust` добавить:

```rust
let rho   = orbital::air_density(dynamics.altitude_m);
let speed = dynamics.speed_ms.max(0.1);
let cd_a  = 0.5 * 55.0;             // Cd≈0.5, A≈55 м² (миделевое сечение SLS)
let drag  = 0.5 * rho * speed * speed * cd_a;
let a_drag = drag / rocket.mass_kg.max(1.0);
let frac_v = dynamics.vertical_speed_ms / speed;
let frac_h = dynamics.horizontal_speed_ms / speed;
dynamics.vertical_speed_ms   -= a_drag * frac_v * dt;
dynamics.horizontal_speed_ms -= a_drag * frac_h * dt;
```

Без drag ракета при падении из 80 км разгонится до >2 км/с — некрасиво. С drag — терминальная скорость пары сот м/с.

### Шаг 5 — HUD‑строка

В `src/ui/hud.rs`, в существующем блоке отрисовки HUD (поверх Launch‑этапа), добавить условный лейбл:

```rust
if rocket.phase == FlightPhase::Coast
   && dynamics.vertical_speed_ms < 0.0
   && dynamics.altitude_m > 1000.0 {
    ui.colored_label(theme::SLS_ORANGE, t.get(*lang, "alert.suborbital"));
}
```

Если ещё лучше — отдельный цвет/рамка.

### Шаг 6 — i18n

`assets/i18n/{ru,en}.ron`:
```
"alert.suborbital":          "СУБОРБИТАЛЬНАЯ ТРАЕКТОРИЯ — ВЫСОТА ПАДАЕТ"
"alert.suborbital_impact":   "Удар о поверхность — миссия провалена"
"alert.unstable_orbit":      "Нестабильная орбита — TLI с потерями"
```
И английские эквиваленты.

### Шаг 7 — следствия для TLI (если делаем unstable orbit)

В `src/stages/tli.rs` или `physics/orbital.rs`, в расчёт точности TLI / траекторной ошибки заложить штраф если `OrbitInsertion::unstable`. Простой вариант: множитель 0.7 на эффективное Δv от ICPS, либо +20% trajectory_error. Это уже отдельная feature, можно делать отдельным коммитом.

---

## Открытые вопросы для пользователя (спросить в начале сессии)

1. **Минимум или полный набор?** Минимальный «вкусный» вариант — шаги 1‑3+5+6 (без drag и без unstable), ~30 минут. Полный — всё вместе + следствия для TLI, ~1.5 часа.
2. **Что с unstable‑орбитой делать в Orbit‑сцене?** Просто пометить и забыть до TLI, или показывать в HUD «orbit decaying — N min until reentry» с реальным таймером?
3. **Drag как Cd·A?** Cd=0.5 и A=55 м² — округления. Если хотим точнее — взять Cd≈1.0 для тупого носового конуса и A=π·(8.4/2)²≈55 м² (8.4 м — диаметр SLS Core Stage). Достаточно для геймплея.

---

## Верификация после реализации

- `cargo clippy` — 0 warnings.
- `cargo test` — 9/9 (уже сейчас).
- Сценарий 1: нормальный полёт → автоматический MECO на T+~510 c при altitude>150 км → переход в Orbit. Должен сохраняться.
- Сценарий 2: ручной `X` сразу после старта на altitude≈0 → ракета остаётся в Launch, gravity тянет вниз, через несколько секунд импакт → `MissionEvent::Abort` → gameover.
- Сценарий 3: `X` на altitude=100 км и vel=7000 м/с → unstable orbit → переход в Orbit с пометкой unstable.
- Сценарий 4: тангаж‑abort на старте → как раньше, gameover один раз.
- В логе НЕ должно быть спама event'ов (мы недавно починили это в `f3793ac`).

---

## Что НЕ делать

- **Полноценный 2D/3D пропагатор орбит** на инерциальных координатах. Это переход с `altitude_m`/`horizontal_speed_ms` на вектор положения — рефакторинг физики на 1‑2 дня + поломка всего что зависит от `FlightDynamics`. Для нужного эффекта избыточно.
- **Реальную модель ISA‑атмосферы** с тропосферой/стратосферой. Экспоненциальной 1.225·exp(−h/8000) хватает.
- **Тепловой нагрев на спуске.** В ракете нет щита. Просто impact.

---

## Команды для быстрого старта новой сессии

```bash
git log --oneline -3        # убедиться что HEAD = f3793ac или новее
cat docs/next-session-suborbital.md   # этот документ
cargo check                 # baseline
```

И первым делом спросить пользователя про п. 1‑3 в «Открытых вопросах».
