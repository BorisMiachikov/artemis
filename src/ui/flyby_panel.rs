use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::i18n::{Lang, Translations};
use crate::stages::lunar_flyby::{FlybyPhase, FlybyState};
use crate::states::MissionStage;
use crate::ui::theme;

pub fn plugin(app: &mut App) {
    app.add_systems(EguiPrimaryContextPass, draw_flyby_panel);
}

fn draw_flyby_panel(
    mut contexts: EguiContexts,
    stage: Res<State<MissionStage>>,
    state: Res<FlybyState>,
    lang: Res<Lang>,
    t: Res<Translations>,
) -> Result {
    if !matches!(stage.get(), MissionStage::LunarFlyby) {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    egui::Window::new("flyby_panel")
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::RIGHT_CENTER, [-12.0, 0.0])
        .frame(egui::Frame::new().fill(theme::PANEL_BG).inner_margin(16.0))
        .show(ctx, |ui| {
            ui.set_width(260.0);

            ui.colored_label(
                theme::NASA_BLUE,
                egui::RichText::new(t.get(*lang, "flyby.title")).size(18.0).strong(),
            );
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            // Фаза
            let phase_key = match state.phase {
                FlybyPhase::Approach => "flyby.phase.approach",
                FlybyPhase::Perilune => "flyby.phase.perilune",
                FlybyPhase::Departure => "flyby.phase.departure",
            };
            let phase_color = match state.phase {
                FlybyPhase::Approach => theme::TEXT_PRIMARY,
                FlybyPhase::Perilune => theme::SLS_ORANGE,
                FlybyPhase::Departure => theme::NASA_BLUE,
            };
            ui.horizontal(|ui| {
                ui.colored_label(theme::TEXT_MUTED, "ФАЗА:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.colored_label(
                        phase_color,
                        egui::RichText::new(t.get(*lang, phase_key)).strong(),
                    );
                });
            });

            ui.add_space(4.0);

            let dist_str = if state.approach_km < 1_000.0 {
                format!("{:.0} км", state.approach_km)
            } else {
                format!("{:.0} тыс. км", state.approach_km / 1_000.0)
            };
            row(ui, *lang, &t, "flyby.dist_moon", &dist_str);
            row(ui, *lang, &t, "flyby.speed", &format!("{:.2} км/с", state.speed_kms));

            if matches!(state.phase, FlybyPhase::Perilune | FlybyPhase::Departure) {
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(4.0);
                row(
                    ui,
                    *lang,
                    &t,
                    "flyby.perilune",
                    &format!("{:.0} км", state.perilune_km),
                );
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
