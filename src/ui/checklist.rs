use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::achievements::AchievementTracker;
use crate::events::MissionEvent;
use crate::i18n::{Lang, Translations};
use crate::stages::prelaunch::PrelaunchCutscene;
use crate::states::MissionStage;
use crate::ui::theme;

#[derive(Resource, Default)]
pub struct ShowAchievementsPanel(pub bool);

const ITEMS: [&str; 10] = [
    "checklist.system.eclss",
    "checklist.system.eps",
    "checklist.system.gnc",
    "checklist.system.comms",
    "checklist.system.rcs",
    "checklist.system.icps",
    "checklist.system.core_stage",
    "checklist.system.srbs",
    "checklist.system.launch_abort",
    "checklist.system.fts",
];

#[derive(Resource, Default)]
pub struct PreflightChecklist {
    pub states: [bool; 10],
}

impl PreflightChecklist {
    pub fn all_ok(&self) -> bool {
        self.states.iter().all(|&v| v)
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<PreflightChecklist>()
        .init_resource::<ShowAchievementsPanel>()
        .add_systems(OnEnter(MissionStage::Prelaunch), reset_checklist)
        .add_systems(EguiPrimaryContextPass, (draw_checklist, draw_achievements_panel));
}

fn reset_checklist(mut checklist: ResMut<PreflightChecklist>) {
    *checklist = PreflightChecklist::default();
}

#[allow(clippy::too_many_arguments)]
fn draw_checklist(
    mut contexts: EguiContexts,
    stage: Res<State<MissionStage>>,
    mut checklist: ResMut<PreflightChecklist>,
    mut next_stage: ResMut<NextState<MissionStage>>,
    mut events: MessageWriter<MissionEvent>,
    lang: Res<Lang>,
    t: Res<Translations>,
    mut show_ach: ResMut<ShowAchievementsPanel>,
    cutscene: Option<Res<PrelaunchCutscene>>,
) -> Result {
    if !matches!(stage.get(), MissionStage::Prelaunch) {
        return Ok(());
    }
    // Скрываем меню пока идёт кат-сцена.
    if cutscene.map(|c| !c.done).unwrap_or(false) {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;
    egui::Window::new("preflight_checklist")
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::LEFT_CENTER, [16.0, 0.0])
        .frame(egui::Frame::new().fill(theme::PANEL_BG).inner_margin(22.0))
        .show(ctx, |ui| {
            ui.set_width(380.0);

            ui.vertical_centered(|ui| {
                ui.colored_label(
                    theme::SLS_ORANGE,
                    egui::RichText::new(t.get(*lang, "menu.title"))
                        .size(28.0)
                        .strong(),
                );
                ui.colored_label(
                    theme::TEXT_MUTED,
                    egui::RichText::new(t.get(*lang, "menu.subtitle")).size(14.0),
                );
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            ui.colored_label(
                theme::NASA_BLUE,
                egui::RichText::new(t.get(*lang, "checklist.title"))
                    .size(18.0)
                    .strong(),
            );
            ui.add_space(8.0);

            for (i, key) in ITEMS.iter().enumerate() {
                let desc_key = format!("{key}.desc");
                ui.checkbox(&mut checklist.states[i], t.get(*lang, key))
                    .on_hover_text(t.get(*lang, &desc_key));
            }

            ui.add_space(14.0);

            let go = checklist.all_ok();
            let fill = if go {
                theme::NASA_BLUE
            } else {
                theme::PANEL_BG
            };
            let label_color = if go {
                theme::TEXT_PRIMARY
            } else {
                theme::TEXT_MUTED
            };
            let btn = egui::Button::new(
                egui::RichText::new(t.get(*lang, "checklist.go"))
                    .size(20.0)
                    .strong()
                    .color(label_color),
            )
            .min_size(egui::vec2(340.0, 44.0))
            .fill(fill);

            if ui.add_enabled(go, btn).clicked() {
                events.write(MissionEvent::Liftoff);
                next_stage.set(MissionStage::Launch);
            }

            ui.add_space(8.0);

            let ach_label = if show_ach.0 {
                t.get(*lang, "checklist.achievements_hide")
            } else {
                t.get(*lang, "checklist.achievements_btn")
            };
            let ach_btn = egui::Button::new(
                egui::RichText::new(ach_label).size(14.0).color(theme::TEXT_MUTED),
            )
            .min_size(egui::vec2(340.0, 28.0))
            .fill(egui::Color32::TRANSPARENT);
            if ui.add(ach_btn).clicked() {
                show_ach.0 = !show_ach.0;
            }
        });

    Ok(())
}

fn draw_achievements_panel(
    mut contexts: EguiContexts,
    stage: Res<State<MissionStage>>,
    show_ach: Res<ShowAchievementsPanel>,
    tracker: Res<AchievementTracker>,
    lang: Res<Lang>,
    t: Res<Translations>,
) -> Result {
    if !matches!(stage.get(), MissionStage::Prelaunch) || !show_ach.0 {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;
    egui::Window::new("achievements_panel")
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::LEFT_CENTER, [430.0, 0.0])
        .frame(
            egui::Frame::new()
                .fill(theme::PANEL_BG)
                .inner_margin(20.0)
                .stroke(egui::Stroke::new(1.0, theme::NASA_BLUE)),
        )
        .show(ctx, |ui| {
            ui.set_width(300.0);
            ui.colored_label(
                theme::NASA_BLUE,
                egui::RichText::new(t.get(*lang, "splashdown.achievements"))
                    .size(15.0)
                    .strong(),
            );
            ui.add_space(8.0);

            let unlocked = tracker.list.iter().filter(|a| a.unlocked).count();
            ui.colored_label(
                theme::TEXT_MUTED,
                egui::RichText::new(format!("{unlocked}/{}", tracker.list.len())).size(12.0),
            );
            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            for ach in &tracker.list {
                let name = if matches!(*lang, crate::i18n::Lang::Ru) { ach.name_ru } else { ach.name_en };
                let desc = if matches!(*lang, crate::i18n::Lang::Ru) { ach.desc_ru } else { ach.desc_en };
                let (icon, color) = if ach.unlocked {
                    ("✓", theme::STATUS_GREEN)
                } else {
                    ("□", theme::TEXT_MUTED)
                };
                ui.horizontal(|ui| {
                    ui.colored_label(color, egui::RichText::new(format!("{icon} {name}")).size(13.0).strong());
                });
                ui.colored_label(theme::TEXT_MUTED, egui::RichText::new(desc).size(11.0));
                ui.add_space(4.0);
            }
        });

    Ok(())
}
