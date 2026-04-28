use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;

use crate::assets::GameAssets;
use crate::events::MissionEvent;
use crate::states::MissionStage;

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
            _ => {} // Реакции на остальные события — в Фазах 4+
        }
    }
}
