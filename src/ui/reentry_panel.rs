use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::i18n::{Lang, Translations};
use crate::stages::reentry::{ReentryPhase, ReentryState};
use crate::states::MissionStage;
use crate::ui::theme;

pub fn plugin(app: &mut App) {
    app.add_systems(EguiPrimaryContextPass, draw_reentry_panel);
}

fn draw_reentry_panel(
    mut contexts: EguiContexts,
    stage: Res<State<MissionStage>>,
    state: Res<ReentryState>,
    lang: Res<Lang>,
    t: Res<Translations>,
) -> Result {
    if !matches!(stage.get(), MissionStage::Reentry) {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    egui::Window::new("reentry_panel")
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::RIGHT_CENTER, [-12.0, 0.0])
        .frame(egui::Frame::new().fill(theme::PANEL_BG).inner_margin(16.0))
        .show(ctx, |ui| {
            ui.set_width(280.0);

            ui.colored_label(
                theme::NASA_BLUE,
                egui::RichText::new(t.get(*lang, "reentry.title")).size(18.0).strong(),
            );
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            row(
                ui,
                *lang,
                &t,
                "reentry.altitude",
                &format!("{:.1} км", state.altitude_km),
            );
            row(
                ui,
                *lang,
                &t,
                "reentry.speed",
                &format!("{:.0} м/с", state.speed_ms.max(0.0)),
            );

            ui.add_space(4.0);

            // Угол входа с цветовым индикатором
            let angle_ok = state.entry_angle_deg >= 6.0 && state.entry_angle_deg <= 6.5;
            let angle_color = if angle_ok {
                theme::NASA_BLUE
            } else {
                theme::SLS_ORANGE
            };
            ui.horizontal(|ui| {
                ui.colored_label(theme::TEXT_MUTED, t.get(*lang, "reentry.angle"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.colored_label(
                        angle_color,
                        egui::RichText::new(format!("{:.2}°", state.entry_angle_deg)).strong(),
                    );
                });
            });
            ui.colored_label(
                theme::TEXT_MUTED,
                egui::RichText::new(t.get(*lang, "reentry.angle_target")).size(11.0),
            );

            ui.add_space(6.0);

            // Тепловой щит
            let heat_color = if state.heat_pct < 60.0 {
                theme::NASA_BLUE
            } else if state.heat_pct < 85.0 {
                egui::Color32::from_rgb(0xFF, 0xA0, 0x00)
            } else {
                theme::SLS_ORANGE
            };
            ui.horizontal(|ui| {
                ui.colored_label(theme::TEXT_MUTED, t.get(*lang, "reentry.heat"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.colored_label(
                        heat_color,
                        egui::RichText::new(format!("{:.0}%", state.heat_pct)).strong(),
                    );
                });
            });
            ui.add(egui::ProgressBar::new((state.heat_pct / 100.0).clamp(0.0, 1.0)));

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(4.0);

            match state.phase {
                ReentryPhase::Approach => {
                    ui.colored_label(
                        theme::TEXT_PRIMARY,
                        egui::RichText::new(t.get(*lang, "reentry.hint_angle")).size(13.0),
                    );
                    ui.colored_label(
                        theme::TEXT_MUTED,
                        egui::RichText::new(t.get(*lang, "reentry.commit_hint")).size(11.0),
                    );
                }
                ReentryPhase::Entry => {
                    ui.colored_label(
                        theme::SLS_ORANGE,
                        egui::RichText::new(t.get(*lang, "reentry.committed")).size(13.0).strong(),
                    );
                }
                ReentryPhase::Parachutes => {
                    ui.colored_label(
                        theme::NASA_BLUE,
                        egui::RichText::new(t.get(*lang, "reentry.parachutes"))
                            .size(14.0)
                            .strong(),
                    );
                }
                _ => {}
            }
        });

    Ok(())
}

fn row(ui: &mut egui::Ui, lang: Lang, t: &Translations, key: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.colored_label(theme::TEXT_MUTED, t.get(lang, key));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.colored_label(theme::TEXT_PRIMARY, egui::RichText::new(value).strong());
        });
    });
}
