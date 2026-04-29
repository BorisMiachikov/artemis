use bevy::asset::AssetPlugin;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

mod assets;
mod audio;
mod camera;
mod config;
mod events;
mod i18n;
mod input;
mod lod;
mod particles;
mod physics;
mod save;
mod stages;
mod starfield;
mod states;
mod ui;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Artemis II — Полёт к Луне".into(),
                        resolution: (1280u32, 720u32).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: resolve_assets_path(),
                    ..default()
                }),
        )
        .add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            states::plugin,
            config::plugin,
            assets::plugin,
            events::plugin,
            input::plugin,
            camera::plugin,
            audio::plugin,
            i18n::plugin,
            save::plugin,
            physics::plugin,
            stages::plugin,
            ui::plugin,
            lod::plugin,
            particles::plugin,
            starfield::plugin,
        ))
        .run();
}

/// Находит папку `assets/`:
/// 1. Через `CARGO_MANIFEST_DIR` (cargo run в dev-режиме).
/// 2. Рядом с исполняемым файлом (production).
/// 3. В текущей рабочей директории (запуск exe из корня проекта).
/// 4. Относительный fallback "assets".
fn resolve_assets_path() -> String {
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        return format!("{manifest}/assets");
    }

    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
    {
        let candidate = parent.join("assets");
        if candidate.exists() {
            return candidate.to_string_lossy().into_owned();
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        let candidate = cwd.join("assets");
        if candidate.exists() {
            return candidate.to_string_lossy().into_owned();
        }
    }

    "assets".into()
}
