# 🚀 Roadmap — «Артемида 2: Полёт к Луне»

> **Версия:** 1.2
> **Дата:** 2026-04-27
> **Статус:** Фаза 0 ✅ · Фаза 1 ✅ · Фаза 2 — следующая
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

### Фаза 2 — HUD, States, i18n, сейвы (1–2 недели)

- [ ] `bevy_egui`: окно с T+ таймером и заглушками (скорость, высота, тяга, топливо). Стиль — современный минимализм: тёмный фон, акценты NASA Blue (#0B3D91) + SLS Orange (#FC3D21), шрифт Inter / Roboto Mono
- [ ] `MissionStage` enum через `States`, `StateScoped` для очистки сцены при переходе
- [ ] Минимальные переходы Prelaunch → Launch по нажатию `Space`
- [ ] `audio.rs`: фон `machinery.mp3` на Prelaunch, `human launch.wav` на старте Launch
- [ ] `events.rs`: `MissionEvent` enum + базовый писатель/читатель
- [ ] `i18n.rs`: ресурс `Lang { Ru, En }`, словари в `assets/i18n/{ru,en}.ron`, макрос `t!(key)`. Все строки HUD сразу через ключи
- [ ] `save.rs`: каркас сохранения — `SaveSlot { stage, fuel_kg, tli_delta_v, systems_state }` сериализуется в RON в `%APPDATA%/artemis/save.ron`. Hook на `OnEnter` каждого `MissionStage`

**DoD:** переход между двумя стейтами с правильной очисткой сцены, HUD каждый кадр обновляется в выбранной локали, в меню есть переключатель RU/EN, после перехода стейта на диске лежит обновлённый `save.ron`.

---

### Фаза 3 — Физика запуска: этапы 0–1 (2–4 недели)

**Физика:**
- [ ] `config.rs`: `SLS_PARAMS` (масса 2 608 000 кг, тяга 39 144 кН, Isp RS-25 = 453 с, Isp SRB = 269 с, длительности 126 с / 510 с) + `Difficulty { Story, Realistic }` с коэффициентами окон допусков (`pitch_tolerance_deg`, `tli_burn_window_sec`, `reentry_angle_window_deg`)
- [ ] `physics/rocket.rs`: компоненты `Rocket { thrust_kn, fuel_kg, stage, mass_kg }`, `FlightDynamics { velocity, altitude_km, pitch_deg, g_load }`
- [ ] Системы: применение тяги (F = Isp × g₀ × ṁ), уменьшение массы по расходу
- [ ] `physics/orbital.rs`: гравитация `g = GM/r²` (GM_earth = 3.986×10¹⁴)

**Этап 0 (Prelaunch):**
- [ ] Кат-сцена с `Gantry.glb` + `Crawler.glb`
- [ ] Мини-UI «проверка систем» (10 чекбоксов перед стартом)

**Этап 1 (Launch):**
- [ ] SRB горят 126 с (32 МН), сброс по событию `MissionEvent::SrbSep`
- [ ] RS-25 продолжают до T+8:30, событие `MecoEvent`
- [ ] Управление тангажом по A/D, fail-state при отклонении > N° (число подобрать)
- [ ] HUD заполнен реальными значениями
- [ ] Звуки: `human launch.wav` → `takeoff.wav` → `afterburner~.wav`

**DoD:** ракета взлетает по реалистичному профилю, выходит на ~200 км за ~8.5 мин, SRB сбрасываются вовремя, экран проигрыша срабатывает при отклонении.

---

### Фаза 4 — Орбита и TLI: этапы 2–3 (2–4 недели)

**Этап 2 (Orbit):**
- [ ] Фоновое движение по круговой LEO (упрощённо — постоянная угловая скорость)
- [ ] Реализация `CameraMode::Cockpit` — вид от первого лица из кабины Orion (без моделей экипажа), `CameraMode::Chase` для красивых кадров
- [ ] Mini-game: 12 кликабельных панелей проверки систем (egui), таймер. На Story — широкое окно времени, на Realistic — реальное
- [ ] Фон `rulei space.mp3`, эффект `scan~.wav` при кликах

**Этап 3 (TLI):**
- [ ] Включение ICPS — окно выбора момента и длительности горения (~360 с ±10%)
- [ ] Точность входа определяет delta-v, которое запоминается в ресурсе и влияет на Фазу 5
- [ ] Звуки: `hyperdrive in.wav` → `hyperdrive.wav` → `hyperdrive out.wav`

**DoD:** проходимый путь Orbit → конец TLI; ошибка в TLI меняет точку перицентра у Луны на следующих фазах.

---

### Фаза 5 — Перелёт и облёт Луны: этапы 4–5 (2–4 недели)

**Этап 4 (Transit):**
- [ ] Свободный полёт с двумя точками притяжения (Земля + Луна)
- [ ] Коррекции курса через AJ10 ESM, расход топлива
- [ ] Случайные события (rand): солнечная вспышка → радиация, микрометеорит → деградация системы
- [ ] HUD: расстояние до Земли/Луны, CO₂ в кабине, температура, уровень радиации

**Этап 5 (Lunar Flyby):**
- [ ] Камера показывает Луну (`our_moon.glb` из `referens/`)
- [ ] Минимальное сближение 6 556 км — рассчитывается из delta-v TLI
- [ ] Гравитационный ассист — скорость растёт, нужно выровнять траекторию возврата
- [ ] Звук `jump drive.wav` в момент перицентра

**DoD:** проходимый путь от выхода с TLI до возврата на траекторию к Земле; точность TLI отражена на расстоянии облёта.

---

### Фаза 6 — Возврат, посадка, полировка (2–4 недели)

**Этап 6 (Reentry):**
- [ ] Подключение `bevy_rapier3d` — коллизии Orion с поверхностью океана
- [ ] Окно угла входа 6.0–6.5°, отклонение → game over (перегрев или рикошет)
- [ ] Тепловой эффект на HUD (подогрев индикатора, шум камеры)
- [ ] Раскрытие 11 парашютов: анимация Bevy + резкое снижение скорости
- [ ] Splashdown: `landing.wav`, кадр Сан-Диего

**Полировка:**
- [ ] Главное меню, пауза, экран победы (`ui/mission.rs`)
- [ ] Замеры производительности
- [ ] Оптимизация загрузки крупных GLB: LOD-версии для `our_moon.glb` (54 МБ) и `earth.glb` (45 МБ) или процедурная сфера с текстурой

**DoD:** полный прогон миссии Prelaunch → Splashdown за ≤ 60 минут реального времени; стабильно 60 FPS.

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
