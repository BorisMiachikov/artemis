use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::config::TliResult;
use crate::physics::rocket::Rocket;
use crate::states::MissionStage;
use crate::ui::mission::MissionFailed;

/// Снимок прогресса миссии.
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

impl SaveSlot {
    /// Есть ли сохранённый прогресс дальше Prelaunch.
    pub fn has_progress(&self) -> bool {
        self.timestamp_unix > 0
            && !matches!(
                self.mission_stage,
                MissionStage::Loading | MissionStage::Prelaunch
            )
    }
}

/// Сигнал: пользователь нажал «ПРОДОЛЖИТЬ» — применить сейв и перейти на сохранённый стейт.
#[derive(Resource, Default)]
pub struct LoadRequested(pub bool);

pub fn plugin(app: &mut App) {
    app.init_resource::<SaveSlot>()
        .init_resource::<LoadRequested>()
        .add_systems(Startup, startup_load)
        .add_systems(Update, apply_load);

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

fn startup_load(mut slot: ResMut<SaveSlot>) {
    let Some(path) = save_path() else { return };
    let Ok(content) = std::fs::read_to_string(&path) else { return };
    match ron::de::from_str::<SaveSlot>(&content) {
        Ok(loaded) => {
            info!(
                "save: загружен сейв — стейт {:?}, топливо {:.0} кг, TLI {:.0} м/с",
                loaded.mission_stage, loaded.fuel_kg, loaded.tli_delta_v_ms
            );
            *slot = loaded;
        }
        Err(e) => warn!("save: ошибка чтения сейва: {e}"),
    }
}

fn apply_load(
    mut requested: ResMut<LoadRequested>,
    slot: Res<SaveSlot>,
    mut tli: ResMut<TliResult>,
    mut failed: ResMut<MissionFailed>,
    mut next_stage: ResMut<NextState<MissionStage>>,
) {
    if !requested.0 {
        return;
    }
    requested.0 = false;

    if slot.tli_delta_v_ms > 0.0 {
        tli.delta_v_ms = slot.tli_delta_v_ms;
        tli.completed = true;
    }
    *failed = MissionFailed::default();
    next_stage.set(slot.mission_stage.clone());
    info!("save: восстановлен стейт {:?}", slot.mission_stage);
}

fn autosave(
    stage: Res<State<MissionStage>>,
    rockets: Query<&Rocket>,
    tli: Res<TliResult>,
    mut slot: ResMut<SaveSlot>,
) {
    slot.mission_stage = stage.get().clone();
    slot.timestamp_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if let Ok(rocket) = rockets.single() {
        slot.fuel_kg = rocket.fuel_kg;
    }
    slot.tli_delta_v_ms = tli.delta_v_ms;

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
