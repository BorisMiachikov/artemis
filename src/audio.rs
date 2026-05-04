use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;

use crate::assets::GameAssets;
use crate::events::MissionEvent;
use crate::physics::rocket::{FlightPhase, Rocket, THROTTLE_MAX, THROTTLE_MIN};
use crate::states::MissionStage;

// ---------------------------------------------------------------------------
// Ресурсы громкости
// ---------------------------------------------------------------------------

#[derive(Resource, Clone, Copy)]
pub struct MusicVolume(pub f32);

#[derive(Resource, Clone, Copy)]
pub struct SfxVolume(pub f32);

impl Default for MusicVolume {
    fn default() -> Self { Self(0.5) }
}
impl Default for SfxVolume {
    fn default() -> Self { Self(0.7) }
}

/// Маркер на ambient-треках, чтобы обновлять громкость на лету.
#[derive(Component)]
pub struct MusicTrack {
    pub base: f32,
}

/// Маркер на сущности, проигрывающей `hyperdrive` в loop.
#[derive(Component)]
struct TliBurnLoop;

/// Маркер на сущности с сиреной аварии. Без него рестарт миссии оставлял
/// `nuke_alarm.wav` доигрывать поверх нового полёта.
#[derive(Component)]
struct AbortAlarm;

/// Маркер на looping-треке двигателей во время Launch.
/// Громкость обновляется каждый кадр из Rocket.throttle_pct и FlightPhase.
#[derive(Component)]
struct EngineLoop;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub fn plugin(app: &mut App) {
    app.init_resource::<MusicVolume>()
        .init_resource::<SfxVolume>()
        .add_systems(OnEnter(MissionStage::MainMenu), start_menu_ambient)
        .add_systems(
            OnEnter(MissionStage::Prelaunch),
            (start_prelaunch_ambient, stop_abort_alarm),
        )
        .add_systems(OnEnter(MissionStage::Transit), start_transit_ambient)
        .add_systems(
            Update,
            (
                react_to_mission_events.run_if(resource_exists::<GameAssets>),
                apply_music_volume,
                update_engine_volume.run_if(in_state(MissionStage::Launch)),
            ),
        );
}

/// Снимаем сирену аварии при старте новой миссии — без этого
/// `nuke_alarm.wav` доигрывает поверх Prelaunch и Launch.
fn stop_abort_alarm(mut commands: Commands, alarms: Query<Entity, With<AbortAlarm>>) {
    for e in &alarms {
        commands.entity(e).despawn();
    }
}

// ---------------------------------------------------------------------------
// Ambient
// ---------------------------------------------------------------------------

fn start_menu_ambient(mut commands: Commands, assets: Res<GameAssets>, vol: Res<MusicVolume>) {
    commands.spawn((
        AudioPlayer(assets.ambient_machinery.clone()),
        PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(0.4 * vol.0)),
        MusicTrack { base: 0.4 },
        DespawnOnExit(MissionStage::MainMenu),
    ));
}

fn start_prelaunch_ambient(
    mut commands: Commands,
    assets: Res<GameAssets>,
    vol: Res<MusicVolume>,
) {
    commands.spawn((
        AudioPlayer(assets.ambient_machinery.clone()),
        PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(0.6 * vol.0)),
        MusicTrack { base: 0.6 },
        DespawnOnExit(MissionStage::Prelaunch),
    ));
}

fn start_transit_ambient(mut commands: Commands, assets: Res<GameAssets>, vol: Res<MusicVolume>) {
    commands.spawn((
        AudioPlayer(assets.ambient_planet.clone()),
        PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(0.36 * vol.0)),
        MusicTrack { base: 0.36 },
        DespawnOnExit(MissionStage::Transit),
    ));
}

// ---------------------------------------------------------------------------
// Динамический микс громкости двигателей
// ---------------------------------------------------------------------------

/// Обновляет громкость engine-loop каждый кадр по throttle_pct и FlightPhase.
///
/// Громкость:
///   BoosterPhase → 0.85 × sfx  (SRB + RS-25, максимальный рёв)
///   CoreOnly     → [0.40–0.70] × sfx, линейно от throttle_pct (67%–109%)
///   Coast        → 0.0          (MECO — тишина)
fn update_engine_volume(
    rockets: Query<&Rocket>,
    sfx: Res<SfxVolume>,
    mut sinks: Query<&mut AudioSink, With<EngineLoop>>,
) {
    let Ok(rocket) = rockets.single() else { return };

    let vol = match rocket.phase {
        FlightPhase::BoosterPhase => 0.85 * sfx.0,
        FlightPhase::CoreOnly => {
            let norm =
                (rocket.throttle_pct - THROTTLE_MIN) / (THROTTLE_MAX - THROTTLE_MIN);
            (0.40 + norm * 0.30) * sfx.0
        }
        FlightPhase::Coast => 0.0,
    };

    for mut sink in &mut sinks {
        sink.set_volume(bevy::audio::Volume::Linear(vol));
    }
}

// ---------------------------------------------------------------------------
// Обновление громкости музыки
// ---------------------------------------------------------------------------

fn apply_music_volume(
    volume: Res<MusicVolume>,
    mut sinks: Query<(&mut AudioSink, &MusicTrack)>,
) {
    if !volume.is_changed() {
        return;
    }
    for (mut sink, track) in &mut sinks {
        sink.set_volume(bevy::audio::Volume::Linear(track.base * volume.0));
    }
}

// ---------------------------------------------------------------------------
// Реакция на события миссии
// ---------------------------------------------------------------------------

fn react_to_mission_events(
    mut commands: Commands,
    mut events: MessageReader<MissionEvent>,
    mut takeoff_timer: Local<Option<Timer>>,
    time: Res<Time>,
    assets: Res<GameAssets>,
    tli_loops: Query<Entity, With<TliBurnLoop>>,
) {
    if let Some(timer) = takeoff_timer.as_mut() {
        timer.tick(time.delta());
        if timer.just_finished() {
            commands.spawn((
                AudioPlayer(assets.takeoff.clone()),
                PlaybackSettings::ONCE,
            ));
            *takeoff_timer = None;
        }
    }

    for event in events.read() {
        match event {
            MissionEvent::Liftoff => {
                // Одноразовый стартовый грохот
                commands.spawn((
                    AudioPlayer(assets.launch_engines.clone()),
                    PlaybackSettings::ONCE,
                ));
                // Непрерывный гул двигателей — громкость обновляется в update_engine_volume
                commands.spawn((
                    AudioPlayer(assets.afterburner.clone()),
                    PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(0.0)),
                    EngineLoop,
                    DespawnOnExit(MissionStage::Launch),
                ));
                *takeoff_timer = Some(Timer::from_seconds(5.0, TimerMode::Once));
                info!("audio: запуск двигателей + engine loop");
            }
            MissionEvent::SrbSep => {
                commands.spawn((
                    AudioPlayer(assets.explosion_medium.clone()),
                    PlaybackSettings::ONCE,
                ));
                info!("audio: SRB sep — explosion medium");
            }
            MissionEvent::Meco => {
                info!("audio: MECO");
            }
            MissionEvent::Abort(reason) => {
                commands.spawn((
                    AudioPlayer(assets.nuke_alarm.clone()),
                    PlaybackSettings::DESPAWN,
                    AbortAlarm,
                ));
                warn!("audio: ABORT — {reason}");
            }
            MissionEvent::TliBurnStart => {
                commands.spawn((
                    AudioPlayer(assets.hyperdrive_in.clone()),
                    PlaybackSettings::ONCE,
                ));
                commands.spawn((
                    AudioPlayer(assets.hyperdrive_loop.clone()),
                    PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(0.5)),
                    TliBurnLoop,
                ));
                info!("audio: TLI burn — hyperdrive in + loop");
            }
            MissionEvent::TliBurnEnd => {
                for e in &tli_loops {
                    commands.entity(e).despawn();
                }
                commands.spawn((
                    AudioPlayer(assets.hyperdrive_out.clone()),
                    PlaybackSettings::ONCE,
                ));
                info!("audio: TLI burn — hyperdrive out");
            }
            MissionEvent::PerilunePassage => {
                info!("audio: PerilunePassage");
            }
            MissionEvent::AtmosphereEntry => {
                commands.spawn((
                    AudioPlayer(assets.afterburner.clone()),
                    PlaybackSettings::ONCE.with_volume(bevy::audio::Volume::Linear(0.6)),
                ));
                info!("audio: вход в атмосферу — afterburner");
            }
            MissionEvent::Splashdown => {
                commands.spawn((
                    AudioPlayer(assets.landing.clone()),
                    PlaybackSettings::ONCE,
                ));
                info!("audio: Splashdown — landing");
            }
        }
    }
}
