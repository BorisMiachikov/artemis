use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;

use crate::assets::GameAssets;
use crate::config::SlsParams;
use crate::physics::rocket;
use crate::states::MissionStage;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(MissionStage::Launch), setup_launch_scene);
}

fn setup_launch_scene(mut commands: Commands, assets: Res<GameAssets>, params: Res<SlsParams>) {
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

    // SLS на стартовом столе. Физические компоненты Rocket/FlightDynamics управляют
    // высотой и тангажом, а Transform ракеты синхронизируется в physics::rocket.
    let (rocket, dynamics) = rocket::initial_components(&params);
    commands.spawn((
        SceneRoot(assets.sls.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0),
        rocket,
        dynamics,
        DespawnOnExit(MissionStage::Launch),
    ));

    info!(
        "stages/launch: spawn SLS, mass={:.0} kg, thrust={:.0} kN",
        params.mass_total_kg, params.thrust_total_kn
    );
}
