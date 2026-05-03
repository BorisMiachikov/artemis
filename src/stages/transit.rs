use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use rand::RngExt;

use crate::assets::GameAssets;
use crate::camera::PlayerVehicle;
use crate::config::{
    IcpsParams, TimeScale, TliResult, TransitOutcome, UNSTABLE_ORBIT_TRAJ_PENALTY,
};
use crate::lod::{DistanceLod, LodMaterials, LodSphere};
use crate::events::MissionEvent;
use crate::physics::rocket::OrbitInsertion;
use crate::states::MissionStage;

/// Реальный перелёт: ~3 суток. В игре при ×1: 120 сек. При ×20: 6 сек.
const TRANSIT_GAME_DURATION_S: f32 = 120.0;
const DIST_EARTH_START_KM: f64 = 6_571.0;
const DIST_MOON_FULL_KM: f64 = 384_400.0;
const ARRIVAL_THRESHOLD_KM: f64 = 50_000.0;
const MCC_FUEL_INITIAL_KG: f32 = 200.0;
const MCC_FUEL_COST_KG: f32 = 50.0;
const EVENT_INTERVAL_S: f32 = 35.0;
const EVENT_DISPLAY_S: f32 = 5.0;

#[derive(Resource, Default, Debug)]
pub struct TransitState {
    pub dist_earth_km: f64,
    pub dist_moon_km: f64,
    pub mcc_fuel_kg: f32,
    pub co2_ppm: f32,
    pub radiation_msv: f32,
    pub elapsed_s: f32,
    pub event_timer: f32,
    pub event_msg: Option<&'static str>,
    pub event_display_s: f32,
}

#[derive(Component)]
struct TransitEarth;

#[derive(Component)]
struct TransitMoon;

#[derive(Component)]
struct FloatingAstronaut {
    base_y: f32,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<TransitState>()
        .add_systems(OnEnter(MissionStage::Transit), setup_transit)
        .add_systems(
            Update,
            (
                tick_transit,
                handle_mcc_input,
                trigger_random_events,
                animate_scene_bodies,
                float_astronaut,
                check_arrival,
            )
                .chain()
                .run_if(in_state(MissionStage::Transit)),
        );
}

#[allow(clippy::too_many_arguments)]
fn setup_transit(
    mut commands: Commands,
    assets: Res<GameAssets>,
    tli: Res<TliResult>,
    icps: Res<IcpsParams>,
    insertion: Res<OrbitInsertion>,
    mut state: ResMut<TransitState>,
    mut outcome: ResMut<TransitOutcome>,
    lod_mats: Res<LodMaterials>,
) {
    // Начальная ошибка траектории из точности TLI: 100% → error=0, 0% → error=1.0
    let accuracy = tli.accuracy_pct(icps.target_delta_v_ms);
    let mut init_error = (1.0 - accuracy / 100.0).clamp(0.0, 1.0);
    // Штраф за выход на нестабильную орбиту: ICPS физически отработал
    // нормально, проблема — в плохой стартовой орбите, что мапится в ошибку
    // траектории к Луне. Игрок может частично компенсировать через MCC.
    let unstable_penalty = if insertion.unstable {
        UNSTABLE_ORBIT_TRAJ_PENALTY
    } else {
        0.0
    };
    init_error = (init_error + unstable_penalty).clamp(0.0, 1.0);

    *state = TransitState {
        dist_earth_km: DIST_EARTH_START_KM,
        dist_moon_km: DIST_MOON_FULL_KM - DIST_EARTH_START_KM,
        mcc_fuel_kg: MCC_FUEL_INITIAL_KG,
        co2_ppm: 400.0,
        radiation_msv: 0.0,
        elapsed_s: 0.0,
        event_timer: EVENT_INTERVAL_S,
        event_msg: None,
        event_display_s: 0.0,
    };
    *outcome = TransitOutcome {
        trajectory_error: init_error,
        mcc_corrections: 0,
    };

    // Освещение — холодный межпланетный свет
    commands.spawn((
        DirectionalLight {
            illuminance: 15_000.0,
            color: Color::srgb(0.95, 0.97, 1.0),
            ..default()
        },
        Transform::from_xyz(50.0, 30.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        DespawnOnExit(MissionStage::Transit),
    ));

    // Земля — маленькая, сзади корабля; масштаб уменьшается по мере удаления
    let earth_lo = commands.spawn((
        Mesh3d(lod_mats.lo_mesh.clone()),
        MeshMaterial3d(lod_mats.earth_mat.clone()),
        Transform::from_xyz(0.0, 0.0, 120.0).with_scale(Vec3::splat(6.0)),
        LodSphere,
        DespawnOnExit(MissionStage::Transit),
    )).id();
    commands.spawn((
        SceneRoot(assets.earth.clone()),
        Transform::from_xyz(0.0, 0.0, 120.0).with_scale(Vec3::splat(6.0)),
        TransitEarth,
        DistanceLod { lo_entity: earth_lo, min_apparent: 0.08 },
        DespawnOnExit(MissionStage::Transit),
    ));

    // Луна — впереди, вырастает по мере сближения
    let moon_lo = commands.spawn((
        Mesh3d(lod_mats.lo_mesh.clone()),
        MeshMaterial3d(lod_mats.moon_mat.clone()),
        Transform::from_xyz(0.0, 0.0, -350.0).with_scale(Vec3::splat(3.0)),
        LodSphere,
        DespawnOnExit(MissionStage::Transit),
    )).id();
    commands.spawn((
        SceneRoot(assets.moon.clone()),
        Transform::from_xyz(0.0, 0.0, -350.0).with_scale(Vec3::splat(3.0)),
        TransitMoon,
        DistanceLod { lo_entity: moon_lo, min_apparent: 0.08 },
        DespawnOnExit(MissionStage::Transit),
    ));

    // Orion в центре
    commands.spawn((
        SceneRoot(assets.orion.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(3.0)),
        PlayerVehicle,
        DespawnOnExit(MissionStage::Transit),
    ));

    // Астронавт в невесомости рядом с Orion
    commands.spawn((
        SceneRoot(assets.astronaut.clone()),
        Transform::from_xyz(3.5, 0.0, 0.0).with_scale(Vec3::splat(0.5)),
        FloatingAstronaut { base_y: 0.0 },
        DespawnOnExit(MissionStage::Transit),
    ));

    // Фоновая музыка
    commands.spawn((
        AudioPlayer(assets.ambient_orbit.clone()),
        PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(0.18)),
        DespawnOnExit(MissionStage::Transit),
    ));

    info!(
        "stages/transit: старт, ошибка траектории TLI={:.1}% (unstable orbit penalty +{:.0}%)",
        init_error * 100.0,
        unstable_penalty * 100.0,
    );
}

fn tick_transit(
    time: Res<Time>,
    timescale: Res<TimeScale>,
    mut state: ResMut<TransitState>,
) {
    let dt = time.delta_secs() * timescale.multiplier;
    if dt <= 0.0 {
        return;
    }

    state.elapsed_s += dt;

    // Дистанция: линейная интерполяция за TRANSIT_GAME_DURATION_S
    let progress = (state.elapsed_s / TRANSIT_GAME_DURATION_S).min(1.0) as f64;
    let dist_end_km = DIST_MOON_FULL_KM - ARRIVAL_THRESHOLD_KM * 0.9;
    state.dist_earth_km = DIST_EARTH_START_KM + (dist_end_km - DIST_EARTH_START_KM) * progress;
    state.dist_moon_km = DIST_MOON_FULL_KM - state.dist_earth_km;

    // CO₂ медленно накапливается (жизнеобеспечение работает)
    state.co2_ppm = 400.0 + state.elapsed_s * 0.3;

    // Фоновая радиация
    state.radiation_msv += dt * 0.01;

    // Таймер отображения события
    if state.event_display_s > 0.0 {
        state.event_display_s -= dt;
        if state.event_display_s <= 0.0 {
            state.event_msg = None;
        }
    }

    // Таймер следующего события
    state.event_timer -= dt;
}

fn handle_mcc_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<TransitState>,
    mut outcome: ResMut<TransitOutcome>,
    mut events: MessageWriter<MissionEvent>,
) {
    if state.mcc_fuel_kg < MCC_FUEL_COST_KG {
        return;
    }

    let press = if keys.just_pressed(KeyCode::KeyQ) {
        -0.12 // улучшить траекторию
    } else if keys.just_pressed(KeyCode::KeyE) {
        0.12 // ухудшить
    } else {
        return;
    };

    state.mcc_fuel_kg -= MCC_FUEL_COST_KG;
    outcome.trajectory_error = (outcome.trajectory_error + press).clamp(0.0, 1.0);
    outcome.mcc_corrections += 1;
    events.write(MissionEvent::Meco); // используем как «импульс» (звук уже есть в audio)
    info!(
        "stages/transit: MCC #{}, ошибка траектории → {:.2}, топливо → {:.0} кг",
        outcome.mcc_corrections, outcome.trajectory_error, state.mcc_fuel_kg
    );
}

fn trigger_random_events(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut state: ResMut<TransitState>,
) {
    if state.event_timer > 0.0 {
        return;
    }
    state.event_timer = EVENT_INTERVAL_S;

    let mut rng = rand::rng();
    let roll: f32 = rng.random();

    if roll < 0.30 {
        // Солнечная вспышка
        state.radiation_msv += 12.0;
        state.event_msg = Some("transit.event.solar_flare");
        state.event_display_s = EVENT_DISPLAY_S;
        commands.spawn((AudioPlayer(assets.alarm.clone()), PlaybackSettings::ONCE));
        warn!("stages/transit: солнечная вспышка! радиация +12 мЗв");
    } else if roll < 0.50 {
        // Микрометеорит
        state.event_msg = Some("transit.event.micrometeorite");
        state.event_display_s = EVENT_DISPLAY_S;
        commands.spawn((AudioPlayer(assets.alarm.clone()), PlaybackSettings::ONCE));
        warn!("stages/transit: микрометеорит! предупреждение систем");
    }
}

/// Обновляет масштаб/позицию Земли и Луны по мере изменения дистанций.
fn animate_scene_bodies(
    state: Res<TransitState>,
    mut earth_q: Query<&mut Transform, (With<TransitEarth>, Without<TransitMoon>)>,
    mut moon_q: Query<&mut Transform, (With<TransitMoon>, Without<TransitEarth>)>,
) {
    // Земля: видимый размер ~пропорционален 1 / dist_earth
    let earth_scale = (6_571.0 / state.dist_earth_km as f32).clamp(0.3, 6.0);
    for mut tr in &mut earth_q {
        tr.scale = Vec3::splat(earth_scale);
    }

    // Луна: видимый размер растёт по мере приближения
    let moon_scale = (6_000.0 / state.dist_moon_km as f32).clamp(0.5, 40.0);
    for mut tr in &mut moon_q {
        tr.scale = Vec3::splat(moon_scale);
    }
}

fn float_astronaut(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &FloatingAstronaut)>,
) {
    let t = time.elapsed_secs();
    let dt = time.delta_secs();
    for (mut tr, fa) in &mut query {
        tr.translation.y = fa.base_y + 0.3 * (t * 0.4).sin();
        tr.rotate_local_y(0.008 * dt);
    }
}

fn check_arrival(
    state: Res<TransitState>,
    mut next_stage: ResMut<NextState<MissionStage>>,
) {
    if state.dist_moon_km <= ARRIVAL_THRESHOLD_KM {
        info!(
            "stages/transit: прибытие в зону Луны, dist_moon={:.0} км",
            state.dist_moon_km
        );
        next_stage.set(MissionStage::LunarFlyby);
    }
}
