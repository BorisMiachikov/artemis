# 🚀 Roadmap — «Артемида 2: Полёт к Луне»

> **Версия:** 1.5
> **Дата:** 2026-04-29
> **Статус:** Фаза 0 ✅ · Фаза 1 ✅ · Фаза 2 ✅ · Фаза 3 ✅ · Фаза 4 ✅ · Фаза 5 ✅ · Фаза 6 ✅ · Фаза 7 ✅ · Фаза 8a ✅ · Фаза 8b ✅ · Фаза 8c ✅ · Фаза 8d ✅ · Фаза 8e ✅ · Фаза 9 ✅ — `cargo build` 0 ошибок, 0 предупреждений, 9 тестов (2026-04-30)
> **Тип:** 3D-симулятор полёта на Rust (Bevy)

---

## 1. Цель проекта

Построить 3D-симулятор реальной миссии NASA **Artemis II** (1–11 апреля 2026): запуск SLS Block 1 с LC-39B, выход на орбиту, манёвр TLI, перелёт к Луне, облёт на минимальной дистанции 6 556 км, возвращение и приводнение в Тихом океане.

Сценарий — **7 последовательных играбельных этапов**. Упор не на «реалистичную орбитальную механику Кеплера», а на узнаваемость профиля миссии и реалистичность ключевых параметров (масса, тяга, длительность горений, скорости).

**Платформы:** Windows (основная), Linux (вторичная). Desktop-only.

---

## 2. Изменения относительно исходного ТЗ (`GAME_PROMPT.md`)

При планировании были пересмотрены пункты ТЗ. Вот что отличается и почему:

| # | Что меняется | Было | Стало | Причина |
|---|---|---|---|---|
| 1 | Bevy | 0.15 | **0.18.1** ✓ | На апрель 2026 актуальна 0.18.1 — новый рендер-граф, мульти-pass egui, изменённый API States/Assets |
| 2 | bevy_egui | 0.30 | **0.39.1** ✓ | Совместима с Bevy 0.18 (см. README крейта) |
| 3 | bevy_rapier3d | 0.27 | **0.33.0** ✓ | Зафиксирована при `cargo add` |
| 4 | bevy_asset_loader | 0.21 | **0.26.0** ✓ | Зафиксирована при `cargo add` |
| 5 | bevy_panorbit_camera | 0.19 | **0.34.0** ✓ | Зафиксирована при `cargo add` |
| 6 | rand | 0.8 | **0.10.1** ✓ | API изменился (`thread_rng()` → `rand::rng()`), учтём в фазах |
| 7 | Edition | 2021 | **2024** ✓ | `cargo init` ставит по умолчанию, поддерживается Rust 1.91 |
| 8 | Физика | Rapier на всём | **Кастомная физика + Rapier только для reentry** | Rapier — движок для столкновений твёрдых тел; орбитальная механика и тяга легко пишутся вручную и точнее под задачу |
| 9 | План | «4 недели на всё» | **6 фаз, 3–6 месяцев** | Реалистичный темп для одиночной разработки |
| 10 | Структура `src/` | 12 файлов | **+ `input.rs`, `events.rs`, `config.rs`, `i18n.rs`, `save.rs`** | Разделение ответственности: ввод, события миссии, константы, локализация, сейвы — в собственных модулях |

---

## 3. Технический стек

```toml
[package]
name = "artemis"
version = "0.1.0"
edition = "2024"

[dependencies]
bevy = "0.18"
bevy_asset_loader = "0.26.0"
bevy_egui = "0.39.1"
bevy_panorbit_camera = "0.34.0"
bevy_rapier3d = "0.33.0"
rand = "0.10.1"
```

**Целевые платформы:** Windows, Linux.
**Rust:** stable 1.91+ (зафиксировано в `rust-toolchain.toml`).
**Лицензии:** код — MIT/Apache 2.0; ассеты NASA — public domain; звуки Endless Sky — GPL v3.

---

## 4. Структура проекта

```
artemis/
├── Cargo.toml
├── rust-toolchain.toml
├── .gitignore
├── README.md
├── roadmap.md                ← этот файл
├── GAME_PROMPT.md
├── artemis2_game_reference.md
├── assets/
│   ├── models/               (14 готовых GLB + перенесённые SLS/Orion/Earth/Moon)
│   └── sounds/
│       ├── endless-sky/      (24 звука)
│       └── nasa-real/        (artemis1_launch.mp3 + переговоры из referens/)
└── src/
    ├── main.rs               # App setup, регистрация плагинов
    ├── states.rs             # MissionStage (States enum)
    ├── config.rs             # SLS_PARAMS, физ. константы (Isp, GM), Difficulty
    ├── assets.rs             # AssetLoader, типизированные хэндлы
    ├── events.rs             # MissionEvent: SrbSep, MecoEvent, TliBurn, Splashdown, Abort
    ├── input.rs              # Маппинг W/S (тяга), A/D (рыскание), Q/E (крен), F1–F4 камеры
    ├── camera.rs             # 4 режима: Cockpit / Chase / External (PanOrbit) / Free
    ├── audio.rs              # Реакция на MissionEvent — проигрывание звуков
    ├── i18n.rs               # RU+EN локализация, Lang resource, t!(key)
    ├── save.rs               # Автосейв после каждого этапа в RON
    ├── physics/
    │   ├── mod.rs
    │   ├── rocket.rs         # Rocket, FlightDynamics, формула Циолковского
    │   ├── orbital.rs        # Гравитация GM/r², 2-body для Земля/Луна
    │   └── reentry.rs        # bevy_rapier3d — коллизии с океаном
    ├── stages/
    │   ├── mod.rs
    │   ├── prelaunch.rs      # Этап 0
    │   ├── launch.rs         # Этап 1
    │   ├── orbit.rs          # Этап 2
    │   ├── tli.rs            # Этап 3
    │   ├── transit.rs        # Этап 4
    │   ├── lunar_flyby.rs    # Этап 5
    │   └── reentry.rs        # Этап 6
    └── ui/
        ├── mod.rs
        ├── hud.rs            # Телеметрия (egui)
        ├── menus.rs          # Главное меню, пауза
        └── mission.rs        # Брифинги, итоги этапов, экран победы
```

---

## 5. Фазы разработки

### Фаза 0 — Подготовка ✅ (завершена 2026-04-27)

- [x] `cargo init --vcs git` (папка не пустая, поэтому `init`, а не `new`)
- [x] Cargo.toml с зафиксированными версиями (см. раздел 3)
- [x] `[profile.dev] opt-level = 1` + `[profile.dev.package."*"] opt-level = 3` для скорости разработки
- [x] `.gitignore` (с `/referens/`), `rust-toolchain.toml` (stable + clippy + rustfmt), `README.md`
- [x] Перенос недостающих ассетов в `assets/models/{SLS,Orion,Earth,Moon}/` и 20 mp3 NASA в `assets/sounds/nasa-real/`
- [x] Базовый `main.rs` с `DefaultPlugins`, заголовком окна и `Camera2d`

**DoD достигнут:** `cargo build` — 10 мин 03 с (первая сборка), 0 warnings.
`cargo run` — окно «Artemis II — Полёт к Луне» открывается, Vulkan на NVIDIA RTX 4060 Ti, никаких WARN/ERROR.

**Замечания, выявленные на фазе:**
- В Bevy 0.18 `WindowResolution::from` принимает `(u32, u32)`, не `(f32, f32)` — учесть в дальнейшем
- `rand` 0.10 — `thread_rng()` переименован в `rand::rng()`, использовать новый API в Фазе 5 (случайные события)
- `Orion.glb` всего 109 КБ — проверить наличие меша в Фазе 1, при необходимости заменить

---

### Фаза 1 — Базовая 3D-сцена ✅ (завершена 2026-04-27)

- [x] Загрузка GLB через `bevy_asset_loader` с loading state (`MissionStage::Loading` → `Prelaunch`). На фазе использован Saturn V вместо Astronaut — см. замечание о Draco ниже
- [x] DirectionalLight 50 000 lux с тенями + AmbientLight (0.18, 0.20, 0.28; brightness 80) — космос тёмный с тёплым ambient
- [x] PanOrbit-камера в позиции (0, 50, 200), смотрит на (0, 50, 0)
- [x] `camera.rs`: enum `CameraMode { Cockpit, Chase, External, Free }`, переключение F1–F4 готово (на этой фазе работает External)
- [x] Скаффолдинг всех 19 модулей из раздела 4: у каждого `pub fn plugin(app: &mut App)`, всё включено в main.rs
- [x] `ClearColor::BLACK` (космический фон); полноценный скайбокс со звёздами — отложен на Фазу 2 как часть UI/атмосферы

**DoD достигнут:** `cargo build` — 12.84 с, 0 warnings; bevy_asset_loader пишет `Loading state ... is done`; FPS стабильно ~60 (avg 60.02) на RTX 4060 Ti.

**API-различия Bevy 0.18, выявленные на фазе** (учесть в дальнейшем):
- `#[derive(Event)]` → `#[derive(Message)]`, `add_event::<T>()` → `add_message::<T>()`, `EventReader/Writer` → `MessageReader/Writer`. Старый `Event` теперь — observer-trigger (`On<E>`)
- `AmbientLight` стал компонентом камеры, а не `Resource`
- `StateScoped(state)` → `DespawnOnExit(state)` / `DespawnOnEnter(state)` (из `bevy::state::state_scoped`)
- `WindowResolution::from((u32, u32))` — целые, не `f32` (зафиксировано в Фазе 0)

**Замечания:**
- ⚠️ **Draco-сжатие в GLB**: 11 из 18 моделей в `assets/models/` сжаты `KHR_draco_mesh_compression`, который bevy_gltf не поддерживает. Список и план конвертации — см. разделы 6 и 7
- Запуск `./target/debug/artemis.exe` напрямую не находит `assets/` (нужен `cargo run` или `CARGO_MANIFEST_DIR=...`). Починим в Фазе 2 через `AssetPlugin { file_path }`

---

### Фаза 2 — HUD, States, i18n, сейвы ✅ (завершена 2026-04-27)

- [x] `bevy_egui`: верхняя HUD-панель (T+ таймер + текущий MissionStage) и окно телеметрии (заглушки скорость/высота/тяга/топливо/G/тангаж). Стиль NASA Blue `#0B3D91` + SLS Orange `#FC3D21`, тёплый текст, тёмный полупрозрачный фон (см. `src/ui/theme.rs`)
- [x] `MissionStage` enum + `DespawnOnExit` для очистки сцены при переходе (вместо устаревшего `StateScoped`)
- [x] Переходы Prelaunch → Launch по `Space` ИЛИ через кнопку «СТАРТ МИССИИ» в главном меню
- [x] `audio.rs`: `machinery.mp3` на Prelaunch с `DespawnOnExit`, `human launch.wav` по `MissionEvent::Liftoff`. В Cargo.toml включены `bevy` features `mp3, wav`
- [x] `events.rs`: `MissionEvent` enum (Liftoff/SrbSep/Meco/TliBurn*/PerilunePassage/AtmosphereEntry/Splashdown/Abort) — `Message` в Bevy 0.18
- [x] `i18n.rs`: ресурсы `Lang { Ru, En }` и `Translations`, словари `assets/i18n/{ru,en}.ron` (по 31 ключу), `Translations::get(lang, key)` метод вместо макроса
- [x] `save.rs`: `SaveSlot { mission_stage, fuel_kg, tli_delta_v_ms, timestamp_unix }`, `OnEnter`-хук на все 8 стейтов миссии, RON в `%APPDATA%/Artemis/Artemis/data/save.ron` через `directories::ProjectDirs`
- [x] Главное меню в `Prelaunch` + persistent settings (combo-box языка и сложности), переключение языка применяется в HUD сразу же
- [x] `AssetPlugin { file_path }` с резолвом по приоритету: `CARGO_MANIFEST_DIR` → рядом с exe → cwd → "assets" — теперь exe запускается и без `cargo run`

**DoD достигнут:** при запуске видим `i18n: загружено 31 ключей для Ru/En`, `Loading state ... is done`, `save: автосейв в C:\Users\lb426\AppData\Roaming\Artemis\Artemis\data\save.ron (стейт Prelaunch)`. FPS ~60. Файл `save.ron` валидный RON. Переключатель языка в меню работает.

**Зависимости, добавленные на фазе:** `serde 1.0.228` (с derive), `ron 0.12.1`, `directories 6.0.0`. Bevy: `features = ["mp3", "wav"]`.

**API-нюансы Bevy 0.18, выявленные на фазе:**
- UI-системы egui идут в schedule `EguiPrimaryContextPass`, не `Update`
- `bevy_audio` декодеры формата за фичами — без явного `mp3`/`wav` в Cargo.toml ассеты не грузятся (`Could not find an asset loader matching`)
- Системы, читающие ресурсы из `LoadingState`-коллекции в `Update`, требуют `.run_if(resource_exists::<GameAssets>)` — иначе паника до завершения загрузки

**Замечания на Фазу 3:**
- Главное меню сейчас живёт в Prelaunch. Когда Prelaunch получит реальный контент (Gantry+Crawler+проверки систем), вынести меню в отдельный `AppState::MainMenu` перед `MissionStage::Loading`
- Реальная телеметрия HUD появится после `Rocket`/`FlightDynamics` компонентов в Фазе 3

---

### Фаза 3 — Физика запуска: этапы 0–1 ✅ (завершена 2026-04-28)

**Физика:**
- [x] `config.rs`: `SLS_PARAMS` (масса 2 608 000 кг, тяга 39 144 кН, Isp RS-25 = 453 с, Isp SRB = 269 с, длительности 126 с / 510 с) + `Difficulty { Story, Realistic }` с коэффициентами окон допусков (`pitch_tolerance_deg`, `tli_burn_window_sec`, `reentry_angle_window_deg`)
- [x] `physics/rocket.rs`: компоненты `Rocket { thrust_kn, fuel_kg, stage, mass_kg }`, `FlightDynamics { velocity, altitude_km, pitch_deg, g_load }`
- [x] Системы: применение тяги (F = Isp × g₀ × ṁ), уменьшение массы по расходу
- [x] `physics/orbital.rs`: гравитация `g = GM/r²` (GM_earth = 3.986×10¹⁴)

**Этап 0 (Prelaunch):**
- [ ] Кат-сцена с `Gantry.glb` + `Crawler.glb` _(отложено: модели Draco-сжаты, нет конвертера)_
- [x] Мини-UI «проверка систем» (10 чекбоксов перед стартом) → `src/ui/checklist.rs`

**Этап 1 (Launch):**
- [x] SRB горят 126 с (32 МН), сброс по событию `MissionEvent::SrbSep`
- [x] RS-25 продолжают до T+8:30, событие `MecoEvent`
- [x] Управление тангажом по A/D, fail-state при отклонении > N° (число подобрать)
- [x] HUD заполнен реальными значениями
- [x] Звуки: `human launch.wav` → `takeoff.wav` → `afterburner~.wav`

**DoD:** ракета взлетает по реалистичному профилю, выходит на ~200 км за ~8.5 мин, SRB сбрасываются вовремя, экран проигрыша срабатывает при отклонении.

---

### Фаза 4 — Орбита и TLI: этапы 2–3 ✅ (завершена 2026-04-29)

**Тонкий вертикальный слой Orbit + TLI ✅ (готов 2026-04-28):**

**Этап 2 (Orbit):**
- [x] Сцена орбиты: `Earth.glb` в центре + `Orion.glb` на круговой траектории, медленное вращение Земли (≈0.05 рад/с) и Orion (≈0.10 рад/с) — визуальная упрощёнка LEO
- [x] Mini-game: `ui/orbit_checklist.rs` — 12 кликабельных систем корабля (egui), кнопка `GO FOR TLI` блокируется до полной чек-листа
- [x] Фоновая музыка `rulei space.mp3` (loop 0.25), `scan~.wav` на каждом активирующем клике чек-листа
- [ ] Реализация `CameraMode::Cockpit` / `CameraMode::Chase` — вынесено за рамки тонкого слоя

**Этап 3 (TLI):**
- [x] `IcpsParams` (тяга 110.1 кН, Isp 462 с, масса связки 56 т, целевые Δv = 3 050 м/с / 1 080 с — реальный Artemis II ICPS burn) — `src/config.rs`
- [x] `stages/tli.rs`: ресурс `TliBurnState { Idle, Burning, Completed }` + `IcpsBurn`; ввод по `Space` начинает burn; ṁ = F/(Isp·g₀), Δv накапливается по Циолковскому пошагово; завершается по достижении длительности, целевого Δv или истощения топлива
- [x] `ui/tli_panel.rs` — окно с прогресс-баром, текущим Δv и итоговой точностью
- [x] Звуки: `hyperdrive in.wav` (one) → `hyperdrive.wav` (loop через маркер `TliBurnLoop`) → `hyperdrive out.wav` (one) на `MissionEvent::TliBurnEnd`
- [x] `TliResult { delta_v_ms, burn_duration_s, completed }` сохраняется в `SaveSlot.tli_delta_v_ms` через автосейв
- [ ] Реальное окно выбора момента (не сразу `Space`, а с T+ дедлайном) — отложено
- [ ] Влияние точности TLI на минимальное сближение в Phase 5 — будет сделано в Phase 5

**Доделано на Phase 4 (2026-04-29):**
- [x] `CameraMode::Cockpit` / `CameraMode::Chase` — `camera.rs`: `PlayerVehicle` маркер, remove/insert `PanOrbitCamera`, follow-система
- [x] `TliResult::accuracy_pct` → `TransitOutcome::trajectory_error` → перицентр в LunarFlyby
- [ ] Окно выбора момента TLI (T+ дедлайн) — отложено на Фазу 6-полировку
- [ ] Доработка HUD TLI: дистанция до Земли, расход ICPS — отложено

---

### Фаза 5 — Перелёт и облёт Луны: этапы 4–5 ✅ (завершена 2026-04-29)

**Этап 4 (Transit) ✅:**
- [x] Сцена: Земля (уменьшается сзади) + Луна (вырастает впереди) + Orion в центре
- [x] `TransitState`: дистанции Земля/Луна, CO₂, радиация, MCC-топливо, таймер событий
- [x] Q/E — коррекция курса (−/+12% ошибки траектории, 50 кг топлива/нажатие)
- [x] Случайные события (rand): солнечная вспышка (радиация +12 мЗв), микрометеорит — каждые 35 с, вероятность 30/20%
- [x] `ui/transit_panel.rs`: дистанции, CO₂, радиация, MCC-топливо, точность, алерты
- [x] Переход → LunarFlyby при dist_moon < 50 000 км

**Этап 5 (Lunar Flyby) ✅:**
- [x] `FlybyState` + `FlybyPhase` {Approach, Perilune, Departure}
- [x] Перицентр = 6 556 км × (1 + trajectory_error) — от точности TLI и MCC
- [x] Сцена: Moon.glb крупным планом, Orion пролетает мимо
- [x] Звук `jump drive.wav` при PerilunePassage
- [x] `FlybyResult::perilune_km` сохраняется для экрана победы
- [x] `ui/flyby_panel.rs`: фаза, дистанция, скорость, перицентр

**DoD:** ✅ полный прогон Transit → LunarFlyby → Reentry; точность TLI отражена на расстоянии облёта.

---

### Фаза 6 — Возврат, посадка, полировка ✅ (завершена 2026-04-29)

**Этап 6 (Reentry) ✅:**
- [x] `ReentryState` + `ReentryPhase` {Approach, Entry, Descent, Parachutes}
- [x] W/S — угол входа (шаг 0.05°/0.15°), Space — зафиксировать и начать вход
- [x] Окно угла входа 6.0–6.5°; в Realistic отклонение → `MissionEvent::Abort`
- [x] Тепловой щит: процент нагрева в HUD + прогресс-бар
- [x] Парашюты на < 3.2 км; Splashdown при высоте < 0.05 км → переход Splashdown state
- [x] `landing.wav` при Splashdown, `afterburner.wav` при AtmosphereEntry

**Полировка ✅:**
- [x] `ui/mission.rs`: экран победы (T+, точность TLI, перицентр, кнопка «НАЧАТЬ ЗАНОВО»)
- [x] `ui/mission.rs`: экран поражения (причина Abort, кнопка «НАЧАТЬ ЗАНОВО»)
- [x] `MissionFailed` resource + `listen_for_abort` система

**DoD:** ✅ `cargo build` — 0 ошибок, 0 предупреждений. Полный путь Prelaunch → Splashdown скомпилирован.

---

### Фаза 7 — Полировка: звёзды, debug, TLI-окно, glow ✅ (завершена 2026-04-29)

- [x] `src/starfield.rs`: 2 000 процедурных звёзд на сфере r=850–1000, unlit emissive материал (синевато-белый)
- [x] `src/ui/debug.rs`: F12-оверлей — FPS, frame time, текущий MissionStage
- [x] `stages/tli.rs`: ресурс `TliWindow { countdown_s: 30.0, window_open }` — обратный отсчёт до открытия окна TLI, guard в `handle_burn_input`
- [x] `ui/tli_panel.rs`: отображает «T− Xс до открытия окна» пока `window_open == false`
- [x] `stages/reentry.rs`: `AtmosphericGlow` сфера + `GlowMaterial` ресурс; `tick_glow` обновляет emissive цвет от оранжевого до красного пропорционально `heat_pct`

**DoD:** ✅ `cargo build` — 0 ошибок.

---

### Фаза 8a — LOD для Earth/Moon ✅ (завершена 2026-04-29)

- [x] `src/lod.rs`: `LodMaterials` (Startup), `DistanceLod` компонент, `LodSphere` маркер, `tick_lod` в PostUpdate
- [x] Алгоритм: `apparent = scale.x / dist`; при `apparent < 0.08` → lo-poly сфера, при `apparent ≥ 0.08` → hi-poly GLB
- [x] `tick_lod` синхронизирует Transform lo-poly с hi-poly (обеспечивает корректную работу при динамическом масштабировании в Transit)
- [x] Все 5 стейджей с Earth/Moon обновлены: `orbit`, `tli`, `transit`, `lunar_flyby`, `reentry`

**DoD:** ✅ `cargo build` — 0 ошибок. GLB грузится один раз; при малом видимом размере заменяется процедурной сферой.

---

### Фаза 8b — Визуальная полировка ✅ (завершена 2026-04-30)

- [x] Particle effects для двигателей (SRB/RS-25/ICPS): спавн частиц через `Mesh3d`-спрайты → `src/particles.rs`
- [x] Bloom для звёзд и emissive-объектов: `bevy::core_pipeline::bloom::Bloom`
- [ ] Lens flare для солнца _(опционально, отложено)_

**DoD:** ✅ `cargo build` — 0 ошибок.

---

### Фаза 8c — Геймплей ✅ (завершена 2026-04-30)

- [x] Система достижений (6 ачивок): «Идеальный burn», «Снайпер входа», «Близкий облёт», «Без коррекций», «Холодный щит», «Мастер-пилот» → `src/achievements.rs`
- [x] Итоговый экран статистики с цветовой кодировкой (зелёный/жёлтый/оранжевый) → `src/ui/mission.rs`
- [x] Replay/Flight log: `FlightRecord` записывает метрики по каждому стейту → `src/replay.rs`

**DoD:** ✅ `cargo build` — 0 ошибок.

---

### Фаза 8d — Аудио ✅ (завершена 2026-04-30)

- [x] Ambient-треки: `ambient_machinery` на Prelaunch, `ambient_planet` (loop 0.18) на Transit → `src/audio.rs`
- [x] Реакция звука на все события: Liftoff, SrbSep, TliBurnStart/End, PerilunePassage, AtmosphereEntry, Splashdown, Abort
- [ ] Динамический микс: громкость двигателей зависит от тяги _(отложено)_

**DoD:** ✅ `cargo build` — 0 ошибок.

---

### Фаза 8e — Тестирование ✅ (завершена 2026-04-30)

- [x] `cargo clippy` — 0 предупреждений (исправлены 3 `collapsible_if`, добавлены `#[allow]` для Bevy-систем)
- [x] 9 unit-тестов: `TliResult::accuracy_pct` (4) + структура достижений (5)
- [ ] Debug-команды быстрого перехода на стейт _(отложено)_
- [ ] Проверка save/load между стейтами _(отложено)_

**DoD:** ✅ `cargo test` — 9/9 passed.

---

### Фаза 9 — Экран достижений в меню + хронология полёта ✅ (завершена 2026-04-30)

- [x] `src/replay.rs`: `FlightRecord` + `PhaseRecord {name_ru, name_en, time_s, detail}` — записывается при `OnEnter` каждого стейта; на экране Splashdown отображается секция «ХРОНОЛОГИЯ ПОЛЁТА»
- [x] `src/ui/checklist.rs`: кнопка «☆ Достижения» на экране Prelaunch открывает боковую панель `draw_achievements_panel` со всеми 6 достижениями и счётчиком разблокированных
- [x] i18n-ключи: `replay.timeline`, `checklist.achievements_btn/hide` (RU + EN)

**DoD:** ✅ `cargo build` — 0 ошибок, 0 предупреждений, 9 тестов.

---

## 6. Риски и mitigation

| Риск | Вероятность | Mitigation |
|---|---|---|
| **Draco-сжатие в 11 GLB** (`KHR_draco_mesh_compression`): Astronaut, Crawler, Gantry, Helmet, ISS, JSC Mission Control, ESAS Crew Module, Apollo LM, LRO, EMU, Space Shuttle. bevy_gltf такие файлы не грузит. | **Высокая** | Конвертировать через `gltf-pipeline -i in.glb -o out.glb -d` (флаг `-d` распаковывает Draco). Без сжатия работают: Earth, Gateway, Gemini, Moon, Orion, SLS, Saturn V — их хватает для Фазы 1. Конвертация критичных моделей (Gantry, Crawler, Astronaut) — до Фазы 3 |
| Большие GLB (Earth/Moon 40–55 МБ) тормозят загрузку | Высокая | LOD-версии в Blender или процедурная сфера + текстура |
| `referens/orion_spacecraft.glb` всего 109 КБ — возможно пустышка | Высокая | Проверить в Фазе 0, при необходимости скачать со Sketchfab (ссылки в `artemis2_game_reference.md`) |
| f32 теряет точность на больших расстояниях (LEO ~6.4×10⁶ м) | Высокая | f64 для физических координат, конвертация в f32 для рендера; либо floating origin |
| Bevy 0.18 — свежая версия, часть туториалов под 0.15/0.16 | Средняя | Опираться на официальные примеры репо `bevyengine/bevy` и Context7 |
| Объём контента (7 этапов) при одиночной разработке | Высокая | Внутри каждой фазы — сначала «тонкий вертикальный слой» (минимально играбельный), затем доработка |
| Игрок проходит через всю миссию каждый раз → 60+ минут на тест | Средняя | Debug-команды для перехода на любой стейт, опционально save/load (см. открытые вопросы) |

---

## 7. Чеклист подготовки ассетов (Фаза 0)

- [x] **SLS Block 1** → `assets/models/SLS/SLS.glb` (1.8 МБ, из `referens/artemis_ii_-_space_launch_system_sls.glb`). Проверить пригодность меша в Фазе 1.
- [x] **Orion** → `assets/models/Orion/Orion.glb` (109 КБ, из `referens/orion_spacecraft.glb`). ⚠️ Очень маленький размер — проверить наличие меша в Фазе 1, при отсутствии — искать замену со Sketchfab (ссылки в `artemis2_game_reference.md` §8).
- [x] **Earth** → `assets/models/Earth/Earth.glb` (5.9 МБ, облегчённая `earth (2).glb`). Тяжёлая 45 МБ-версия не используется.
- [x] **Moon** → `assets/models/Moon/Moon.glb` (40 МБ, `our_moon (1).glb`).
- [x] **Аудио NASA Artemis I/II** → `assets/sounds/nasa-real/` (20 mp3: переговоры, отсчёт, liftoff, splashdown).
- [ ] **Astronaut с текстурами**: уже есть `assets/models/Astronaut/Astronaut.glb`; в `referens/astronaut/textures/` есть исходные текстуры — использовать только если возникнут проблемы с встроенными (проверить в Фазе 1).
- [ ] **Конвертация Draco-моделей** (до Фазы 3): запустить `gltf-pipeline -i <input>.glb -o <output>.glb -d` для 11 моделей: `Astronaut`, `Crawler`, `Gantry`, `Helmet`, `International Space Station (ISS) (A)`, `JSC Mission Control Room`, `ESAS Crew Module`, `Apollo Lunar Module`, `Lunar Reconnaissance Orbiter (A)`, `Extravehicular Mobility Unit`, `Space Shuttle (A)`. Установка: `npm install -g gltf-pipeline`. После конвертации файлы станут крупнее — учесть размер репозитория.

---

## 8. Верификация

- Каждая фаза заканчивается ручным прогоном на Windows: `cargo run --release`, проверка DoD.
- После Фазы 3 — интеграционный тест в `physics/rocket.rs`: уравнение Циолковского для известных входов (Δv = Isp × g₀ × ln(m₀/m₁)) даёт ожидаемое значение в пределах 1%.
- На Фазе 6 — записать видео полного прогона миссии и сравнить таймлайн с реальным (T+8:30 MECO, T+~24h TLI, T+~5d perilune, T+~10d splashdown).

---

## 9. Принятые дизайн-решения

| # | Вопрос | Решение |
|---|---|---|
| 1 | Стилистика HUD | **Современный минимализм с акцентами NASA**: тёмный фон, синий NASA `#0B3D91`, оранжевый SLS `#FC3D21`, Inter / Roboto Mono. Без CRT-эффектов |
| 2 | Уровни сложности | **Два режима: `Story` и `Realistic`**. Story — широкие окна допусков, подсказки, без Game Over по случайным авариям. Realistic — реальные допуски и события |
| 3 | Локализация | **RU + EN с первого дня**. Реализация: `i18n.rs` + `assets/i18n/{ru,en}.ron`, переключение в главном меню |
| 4 | Сохранения | **Автосейв 1 слот после каждого этапа**. RON-файл в `%APPDATA%/artemis/save.ron` (Win) / `~/.config/artemis/save.ron` (Linux). Сохраняем `MissionStage`, топливо, delta-v TLI, состояние систем |
| 5 | Режимы камеры | **4 режима: `Cockpit` / `Chase` / `External` / `Free`**. Переключение F1–F4. Free используется для скриншотов и кат-сцен |
| 6 | Экипаж в кадре | **От первого лица, без моделей экипажа**. Экипаж присутствует в радиопереговорах (звуки `referens/artemis-ii-*.mp3`) |

---

## 10. Источники и связанные документы

- `GAME_PROMPT.md` — исходный ТЗ-промпт со стеком, ассетами, физикой, HUD
- `artemis2_game_reference.md` — справочник миссии (точные числа, ссылки на ассеты)
- [Bevy 0.18 docs](https://docs.rs/bevy/0.18.1/bevy/)
- [bevy_egui 0.39 README](https://github.com/vladbat00/bevy_egui)
- [bevy_rapier docs](https://github.com/dimforge/bevy_rapier)
- [NASA Artemis II Official](https://www.nasa.gov/mission/artemis-ii/)
