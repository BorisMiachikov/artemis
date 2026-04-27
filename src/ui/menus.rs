use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::config::Difficulty;
use crate::events::MissionEvent;
use crate::i18n::{Lang, Translations};
use crate::states::MissionStage;
use crate::ui::theme;

pub fn plugin(app: &mut App) {
    app.add_systems(EguiPrimaryContextPass, (draw_main_menu, draw_settings));
}

fn draw_main_menu(
    mut contexts: EguiContexts,
    stage: Res<State<MissionStage>>,
    mut next_stage: ResMut<NextState<MissionStage>>,
    mut events: MessageWriter<MissionEvent>,
    lang: Res<Lang>,
    t: Res<Translations>,
) -> Result {
    if !matches!(stage.get(), MissionStage::Prelaunch) {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;
    egui::Window::new("main_menu")
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(egui::Frame::new().fill(theme::PANEL_BG).inner_margin(24.0))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.set_width(360.0);

                ui.colored_label(
                    theme::SLS_ORANGE,
                    egui::RichText::new(t.get(*lang, "menu.title"))
                        .size(36.0)
                        .strong(),
                );
                ui.colored_label(
                    theme::TEXT_MUTED,
                    egui::RichText::new(t.get(*lang, "menu.subtitle")).size(16.0),
                );

                ui.add_space(20.0);

                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(t.get(*lang, "menu.start"))
                                .size(20.0)
                                .strong()
                                .color(theme::TEXT_PRIMARY),
                        )
                        .min_size(egui::vec2(280.0, 48.0))
                        .fill(theme::NASA_BLUE),
                    )
                    .clicked()
                {
                    events.write(MissionEvent::Liftoff);
                    next_stage.set(MissionStage::Launch);
                }

                ui.add_space(8.0);
                ui.colored_label(
                    theme::TEXT_MUTED,
                    egui::RichText::new(t.get(*lang, "hint.press_space")).size(12.0),
                );
            });
        });

    Ok(())
}

fn draw_settings(
    mut contexts: EguiContexts,
    mut lang: ResMut<Lang>,
    mut difficulty: ResMut<Difficulty>,
    t: Res<Translations>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    let current_lang = *lang;

    egui::Window::new("settings")
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::LEFT_BOTTOM, [12.0, -12.0])
        .frame(egui::Frame::new().fill(theme::PANEL_BG).inner_margin(10.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.colored_label(theme::TEXT_MUTED, t.get(current_lang, "menu.language"));
                egui::ComboBox::from_id_salt("lang_combo")
                    .selected_text(lang.label())
                    .show_ui(ui, |ui| {
                        for opt in Lang::ALL {
                            ui.selectable_value(&mut *lang, opt, opt.label());
                        }
                    });
            });
            ui.horizontal(|ui| {
                ui.colored_label(theme::TEXT_MUTED, t.get(current_lang, "menu.difficulty"));
                egui::ComboBox::from_id_salt("diff_combo")
                    .selected_text(match *difficulty {
                        Difficulty::Story => t.get(current_lang, "difficulty.story"),
                        Difficulty::Realistic => t.get(current_lang, "difficulty.realistic"),
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut *difficulty,
                            Difficulty::Story,
                            t.get(current_lang, "difficulty.story"),
                        );
                        ui.selectable_value(
                            &mut *difficulty,
                            Difficulty::Realistic,
                            t.get(current_lang, "difficulty.realistic"),
                        );
                    });
            });
        });

    Ok(())
}
