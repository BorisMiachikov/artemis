use std::time::Duration;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::achievements::AchievementTracker;
use crate::config::{Difficulty, FlybyResult, IcpsParams, TliResult, TransitOutcome};
use crate::events::MissionEvent;
use crate::i18n::{Lang, Translations};
use crate::replay::FlightRecord;
use crate::stages::reentry::ReentryState;
use crate::states::MissionStage;
use crate::ui::hud::MissionTime;
use crate::ui::theme;

/// Причина провала миссии. Пустая — миссия в процессе или завершена успешно.
#[derive(Resource, Default)]
pub struct MissionFailed {
    pub reason: Option<String>,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<MissionFailed>()
        .add_systems(Update, listen_for_abort)
        .add_systems(EguiPrimaryContextPass, (draw_splashdown_screen, draw_gameover_screen));
}

/// Слушаем MissionEvent::Abort и сохраняем причину провала.
fn listen_for_abort(
    mut events: MessageReader<MissionEvent>,
    mut failed: ResMut<MissionFailed>,
) {
    for event in events.read() {
        if let MissionEvent::Abort(reason) = event
            && failed.reason.is_none()
        {
            failed.reason = Some(reason.clone());
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_splashdown_screen(
    mut contexts: EguiContexts,
    stage: Res<State<MissionStage>>,
    mission_time: Res<MissionTime>,
    tli: Res<TliResult>,
    flyby: Res<FlybyResult>,
    icps: Res<IcpsParams>,
    reentry: Res<ReentryState>,
    outcome: Res<TransitOutcome>,
    diff: Res<Difficulty>,
    tracker: Res<AchievementTracker>,
    flight: Res<FlightRecord>,
    lang: Res<Lang>,
    t: Res<Translations>,
    mut next_stage: ResMut<NextState<MissionStage>>,
    keys: Res<ButtonInput<KeyCode>>,
) -> Result {
    if !matches!(stage.get(), MissionStage::Splashdown) {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    egui::Window::new("splashdown_screen")
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::new()
                .fill(theme::PANEL_BG)
                .inner_margin(40.0)
                .stroke(egui::Stroke::new(2.0, theme::NASA_BLUE)),
        )
        .show(ctx, |ui| {
            ui.set_width(520.0);
            ui.vertical_centered(|ui| {
                ui.colored_label(
                    theme::SLS_ORANGE,
                    egui::RichText::new(t.get(*lang, "splashdown.title"))
                        .size(32.0)
                        .strong(),
                );
                ui.add_space(8.0);
                ui.colored_label(
                    theme::TEXT_PRIMARY,
                    egui::RichText::new(t.get(*lang, "splashdown.subtitle")).size(16.0),
                );
            });

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(12.0);

            // --- Mission time (always white) ---
            let total = mission_time.elapsed.as_secs();
            let h = total / 3600;
            let m = (total / 60) % 60;
            let s = total % 60;
            stat_row(ui, *lang, &t, "splashdown.mission_time", &format!("{h:02}:{m:02}:{s:02}"));

            // --- TLI accuracy (colored) ---
            let accuracy = tli.accuracy_pct(icps.target_delta_v_ms);
            let acc_color = if accuracy >= 95.0 {
                theme::STATUS_GREEN
            } else if accuracy >= 80.0 {
                theme::STATUS_YELLOW
            } else {
                theme::SLS_ORANGE
            };
            colored_stat_row(ui, *lang, &t, "splashdown.tli_accuracy", &format!("{accuracy:.1}%"), acc_color);

            // --- Perilune ---
            stat_row(ui, *lang, &t, "splashdown.perilune", &format!("{:.0} км", flyby.perilune_km));

            // --- Entry angle (colored) ---
            let angle = reentry.entry_angle_deg;
            let angle_color = if (6.0..=6.5).contains(&angle) {
                theme::STATUS_GREEN
            } else if (5.5..=7.0).contains(&angle) {
                theme::STATUS_YELLOW
            } else {
                theme::SLS_ORANGE
            };
            colored_stat_row(ui, *lang, &t, "splashdown.entry_angle", &format!("{angle:.2}°"), angle_color);

            // --- Heat shield (colored) ---
            let heat = reentry.heat_pct;
            let heat_color = if heat <= 60.0 {
                theme::STATUS_GREEN
            } else if heat <= 80.0 {
                theme::STATUS_YELLOW
            } else {
                theme::SLS_ORANGE
            };
            colored_stat_row(ui, *lang, &t, "splashdown.heat_shield", &format!("{heat:.0}%"), heat_color);

            // --- MCC corrections ---
            stat_row(ui, *lang, &t, "splashdown.mcc", &outcome.mcc_corrections.to_string());

            // --- Difficulty ---
            let diff_str = match *diff {
                Difficulty::Story => t.get(*lang, "difficulty.story"),
                Difficulty::Realistic => t.get(*lang, "difficulty.realistic"),
            };
            stat_row(ui, *lang, &t, "splashdown.difficulty", diff_str);

            // --- Achievements section ---
            ui.add_space(16.0);
            ui.separator();
            ui.add_space(10.0);

            ui.colored_label(
                theme::NASA_BLUE,
                egui::RichText::new(t.get(*lang, "splashdown.achievements"))
                    .size(15.0)
                    .strong(),
            );
            ui.add_space(6.0);

            for ach in &tracker.list {
                let name = if matches!(*lang, Lang::Ru) { ach.name_ru } else { ach.name_en };
                let desc = if matches!(*lang, Lang::Ru) { ach.desc_ru } else { ach.desc_en };
                let (icon, name_color) = if ach.unlocked {
                    ("✓", theme::STATUS_GREEN)
                } else {
                    ("□", theme::TEXT_MUTED)
                };
                ui.horizontal(|ui| {
                    ui.colored_label(name_color, egui::RichText::new(format!("{icon} {name}")).size(13.0).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.colored_label(theme::TEXT_MUTED, egui::RichText::new(desc).size(11.0));
                    });
                });
            }

            // --- Flight timeline ---
            if !flight.phases.is_empty() {
                ui.add_space(16.0);
                ui.separator();
                ui.add_space(10.0);
                ui.colored_label(
                    theme::NASA_BLUE,
                    egui::RichText::new(t.get(*lang, "replay.timeline"))
                        .size(15.0)
                        .strong(),
                );
                ui.add_space(6.0);
                for rec in &flight.phases {
                    let name = if matches!(*lang, Lang::Ru) { rec.name_ru } else { rec.name_en };
                    ui.horizontal(|ui| {
                        ui.colored_label(
                            theme::TEXT_MUTED,
                            egui::RichText::new(rec.fmt_time()).size(12.0),
                        );
                        ui.colored_label(
                            theme::TEXT_PRIMARY,
                            egui::RichText::new(name).size(12.0).strong(),
                        );
                        if !rec.detail.is_empty() {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.colored_label(
                                    theme::STATUS_GREEN,
                                    egui::RichText::new(&rec.detail).size(12.0),
                                );
                            });
                        }
                    });
                }
            }

            ui.add_space(20.0);

            ui.vertical_centered(|ui| {
                let btn = egui::Button::new(
                    egui::RichText::new(t.get(*lang, "splashdown.restart"))
                        .size(18.0)
                        .strong()
                        .color(theme::TEXT_PRIMARY),
                )
                .min_size(egui::vec2(220.0, 44.0))
                .fill(theme::NASA_BLUE);

                if ui.add(btn).clicked() || keys.just_pressed(KeyCode::Space) {
                    next_stage.set(MissionStage::Prelaunch);
                }
            });
        });

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn draw_gameover_screen(
    mut contexts: EguiContexts,
    mut failed: ResMut<MissionFailed>,
    stage: Res<State<MissionStage>>,
    lang: Res<Lang>,
    t: Res<Translations>,
    mut next_stage: ResMut<NextState<MissionStage>>,
    mut tli: ResMut<TliResult>,
    mut outcome: ResMut<TransitOutcome>,
    mut mission_time: ResMut<MissionTime>,
    keys: Res<ButtonInput<KeyCode>>,
) -> Result {
    let Some(reason) = failed.reason.clone() else {
        return Ok(());
    };
    if matches!(stage.get(), MissionStage::Splashdown) {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    egui::Window::new("gameover_screen")
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::new()
                .fill(theme::PANEL_BG)
                .inner_margin(40.0)
                .stroke(egui::Stroke::new(2.0, theme::SLS_ORANGE)),
        )
        .show(ctx, |ui| {
            ui.set_width(440.0);
            ui.vertical_centered(|ui| {
                ui.colored_label(
                    theme::SLS_ORANGE,
                    egui::RichText::new(t.get(*lang, "gameover.title"))
                        .size(28.0)
                        .strong(),
                );
                ui.add_space(10.0);
                ui.colored_label(
                    theme::TEXT_MUTED,
                    egui::RichText::new(reason.clone()).size(14.0),
                );
            });

            ui.add_space(24.0);
            ui.vertical_centered(|ui| {
                let btn = egui::Button::new(
                    egui::RichText::new(t.get(*lang, "gameover.restart"))
                        .size(18.0)
                        .strong()
                        .color(theme::TEXT_PRIMARY),
                )
                .min_size(egui::vec2(200.0, 44.0))
                .fill(theme::SLS_ORANGE);

                if ui.add(btn).clicked() || keys.just_pressed(KeyCode::Enter) {
                    // Сброс ключевых ресурсов
                    *tli = TliResult::default();
                    *outcome = TransitOutcome::default();
                    mission_time.elapsed = Duration::ZERO;
                    *failed = MissionFailed::default();
                    next_stage.set(MissionStage::Prelaunch);
                }
            });
        });

    Ok(())
}

fn stat_row(ui: &mut egui::Ui, lang: Lang, t: &Translations, key: &str, value: &str) {
    colored_stat_row(ui, lang, t, key, value, theme::TEXT_PRIMARY);
}

fn colored_stat_row(
    ui: &mut egui::Ui,
    lang: Lang,
    t: &Translations,
    key: &str,
    value: &str,
    color: egui::Color32,
) {
    ui.horizontal(|ui| {
        ui.colored_label(theme::TEXT_MUTED, egui::RichText::new(t.get(lang, key)).size(14.0));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.colored_label(color, egui::RichText::new(value).size(14.0).strong());
        });
    });
}
