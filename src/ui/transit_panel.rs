use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::config::TransitOutcome;
use crate::i18n::{Lang, Translations};
use crate::stages::transit::TransitState;
use crate::states::MissionStage;
use crate::ui::theme;

pub fn plugin(app: &mut App) {
    app.add_systems(EguiPrimaryContextPass, draw_transit_panel);
}

fn draw_transit_panel(
    mut contexts: EguiContexts,
    stage: Res<State<MissionStage>>,
    state: Res<TransitState>,
    outcome: Res<TransitOutcome>,
    lang: Res<Lang>,
    t: Res<Translations>,
) -> Result {
    if !matches!(stage.get(), MissionStage::Transit) {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    egui::Window::new("transit_panel")
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::RIGHT_CENTER, [-12.0, 0.0])
        .frame(egui::Frame::new().fill(theme::PANEL_BG).inner_margin(16.0))
        .show(ctx, |ui| {
            ui.set_width(280.0);

            ui.colored_label(
                theme::NASA_BLUE,
                egui::RichText::new(t.get(*lang, "transit.title")).size(18.0).strong(),
            );
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            let dist_earth = if state.dist_earth_km < 10_000.0 {
                format!("{:.0} км", state.dist_earth_km)
            } else {
                format!("{:.0} тыс. км", state.dist_earth_km / 1_000.0)
            };
            let dist_moon = if state.dist_moon_km < 10_000.0 {
                format!("{:.0} км", state.dist_moon_km)
            } else {
                format!("{:.0} тыс. км", state.dist_moon_km / 1_000.0)
            };

            row(ui, *lang, &t, "hud.distance_earth", &dist_earth);
            row(ui, *lang, &t, "hud.distance_moon", &dist_moon);
            row(ui, *lang, &t, "transit.mcc_fuel", &format!("{:.0} кг", state.mcc_fuel_kg));
            row(ui, *lang, &t, "transit.co2", &format!("{:.0}", state.co2_ppm));
            row(ui, *lang, &t, "transit.radiation", &format!("{:.1}", state.radiation_msv));

            ui.add_space(6.0);

            // Точность траектории (из MCC)
            let accuracy_pct = (1.0 - outcome.trajectory_error) * 100.0;
            let acc_color = if accuracy_pct >= 80.0 {
                theme::NASA_BLUE
            } else if accuracy_pct >= 50.0 {
                egui::Color32::from_rgb(0xFF, 0xA0, 0x00)
            } else {
                theme::SLS_ORANGE
            };
            ui.horizontal(|ui| {
                ui.colored_label(theme::TEXT_MUTED, "ТОЧНОСТЬ:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.colored_label(
                        acc_color,
                        egui::RichText::new(format!("{:.0}%", accuracy_pct)).strong(),
                    );
                });
            });

            ui.add_space(6.0);
            ui.colored_label(
                theme::TEXT_MUTED,
                egui::RichText::new(t.get(*lang, "transit.mcc_hint")).size(11.0),
            );

            // Показываем событие если есть
            if let Some(key) = state.event_msg {
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);
                ui.colored_label(
                    theme::SLS_ORANGE,
                    egui::RichText::new(t.get(*lang, key)).size(13.0).strong(),
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
