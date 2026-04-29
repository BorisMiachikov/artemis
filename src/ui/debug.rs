use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::states::MissionStage;

#[derive(Resource, Default)]
pub struct DebugOverlay {
    pub visible: bool,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<DebugOverlay>()
        .add_systems(Update, toggle_debug)
        .add_systems(EguiPrimaryContextPass, draw_debug);
}

fn toggle_debug(keys: Res<ButtonInput<KeyCode>>, mut overlay: ResMut<DebugOverlay>) {
    if keys.just_pressed(KeyCode::F12) {
        overlay.visible = !overlay.visible;
        info!("debug overlay: {}", if overlay.visible { "ON" } else { "OFF" });
    }
}

fn draw_debug(
    mut contexts: EguiContexts,
    overlay: Res<DebugOverlay>,
    diagnostics: Res<DiagnosticsStore>,
    stage: Res<State<MissionStage>>,
) -> Result {
    if !overlay.visible {
        return Ok(());
    }

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);
    let frame_ms = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.smoothed())
        .map(|v| v * 1000.0)
        .unwrap_or(0.0);

    let ctx = contexts.ctx_mut()?;
    egui::Window::new("Debug")
        .anchor(egui::Align2::RIGHT_TOP, [-8.0, 8.0])
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.monospace(format!("FPS:   {fps:.1}"));
            ui.monospace(format!("Frame: {frame_ms:.2} ms"));
            ui.separator();
            ui.monospace(format!("Stage: {:?}", stage.get()));
        });

    Ok(())
}
