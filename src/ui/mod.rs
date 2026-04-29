use bevy::prelude::*;
use bevy_egui::EguiPlugin;

pub mod checklist;
pub mod debug;
pub mod flyby_panel;
pub mod hud;
pub mod menus;
pub mod mission;
pub mod orbit_checklist;
pub mod reentry_panel;
pub mod theme;
pub mod tli_panel;
pub mod transit_panel;

pub fn plugin(app: &mut App) {
    app.add_plugins(EguiPlugin::default()).add_plugins((
        menus::plugin,
        checklist::plugin,
        orbit_checklist::plugin,
        tli_panel::plugin,
        transit_panel::plugin,
        flyby_panel::plugin,
        reentry_panel::plugin,
        mission::plugin,
        hud::plugin,
        debug::plugin,
    ));
}
