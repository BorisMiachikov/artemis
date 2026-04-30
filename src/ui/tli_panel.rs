use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::config::{G0, IcpsParams, TliResult};
use crate::i18n::{Lang, Translations};
use crate::stages::tli::{IcpsBurn, TliBurnState, TliWindow};
use crate::states::MissionStage;
use crate::ui::theme;

pub fn plugin(app: &mut App) {
    app.add_systems(EguiPrimaryContextPass, draw_tli_panel);
}

#[allow(clippy::too_many_arguments)]
fn draw_tli_panel(
    mut contexts: EguiContexts,
    stage: Res<State<MissionStage>>,
    burn_state: Res<TliBurnState>,
    icps: Res<IcpsParams>,
    tli: Res<TliResult>,
    burn: Res<IcpsBurn>,
    window: Res<TliWindow>,
    lang: Res<Lang>,
    t: Res<Translations>,
) -> Result {
    if !matches!(stage.get(), MissionStage::TLI) {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;
    egui::Window::new("tli_panel")
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -28.0])
        .frame(egui::Frame::new().fill(theme::PANEL_BG).inner_margin(20.0))
        .show(ctx, |ui| {
            ui.set_width(420.0);
            ui.vertical_centered(|ui| {
                ui.colored_label(
                    theme::SLS_ORANGE,
                    egui::RichText::new(t.get(*lang, "tli.title"))
                        .size(22.0)
                        .strong(),
                );
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            let progress = (tli.burn_duration_s / icps.target_burn_duration_s).clamp(0.0, 1.0);

            // Расчёт топлива ICPS (доступен во всех состояниях).
            let prop_total = icps.stack_mass_kg - icps.stack_dry_mass_kg;
            let prop_rem = (burn.mass_kg - icps.stack_dry_mass_kg).max(0.0);
            let prop_pct = if prop_total > 0.0 { prop_rem / prop_total * 100.0 } else { 0.0 };
            let mass_flow = (icps.thrust_kn * 1_000.0) / (icps.isp_s * G0);

            // Орбита — 200 км над поверхностью; burn происходит в перицентре, поэтому
            // дистанция до центра Земли меняется слабо. Показываем лёгкий рост по Δv.
            let dist_km = 6_371.0 + 200.0 + tli.delta_v_ms * 0.04;

            match *burn_state {
                TliBurnState::Idle => {
                    if !window.window_open {
                        ui.colored_label(
                            theme::TEXT_MUTED,
                            egui::RichText::new(format!(
                                "T−  {:.0} с до открытия окна",
                                window.countdown_s
                            ))
                            .size(16.0),
                        );
                    } else {
                        ui.colored_label(
                            theme::TEXT_PRIMARY,
                            egui::RichText::new(t.get(*lang, "tli.start_hint"))
                                .size(16.0)
                                .strong(),
                        );
                    }
                    ui.add_space(6.0);
                    row(ui, *lang, &t, "tli.target",
                        &format!("{:.0} м/с", icps.target_delta_v_ms));
                    row(ui, *lang, &t, "tli.distance_earth",
                        &format!("{:.0} км", dist_km));
                    row(ui, *lang, &t, "tli.propellant",
                        &format!("{:.0} кг  ({:.0}%)", prop_rem, prop_pct));
                }
                TliBurnState::Burning => {
                    ui.colored_label(
                        theme::SLS_ORANGE,
                        egui::RichText::new(t.get(*lang, "tli.burning"))
                            .size(18.0)
                            .strong(),
                    );
                    ui.add_space(6.0);
                    ui.add(egui::ProgressBar::new(progress).show_percentage());
                    ui.add_space(6.0);
                    row(ui, *lang, &t, "tli.delta_v",
                        &format!("{:.0} м/с", tli.delta_v_ms));
                    row(ui, *lang, &t, "tli.burn_time",
                        &format!("{:.0} / {:.0} с",
                            tli.burn_duration_s, icps.target_burn_duration_s));
                    row(ui, *lang, &t, "tli.distance_earth",
                        &format!("{:.0} км", dist_km));
                    row(ui, *lang, &t, "tli.propellant",
                        &format!("{:.0} кг  ({:.0}%)", prop_rem, prop_pct));
                    row(ui, *lang, &t, "tli.mass_flow",
                        &format!("{:.1} кг/с", mass_flow));
                }
                TliBurnState::Completed => {
                    ui.colored_label(
                        theme::NASA_BLUE,
                        egui::RichText::new(t.get(*lang, "tli.complete"))
                            .size(18.0)
                            .strong(),
                    );
                    ui.add_space(6.0);
                    row(ui, *lang, &t, "tli.delta_v",
                        &format!("{:.0} м/с", tli.delta_v_ms));
                    row(ui, *lang, &t, "tli.accuracy",
                        &format!("{:.1} %", tli.accuracy_pct(icps.target_delta_v_ms)));
                    row(ui, *lang, &t, "tli.distance_earth",
                        &format!("{:.0} км", dist_km));
                    row(ui, *lang, &t, "tli.propellant",
                        &format!("{:.0} кг  ({:.0}%)", prop_rem, prop_pct));
                }
            }
        });

    Ok(())
}

fn row(ui: &mut egui::Ui, lang: Lang, t: &Translations, key: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.colored_label(theme::TEXT_MUTED, t.get(lang, key));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.colored_label(
                theme::TEXT_PRIMARY,
                egui::RichText::new(value).strong(),
            );
        });
    });
}
