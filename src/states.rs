#![allow(dead_code)] // варианты используются с Фазы 3 (переходы между этапами)

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)] // TLI = Trans-Lunar Injection — официальная аббревиатура NASA
pub enum MissionStage {
    #[default]
    Loading,
    Prelaunch,
    Launch,
    Orbit,
    TLI,
    Transit,
    LunarFlyby,
    Reentry,
    Splashdown,
}

pub fn plugin(app: &mut App) {
    app.init_state::<MissionStage>();
}
