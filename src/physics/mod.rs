use bevy::prelude::*;

pub mod orbital;
pub mod reentry;
pub mod rocket;

pub fn plugin(app: &mut App) {
    app.add_plugins((rocket::plugin, orbital::plugin));
}
