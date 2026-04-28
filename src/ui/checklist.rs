use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::events::MissionEvent;
use crate::i18n::{Lang, Translations};
use crate::states::MissionStage;
use crate::ui::theme;

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
        .add_systems(OnEnter(MissionStage::Prelaunch), reset_checklist)
        .add_systems(EguiPrimaryContextPass, draw_checklist);
}

fn reset_checklist(mut checklist: ResMut<PreflightChecklist>) {
    *checklist = PreflightChecklist::default();
}

fn draw_checklist(
    mut contexts: EguiContexts,
    stage: Res<State<MissionStage>>,
    mut checklist: ResMut<PreflightChecklist>,
    mut next_stage: ResMut<NextState<MissionStage>>,
    mut events: MessageWriter<MissionEvent>,
    lang: Res<Lang>,
    t: Res<Translations>,
) -> Result {
    if !matches!(stage.get(), MissionStage::Prelaunch) {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;
    egui::Window::new("preflight_checklist")
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
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
                ui.checkbox(&mut checklist.states[i], t.get(*lang, key));
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
        });

    Ok(())
}
