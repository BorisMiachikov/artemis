use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;

use crate::assets::GameAssets;
use crate::camera::PlayerVehicle;
use crate::config::{FlybyResult, TransitOutcome};
use crate::lod::{DistanceLod, LodMaterials, LodSphere};
use crate::events::MissionEvent;
use crate::states::MissionStage;

/// Реальный перицентр Artemis II: 6 556 км от центра Луны.
const PERILUNE_TARGET_KM: f32 = 6_556.0;
/// При ошибке 0% — точно 6 556 км; при 100% — до 13 112 км.
const PERILUNE_ERROR_RANGE_KM: f32 = 6_556.0;
/// Начальная дистанция при входе в этап, км.
const APPROACH_START_KM: f32 = 50_000.0;
/// Дистанция для перехода к возврату после перицентра.
const DEPARTURE_END_KM: f32 = 25_000.0;
/// Скорость сближения: км/сек в игровом времени (≈ 0.88 км/с).
const APPROACH_SPEED_KMS: f32 = 450.0;

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

pub fn plugin(app: &mut App) {
    app.init_resource::<FlybyState>()
        .add_systems(OnEnter(MissionStage::LunarFlyby), setup_flyby)
        .add_systems(
            Update,
            (tick_flyby, check_flyby_events, animate_moon, check_departure)
                .chain()
                .run_if(in_state(MissionStage::LunarFlyby)),
        );
}

fn setup_flyby(
    mut commands: Commands,
    assets: Res<GameAssets>,
    outcome: Res<TransitOutcome>,
    mut state: ResMut<FlybyState>,
    mut result: ResMut<FlybyResult>,
    lod_mats: Res<LodMaterials>,
) {
    // Перицентр: при ошибке 0 → 6 556 км, ошибке 1.0 → 13 112 км
    let perilune = PERILUNE_TARGET_KM + outcome.trajectory_error * PERILUNE_ERROR_RANGE_KM;

    *state = FlybyState {
        phase: FlybyPhase::Approach,
        approach_km: APPROACH_START_KM,
        perilune_km: perilune,
        speed_kms: 0.88,
        perilune_fired: false,
        departure_started: false,
    };
    result.perilune_km = perilune;

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

    // Луна занимает весь задний план
    let moon_lo = commands.spawn((
        Mesh3d(lod_mats.lo_mesh.clone()),
        MeshMaterial3d(lod_mats.moon_mat.clone()),
        Transform::from_xyz(0.0, -20.0, -80.0).with_scale(Vec3::splat(60.0)),
        LodSphere,
        Visibility::Hidden,
        DespawnOnExit(MissionStage::LunarFlyby),
    )).id();
    commands.spawn((
        SceneRoot(assets.moon.clone()),
        Transform::from_xyz(0.0, -20.0, -80.0).with_scale(Vec3::splat(60.0)),
        FlybyMoon,
        DistanceLod { lo_entity: moon_lo, min_apparent: 0.08 },
        DespawnOnExit(MissionStage::LunarFlyby),
    ));

    // Orion — летит мимо
    commands.spawn((
        SceneRoot(assets.orion.clone()),
        Transform::from_xyz(-30.0, 0.0, 0.0).with_scale(Vec3::splat(3.0)),
        PlayerVehicle,
        DespawnOnExit(MissionStage::LunarFlyby),
    ));

    // Фон: тихая космическая музыка
    commands.spawn((
        AudioPlayer(assets.ambient_orbit.clone()),
        PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(0.15)),
        DespawnOnExit(MissionStage::LunarFlyby),
    ));

    info!(
        "stages/lunar_flyby: перицентр={:.0} км (ошибка траектории={:.2})",
        perilune, outcome.trajectory_error
    );
}

fn tick_flyby(
    time: Res<Time>,
    mut state: ResMut<FlybyState>,
) {
    let dt = time.delta_secs();

    match state.phase {
        FlybyPhase::Approach => {
            state.approach_km -= APPROACH_SPEED_KMS * dt;
            // Гравитационное ускорение: скорость растёт при сближении
            state.speed_kms = 0.88 + (APPROACH_START_KM - state.approach_km) / APPROACH_START_KM * 0.4;

            if state.approach_km <= state.perilune_km {
                state.approach_km = state.perilune_km;
                state.phase = FlybyPhase::Perilune;
            }
        }
        FlybyPhase::Perilune => {
            // Задержка 3 сек в перицентре
            if !state.perilune_fired {
                state.perilune_fired = true;
            }
        }
        FlybyPhase::Departure => {
            state.approach_km += APPROACH_SPEED_KMS * dt * 0.8;
        }
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
                // Гравитационный ассист завершён — уходим
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
        _ => {}
    }
}

fn animate_moon(
    state: Res<FlybyState>,
    mut q: Query<&mut Transform, With<FlybyMoon>>,
) {
    // Луна немного движется в поле зрения по мере прохода перицентра
    for mut tr in &mut q {
        let phase_offset = match state.phase {
            FlybyPhase::Approach => 0.0f32,
            FlybyPhase::Perilune => 5.0,
            FlybyPhase::Departure => 15.0,
        };
        tr.translation.x = phase_offset;
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
