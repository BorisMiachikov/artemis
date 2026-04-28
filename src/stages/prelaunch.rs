use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;

use crate::assets::GameAssets;
use crate::states::MissionStage;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(MissionStage::Prelaunch), setup_scene);
}

fn setup_scene(mut commands: Commands, assets: Res<GameAssets>) {
    // Солнце: яркий направленный свет с тенями.
    commands.spawn((
        DirectionalLight {
            illuminance: 50_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(8.0, 12.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
        DespawnOnExit(MissionStage::Prelaunch),
    ));

    // Crawler-transporter — внизу, у поверхности.
    commands.spawn((
        SceneRoot(assets.crawler.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0),
        DespawnOnExit(MissionStage::Prelaunch),
    ));

    // Стартовая башня — на крауле.
    commands.spawn((
        SceneRoot(assets.gantry.clone()),
        Transform::from_xyz(0.0, 4.0, 0.0),
        DespawnOnExit(MissionStage::Prelaunch),
    ));

    // SLS — на стартовом столе рядом с башней.
    commands.spawn((
        SceneRoot(assets.sls.clone()),
        Transform::from_xyz(-3.0, 4.0, 0.0),
        DespawnOnExit(MissionStage::Prelaunch),
    ));
}
