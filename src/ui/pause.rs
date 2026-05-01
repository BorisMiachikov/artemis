use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::i18n::{Lang, Translations};
use crate::states::MissionStage;
use crate::ui::theme;

#[derive(Resource, Default)]
pub struct Paused(pub bool);

/// ESC работает во всех стейтах, кроме Loading и MainMenu.
fn is_pauseable(stage: &MissionStage) -> bool {
    !matches!(stage, MissionStage::Loading | MissionStage::MainMenu)
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Paused>()
        .add_systems(Update, toggle_pause)
        .add_systems(EguiPrimaryContextPass, draw_pause_overlay);
}

fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    mut paused: ResMut<Paused>,
    mut time: ResMut<Time<Virtual>>,
    stage: Res<State<MissionStage>>,
) {
    if !is_pauseable(stage.get()) {
        // Сбрасываем паузу при выходе за пределы pauseable-стейтов.
        if paused.0 {
            paused.0 = false;
            time.unpause();
        }
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        paused.0 = !paused.0;
        if paused.0 {
            time.pause();
        } else {
            time.unpause();
        }
    }
}

fn draw_pause_overlay(
    mut contexts: EguiContexts,
    mut paused: ResMut<Paused>,
    mut time: ResMut<Time<Virtual>>,
    mut next_stage: ResMut<NextState<MissionStage>>,
    lang: Res<Lang>,
    t: Res<Translations>,
) -> Result {
    let cur_lang = *lang;
    if !paused.0 {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;
    let screen = ctx.input(|i| {
        i.viewport().inner_rect
            .or(i.viewport().outer_rect)
            .unwrap_or(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1920.0, 1080.0)))
    });

    // Полупрозрачная подложка на слое Foreground.
    ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, "pause_dim".into()))
        .rect_filled(screen, 0.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 170));

    // Панель паузы поверх подложки (TOP_MOST).
    let mut resume   = false;
    let mut go_menu  = false;
    let mut do_exit  = false;

    egui::Area::new("pause_panel".into())
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .order(egui::Order::Tooltip) // выше Foreground → поверх диммера
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(theme::PANEL_BG)
                .inner_margin(egui::Margin::same(32))
                .corner_radius(6.0)
                .show(ui, |ui| {
                    ui.set_width(260.0);

                    ui.vertical_centered(|ui| {
                        ui.colored_label(
                            theme::SLS_ORANGE,
                            egui::RichText::new(t.get(cur_lang, "pause.title"))
                                .size(24.0)
                                .strong(),
                        );
                    });

                    ui.add_space(20.0);

                    // Продолжить
                    let resume_btn = egui::Button::new(
                        egui::RichText::new(t.get(cur_lang, "pause.resume"))
                            .size(16.0)
                            .strong()
                            .color(theme::TEXT_PRIMARY),
                    )
                    .min_size(egui::vec2(196.0, 38.0))
                    .fill(theme::NASA_BLUE);
                    if ui.add(resume_btn).clicked() {
                        resume = true;
                    }

                    ui.add_space(8.0);

                    // Главное меню
                    let menu_btn = egui::Button::new(
                        egui::RichText::new(t.get(cur_lang, "pause.main_menu"))
                            .size(14.0)
                            .color(theme::TEXT_MUTED),
                    )
                    .min_size(egui::vec2(196.0, 32.0))
                    .fill(egui::Color32::TRANSPARENT)
                    .stroke(egui::Stroke::new(1.0, theme::TEXT_MUTED));
                    if ui.add(menu_btn).clicked() {
                        go_menu = true;
                    }

                    ui.add_space(8.0);

                    // Выход
                    let exit_btn = egui::Button::new(
                        egui::RichText::new(t.get(cur_lang, "pause.exit"))
                            .size(13.0)
                            .color(theme::TEXT_MUTED),
                    )
                    .min_size(egui::vec2(196.0, 28.0))
                    .fill(egui::Color32::TRANSPARENT);
                    if ui.add(exit_btn).clicked() {
                        do_exit = true;
                    }
                });
        });

    // Обработка кнопок после отрисовки.
    if resume {
        paused.0 = false;
        time.unpause();
    }
    if go_menu {
        paused.0 = false;
        time.unpause();
        next_stage.set(MissionStage::MainMenu);
    }
    if do_exit {
        std::process::exit(0);
    }

    Ok(())
}
