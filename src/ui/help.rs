//! Справка по управлению — F1 показывает/скрывает оверлей со всеми хоткеями.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::i18n::{Lang, Translations};
use crate::states::MissionStage;
use crate::ui::theme;

#[derive(Resource, Default)]
pub struct HelpOverlay {
    pub visible: bool,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<HelpOverlay>()
        .add_systems(Update, toggle_help)
        .add_systems(EguiPrimaryContextPass, draw_help);
}

fn toggle_help(keys: Res<ButtonInput<KeyCode>>, mut overlay: ResMut<HelpOverlay>) {
    if keys.just_pressed(KeyCode::F1) {
        overlay.visible = !overlay.visible;
    }
    // ESC закрывает справку, не трогая основную паузу.
    if overlay.visible && keys.just_pressed(KeyCode::Escape) {
        overlay.visible = false;
    }
}

fn draw_help(
    mut contexts: EguiContexts,
    overlay: Res<HelpOverlay>,
    stage: Res<State<MissionStage>>,
    lang: Res<Lang>,
    t: Res<Translations>,
) -> Result {
    if !overlay.visible {
        return Ok(());
    }
    // На Loading нет смысла показывать.
    if matches!(stage.get(), MissionStage::Loading) {
        return Ok(());
    }
    let ctx = contexts.ctx_mut()?;
    let cur = *lang;

    egui::Window::new("help_overlay")
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::new()
                .fill(theme::PANEL_BG)
                .inner_margin(28.0)
                .stroke(egui::Stroke::new(1.5, theme::NASA_BLUE)),
        )
        .show(ctx, |ui| {
            ui.set_width(420.0);

            ui.vertical_centered(|ui| {
                ui.colored_label(
                    theme::SLS_ORANGE,
                    egui::RichText::new(t.get(cur, "help.title")).size(22.0).strong(),
                );
            });
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(8.0);

            section(ui, cur, &t, "help.section.camera", &[
                "hint.cam_far",
                "hint.cam_up",
                "hint.cam_down",
                "hint.cam_cockpit",
                "hint.cam_chase",
                "hint.cam_free",
            ]);

            section(ui, cur, &t, "help.section.flight", &[
                "hint.pitch",
                "hint.throttle",
                "hint.cutoff",
                "hint.timescale",
                "hint.tli_burn",
                "hint.mcc",
                "hint.entry_angle",
                "hint.commit_entry",
            ]);

            section(ui, cur, &t, "help.section.mouse", &[
                "hint.mouse_orbit",
                "hint.mouse_pan",
                "hint.mouse_zoom",
            ]);

            section(ui, cur, &t, "help.section.system", &[
                "hint.pause",
                "hint.help_toggle",
                "hint.debug_toggle",
            ]);

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(6.0);
            ui.vertical_centered(|ui| {
                ui.colored_label(
                    theme::TEXT_MUTED,
                    egui::RichText::new(t.get(cur, "help.close_hint")).size(11.0),
                );
            });
        });

    Ok(())
}

fn section(ui: &mut egui::Ui, lang: Lang, t: &Translations, header_key: &str, lines: &[&str]) {
    ui.colored_label(
        theme::NASA_BLUE,
        egui::RichText::new(t.get(lang, header_key)).size(14.0).strong(),
    );
    ui.add_space(4.0);
    for key in lines {
        ui.colored_label(
            theme::TEXT_PRIMARY,
            egui::RichText::new(t.get(lang, key)).size(13.0),
        );
    }
    ui.add_space(10.0);
}
