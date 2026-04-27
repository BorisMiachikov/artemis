use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::states::MissionStage;

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    // На Фазе 1 — Saturn V как тестовая модель (Astronaut.glb сжат Draco,
    // bevy_gltf такой формат не поддерживает; вернёмся к нему после конвертации).
    #[asset(path = "models/Saturn V/Saturn V.glb#Scene0")]
    pub saturn_v: Handle<Scene>,
}

pub fn plugin(app: &mut App) {
    app.add_loading_state(
        LoadingState::new(MissionStage::Loading)
            .continue_to_state(MissionStage::Prelaunch)
            .load_collection::<GameAssets>(),
    );
}
