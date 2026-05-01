use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::config::Difficulty;
use crate::i18n::{Lang, Translations};
use crate::states::MissionStage;
use crate::ui::theme;

pub fn plugin(app: &mut App) {
    app.add_systems(EguiPrimaryContextPass, draw_settings);
}

/// Окно настроек (язык + сложность) — всегда в нижнем левом углу.
/// Главное меню (заголовок + чеклист) живёт в [`crate::ui::checklist`].
fn draw_settings(
    mut contexts: EguiContexts,
    stage: Res<State<MissionStage>>,
    mut lang: ResMut<Lang>,
    mut difficulty: ResMut<Difficulty>,
    t: Res<Translations>,
) -> Result {
    // В MainMenu настройки встроены в главную панель.
    if matches!(stage.get(), MissionStage::MainMenu | MissionStage::Loading) {
        return Ok(());
    }
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
