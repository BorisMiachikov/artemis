# Artemis II — Полёт к Луне

3D-симулятор полёта на Луну по реальной миссии NASA **Artemis II** (1–11 апреля 2026).
Проект на Rust + [Bevy](https://bevyengine.org/) 0.18.

> **Статус:** Фаза 0 — Подготовка ✅
> Полный план развития — в [`roadmap.md`](./roadmap.md)

## О миссии

- Ракета SLS Block 1, корабль Orion «Integrity»
- Экипаж: Reid Wiseman, Victor Glover, Christina Koch, Jeremy Hansen
- Старт: LC-39B, Космический центр Кеннеди, 1 апреля 2026, 18:35 EDT
- Рекордное удаление от Земли: 252 760 миль (406 840 км)
- Ближайший подлёт к Луне: 6 556 км
- Приводнение: Тихий океан, у побережья Сан-Диего

Подробный справочник — в [`artemis2_game_reference.md`](./artemis2_game_reference.md).

## Стек

| Крейт | Версия | Назначение |
|---|---|---|
| `bevy` | 0.18 | Движок (ECS, рендер, ассеты, аудио, GLTF) |
| `bevy_egui` | 0.39 | HUD, телеметрия, меню |
| `bevy_rapier3d` | 0.33 | Физика для входа в атмосферу и приводнения |
| `bevy_asset_loader` | 0.26 | Предзагрузка ассетов с состояниями |
| `bevy_panorbit_camera` | 0.34 | Внешний обзор сцены |
| `rand` | 0.10 | Случайные события (солнечная вспышка, микрометеориты) |

**Rust:** stable (1.91+), edition 2024.
**Платформы:** Windows (основная), Linux.

## Запуск

```bash
# Сборка и запуск (debug, быстрая компиляция)
cargo run

# Релизная сборка (медленнее компилируется, быстрее в работе)
cargo run --release
```

## Структура проекта

```
artemis/
├── Cargo.toml              # Манифест с зависимостями
├── rust-toolchain.toml     # Фиксация stable-toolchain
├── roadmap.md              # План разработки (6 фаз, 3–6 мес)
├── GAME_PROMPT.md          # Исходный ТЗ-промпт
├── artemis2_game_reference.md   # Справочник миссии
├── assets/
│   ├── models/             # GLB-модели (SLS, Orion, Earth, Moon, Astronaut, ...)
│   └── sounds/
│       ├── endless-sky/    # Звуки запуска и космоса (GPL v3)
│       └── nasa-real/      # Реальные переговоры NASA Artemis I/II
├── src/
│   └── main.rs             # Точка входа (на Фазе 0 — пустое окно Bevy)
└── referens/               # Референс-материалы (НЕ попадают в git)
```

Полная целевая структура `src/` — в [`roadmap.md`](./roadmap.md#4-структура-проекта).

## Лицензии

- **Код проекта:** MIT OR Apache-2.0
- **3D-модели NASA:** public domain (NASA media policy)
- **Звуки Endless Sky** (`assets/sounds/endless-sky/`): GPL v3
- **Аудио NASA Artemis I/II** (`assets/sounds/nasa-real/`): public domain
