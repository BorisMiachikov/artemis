use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;

use crate::assets::GameAssets;
use crate::events::MissionEvent;
use crate::states::MissionStage;

/// Маркер на сущности, проигрывающей `hyperdrive` в loop, чтобы её можно было снять
/// в момент `TliBurnEnd` и заменить звуком `hyperdrive out`.
#[derive(Component)]
struct TliBurnLoop;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(MissionStage::Prelaunch), start_ambient)
        .add_systems(
            Update,
            react_to_mission_events.run_if(resource_exists::<GameAssets>),
        );
}

fn start_ambient(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((
        AudioPlayer(assets.ambient_machinery.clone()),
        PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(0.3)),
        DespawnOnExit(MissionStage::Prelaunch),
    ));
}

fn react_to_mission_events(
    mut commands: Commands,
    mut events: MessageReader<MissionEvent>,
    mut takeoff_timer: Local<Option<Timer>>,
    time: Res<Time>,
    assets: Res<GameAssets>,
    tli_loops: Query<Entity, With<TliBurnLoop>>,
) {
    // Отложенный takeoff.wav после Liftoff (запускается через 5 с реального времени).
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
                commands.spawn((
                    AudioPlayer(assets.launch_engines.clone()),
                    PlaybackSettings::ONCE,
                ));
                *takeoff_timer = Some(Timer::from_seconds(5.0, TimerMode::Once));
                info!("audio: запуск двигателей");
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
                    PlaybackSettings::ONCE,
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
                // jump_drive уже запускается в lunar_flyby.rs напрямую; здесь только лог
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
