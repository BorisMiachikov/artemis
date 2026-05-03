use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::config::{TransitOutcome, WarpLevel, WARP_LEVELS};
use crate::i18n::{Lang, Translations};
use crate::physics::trajectory::TrajectorySim;
use crate::stages::transit::TransitState;
use crate::states::MissionStage;
use crate::ui::theme;

pub fn plugin(app: &mut App) {
    app.add_systems(EguiPrimaryContextPass, draw_transit_panel);
}

#[allow(clippy::too_many_arguments)]
fn draw_transit_panel(
    mut contexts: EguiContexts,
    stage: Res<State<MissionStage>>,
    state: Res<TransitState>,
    sim: Res<TrajectorySim>,
    warp: Res<WarpLevel>,
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
            ui.set_width(300.0);

            ui.colored_label(
                theme::NASA_BLUE,
                egui::RichText::new(t.get(*lang, "transit.title"))
                    .size(18.0)
                    .strong(),
            );
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            // Warp
            let warp_label = format!("×{}", WARP_LEVELS[warp.0] as u32);
            row(ui, *lang, &t, "transit.warp", &warp_label);
            ui.colored_label(
                theme::TEXT_MUTED,
                egui::RichText::new(t.get(*lang, "transit.warp_hint")).size(10.0),
            );
            ui.add_space(4.0);

            // Дистанции из TrajectorySim
            let dist_earth_km = sim.orion_pos_km.length();
            let dist_moon_km = (sim.orion_pos_km - sim.moon_pos_km).length();
            let dist_earth = fmt_km(dist_earth_km);
            let dist_moon = fmt_km(dist_moon_km);
            row(ui, *lang, &t, "hud.distance_earth", &dist_earth);
            row(ui, *lang, &t, "hud.distance_moon", &dist_moon);

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // Predicted perilune
            let perilune_str = if sim.predicted_perilune_km < 999_000.0 {
                fmt_km(sim.predicted_perilune_km)
            } else {
                "—".to_owned()
            };
            let perilune_color = if sim.predicted_perilune_km < 20_000.0 {
                theme::NASA_BLUE
            } else if sim.predicted_perilune_km < 80_000.0 {
                egui::Color32::from_rgb(0xFF, 0xA0, 0x00)
            } else {
                theme::SLS_ORANGE
            };
            ui.horizontal(|ui| {
                ui.colored_label(
                    theme::TEXT_MUTED,
                    t.get(*lang, "transit.predicted_perilune"),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.colored_label(
                        perilune_color,
                        egui::RichText::new(&perilune_str).strong(),
                    );
                });
            });

            // MCC fuel
            row(
                ui,
                *lang,
                &t,
                "transit.mcc_fuel",
                &format!("{:.0} кг", sim.mcc_fuel_kg),
            );
            ui.colored_label(
                theme::TEXT_MUTED,
                egui::RichText::new(t.get(*lang, "transit.mcc_hint")).size(11.0),
            );

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // CO₂ и радиация
            row(
                ui,
                *lang,
                &t,
                "transit.co2",
                &format!("{:.0}", state.co2_ppm),
            );
            row(
                ui,
                *lang,
                &t,
                "transit.radiation",
                &format!("{:.1}", state.radiation_msv),
            );

            // MCC count
            if outcome.mcc_corrections > 0 {
                ui.add_space(4.0);
                ui.colored_label(
                    theme::TEXT_MUTED,
                    egui::RichText::new(format!("MCC: {}", outcome.mcc_corrections)).size(11.0),
                );
            }

            // Алерт события
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

fn fmt_km(km: f64) -> String {
    if km < 10_000.0 {
        format!("{:.0} км", km)
    } else {
        format!("{:.0} тыс. км", km / 1_000.0)
    }
}

fn row(ui: &mut egui::Ui, lang: Lang, t: &Translations, key: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.colored_label(theme::TEXT_MUTED, t.get(lang, key));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.colored_label(theme::TEXT_PRIMARY, egui::RichText::new(value).strong());
        });
    });
}
