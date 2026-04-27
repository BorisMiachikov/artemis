use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;

use crate::assets::GameAssets;
use crate::states::MissionStage;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(MissionStage::Launch), setup_launch_scene);
}

fn setup_launch_scene(mut commands: Commands, assets: Res<GameAssets>) {
    // Свет старта (тёплый, как закат во Флориде).
    commands.spawn((
        DirectionalLight {
            illuminance: 60_000.0,
            shadows_enabled: true,
            color: Color::srgb(1.0, 0.92, 0.85),
            ..default()
        },
        Transform::from_xyz(8.0, 12.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
        DespawnOnExit(MissionStage::Launch),
    ));

    // Тестовая ракета (на Фазе 3 заменим на SLS + физику тяги).
    commands.spawn((
        SceneRoot(assets.saturn_v.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0),
        DespawnOnExit(MissionStage::Launch),
    ));

    info!("stages/launch: сцена старта построена");
}
