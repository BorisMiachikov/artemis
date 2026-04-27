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
    assets: Res<GameAssets>,
) {
    for event in events.read() {
        match event {
            MissionEvent::Liftoff => {
                commands.spawn((
                    AudioPlayer(assets.launch_engines.clone()),
                    PlaybackSettings::ONCE,
                ));
                info!("audio: запуск двигателей");
            }
            _ => {} // Реакции на остальные события — в Фазах 3+
        }
    }
}
