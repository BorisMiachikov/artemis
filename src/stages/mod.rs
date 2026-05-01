use bevy::prelude::*;

pub mod launch;
pub mod launch_pad;
pub mod lunar_flyby;
pub mod main_menu;
pub mod orbit;
pub mod prelaunch;
pub mod reentry;
pub mod tli;
pub mod transit;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        main_menu::plugin,
        prelaunch::plugin,
        launch::plugin,
        orbit::plugin,
        tli::plugin,
        transit::plugin,
        lunar_flyby::plugin,
        reentry::plugin,
    ));
}
