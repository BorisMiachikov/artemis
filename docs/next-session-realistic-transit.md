# Реалистичная траектория Transit: TLI к Луне (n-body RK4)

> **Базовый коммит:** `a86ecc6` (HEAD на момент написания).
> **Контекст:** В этой сессии добавили ISS+Astronaut в сцены. Следующая сессия —
> переписать стадию Transit: вместо линейной интерполяции `dist_earth_km` сделать
> настоящие 3D-координаты Orion и Луны + RK4 интегрирование гравитации
> Земли и Луны. Цель — траектория эллипса от LEO к Луне, приближённая к реальной
> Apollo/Artemis.

---

## TL;DR

Сейчас Transit — это «фоновая картинка с таймером»:
`elapsed_s / 120` → масштаб Земли уменьшается, масштаб Луны растёт. Нет
векторов, нет физики, MCC `±0.12` к ошибке.

После рефакторинга:
- `TrajectorySim { orion_pos_km, orion_vel_kms, moon_pos_km, moon_vel_kms }`
  в инерциальных координатах с центром в Земле
- RK4 интегрирование с гравитацией Земли (μ=398600) + Луны (μ=4903)
- Реальное время перелёта ~3 суток сжимается через KSP-style time-warp
- Прогноз траектории gizmos-линиями на 3 суток вперёд
- 4 режима камеры (Wide-3D / Wide-2D top-down / OrionChase / Cockpit)
- Auto-target MCC: Q/E применяет Δv в сторону уменьшения/увеличения
  predicted perilune

---

## Принятые решения (из вопросов в текущей сессии)

| Вопрос | Ответ |
|---|---|
| Сжатие времени | **Только через timescale** — игрок крутит warp сам, базового жёсткого времени нет |
| Камера | **Переключаемая** между Wide-3D/Wide-2D/Chase/Cockpit |
| MCC (Q/E) | **Auto-target** — Δv в направлении приближения/отдаления от расчётной точки встречи |
| 2D vs 3D | **Оба переключаемы клавишей** — внутренне 3D, projection toggle между ortho/perspective |

---

## Архитектура

### 1. Новый ресурс `TrajectorySim`

```rust
// src/physics/trajectory.rs (НОВЫЙ файл)
use bevy::math::DVec3;

#[derive(Resource, Debug, Clone)]
pub struct TrajectorySim {
    pub orion_pos_km:   DVec3,    // Earth-centered inertial
    pub orion_vel_kms:  DVec3,
    pub moon_pos_km:    DVec3,
    pub moon_vel_kms:   DVec3,
    pub elapsed_sim_s:  f64,

    pub closest_approach_km:    f64,    // running min |orion - moon|
    pub closest_approach_t_s:   f64,
    pub predicted_perilune_km:  f64,    // forward-prop ~3 days
    pub predicted_perilune_t_s: f64,

    pub mcc_fuel_kg:    f32,
    pub trail_buffer:   VecDeque<DVec3>,    // последние ~500 позиций для рендера трасы
}
```

Итого ресурс ~150 байт + буфер. Использование `DVec3` критично — `f32` накопит
ошибку за 3 дня RK4.

### 2. Константы (`src/config.rs`)

```rust
// SI/астрономические
pub const MU_EARTH_KM3_S2:  f64 = 398_600.4418;
pub const MU_MOON_KM3_S2:   f64 =   4_902.800066;
pub const EARTH_R_KM:       f64 = 6_371.0;
pub const MOON_ORBIT_R_KM:  f64 = 384_400.0;
pub const MOON_ORBIT_V_KMS: f64 = 1.022;
pub const SOI_MOON_KM:      f64 =  66_100.0;
pub const LEO_ALT_KM:       f64 =     200.0;
pub const LEO_R_KM:         f64 = EARTH_R_KM + LEO_ALT_KM;
pub const LEO_V_KMS:        f64 = 7.784;
pub const TLI_DV_NOMINAL:   f64 = 3.050;     // ICPS Δv

// Игровое масштабирование (km → game units)
pub const KM_TO_UNITS: f32 = 6.0 / 6_371.0;  // Earth radius 6 units = 6371 km
                                              // → Moon at 384400 km = ~362 units

// Time-warp уровни (KSP style)
pub const WARP_LEVELS: &[f32] = &[1.0, 5.0, 20.0, 100.0, 500.0, 2000.0];
pub const WARP_DEFAULT_INDEX: usize = 3;     // ×100 на старте

// MCC Δv параметры
pub const MCC_PRESS_DV_MS:    f32 = 1.5;     // м/с за нажатие
pub const MCC_PRESS_FUEL_KG:  f32 = 0.8;     // кг за нажатие
pub const MCC_FUEL_INITIAL:   f32 = 200.0;
```

### 3. RK4 интегратор + sub-stepping

```rust
fn rk4_step(state: &mut TrajectorySim, dt_s: f64) {
    // accel(pos_orion) = -μ_e·r_orion/|r_orion|³ - μ_m·(r_orion - r_moon)/|...|³
    // accel(pos_moon)  = -μ_e·r_moon/|r_moon|³  (двух-тельная задача относит. Земли)
    // Стандартный RK4 на 12-мерном векторе [pos_orion, vel_orion, pos_moon, vel_moon]
}

// Sub-stepping: при warp ×2000 один игровой кадр (16мс) → 32 sec sim time
// RK4 с шагом 32 sec разойдётся → бьём на N подшагов по MAX_DT=2.0 sec
const MAX_DT_S: f64 = 2.0;
fn integrate(state: &mut TrajectorySim, dt_total: f64) {
    let n = (dt_total / MAX_DT_S).ceil().max(1.0) as usize;
    let dt = dt_total / n as f64;
    for _ in 0..n { rk4_step(state, dt); }
}
```

### 4. Auto-target MCC

```rust
fn apply_mcc(state: &mut TrajectorySim, sign: f32) {
    // 1. Forward-propagate копию state на 3 дня
    let mut sim = state.clone();
    let target_t = state.elapsed_sim_s + 3.0 * 86400.0;
    let mut min_dist = f64::INFINITY;
    let mut min_pos = DVec3::ZERO;
    while sim.elapsed_sim_s < target_t {
        rk4_step(&mut sim, 30.0);
        let d = (sim.orion_pos_km - sim.moon_pos_km).length();
        if d < min_dist { min_dist = d; min_pos = sim.orion_pos_km; }
        if d < EARTH_R_KM { break; }    // упал
    }
    state.predicted_perilune_km = min_dist;

    // 2. Δv в направлении уменьшения min_dist (Q) или увеличения (E)
    //    Используем gradient-descent-эвристику: Δv по направлению
    //    (target_predicted - orion) с малой амплитудой.
    let to_target = (min_pos - state.orion_pos_km).normalize();
    let dv = sign * (MCC_PRESS_DV_MS as f64 / 1000.0);    // m/s → km/s
    state.orion_vel_kms += to_target * dv;
    state.mcc_fuel_kg -= MCC_PRESS_FUEL_KG;
}
```

Q (sign=+1) — Δv в сторону точки встречи (уменьшает perilune).
E (sign=−1) — Δv от точки встречи (увеличивает perilune).
По сути «магнитный курс» — игрок не разбирается в орбитальной механике, просто
жмёт Q пока predicted perilune не будет ~100 км.

### 5. Time-warp (KSP-style)

```rust
#[derive(Resource, Default)]
pub struct WarpLevel(pub usize);    // index в WARP_LEVELS

// hotkeys в src/input.rs:
//   .  → warp += 1 (cap WARP_LEVELS.len()-1)
//   ,  → warp -= 1 (min 0)
// HUD показывает текущий ×N
```

`TimeScale.multiplier` обновляется = `WARP_LEVELS[warp_level.0]`. Существующая
система `tick_transit(time, timescale)` интегрирует через timescale ×
delta_secs — никакой ломки.

### 6. Камеры (4 режима)

```rust
// src/camera.rs дополнить:
#[derive(Resource, Default)]
pub enum TransitCameraMode {
    #[default]
    Wide3D,         // перспектива, Earth+Orion+Moon в кадре
    Wide2D,         // ortho top-down (камера +Y, projection ortho)
    OrionChase,     // как Chase в Launch — за кораблём
    Cockpit,        // изнутри
}

// Хоткеи: 1/2/3/4 (как сейчас в orbit/launch)
```

Wide3D: камера в фиксированной точке (например, +0, +200, +600 game units),
смотрит на середину между Земли и Луны. По мере полёта center-of-view
интерполируется.

Wide2D: камера сверху (+0, +1000, +0), `OrthographicProjection { scale: 800 }`,
показывает плоскость орбиты. Это самый «карто-подобный» режим.

OrionChase: повторяет существующий Chase из Launch — `transform_translation =
orion_pos + Vec3::new(0, 5, 15)`.

Cockpit: камера в позиции Orion, look_at = velocity_direction.

### 7. Визуализация траектории (Gizmos)

```rust
// src/render/trajectory_gizmos.rs (НОВЫЙ)
fn draw_trajectory(
    state: Res<TrajectorySim>,
    mut gizmos: Gizmos,
) {
    // Прошлый путь (буфер trail_buffer): белая линия
    for window in state.trail_buffer.iter().collect::<Vec<_>>().windows(2) {
        gizmos.line(km_to_unit(*window[0]), km_to_unit(*window[1]), Color::WHITE);
    }
    // Прогноз: 3 дня вперёд, зелёный пунктир (clone+propagate)
    // Закрытая орбита Луны: круг радиуса 384400 км в плоскости XZ
    // Closest approach marker: оранжевая точка на predicted_perilune
}
```

Буфер `trail_buffer` пушится каждые ~5 sim-секунд, ёмкость 500 (~40 мин трасы).

### 8. Прибытие в Лунную SOI

Заменить `check_arrival` на:
```rust
fn check_lunar_soi(state: Res<TrajectorySim>, mut next: ResMut<NextState<MissionStage>>) {
    let dist_to_moon = (state.orion_pos_km - state.moon_pos_km).length();
    if dist_to_moon < SOI_MOON_KM {
        next.set(MissionStage::LunarFlyby);
    }
}
```

`LunarFlyby` (стадия [src/stages/lunar_flyby.rs](src/stages/lunar_flyby.rs))
сейчас читает `TliResult.accuracy_pct` для расчёта perilune. Нужно поправить
чтобы брал `state.closest_approach_km` напрямую (или его forward-projected
вариант от момента входа в SOI).

### 9. Условие провала

```rust
fn check_miss(state: Res<TrajectorySim>, ...) {
    // Если прошли максимум сближения и удаляемся — миссия провалена
    let dist = (state.orion_pos_km - state.moon_pos_km).length();
    let v_radial = (state.orion_vel_kms - state.moon_vel_kms)
                       .dot((state.orion_pos_km - state.moon_pos_km).normalize());
    if state.elapsed_sim_s > 4.0 * 86400.0 && v_radial > 0.0 && dist > SOI_MOON_KM * 5.0 {
        events.write(MissionEvent::Abort("alert.missed_lunar_approach".into()));
    }
}
```

---

## Файлы и изменения

| Файл | Действие | Объём |
|---|---|---|
| `src/physics/trajectory.rs` | **НОВЫЙ** — `TrajectorySim`, RK4, sub-stepping, gravity | ~250 строк |
| `src/physics/mod.rs` | + `pub mod trajectory;` | +1 строка |
| `src/render/trajectory_gizmos.rs` | **НОВЫЙ** — линии трасы и прогноза | ~100 строк |
| `src/render/mod.rs` (если нет — создать) | + `pub mod trajectory_gizmos;` | +1-5 строк |
| `src/stages/transit.rs` | **переписать** — `setup_transit` ставит initial conditions, `tick_transit` интегрирует, удалить linear interp + `animate_scene_bodies` | ~250 строк (было 290) |
| `src/stages/lunar_flyby.rs` | поправить `setup_lunar_flyby` чтобы брал `TrajectorySim.closest_approach_km` вместо `accuracy_pct` | +20 строк |
| `src/ui/transit_hud.rs` | **НОВЫЙ** — окно с warp level, predicted perilune, Δv доступно, time-to-perilune | ~150 строк |
| `src/ui/hud.rs` | старое окно `transit telemetry` оставить, добавить вызов нового HUD | +5 строк |
| `src/camera.rs` | + `TransitCameraMode` enum + системы переключения | +80 строк |
| `src/input.rs` | warp `.`/`,`, camera switch `1`/`2`/`3`/`4`, MCC `Q`/`E` mapping на новую функцию | +40 строк |
| `src/config.rs` | + 12 констант (см. секцию Константы) | +25 строк |
| `assets/i18n/{ru,en}.ron` | i18n keys: `transit.warp`, `transit.perilune`, `transit.time_to_periapsis`, `alert.missed_lunar_approach` | +10 строк × 2 |
| `src/save.rs` (если есть) | расширить save-формат на `TrajectorySim` или запретить save в Transit | TBD |

**Итого:** ~700 строк нового кода, ~200 строк удалено/изменено.

---

## Этапы реализации (по 1-2 часа)

1. **Constants + skeleton** — добавить константы в `config.rs`, создать пустой `physics/trajectory.rs` с `TrajectorySim` ресурсом и регистрацией в plugin. Юнит-тесты-заглушки. **30 мин**.

2. **RK4 ядро** — gravity-функции (Earth+Moon), `rk4_step`, `integrate` с sub-stepping. Тесты: круговая орбита (период 2π√(r³/μ)), сохранение энергии (drift <1% за 3 дня). **1 ч**.

3. **Initial conditions** — `setup_transit` ставит Orion в перигее эллипса с правильным Δv от ICPS, Луну в 60° впереди (упреждение). Удалить старый linear interp и random events временно. **45 мин**.

4. **Sync 3D мира** — каждый кадр обновлять `Transform` Земли/Луны/Orion из km-позиций × `KM_TO_UNITS`. Удалить `animate_scene_bodies`. **30 мин**.

5. **Time-warp** — `WarpLevel` ресурс, hotkeys `.`/`,`, обновление `TimeScale.multiplier`. HUD-индикатор. **30 мин**.

6. **Камеры** — 4 режима, переключение `1`/`2`/`3`/`4`. Wide2D через `OrthographicProjection`. **1.5 ч**.

7. **Trajectory gizmos** — буфер прошлых позиций, прогноз forward-prop, орбита Луны, perilune marker. **1 ч**.

8. **Auto-target MCC** — forward-prop при каждом нажатии Q/E, Δv в нужном направлении, отображение predicted perilune. Random events вернуть. **1.5 ч**.

9. **SOI transit + miss detection** — `check_lunar_soi`, `check_miss`, переход в `LunarFlyby` с правильным perilune. **45 мин**.

10. **HUD-панель** — warp level, perilune, Δv, time-to-perilune, MCC fuel. i18n. **1 ч**.

11. **Интеграция LunarFlyby** — `setup_lunar_flyby` читает `TrajectorySim` вместо `TliResult.accuracy_pct`. **30 мин**.

12. **Полировка + clippy + тесты + ручные сценарии** — **1 ч**.

**Итого: ~10-11 часов работы.**

---

## Открытые вопросы (решить в начале следующей сессии)

1. **Default warp на старте Transit:** ×1 (real-time, нужно warp руками) или ×100 (стартует в warp)?
2. **2D режим — настоящая орто-проекция или just look-down?** Если ortho — нужна `OrthographicProjection` (Bevy 0.18 API проверить).
3. **Trajectory line стиль:** прошлая орбита белая solid или градиент по времени? Прогноз — пунктир или solid? Перекраска при miss-trajectory в красный?
4. **Пропуск warp при close approach:** автоматически снижать warp до ×5 при `dist_moon < 10000 km` чтобы игрок не пропустил вход в SOI?
5. **MCC точность авто-target:** 1 нажатие = 1.5 m/s или динамически (например, амплитуда зависит от расстояния до perilune)?
6. **Free-return геометрия:** делать так, чтобы при идеальном TLI (accuracy 100%) траектория была реально free-return (возвращается к Земле если не делать LOI)? Или просто прицеливаемся в перилуну Луны?
7. **MCC топливо 200 кг → сколько нажатий:** при 0.8 кг/нажатие = 250 нажатий, это много. Уменьшить до 50 кг или увеличить расход?
8. **Save/load в Transit:** запрещаем (флаг в save UI) или сериализуем `TrajectorySim` (DVec3 → 24 байта × 4 = 96 байт + буфер)?
9. **LunarFlyby интерфейс:** оставляем текущую сцену или тоже переписываем под векторы (логично, но ещё +5 ч работы)?
10. **Достижения:** новое достижение «Свободный возврат» (free-return геометрия без MCC), «Точное попадание» (perilune <100 км без MCC)?
11. **Pause при warp:** жать `Esc` останавливает sim (обычная пауза) или есть отдельный SimPaused для warp ×0?

---

## Риски и регрессии

- **RK4 stability при warp ×2000:** 32 sec/frame внутри одного кадра — sub-stepping обязателен, иначе орбита разойдётся за минуты. **Тест:** круговая LEO-орбита 100 витков на ×2000 — должна остаться круговой ±1%.

- **f32 vs f64:** Все km-координаты в `f64` (DVec3). Конверсия в `Transform` через `f32 × KM_TO_UNITS` — точности хватит до ~10⁷ km, а Луна на 4×10⁵ km, запас есть.

- **LunarFlyby breakage:** текущая стадия рассчитывает perilune от `TliResult.accuracy_pct`. Сейчас 9 unit-тестов на `accuracy_pct_*` — их **не трогаем**, просто игнорируем `accuracy_pct` в `setup_lunar_flyby`, заменяем на `closest_approach_km` из `TrajectorySim`.

- **Save format:** RON-файлы старых сейвов сломаются. Решение: добавить version field, при mismatch — игнорировать transit-state и стартовать с initial conditions.

- **Performance gizmos:** ~500 line segments × 60 FPS = 30000 draw calls/sec. Bevy gizmos оптимизированы под это, но проверить frame time. Альтернатива: meshline через mesh.

- **Production scale:** Wide-3D camera на расстоянии 600 game units — рендер всего в радиусе 1000 units. Skybox должен быть на радиусе ≥2000. Проверить `Starfield` ([src/starfield.rs](src/starfield.rs)) на скейл.

---

## Условия завершения (acceptance)

```bash
cargo clippy -- -D warnings        # 0 предупреждений
cargo test                         # 9 старых + ~3 новых на trajectory.rs
cargo run --release                # 60 FPS на Wide-3D
```

**Ручные сценарии:**

1. **Идеальный TLI (accuracy ≥95%):** старт Transit → predicted perilune ~100 км
   без MCC → time-warp ×2000 → автопереход в LunarFlyby через ~3 дня sim-time
   (~7 сек wall-time).

2. **TLI 50%:** старт → predicted perilune ~5000 км → 2-3 нажатия Q → perilune
   ~150 км → переход в LunarFlyby.

3. **TLI 10%:** старт → predicted perilune ~50000 км → 10 нажатий Q не хватает
   (топлива не хватит) → траектория проходит мимо Луны → через 4 sim-дня
   `Abort("alert.missed_lunar_approach")` → gameover.

4. **Camera switch:** клавиши 1/2/3/4 переключают режимы плавно, без skip
   кадров. Wide-2D показывает ортогональную проекцию плоскости орбит.

5. **Warp:** `,` и `.` крутят ×1 → ×5 → ×20 → ×100 → ×500 → ×2000. На ×2000
   орбита Луны замыкается за ~13 sim-секунд.

6. **Регрессия:** Launch → MECO → Orbit → TLI burn → Transit → переход
   проходит без паник. Random events (solar flare, micrometeorite) работают.

---

## Что НЕ делаем

- **Полная n-body N>2:** учитываем только Землю и Луну. Солнце/планеты —
  излишне для TLI-перелёта.
- **Patched conics:** простой n-body RK4 даёт ту же точность без сложности
  переключения SOI.
- **Атмосферное торможение в Transit:** уже на 200 км альтитуде нет; в SOI
  Луны — там нет атмосферы.
- **Лагранжевы точки, гало-орбиты:** Artemis II не идёт через NRHO — летим
  прямо к Луне.
- **Реальная относительная скорость испарения топлива (Isp):** упрощаем,
  фиксированный кг/нажатие.

---

## Команды для быстрого старта новой сессии

```bash
git log --oneline -5                                        # HEAD = a86ecc6
cat docs/next-session-realistic-transit.md                  # этот документ
cat src/stages/transit.rs                                   # что переписываем
cat src/stages/lunar_flyby.rs                               # с чем интегрируем
grep -n "DVec3\|f64" src/ -r                                # где уже f64
cargo check
```

И первым делом — **ответить на 11 открытых вопросов выше**, особенно п.1, 4, 7, 9.
