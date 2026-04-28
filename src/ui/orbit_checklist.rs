use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::assets::GameAssets;
use crate::i18n::{Lang, Translations};
use crate::states::MissionStage;
use crate::ui::theme;

const ITEMS: [&str; 12] = [
    "orbit.system.eclss",
    "orbit.system.tcs",
    "orbit.system.eps",
    "orbit.system.gnc",
    "orbit.system.imu",
    "orbit.system.comms",
    "orbit.system.docking",
    "orbit.system.rcs",
    "orbit.system.icps_pressure",
    "orbit.system.icps_ignite",
    "orbit.system.crew_health",
    "orbit.system.trajectory",
];

#[derive(Resource, Default)]
pub struct OrbitChecklist {
    pub states: [bool; 12],
}

impl OrbitChecklist {
    pub fn all_ok(&self) -> bool {
        self.states.iter().all(|&v| v)
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<OrbitChecklist>()
        .add_systems(OnEnter(MissionStage::Orbit), reset_checklist)
        .add_systems(EguiPrimaryContextPass, draw_checklist);
}

fn reset_checklist(mut checklist: ResMut<OrbitChecklist>) {
    *checklist = OrbitChecklist::default();
}

fn draw_checklist(
    mut commands: Commands,
    mut contexts: EguiContexts,
    stage: Res<State<MissionStage>>,
    mut checklist: ResMut<OrbitChecklist>,
    mut next_stage: ResMut<NextState<MissionStage>>,
    lang: Res<Lang>,
    t: Res<Translations>,
    assets: Option<Res<GameAssets>>,
) -> Result {
    if !matches!(stage.get(), MissionStage::Orbit) {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;
    egui::Window::new("orbit_checklist")
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::LEFT_CENTER, [16.0, 0.0])
        .frame(egui::Frame::new().fill(theme::PANEL_BG).inner_margin(20.0))
        .show(ctx, |ui| {
            ui.set_width(360.0);

            ui.colored_label(
                theme::NASA_BLUE,
                egui::RichText::new(t.get(*lang, "orbit.checklist.title"))
                    .size(18.0)
                    .strong(),
            );
            ui.add_space(10.0);

            for (i, key) in ITEMS.iter().enumerate() {
                let response = ui.checkbox(&mut checklist.states[i], t.get(*lang, key));
                if response.changed()
                    && checklist.states[i]
                    && let Some(a) = assets.as_ref()
                {
                    commands.spawn((AudioPlayer(a.scan.clone()), PlaybackSettings::ONCE));
                }
            }

            ui.add_space(14.0);

            let go = checklist.all_ok();
            let fill = if go {
                theme::SLS_ORANGE
            } else {
                theme::PANEL_BG
            };
            let label_color = if go {
                theme::TEXT_PRIMARY
            } else {
                theme::TEXT_MUTED
            };
            let btn = egui::Button::new(
                egui::RichText::new(t.get(*lang, "orbit.checklist.go"))
                    .size(20.0)
                    .strong()
                    .color(label_color),
            )
            .min_size(egui::vec2(320.0, 44.0))
            .fill(fill);

            if ui.add_enabled(go, btn).clicked() {
                next_stage.set(MissionStage::TLI);
            }
        });

    Ok(())
}
