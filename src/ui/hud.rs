use std::time::Duration;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::config::{SlsParams, TimeScale};
use crate::i18n::{Lang, Translations};
use crate::physics::rocket::{FlightDynamics, Rocket};
use crate::states::MissionStage;
use crate::ui::theme;

/// Время с момента старта миссии (T+). Тикает только в Launch и далее.
/// Учитывает игровое ускорение времени `TimeScale`.
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
    timescale: Res<TimeScale>,
    stage: Res<State<MissionStage>>,
    mut mission_time: ResMut<MissionTime>,
) {
    let active = !matches!(
        stage.get(),
        MissionStage::Loading | MissionStage::Prelaunch
    );
    if !active {
        return;
    }
    let scaled = time.delta().as_secs_f64() * timescale.multiplier as f64;
    mission_time.elapsed += Duration::from_secs_f64(scaled);
}

#[allow(clippy::too_many_arguments)] // HUD читает state, время, физику и i18n — всё нужно
fn draw_hud(
    mut contexts: EguiContexts,
    mission_time: Res<MissionTime>,
    timescale: Res<TimeScale>,
    stage: Res<State<MissionStage>>,
    params: Res<SlsParams>,
    rockets: Query<(&Rocket, &FlightDynamics)>,
    lang: Res<Lang>,
    t: Res<Translations>,
) -> Result {
    if matches!(stage.get(), MissionStage::Loading) {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    egui::TopBottomPanel::top("hud_top")
        .frame(egui::Frame::new().fill(theme::PANEL_BG).inner_margin(8.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
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

                ui.colored_label(
                    theme::TEXT_MUTED,
                    egui::RichText::new(format!("×{:.0}", timescale.multiplier)).size(16.0),
                );

                ui.separator();

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

    if matches!(stage.get(), MissionStage::Prelaunch) {
        return Ok(());
    }

    // Окно телеметрии. Если ракеты на сцене нет (этапы после MECO без Rocket-сущности),
    // показываем нули — поведение совпадает с Phase 2 заглушками.
    let (speed, altitude, thrust_pct, fuel_pct, gload, pitch) = match rockets.single() {
        Ok((rocket, d)) => (
            format!("{:.0} м/с", d.speed_ms),
            format!("{:.1} км", d.altitude_m / 1000.0),
            format!("{:.0} %", rocket.thrust_kn / params.thrust_total_kn * 100.0),
            format!(
                "{:.0} %",
                if rocket.fuel_initial_kg > 0.0 {
                    rocket.fuel_kg / rocket.fuel_initial_kg * 100.0
                } else {
                    0.0
                }
            ),
            format!("{:.1} G", d.g_load),
            format!("{:.0}°", d.pitch_deg),
        ),
        Err(_) => (
            "0 м/с".into(),
            "0 км".into(),
            "0 %".into(),
            "100 %".into(),
            "0.0 G".into(),
            "0°".into(),
        ),
    };

    egui::Window::new("telemetry")
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::RIGHT_TOP, [-12.0, 64.0])
        .frame(egui::Frame::new().fill(theme::PANEL_BG).inner_margin(12.0))
        .show(ctx, |ui| {
            ui.set_width(220.0);
            row(ui, *lang, &t, "hud.speed", &speed);
            row(ui, *lang, &t, "hud.altitude", &altitude);
            row(ui, *lang, &t, "hud.thrust", &thrust_pct);
            row(ui, *lang, &t, "hud.fuel", &fuel_pct);
            row(ui, *lang, &t, "hud.gload", &gload);
            row(ui, *lang, &t, "hud.pitch", &pitch);
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
