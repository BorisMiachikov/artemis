use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::states::MissionStage;

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    // --- Модели ---
    #[asset(path = "models/SLS/SLS.glb#Scene0")]
    pub sls: Handle<Scene>,

    #[asset(path = "models/Gantry/Gantry.glb#Scene0")]
    pub gantry: Handle<Scene>,

    #[asset(path = "models/Crawler/Crawler.glb#Scene0")]
    pub crawler: Handle<Scene>,

    // --- Аудио ---
    #[asset(path = "sounds/endless-sky/machinery.mp3")]
    pub ambient_machinery: Handle<AudioSource>,

    #[asset(path = "sounds/endless-sky/human launch.wav")]
    pub launch_engines: Handle<AudioSource>,

    #[asset(path = "sounds/endless-sky/takeoff.wav")]
    pub takeoff: Handle<AudioSource>,

    #[asset(path = "sounds/endless-sky/explosion medium.wav")]
    pub explosion_medium: Handle<AudioSource>,

    #[asset(path = "sounds/endless-sky/nuke alarm.wav")]
    pub nuke_alarm: Handle<AudioSource>,
}

pub fn plugin(app: &mut App) {
    app.add_loading_state(
        LoadingState::new(MissionStage::Loading)
            .continue_to_state(MissionStage::Prelaunch)
            .load_collection::<GameAssets>(),
    );
}
