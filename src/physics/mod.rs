use bevy::prelude::*;

pub mod orbital;
pub mod reentry;
pub mod rocket;

pub fn plugin(_app: &mut App) {
    // Подключим в Фазе 3: rocket::plugin (тяга, расход массы),
    // orbital::plugin (гравитация), reentry::plugin (Rapier) — в Фазе 6.
}
