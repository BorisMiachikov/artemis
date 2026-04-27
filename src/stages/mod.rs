use bevy::prelude::*;

pub mod launch;
pub mod lunar_flyby;
pub mod orbit;
pub mod prelaunch;
pub mod reentry;
pub mod tli;
pub mod transit;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        prelaunch::plugin,
        launch::plugin,
        orbit::plugin,
        tli::plugin,
        transit::plugin,
        lunar_flyby::plugin,
        reentry::plugin,
    ));
}
