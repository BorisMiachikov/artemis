// 105.5° — оптимальный угол фазы Луны: перилуний ~6789 км от центра (alt ~5052 км)
const MOON_PHASE_ANGLE: f64 = 1.841_296_4; // 105.5_f64.to_radians()

use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use rand::RngExt;

use crate::assets::GameAssets;
use crate::camera::{CameraMode, GameCamera, PlayerVehicle};
use crate::config::{
    IcpsParams, KM_TO_UNITS, MCC_FUEL_INITIAL, MOON_ORBIT_R_KM, MU_EARTH_KM3_S2, LEO_R_KM,
    SOI_MOON_KM, TLI_PHYSICS_DV_KMS, TimeScale, TliResult, TransitOutcome, WarpLevel, WARP_LEVELS,
};
use crate::events::MissionEvent;
use crate::input::MccCommand;
use crate::lod::{DistanceLod, LodMaterials, LodSphere};
use crate::physics::rocket::OrbitInsertion;
use crate::physics::trajectory::{self, TrajectorySim};
use crate::states::MissionStage;

const EVENT_INTERVAL_S: f32 = 35.0;
const EVENT_DISPLAY_S: f32 = 5.0;
const TRAIL_PUSH_INTERVAL_S: f64 = 5.0;
const TRAIL_MAX: usize = 500;

/// Non-physics состояние Transit: CO₂, радиация, случайные события.
#[derive(Resource, Default, Debug)]
pub struct TransitState {
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
    app.init_resource::<TransitState>().add_systems(
        OnEnter(MissionStage::Transit),
        (setup_transit, reset_transit_camera).chain(),
    );
    // Физика и ввод — строгий порядок
    app.add_systems(
        Update,
        (
            tick_transit,
            handle_mcc_input,
            sync_transforms,
            trigger_random_events,
            float_astronaut,
        )
            .chain()
            .run_if(in_state(MissionStage::Transit)),
    );
    // Проверки и визуализация — независимы, запускаются после физики
    app.add_systems(
        Update,
        (
            update_trajectory_prediction,
            draw_trajectory_gizmos,
            warp_auto_drop,
            check_lunar_soi,
            check_miss,
        )
            .run_if(in_state(MissionStage::Transit))
            .after(tick_transit),
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
    mut sim: ResMut<TrajectorySim>,
    mut warp: ResMut<WarpLevel>,
    mut timescale: ResMut<TimeScale>,
    lod_mats: Res<LodMaterials>,
) {
    // --- Физические начальные условия ---
    // Круговая скорость на LEO 200 км
    let v_leo_circ = (MU_EARTH_KM3_S2 / LEO_R_KM).sqrt();

    // Масштабируем game-точность TLI в physics-корректное Δv:
    // при 100% → 3.132 km/s (Гомановский переход LEO → лунная орбита).
    let accuracy = if icps.target_delta_v_ms > 0.0 {
        (tli.delta_v_ms / icps.target_delta_v_ms).clamp(0.0, 1.5) as f64
    } else {
        0.0
    };
    let phys_dv = accuracy * TLI_PHYSICS_DV_KMS;

    // Штраф за нестабильную орбиту: -0.15 km/s
    let penalty = if insertion.unstable { 0.15 } else { 0.0 };
    let final_dv = (phys_dv - penalty).max(0.0);

    // Луна на 60° впереди Orion (упреждение для перехвата)
    let moon_v_exact = (MU_EARTH_KM3_S2 / MOON_ORBIT_R_KM).sqrt();

    *sim = TrajectorySim {
        orion_pos_km: bevy::math::DVec3::new(LEO_R_KM, 0.0, 0.0),
        orion_vel_kms: bevy::math::DVec3::new(0.0, 0.0, v_leo_circ + final_dv),
        moon_pos_km: bevy::math::DVec3::new(
            MOON_ORBIT_R_KM * MOON_PHASE_ANGLE.cos(),
            0.0,
            MOON_ORBIT_R_KM * MOON_PHASE_ANGLE.sin(),
        ),
        moon_vel_kms: bevy::math::DVec3::new(
            -moon_v_exact * MOON_PHASE_ANGLE.sin(),
            0.0,
            moon_v_exact * MOON_PHASE_ANGLE.cos(),
        ),
        mcc_fuel_kg: MCC_FUEL_INITIAL,
        ..Default::default()
    };

    // Сброс warp → ×1
    *warp = WarpLevel(0);
    timescale.multiplier = 1.0;

    // Сброс игрового состояния
    *state = TransitState {
        co2_ppm: 400.0,
        radiation_msv: 0.0,
        elapsed_s: 0.0,
        event_timer: EVENT_INTERVAL_S,
        event_msg: None,
        event_display_s: 0.0,
    };
    *outcome = TransitOutcome {
        trajectory_error: 0.0,
        mcc_corrections: 0,
    };

    // --- Сцена ---
    // Направленный свет
    commands.spawn((
        DirectionalLight {
            illuminance: 15_000.0,
            color: Color::srgb(0.95, 0.97, 1.0),
            ..default()
        },
        Transform::from_xyz(50.0, 30.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        DespawnOnExit(MissionStage::Transit),
    ));

    // Земля — фиксирована в начале координат (ECI ref frame)
    let earth_scale = Vec3::splat(6.0);
    let earth_lo = commands
        .spawn((
            Mesh3d(lod_mats.lo_mesh.clone()),
            MeshMaterial3d(lod_mats.earth_mat.clone()),
            Transform::from_translation(Vec3::ZERO).with_scale(earth_scale),
            LodSphere,
            DespawnOnExit(MissionStage::Transit),
        ))
        .id();
    commands.spawn((
        SceneRoot(assets.earth.clone()),
        Transform::from_translation(Vec3::ZERO).with_scale(earth_scale),
        TransitEarth,
        DistanceLod {
            lo_entity: earth_lo,
            min_apparent: 0.08,
        },
        DespawnOnExit(MissionStage::Transit),
    ));

    // Луна — начальная позиция из sim
    let moon_pos = sim.moon_pos_km.as_vec3() * KM_TO_UNITS;
    let moon_scale = Vec3::splat(2.5);
    let moon_lo = commands
        .spawn((
            Mesh3d(lod_mats.lo_mesh.clone()),
            MeshMaterial3d(lod_mats.moon_mat.clone()),
            Transform::from_translation(moon_pos).with_scale(moon_scale),
            LodSphere,
            DespawnOnExit(MissionStage::Transit),
        ))
        .id();
    commands.spawn((
        SceneRoot(assets.moon.clone()),
        Transform::from_translation(moon_pos).with_scale(moon_scale),
        TransitMoon,
        DistanceLod {
            lo_entity: moon_lo,
            min_apparent: 0.08,
        },
        DespawnOnExit(MissionStage::Transit),
    ));

    // Orion — начальная позиция из sim
    let orion_pos = sim.orion_pos_km.as_vec3() * KM_TO_UNITS;
    commands.spawn((
        SceneRoot(assets.orion.clone()),
        Transform::from_translation(orion_pos).with_scale(Vec3::splat(3.0)),
        PlayerVehicle,
        DespawnOnExit(MissionStage::Transit),
    ));

    // Астронавт в невесомости рядом с Orion
    commands.spawn((
        SceneRoot(assets.astronaut.clone()),
        Transform::from_translation(orion_pos + Vec3::new(3.5, 0.0, 0.0))
            .with_scale(Vec3::splat(0.5)),
        FloatingAstronaut { base_y: orion_pos.y },
        DespawnOnExit(MissionStage::Transit),
    ));

    // Фоновая музыка
    commands.spawn((
        AudioPlayer(assets.ambient_orbit.clone()),
        PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(0.18)),
        DespawnOnExit(MissionStage::Transit),
    ));

    info!(
        "stages/transit: старт — Δv={:.3} km/s (accuracy={:.0}%, unstable penalty {:.0}%), warp=×1",
        final_dv,
        accuracy * 100.0,
        penalty * 100.0,
    );
}

fn reset_transit_camera(
    mut mode: ResMut<CameraMode>,
    mut cameras: Query<&mut Transform, With<GameCamera>>,
) {
    *mode = CameraMode::External;
    if let Ok(mut tr) = cameras.single_mut() {
        // Wide3D: камера ~400 ед. по Z+Y, смотрит на точку 60% пути к Луне
        *tr = Transform::from_xyz(40.0, 80.0, 400.0)
            .looking_at(Vec3::new(0.0, 0.0, 150.0), Vec3::Y);
    }
}

fn tick_transit(
    time: Res<Time>,
    timescale: Res<TimeScale>,
    mut sim: ResMut<TrajectorySim>,
    mut state: ResMut<TransitState>,
    mut trail_timer: Local<f64>,
) {
    let dt_sim = (time.delta_secs() * timescale.multiplier) as f64;
    if dt_sim <= 0.0 {
        return;
    }

    trajectory::integrate(&mut sim, dt_sim);
    state.elapsed_s += time.delta_secs() * timescale.multiplier;

    // CO₂ накапливается медленно (жизнеобеспечение работает)
    state.co2_ppm = (400.0 + state.elapsed_s * 0.3).min(2000.0);

    // Фоновая радиация (по реальному sim-времени, не game-времени)
    state.radiation_msv += (dt_sim as f32) * 0.01;

    // Таймер отображения события
    if state.event_display_s > 0.0 {
        state.event_display_s -= time.delta_secs();
        if state.event_display_s <= 0.0 {
            state.event_msg = None;
        }
    }

    // Таймер следующего события (по game-времени, не sim)
    state.event_timer -= time.delta_secs();

    // Trail: пушим каждые TRAIL_PUSH_INTERVAL_S sim-сек
    *trail_timer += dt_sim;
    if *trail_timer >= TRAIL_PUSH_INTERVAL_S {
        *trail_timer = 0.0;
        let pos = sim.orion_pos_km; // copy перед мутабельным заимствованием trail_buffer
        if sim.trail_buffer.len() >= TRAIL_MAX {
            sim.trail_buffer.pop_front();
        }
        sim.trail_buffer.push_back(pos);
    }
}

/// Синхронизирует Transform Земли/Луны/Orion из km-координат TrajectorySim.
#[allow(clippy::type_complexity)]
fn sync_transforms(
    sim: Res<TrajectorySim>,
    mut earth_q: Query<
        &mut Transform,
        (
            With<TransitEarth>,
            Without<TransitMoon>,
            Without<PlayerVehicle>,
        ),
    >,
    mut moon_q: Query<
        &mut Transform,
        (
            With<TransitMoon>,
            Without<TransitEarth>,
            Without<PlayerVehicle>,
        ),
    >,
    mut orion_q: Query<
        &mut Transform,
        (
            With<PlayerVehicle>,
            Without<TransitEarth>,
            Without<TransitMoon>,
        ),
    >,
) {
    for mut tr in &mut earth_q {
        tr.translation = Vec3::ZERO;
    }
    let moon_pos = sim.moon_pos_km.as_vec3() * KM_TO_UNITS;
    for mut tr in &mut moon_q {
        tr.translation = moon_pos;
    }
    let orion_pos = sim.orion_pos_km.as_vec3() * KM_TO_UNITS;
    for mut tr in &mut orion_q {
        tr.translation = orion_pos;
    }
}

fn handle_mcc_input(
    mut sim: ResMut<TrajectorySim>,
    mut events: MessageReader<MccCommand>,
    mut outcome: ResMut<TransitOutcome>,
) {
    for MccCommand(sign) in events.read() {
        trajectory::apply_mcc(&mut sim, *sign);
        outcome.mcc_corrections = outcome.mcc_corrections.saturating_add(1);
    }
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
        state.radiation_msv += 12.0;
        state.event_msg = Some("transit.event.solar_flare");
        state.event_display_s = EVENT_DISPLAY_S;
        commands.spawn((AudioPlayer(assets.alarm.clone()), PlaybackSettings::ONCE));
        warn!("stages/transit: солнечная вспышка! радиация +12 мЗв");
    } else if roll < 0.50 {
        state.event_msg = Some("transit.event.micrometeorite");
        state.event_display_s = EVENT_DISPLAY_S;
        commands.spawn((AudioPlayer(assets.alarm.clone()), PlaybackSettings::ONCE));
        warn!("stages/transit: микрометеорит!");
    }
}

fn float_astronaut(time: Res<Time>, mut query: Query<(&mut Transform, &FloatingAstronaut)>) {
    let t = time.elapsed_secs();
    let dt = time.delta_secs();
    for (mut tr, fa) in &mut query {
        tr.translation.y = fa.base_y + 0.3 * (t * 0.4).sin();
        tr.rotate_local_y(0.008 * dt);
    }
}

/// Кеширует предиктивную траекторию раз в 2 game-секунды.
/// Forward-prop 3 дня с шагом 60 сек → sample каждые 9 шагов (~480 отрезков).
fn update_trajectory_prediction(
    mut sim: ResMut<TrajectorySim>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < 2.0 {
        return;
    }
    *timer = 0.0;

    use crate::config::EARTH_RADIUS_KM;
    let mut tmp = sim.clone();
    let t_end = sim.elapsed_sim_s + 3.0 * 86400.0;
    let mut pts: Vec<bevy::math::DVec3> = Vec::with_capacity(500);
    pts.push(tmp.orion_pos_km);
    let step = 60.0_f64;
    let mut i = 0_usize;
    let mut min_dist = f64::INFINITY;

    while tmp.elapsed_sim_s < t_end {
        trajectory::integrate(&mut tmp, step);
        i += 1;
        if i.is_multiple_of(9) {
            pts.push(tmp.orion_pos_km);
        }
        let d = (tmp.orion_pos_km - tmp.moon_pos_km).length();
        if d < min_dist {
            min_dist = d;
        }
        if tmp.orion_pos_km.length() < EARTH_RADIUS_KM {
            break;
        }
    }
    sim.predicted_trajectory = pts;
    // Обновляем предсказанный перилуний (минимум за всю forward-prop)
    if min_dist < f64::INFINITY {
        sim.predicted_perilune_km = min_dist;
    }
}

/// Рисует gizmos: прошлый путь (белый), прогноз (зелёный), орбита Луны (серая).
fn draw_trajectory_gizmos(sim: Res<TrajectorySim>, mut gizmos: Gizmos) {
    // Прошлый путь
    let trail: Vec<Vec3> = sim
        .trail_buffer
        .iter()
        .map(|p| p.as_vec3() * KM_TO_UNITS)
        .collect();
    for w in trail.windows(2) {
        gizmos.line(w[0], w[1], Color::srgba(1.0, 1.0, 1.0, 0.35));
    }

    // Предиктивная траектория
    let pred: Vec<Vec3> = sim
        .predicted_trajectory
        .iter()
        .map(|p| p.as_vec3() * KM_TO_UNITS)
        .collect();
    for w in pred.windows(2) {
        gizmos.line(w[0], w[1], Color::srgba(0.0, 1.0, 0.3, 0.55));
    }

    // Орбита Луны — тонкий серый круг в плоскости XZ
    gizmos.circle(
        Isometry3d::new(Vec3::ZERO, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        MOON_ORBIT_R_KM as f32 * KM_TO_UNITS,
        Color::srgba(0.4, 0.4, 0.4, 0.2),
    );

    // Текущие позиции Луны и Orion
    let moon_pos = sim.moon_pos_km.as_vec3() * KM_TO_UNITS;
    gizmos.sphere(
        Isometry3d::from_translation(moon_pos),
        1.0,
        Color::srgba(0.7, 0.7, 1.0, 0.6),
    );
}

/// Автоматически снижает warp до ≤ ×5 при dist_moon < 10 000 км.
fn warp_auto_drop(
    sim: Res<TrajectorySim>,
    mut warp: ResMut<WarpLevel>,
    mut timescale: ResMut<TimeScale>,
    mut notified: Local<bool>,
    mut state: ResMut<TransitState>,
) {
    let d = (sim.orion_pos_km - sim.moon_pos_km).length();
    if d < 10_000.0 && warp.0 > 1 {
        warp.0 = 1; // ≤ ×5
        timescale.multiplier = WARP_LEVELS[1];
        if !*notified {
            *notified = true;
            state.event_msg = Some("alert.warp_auto_drop");
            state.event_display_s = 5.0;
            info!("warp_auto_drop: dist_moon={:.0} км → warp снижен до ×5", d);
        }
    } else if d >= 10_000.0 {
        *notified = false;
    }
}

fn check_lunar_soi(
    sim: Res<TrajectorySim>,
    mut next: ResMut<NextState<MissionStage>>,
) {
    let d = (sim.orion_pos_km - sim.moon_pos_km).length();
    if d < SOI_MOON_KM {
        info!(
            "stages/transit: вход в SOI Луны, dist={:.0} км → LunarFlyby",
            d
        );
        next.set(MissionStage::LunarFlyby);
    }
}

fn check_miss(
    sim: Res<TrajectorySim>,
    mut events: MessageWriter<MissionEvent>,
    mut fired: Local<bool>,
) {
    if *fired {
        return;
    }
    let d = (sim.orion_pos_km - sim.moon_pos_km).length();
    // Положительная радиальная скорость (Orion удаляется от Луны)
    let v_radial = sim
        .orion_vel_kms
        .dot((sim.orion_pos_km - sim.moon_pos_km).normalize_or_zero());
    if sim.elapsed_sim_s > 4.0 * 86400.0 && v_radial > 0.0 && d > SOI_MOON_KM * 5.0 {
        *fired = true;
        warn!(
            "stages/transit: Луна пропущена (elapsed={:.1}d, dist={:.0} км)",
            sim.elapsed_sim_s / 86400.0,
            d
        );
        events.write(MissionEvent::Abort("alert.missed_lunar_approach".into()));
    }
}
