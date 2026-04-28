use bevy::prelude::*;
use bevy_egui::EguiPlugin;

pub mod checklist;
pub mod hud;
pub mod menus;
pub mod mission;
pub mod orbit_checklist;
pub mod tli_panel;
pub mod theme;

pub fn plugin(app: &mut App) {
    app.add_plugins(EguiPlugin::default()).add_plugins((
        menus::plugin,
        checklist::plugin,
        orbit_checklist::plugin,
        tli_panel::plugin,
        hud::plugin,
    ));
}
