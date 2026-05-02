//! Боковая панель в Launch со стилизованными кнопками `=`/`−`/`X`:
//! дроссель RS-25 и manual MECO. Кликаются мышью И активируются с клавиатуры
//! (см. `input::handle_throttle_input` и `handle_engine_cutoff`).

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::events::MissionEvent;
use crate::i18n::{Lang, Translations};
use crate::physics::rocket::{FlightPhase, Rocket, THROTTLE_MAX, THROTTLE_MIN};
use crate::states::MissionStage;
use crate::ui::theme;

/// Шаг изменения дросселя при клике мышью.
const STEP_PER_CLICK: f32 = 0.05;

pub fn plugin(app: &mut App) {
    app.add_systems(
        EguiPrimaryContextPass,
        draw_panel.run_if(in_state(MissionStage::Launch)),
    );
}

fn draw_panel(
    mut contexts: EguiContexts,
    mut rockets: Query<&mut Rocket>,
    mut events: MessageWriter<MissionEvent>,
    mut next_stage: ResMut<NextState<MissionStage>>,
    lang: Res<Lang>,
    t: Res<Translations>,
) -> Result {
    let Ok(mut rocket) = rockets.single_mut() else {
        return Ok(());
    };
    let throttle_active = matches!(rocket.phase, FlightPhase::CoreOnly);
    let cutoff_active = !matches!(rocket.phase, FlightPhase::Coast);
    let cur = *lang;

    let ctx = contexts.ctx_mut()?;
    egui::Window::new("launch_controls")
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::RIGHT_CENTER, [-12.0, 0.0])
        .frame(egui::Frame::new().fill(theme::PANEL_BG).inner_margin(12.0))
        .show(ctx, |ui| {
            ui.set_width(96.0);
            ui.vertical_centered(|ui| {
                ui.colored_label(
                    theme::TEXT_MUTED,
                    egui::RichText::new(t.get(cur, "controls.throttle"))
                        .size(11.0)
                        .strong(),
                );
                ui.add_space(2.0);

                let throttle_color = if throttle_active {
                    theme::TEXT_PRIMARY
                } else {
                    theme::TEXT_MUTED
                };
                ui.colored_label(
                    throttle_color,
                    egui::RichText::new(format!("{:.0}%", rocket.throttle_pct * 100.0))
                        .size(22.0)
                        .strong(),
                );
                ui.add_space(8.0);

                // [+] — клавиша =
                if key_button(ui, "+", throttle_active)
                    .on_hover_text(t.get(cur, "controls.throttle_up_hint"))
                    .clicked()
                {
                    rocket.throttle_pct =
                        (rocket.throttle_pct + STEP_PER_CLICK).min(THROTTLE_MAX);
                }
                ui.add_space(4.0);

                // [−] — клавиша -
                if key_button(ui, "−", throttle_active)
                    .on_hover_text(t.get(cur, "controls.throttle_down_hint"))
                    .clicked()
                {
                    rocket.throttle_pct =
                        (rocket.throttle_pct - STEP_PER_CLICK).max(THROTTLE_MIN);
                }
                ui.add_space(12.0);

                ui.separator();
                ui.add_space(8.0);

                ui.colored_label(
                    theme::TEXT_MUTED,
                    egui::RichText::new(t.get(cur, "controls.cutoff"))
                        .size(10.0)
                        .strong(),
                );
                ui.add_space(2.0);

                // [X] — клавиша X — отдельный «опасный» цвет SLS_ORANGE.
                if cutoff_button(ui, cutoff_active)
                    .on_hover_text(t.get(cur, "controls.cutoff_hint"))
                    .clicked()
                {
                    rocket.phase = FlightPhase::Coast;
                    rocket.thrust_kn = 0.0;
                    events.write(MissionEvent::Meco);
                    next_stage.set(MissionStage::Orbit);
                }
            });
        });

    Ok(())
}

/// Кнопка-капс клавиши. Активная — синий фон + светлая обводка; неактивная —
/// прозрачная с серой обводкой и приглушённым лейблом.
fn key_button(ui: &mut egui::Ui, label: &str, active: bool) -> egui::Response {
    let (fg, bg, stroke) = if active {
        (
            theme::TEXT_PRIMARY,
            theme::NASA_BLUE,
            egui::Stroke::new(1.0, theme::TEXT_PRIMARY),
        )
    } else {
        (
            theme::TEXT_MUTED,
            egui::Color32::TRANSPARENT,
            egui::Stroke::new(1.0, theme::TEXT_MUTED),
        )
    };
    let btn = egui::Button::new(
        egui::RichText::new(label)
            .size(20.0)
            .strong()
            .monospace()
            .color(fg),
    )
    .min_size(egui::vec2(56.0, 36.0))
    .fill(bg)
    .stroke(stroke)
    .corner_radius(4.0);
    ui.add_enabled(active, btn)
}

/// Кнопка аварийного выключения двигателя — оранжевая, чтобы выделять.
fn cutoff_button(ui: &mut egui::Ui, active: bool) -> egui::Response {
    let (fg, bg, stroke) = if active {
        (
            theme::TEXT_PRIMARY,
            theme::SLS_ORANGE,
            egui::Stroke::new(1.0, theme::TEXT_PRIMARY),
        )
    } else {
        (
            theme::TEXT_MUTED,
            egui::Color32::TRANSPARENT,
            egui::Stroke::new(1.0, theme::TEXT_MUTED),
        )
    };
    let btn = egui::Button::new(
        egui::RichText::new("X")
            .size(20.0)
            .strong()
            .monospace()
            .color(fg),
    )
    .min_size(egui::vec2(56.0, 36.0))
    .fill(bg)
    .stroke(stroke)
    .corner_radius(4.0);
    ui.add_enabled(active, btn)
}
