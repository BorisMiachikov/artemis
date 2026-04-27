#![allow(dead_code)] // варианты используются с Фазы 3

use bevy::prelude::*;

#[derive(Message, Clone, Debug)]
pub enum MissionEvent {
    Liftoff,
    SrbSep,
    Meco,
    TliBurnStart,
    TliBurnEnd,
    PerilunePassage,
    AtmosphereEntry,
    Splashdown,
    Abort(String),
}

pub fn plugin(app: &mut App) {
    app.add_message::<MissionEvent>();
}
