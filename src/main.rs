use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

mod assets;
mod audio;
mod camera;
mod config;
mod events;
mod i18n;
mod input;
mod physics;
mod save;
mod stages;
mod states;
mod ui;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Artemis II — Полёт к Луне".into(),
                resolution: (1280u32, 720u32).into(),
                ..default()
            }),
            ..default()
        }))
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
        ))
        .run();
}
