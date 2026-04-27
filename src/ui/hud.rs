use std::time::Duration;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::i18n::{Lang, Translations};
use crate::states::MissionStage;
use crate::ui::theme;

/// Время с момента старта миссии (T+). Тикает только в Launch и далее.
#[derive(Resource, Default)]
pub struct MissionTime {
    pub elapsed: Duration,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<MissionTime>()
        .add_systems(Update, tick_mission_time)
        .add_systems(EguiPrimaryContextPass, draw_hud);
}

fn tick_mission_time(
    time: Res<Time>,
    stage: Res<State<MissionStage>>,
    mut mission_time: ResMut<MissionTime>,
) {
    let active = !matches!(
        stage.get(),
        MissionStage::Loading | MissionStage::Prelaunch
    );
    if active {
        mission_time.elapsed += time.delta();
    }
}

fn draw_hud(
    mut contexts: EguiContexts,
    mission_time: Res<MissionTime>,
    stage: Res<State<MissionStage>>,
    lang: Res<Lang>,
    t: Res<Translations>,
) -> Result {
    // В Loading HUD не показываем — там пока что нет данных.
    if matches!(stage.get(), MissionStage::Loading) {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    egui::TopBottomPanel::top("hud_top")
        .frame(egui::Frame::new().fill(theme::PANEL_BG).inner_margin(8.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // T+ таймер
                let total = mission_time.elapsed.as_secs();
                let h = total / 3600;
                let m = (total / 60) % 60;
                let s = total % 60;
                ui.colored_label(
                    theme::SLS_ORANGE,
                    egui::RichText::new(format!(
                        "{} {:02}:{:02}:{:02}",
                        t.get(*lang, "hud.time"),
                        h,
                        m,
                        s
                    ))
                    .size(20.0)
                    .strong(),
                );

                ui.separator();

                // Текущий этап миссии
                let stage_key = match stage.get() {
                    MissionStage::Loading => "stage.loading",
                    MissionStage::Prelaunch => "stage.prelaunch",
                    MissionStage::Launch => "stage.launch",
                    MissionStage::Orbit => "stage.orbit",
                    MissionStage::TLI => "stage.tli",
                    MissionStage::Transit => "stage.transit",
                    MissionStage::LunarFlyby => "stage.lunar_flyby",
                    MissionStage::Reentry => "stage.reentry",
                    MissionStage::Splashdown => "stage.splashdown",
                };
                ui.colored_label(
                    theme::NASA_BLUE,
                    egui::RichText::new(t.get(*lang, stage_key))
                        .size(18.0)
                        .strong(),
                );
            });
        });

    // Телеметрия — пока заглушки, заполнятся в Фазе 3.
    if !matches!(stage.get(), MissionStage::Prelaunch) {
        egui::Window::new("telemetry")
            .title_bar(false)
            .resizable(false)
            .anchor(egui::Align2::RIGHT_TOP, [-12.0, 64.0])
            .frame(egui::Frame::new().fill(theme::PANEL_BG).inner_margin(12.0))
            .show(ctx, |ui| {
                ui.set_width(200.0);
                row(ui, *lang, &t, "hud.speed", "0 м/с");
                row(ui, *lang, &t, "hud.altitude", "0 км");
                row(ui, *lang, &t, "hud.thrust", "0 %");
                row(ui, *lang, &t, "hud.fuel", "100 %");
                row(ui, *lang, &t, "hud.gload", "0.0 G");
                row(ui, *lang, &t, "hud.pitch", "0°");
            });
    }

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
