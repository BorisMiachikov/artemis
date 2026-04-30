use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::states::MissionStage;
use crate::ui::mission::MissionFailed;

#[derive(Resource, Default)]
pub struct DebugOverlay {
    pub visible: bool,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<DebugOverlay>()
        .add_systems(Update, (toggle_debug, handle_stage_jump));
    app.add_systems(EguiPrimaryContextPass, draw_debug);
}

fn toggle_debug(keys: Res<ButtonInput<KeyCode>>, mut overlay: ResMut<DebugOverlay>) {
    if keys.just_pressed(KeyCode::F12) {
        overlay.visible = !overlay.visible;
        info!("debug overlay: {}", if overlay.visible { "ON" } else { "OFF" });
    }
}

fn handle_stage_jump(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_stage: ResMut<NextState<MissionStage>>,
    mut failed: ResMut<MissionFailed>,
) {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if !ctrl {
        return;
    }

    let target = match () {
        _ if keys.just_pressed(KeyCode::Digit1) => Some(MissionStage::Prelaunch),
        _ if keys.just_pressed(KeyCode::Digit2) => Some(MissionStage::Launch),
        _ if keys.just_pressed(KeyCode::Digit3) => Some(MissionStage::Orbit),
        _ if keys.just_pressed(KeyCode::Digit4) => Some(MissionStage::TLI),
        _ if keys.just_pressed(KeyCode::Digit5) => Some(MissionStage::Transit),
        _ if keys.just_pressed(KeyCode::Digit6) => Some(MissionStage::LunarFlyby),
        _ if keys.just_pressed(KeyCode::Digit7) => Some(MissionStage::Reentry),
        _ if keys.just_pressed(KeyCode::Digit8) => Some(MissionStage::Splashdown),
        _ => None,
    };

    if let Some(stage) = target {
        info!("debug jump → {:?}", stage);
        *failed = MissionFailed::default();
        next_stage.set(stage);
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
            ui.separator();
            ui.monospace("Ctrl+1  Prelaunch");
            ui.monospace("Ctrl+2  Launch");
            ui.monospace("Ctrl+3  Orbit");
            ui.monospace("Ctrl+4  TLI");
            ui.monospace("Ctrl+5  Transit");
            ui.monospace("Ctrl+6  LunarFlyby");
            ui.monospace("Ctrl+7  Reentry");
            ui.monospace("Ctrl+8  Splashdown");
        });

    Ok(())
}
