use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::states::MissionStage;

/// Снимок прогресса миссии. На Фазе 2 хранятся только этап и таймстамп —
/// топливо/delta-v/состояние систем подключим в Фазе 3, когда появятся ECS-компоненты.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct SaveSlot {
    pub mission_stage: MissionStage,
    pub fuel_kg: f32,
    pub tli_delta_v_ms: f32,
    pub timestamp_unix: u64,
}

impl Default for SaveSlot {
    fn default() -> Self {
        Self {
            mission_stage: MissionStage::Prelaunch,
            fuel_kg: 0.0,
            tli_delta_v_ms: 0.0,
            timestamp_unix: 0,
        }
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<SaveSlot>();

    // Автосейв на входе в каждый стейт миссии (кроме Loading — там сейвить нечего).
    for stage in [
        MissionStage::Prelaunch,
        MissionStage::Launch,
        MissionStage::Orbit,
        MissionStage::TLI,
        MissionStage::Transit,
        MissionStage::LunarFlyby,
        MissionStage::Reentry,
        MissionStage::Splashdown,
    ] {
        app.add_systems(OnEnter(stage), autosave);
    }
}

fn autosave(stage: Res<State<MissionStage>>, mut slot: ResMut<SaveSlot>) {
    slot.mission_stage = stage.get().clone();
    slot.timestamp_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let Some(path) = save_path() else {
        warn!("save: не удалось определить директорию для сейва");
        return;
    };

    if let Some(parent) = path.parent()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        warn!("save: не удалось создать {}: {}", parent.display(), e);
        return;
    }

    match ron::ser::to_string_pretty(&*slot, ron::ser::PrettyConfig::default()) {
        Ok(content) => match std::fs::write(&path, content) {
            Ok(()) => info!(
                "save: автосейв в {} (стейт {:?})",
                path.display(),
                slot.mission_stage
            ),
            Err(e) => warn!("save: не удалось записать {}: {}", path.display(), e),
        },
        Err(e) => warn!("save: ошибка сериализации: {}", e),
    }
}

fn save_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "Artemis", "Artemis").map(|dirs| dirs.data_dir().join("save.ron"))
}
