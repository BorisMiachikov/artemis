use std::time::Duration;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::config::{FlybyResult, TliResult, TransitOutcome};
use crate::events::MissionEvent;
use crate::i18n::{Lang, Translations};
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
        if let MissionEvent::Abort(reason) = event {
            if failed.reason.is_none() {
                failed.reason = Some(reason.clone());
            }
        }
    }
}

fn draw_splashdown_screen(
    mut contexts: EguiContexts,
    stage: Res<State<MissionStage>>,
    mission_time: Res<MissionTime>,
    tli: Res<TliResult>,
    flyby: Res<FlybyResult>,
    icps: Res<crate::config::IcpsParams>,
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
            ui.set_width(500.0);
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
            ui.add_space(16.0);

            let total = mission_time.elapsed.as_secs();
            let h = total / 3600;
            let m = (total / 60) % 60;
            let s = total % 60;
            stat_row(
                ui,
                *lang,
                &t,
                "splashdown.mission_time",
                &format!("{:02}:{:02}:{:02}", h, m, s),
            );
            stat_row(
                ui,
                *lang,
                &t,
                "splashdown.tli_accuracy",
                &format!("{:.1}%", tli.accuracy_pct(icps.target_delta_v_ms)),
            );
            stat_row(
                ui,
                *lang,
                &t,
                "splashdown.perilune",
                &format!("{:.0} км", flyby.perilune_km),
            );

            ui.add_space(24.0);

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

fn draw_gameover_screen(
    mut contexts: EguiContexts,
    failed: Res<MissionFailed>,
    stage: Res<State<MissionStage>>,
    lang: Res<Lang>,
    t: Res<Translations>,
    mut next_stage: ResMut<NextState<MissionStage>>,
    mut failed_resource: ResMut<MissionFailed>,
    mut tli: ResMut<TliResult>,
    mut outcome: ResMut<TransitOutcome>,
    mut mission_time: ResMut<MissionTime>,
    keys: Res<ButtonInput<KeyCode>>,
) -> Result {
    let Some(ref reason) = failed.reason else {
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
                    egui::RichText::new(reason.as_str()).size(14.0),
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
                    *failed_resource = MissionFailed::default();
                    next_stage.set(MissionStage::Prelaunch);
                }
            });
        });

    Ok(())
}

fn stat_row(ui: &mut egui::Ui, lang: Lang, t: &Translations, key: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.colored_label(theme::TEXT_MUTED, egui::RichText::new(t.get(lang, key)).size(14.0));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.colored_label(
                theme::TEXT_PRIMARY,
                egui::RichText::new(value).size(14.0).strong(),
            );
        });
    });
}
