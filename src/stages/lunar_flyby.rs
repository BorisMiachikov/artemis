use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;

use crate::assets::GameAssets;
use crate::camera::{CameraMode, GameCamera, PlayerVehicle};
use crate::config::{FlybyResult, KM_TO_UNITS, MOON_RADIUS_KM, TimeScale, TransitOutcome};
use crate::events::MissionEvent;
use crate::lod::{DistanceLod, LodMaterials, LodSphere};
use crate::physics::trajectory::{self, TrajectorySim};
use crate::states::MissionStage;

/// Реальный перицентр Artemis II: 6 556 км от центра Луны.
const PERILUNE_TARGET_KM: f32 = 6_556.0;
/// При ошибке 0% — точно 6 556 км; при 100% — до 13 112 км.
const PERILUNE_ERROR_RANGE_KM: f32 = 6_556.0;
/// Дистанция (км от Луны) для перехода к возврату после перицентра.
const DEPARTURE_END_KM: f32 = 25_000.0;

/// Фиксированная позиция Moon-backdrop в сцене.
const MOON_SCENE_POS: Vec3 = Vec3::new(0.0, -20.0, -80.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FlybyPhase {
    #[default]
    Approach,
    Perilune,
    Departure,
}

#[derive(Resource, Default, Debug)]
pub struct FlybyState {
    pub phase: FlybyPhase,
    pub approach_km: f32,
    pub perilune_km: f32,
    pub speed_kms: f32,
    pub perilune_fired: bool,
    pub departure_started: bool,
}

#[derive(Component)]
struct FlybyMoon;

/// Маркер Orion в стадии LunarFlyby — отдельно от PlayerVehicle, чтобы
/// не конфликтовать с Query-фильтрами Transit (sync_transforms там тоже активна).
#[derive(Component)]
struct FlybyOrion;

pub fn plugin(app: &mut App) {
    app.init_resource::<FlybyState>()
        .add_systems(
            OnEnter(MissionStage::LunarFlyby),
            (setup_flyby, reset_flyby_camera).chain(),
        )
        .add_systems(
            Update,
            (tick_flyby, sync_flyby_transforms, check_flyby_events, check_departure)
                .chain()
                .run_if(in_state(MissionStage::LunarFlyby)),
        );
}

#[allow(clippy::too_many_arguments)]
fn setup_flyby(
    mut commands: Commands,
    assets: Res<GameAssets>,
    outcome: Res<TransitOutcome>,
    sim: Res<TrajectorySim>,
    mut state: ResMut<FlybyState>,
    mut result: ResMut<FlybyResult>,
    mut timescale: ResMut<TimeScale>,
    lod_mats: Res<LodMaterials>,
) {
    // Перилуний: приоритет — predicted_perilune_km (forward-prop 3 дня из Transit),
    // затем closest_approach_km, затем fallback через trajectory_error.
    // closest_approach_km на входе в SOI ≈ 66 100 км (не реальный перилуний!),
    // поэтому predicted_perilune_km является корректным источником.
    let perilune = if sim.predicted_perilune_km < 1_000_000.0 {
        sim.predicted_perilune_km.clamp(MOON_RADIUS_KM, 50_000.0) as f32
    } else if sim.closest_approach_km < 1_000_000.0 {
        sim.closest_approach_km.clamp(MOON_RADIUS_KM, 50_000.0) as f32
    } else {
        PERILUNE_TARGET_KM + outcome.trajectory_error * PERILUNE_ERROR_RANGE_KM
    };

    let init_dist = (sim.orion_pos_km - sim.moon_pos_km).length() as f32;
    let init_speed = sim.orion_vel_kms.length() as f32;

    *state = FlybyState {
        phase: FlybyPhase::Approach,
        approach_km: init_dist,
        perilune_km: perilune,
        speed_kms: init_speed,
        perilune_fired: false,
        departure_started: false,
    };
    result.perilune_km = perilune;

    // Сброс warp — LunarFlyby идёт в реальном времени (пролёт очень короткий)
    timescale.multiplier = 1.0;

    // Свет — более яркий (близко к Луне)
    commands.spawn((
        DirectionalLight {
            illuminance: 25_000.0,
            color: Color::srgb(0.98, 0.96, 0.92),
            ..default()
        },
        Transform::from_xyz(60.0, 40.0, -20.0).looking_at(Vec3::ZERO, Vec3::Y),
        DespawnOnExit(MissionStage::LunarFlyby),
    ));

    // Луна-backdrop занимает весь задний план (декоративный масштаб ×60)
    let moon_lo = commands
        .spawn((
            Mesh3d(lod_mats.lo_mesh.clone()),
            MeshMaterial3d(lod_mats.moon_mat.clone()),
            Transform::from_translation(MOON_SCENE_POS).with_scale(Vec3::splat(60.0)),
            LodSphere,
            Visibility::Hidden,
            DespawnOnExit(MissionStage::LunarFlyby),
        ))
        .id();
    commands.spawn((
        SceneRoot(assets.moon.clone()),
        Transform::from_translation(MOON_SCENE_POS).with_scale(Vec3::splat(60.0)),
        FlybyMoon,
        DistanceLod {
            lo_entity: moon_lo,
            min_apparent: 0.08,
        },
        DespawnOnExit(MissionStage::LunarFlyby),
    ));

    // Orion — физическая позиция относительно Moon-backdrop
    let rel_km = (sim.orion_pos_km - sim.moon_pos_km).as_vec3();
    let orion_scene_pos = MOON_SCENE_POS + rel_km * KM_TO_UNITS;
    commands.spawn((
        SceneRoot(assets.orion.clone()),
        Transform::from_translation(orion_scene_pos).with_scale(Vec3::splat(3.0)),
        PlayerVehicle,
        FlybyOrion,
        DespawnOnExit(MissionStage::LunarFlyby),
    ));

    // Фоновая музыка
    commands.spawn((
        AudioPlayer(assets.ambient_orbit.clone()),
        PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(0.15)),
        DespawnOnExit(MissionStage::LunarFlyby),
    ));

    info!(
        "stages/lunar_flyby: перицентр={:.0} км (predicted={:.0} км, closest={:.0} км, ошибка={:.2})",
        perilune,
        sim.predicted_perilune_km,
        sim.closest_approach_km,
        outcome.trajectory_error,
    );
}

fn reset_flyby_camera(
    mut mode: ResMut<CameraMode>,
    mut cameras: Query<&mut Transform, With<GameCamera>>,
) {
    *mode = CameraMode::External;
    if let Ok(mut tr) = cameras.single_mut() {
        *tr = Transform::from_xyz(0.0, 40.0, 80.0)
            .looking_at(Vec3::new(0.0, -20.0, -80.0), Vec3::Y);
    }
}

fn tick_flyby(
    time: Res<Time>,
    timescale: Res<TimeScale>,
    mut sim: ResMut<TrajectorySim>,
    mut state: ResMut<FlybyState>,
) {
    let dt_sim = (time.delta_secs() * timescale.multiplier) as f64;
    if dt_sim > 0.0 {
        trajectory::integrate(&mut sim, dt_sim);
    }

    let dist = (sim.orion_pos_km - sim.moon_pos_km).length() as f32;
    state.approach_km = dist;
    state.speed_kms = sim.orion_vel_kms.length() as f32;

    // Радиальная скорость Orion относительно Луны:
    // отрицательная = сближение, положительная = удаление (перицентр пройден).
    let r_om = sim.orion_pos_km - sim.moon_pos_km;
    let v_om = sim.orion_vel_kms - sim.moon_vel_kms;
    let v_radial = v_om.dot(r_om.normalize_or_zero()) as f32;

    if state.phase == FlybyPhase::Approach && v_radial > 0.0 {
        state.phase = FlybyPhase::Perilune;
    }
}

/// Синхронизирует позицию Orion из физических векторов TrajectorySim.
/// Orion отображается в позиции, смещённой относительно Moon-backdrop.
fn sync_flyby_transforms(
    sim: Res<TrajectorySim>,
    mut orion_q: Query<&mut Transform, With<FlybyOrion>>,
) {
    let rel_km = (sim.orion_pos_km - sim.moon_pos_km).as_vec3();
    let orion_world = MOON_SCENE_POS + rel_km * KM_TO_UNITS;
    for mut tr in &mut orion_q {
        tr.translation = orion_world;
    }
}

fn check_flyby_events(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut state: ResMut<FlybyState>,
    mut events: MessageWriter<MissionEvent>,
    mut local_timer: Local<f32>,
    time: Res<Time>,
) {
    match state.phase {
        FlybyPhase::Perilune if state.perilune_fired && !state.departure_started => {
            *local_timer += time.delta_secs();
            if *local_timer >= 3.0 {
                events.write(MissionEvent::PerilunePassage);
                commands.spawn((
                    AudioPlayer(assets.jump_drive.clone()),
                    PlaybackSettings::ONCE,
                ));
                state.phase = FlybyPhase::Departure;
                state.departure_started = true;
                *local_timer = 0.0;
                info!(
                    "stages/lunar_flyby: перицентр пройден @ {:.0} км, скорость {:.2} км/с",
                    state.perilune_km, state.speed_kms
                );
            }
        }
        FlybyPhase::Perilune if !state.perilune_fired => {
            state.perilune_fired = true;
        }
        _ => {}
    }
}

fn check_departure(
    state: Res<FlybyState>,
    mut next_stage: ResMut<NextState<MissionStage>>,
) {
    if state.phase == FlybyPhase::Departure && state.approach_km >= DEPARTURE_END_KM {
        info!(
            "stages/lunar_flyby: отлёт завершён @ {:.0} км, переход к Reentry",
            state.approach_km
        );
        next_stage.set(MissionStage::Reentry);
    }
}
