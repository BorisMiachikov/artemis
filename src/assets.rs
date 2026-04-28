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

    #[asset(path = "models/Earth/Earth.glb#Scene0")]
    pub earth: Handle<Scene>,

    #[asset(path = "models/Orion/Orion.glb#Scene0")]
    pub orion: Handle<Scene>,

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

    #[asset(path = "sounds/endless-sky/rulei space.mp3")]
    pub ambient_orbit: Handle<AudioSource>,

    #[asset(path = "sounds/endless-sky/scan~.wav")]
    pub scan: Handle<AudioSource>,

    #[asset(path = "sounds/endless-sky/hyperdrive in.wav")]
    pub hyperdrive_in: Handle<AudioSource>,

    #[asset(path = "sounds/endless-sky/hyperdrive.wav")]
    pub hyperdrive_loop: Handle<AudioSource>,

    #[asset(path = "sounds/endless-sky/hyperdrive out.wav")]
    pub hyperdrive_out: Handle<AudioSource>,
}

pub fn plugin(app: &mut App) {
    app.add_loading_state(
        LoadingState::new(MissionStage::Loading)
            .continue_to_state(MissionStage::Prelaunch)
            .load_collection::<GameAssets>(),
    );
}
