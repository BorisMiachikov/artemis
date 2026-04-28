use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;

use crate::assets::GameAssets;
use crate::states::MissionStage;

/// Кинематический ярлык: вращаемся вокруг центра сцены с этой угловой скоростью (рад/с).
const EARTH_SPIN_RATE_RAD_S: f32 = 0.05;
const ORION_ORBIT_RATE_RAD_S: f32 = 0.10;

#[derive(Component)]
struct EarthBody;

#[derive(Component)]
struct OrionInOrbit;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(MissionStage::Orbit), setup_orbit_scene)
        .add_systems(
            Update,
            (rotate_earth, orbit_orion).run_if(in_state(MissionStage::Orbit)),
        );
}

fn setup_orbit_scene(mut commands: Commands, assets: Res<GameAssets>) {
    // Тёплое солнце над сценой.
    commands.spawn((
        DirectionalLight {
            illuminance: 35_000.0,
            shadows_enabled: false,
            color: Color::srgb(1.0, 0.97, 0.92),
            ..default()
        },
        Transform::from_xyz(60.0, 30.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
        DespawnOnExit(MissionStage::Orbit),
    ));

    // Земля в центре сцены. Радиус GLB ≈ 1; масштабируем до визуально удобного размера.
    commands.spawn((
        SceneRoot(assets.earth.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(40.0)),
        EarthBody,
        DespawnOnExit(MissionStage::Orbit),
    ));

    // Orion на «низкой орбите» — кружит вокруг центра. Сама модель маленькая,
    // вынесем её на радиус 55, что визуально читается как высокая LEO.
    commands.spawn((
        SceneRoot(assets.orion.clone()),
        Transform::from_xyz(55.0, 0.0, 0.0).with_scale(Vec3::splat(2.0)),
        OrionInOrbit,
        DespawnOnExit(MissionStage::Orbit),
    ));

    // Фоновая музыка orbit.
    commands.spawn((
        AudioPlayer(assets.ambient_orbit.clone()),
        PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(0.25)),
        DespawnOnExit(MissionStage::Orbit),
    ));

    info!("stages/orbit: setup сцены LEO (Earth + Orion)");
}

fn rotate_earth(time: Res<Time>, mut q: Query<&mut Transform, With<EarthBody>>) {
    for mut tr in &mut q {
        tr.rotate_y(EARTH_SPIN_RATE_RAD_S * time.delta_secs());
    }
}

fn orbit_orion(time: Res<Time>, mut q: Query<&mut Transform, With<OrionInOrbit>>) {
    let dt = time.delta_secs();
    for mut tr in &mut q {
        let angle = ORION_ORBIT_RATE_RAD_S * dt;
        let (sa, ca) = angle.sin_cos();
        let (x, z) = (tr.translation.x, tr.translation.z);
        tr.translation.x = x * ca - z * sa;
        tr.translation.z = x * sa + z * ca;
        // Корабль смотрит против вектора движения.
        let look_dir = Vec3::new(-tr.translation.z, 0.0, tr.translation.x).normalize_or_zero();
        if look_dir.length_squared() > 0.0 {
            tr.look_to(look_dir, Vec3::Y);
        }
    }
}
