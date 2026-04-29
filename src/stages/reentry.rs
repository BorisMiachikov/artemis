use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;

use crate::assets::GameAssets;
use crate::camera::PlayerVehicle;
use crate::config::{Difficulty, TimeScale};
use crate::lod::{DistanceLod, LodMaterials, LodSphere};
use crate::events::MissionEvent;
use crate::states::MissionStage;

/// Целевое окно угла входа: 6.0°–6.5°. Центр — 6.25°.
const ENTRY_ANGLE_MIN: f32 = 6.0;
const ENTRY_ANGLE_MAX: f32 = 6.5;
const ENTRY_ANGLE_TARGET: f32 = 6.25;
/// Начальная высота (за пределами атмосферы), км.
const ALTITUDE_START_KM: f32 = 400.0;
/// Высота перехода к активному входу (атмосфера начинается), км.
const ENTRY_ALTITUDE_KM: f32 = 120.0;
/// Высота раскрытия парашютов, км.
const CHUTE_ALTITUDE_KM: f32 = 8.0;
/// Скорость входа (вектор суммарный), м/с.
const ENTRY_SPEED_MS: f32 = 11_000.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ReentryPhase {
    #[default]
    Approach,   // игрок выставляет угол, ещё высоко
    Entry,      // вошли в атмосферу, угол зафиксирован
    Descent,    // дозвуковой, снижение
    Parachutes, // раскрыты парашюты
}

#[derive(Resource, Default, Debug)]
pub struct ReentryState {
    pub phase: ReentryPhase,
    pub entry_angle_deg: f32,
    pub altitude_km: f32,
    pub speed_ms: f32,
    pub heat_pct: f32,
    pub elapsed_s: f32,
    pub abort_reason: Option<String>,
}

#[derive(Component)]
struct ReentryEarth;

#[derive(Component)]
struct AtmosphericGlow;

#[derive(Resource)]
struct GlowMaterial(Handle<StandardMaterial>);

pub fn plugin(app: &mut App) {
    app.init_resource::<ReentryState>()
        .add_systems(OnEnter(MissionStage::Reentry), setup_reentry)
        .add_systems(
            Update,
            (
                handle_angle_input,
                commit_entry,
                tick_reentry_physics,
                check_reentry_outcome,
            )
                .chain()
                .run_if(in_state(MissionStage::Reentry)),
        )
        .add_systems(
            Update,
            tick_glow.run_if(in_state(MissionStage::Reentry)),
        );
}

fn setup_reentry(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut state: ResMut<ReentryState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    lod_mats: Res<LodMaterials>,
) {
    *state = ReentryState {
        phase: ReentryPhase::Approach,
        entry_angle_deg: ENTRY_ANGLE_TARGET,
        altitude_km: ALTITUDE_START_KM,
        speed_ms: ENTRY_SPEED_MS,
        heat_pct: 0.0,
        elapsed_s: 0.0,
        abort_reason: None,
    };

    // Свет: закат над Тихим океаном
    commands.spawn((
        DirectionalLight {
            illuminance: 45_000.0,
            color: Color::srgb(1.0, 0.88, 0.75),
            ..default()
        },
        Transform::from_xyz(20.0, -10.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
        DespawnOnExit(MissionStage::Reentry),
    ));

    // Земля под кораблём
    let earth_lo = commands.spawn((
        Mesh3d(lod_mats.lo_mesh.clone()),
        MeshMaterial3d(lod_mats.earth_mat.clone()),
        Transform::from_xyz(0.0, -120.0, 0.0).with_scale(Vec3::splat(100.0)),
        LodSphere,
        Visibility::Hidden,
        DespawnOnExit(MissionStage::Reentry),
    )).id();
    commands.spawn((
        SceneRoot(assets.earth.clone()),
        Transform::from_xyz(0.0, -120.0, 0.0).with_scale(Vec3::splat(100.0)),
        ReentryEarth,
        DistanceLod { lo_entity: earth_lo, min_apparent: 0.08 },
        DespawnOnExit(MissionStage::Reentry),
    ));

    // Orion
    commands.spawn((
        SceneRoot(assets.orion.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(3.0)),
        PlayerVehicle,
        DespawnOnExit(MissionStage::Reentry),
    ));

    // Атмосферное свечение — прозрачная сфера вокруг Ориона, разгорается при нагреве
    let glow_handle = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.0, 0.0, 0.0),
        emissive: LinearRgba::BLACK,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    commands.insert_resource(GlowMaterial(glow_handle.clone()));
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(2.0))),
        MeshMaterial3d(glow_handle),
        Transform::IDENTITY,
        AtmosphericGlow,
        DespawnOnExit(MissionStage::Reentry),
    ));

    info!("stages/reentry: старт, высота={:.0} км, скорость={:.0} м/с", ALTITUDE_START_KM, ENTRY_SPEED_MS);
}

/// Игрок настраивает угол W/S, пока не вошли в атмосферу.
fn handle_angle_input(
    keys: Res<ButtonInput<KeyCode>>,
    diff: Res<Difficulty>,
    mut state: ResMut<ReentryState>,
) {
    if state.phase != ReentryPhase::Approach {
        return;
    }

    let step = match *diff {
        Difficulty::Story => 0.15,
        Difficulty::Realistic => 0.05,
    };

    if keys.just_pressed(KeyCode::KeyW) {
        state.entry_angle_deg = (state.entry_angle_deg + step).min(9.0);
    }
    if keys.just_pressed(KeyCode::KeyS) {
        state.entry_angle_deg = (state.entry_angle_deg - step).max(3.0);
    }
}

/// Пробел фиксирует курс и начинает вход.
fn commit_entry(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<ReentryState>,
    mut events: MessageWriter<MissionEvent>,
    mut commands: Commands,
    assets: Res<GameAssets>,
) {
    if state.phase != ReentryPhase::Approach {
        return;
    }
    if keys.just_pressed(KeyCode::Space) {
        state.phase = ReentryPhase::Entry;
        events.write(MissionEvent::AtmosphereEntry);
        commands.spawn((
            AudioPlayer(assets.afterburner.clone()),
            PlaybackSettings::ONCE,
        ));
        info!(
            "stages/reentry: вход зафиксирован, угол={:.2}°",
            state.entry_angle_deg
        );
    }
}

fn tick_reentry_physics(
    time: Res<Time>,
    timescale: Res<TimeScale>,
    diff: Res<Difficulty>,
    mut state: ResMut<ReentryState>,
    mut events: MessageWriter<MissionEvent>,
    mut commands: Commands,
    assets: Res<GameAssets>,
) {
    if !matches!(state.phase, ReentryPhase::Entry | ReentryPhase::Descent | ReentryPhase::Parachutes) {
        return;
    }

    let dt = time.delta_secs() * timescale.multiplier;
    state.elapsed_s += dt;

    let angle = state.entry_angle_deg;

    match state.phase {
        ReentryPhase::Entry => {
            // Торможение: упрощённое (квадратичное сопротивление нарастает с ростом плотности)
            let drag = (0.5 + (ENTRY_ALTITUDE_KM - state.altitude_km) / ENTRY_ALTITUDE_KM * 3.0).max(0.5);
            state.speed_ms -= drag * 900.0 * dt;
            state.altitude_km -= state.speed_ms * angle.to_radians().sin() / 1000.0 * dt;

            // Нагрев: максимальный при скорости > 7 000 м/с, зависит от отклонения угла
            let angle_error = ((angle - ENTRY_ANGLE_TARGET) / (ENTRY_ANGLE_MAX - ENTRY_ANGLE_MIN)).abs();
            let speed_factor = (state.speed_ms / ENTRY_SPEED_MS).powi(2);
            state.heat_pct += speed_factor * (0.8 + angle_error * 2.0) * dt * 4.0;
            state.heat_pct = state.heat_pct.min(110.0);

            if state.altitude_km <= CHUTE_ALTITUDE_KM {
                state.phase = ReentryPhase::Descent;
            }
        }
        ReentryPhase::Descent => {
            // Дозвуковой: тормозим быстрее
            state.speed_ms -= 600.0 * dt;
            state.altitude_km -= state.speed_ms / 1000.0 * dt;
            state.heat_pct = (state.heat_pct - dt * 5.0).max(0.0);

            if state.altitude_km <= CHUTE_ALTITUDE_KM * 0.4 {
                state.phase = ReentryPhase::Parachutes;
                events.write(MissionEvent::Splashdown); // пока используем как «парашюты»
                commands.spawn((
                    AudioPlayer(assets.afterburner.clone()),
                    PlaybackSettings::ONCE,
                ));
                info!("stages/reentry: парашюты раскрыты @ {:.1} км", state.altitude_km);
            }
        }
        ReentryPhase::Parachutes => {
            // Парашюты: резкое торможение
            state.speed_ms = (state.speed_ms - 2000.0 * dt).max(7.0);
            state.altitude_km = (state.altitude_km - state.speed_ms / 1000.0 * dt).max(0.0);
            state.heat_pct = (state.heat_pct - dt * 20.0).max(0.0);

            // Подавляем лишние сигналы: реальный splashdown только на земле
        }
        _ => {}
    }

    // Нагрев выше допустимого при Story разрешён (только визуальное предупреждение)
    if state.heat_pct >= 100.0 && matches!(*diff, Difficulty::Realistic) && state.abort_reason.is_none() {
        state.abort_reason = Some("перегрев теплового щита".into());
        events.write(MissionEvent::Abort("перегрев теплового щита".into()));
        warn!("stages/reentry: ABORT — перегрев теплового щита!");
    }
}

fn tick_glow(
    state: Res<ReentryState>,
    glow: Option<Res<GlowMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(glow) = glow else { return };
    let intensity = (state.heat_pct / 100.0).clamp(0.0, 1.0);
    if let Some(mat) = materials.get_mut(&glow.0) {
        mat.emissive = LinearRgba::rgb(intensity * 5.0, intensity * 1.2, 0.0);
        mat.base_color = Color::srgba(1.0, 0.4, 0.0, intensity * 0.35);
    }
}

fn check_reentry_outcome(
    state: Res<ReentryState>,
    diff: Res<Difficulty>,
    mut events: MessageWriter<MissionEvent>,
    mut next_stage: ResMut<NextState<MissionStage>>,
    mut local_done: Local<bool>,
) {
    if *local_done {
        return;
    }

    // Проверяем опасные углы при фиксации (Realistic → Abort, Story → предупреждение)
    if state.phase == ReentryPhase::Entry && state.abort_reason.is_none() {
        let too_shallow = state.entry_angle_deg < ENTRY_ANGLE_MIN;
        let too_steep = state.entry_angle_deg > ENTRY_ANGLE_MAX;

        if (too_shallow || too_steep) && matches!(*diff, Difficulty::Realistic) {
            let reason = if too_shallow {
                "слишком малый угол входа — скользящий отскок".into()
            } else {
                "слишком крутой угол входа — перегрев".into()
            };
            events.write(MissionEvent::Abort(reason));
            return;
        }
    }

    // Приводнение: высота практически нулевая, фаза парашютов
    if state.phase == ReentryPhase::Parachutes && state.altitude_km <= 0.05 {
        *local_done = true;
        events.write(MissionEvent::Splashdown);
        info!(
            "stages/reentry: SPLASHDOWN! скорость={:.1} м/с, тепловой щит={:.1}%",
            state.speed_ms, state.heat_pct
        );
        next_stage.set(MissionStage::Splashdown);
    }
}
