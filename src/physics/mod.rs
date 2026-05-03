use bevy::prelude::*;

pub mod orbital;
pub mod reentry;
pub mod rocket;
pub mod trajectory;

pub fn plugin(app: &mut App) {
    app.add_plugins((rocket::plugin, orbital::plugin, trajectory::plugin));
}
