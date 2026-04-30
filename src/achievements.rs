use bevy::prelude::*;

use crate::config::{Difficulty, FlybyResult, IcpsParams, TliResult, TransitOutcome};
use crate::stages::reentry::ReentryState;
use crate::states::MissionStage;

pub fn plugin(app: &mut App) {
    app.init_resource::<AchievementTracker>()
        .add_systems(Startup, setup_achievements)
        .add_systems(OnEnter(MissionStage::Prelaunch), reset_achievements)
        .add_systems(OnEnter(MissionStage::Splashdown), check_achievements);
}

#[derive(Clone, Debug)]
pub struct Achievement {
    pub id: &'static str,
    pub name_ru: &'static str,
    pub name_en: &'static str,
    pub desc_ru: &'static str,
    pub desc_en: &'static str,
    pub unlocked: bool,
}

#[derive(Resource, Default)]
pub struct AchievementTracker {
    pub list: Vec<Achievement>,
}

fn default_achievements() -> Vec<Achievement> {
    vec![
        Achievement {
            id: "perfect_tli",
            name_ru: "Идеальный burn",
            name_en: "Perfect Burn",
            desc_ru: "Точность TLI ≥ 99%",
            desc_en: "TLI accuracy ≥ 99%",
            unlocked: false,
        },
        Achievement {
            id: "precision_entry",
            name_ru: "Снайпер входа",
            name_en: "Precision Entry",
            desc_ru: "Угол входа ±0.1° от 6.25°",
            desc_en: "Entry angle within ±0.1° of 6.25°",
            unlocked: false,
        },
        Achievement {
            id: "close_flyby",
            name_ru: "Близкий облёт",
            name_en: "Close Flyby",
            desc_ru: "Перицентр ≤ 6 600 км",
            desc_en: "Perilune ≤ 6,600 km",
            unlocked: false,
        },
        Achievement {
            id: "no_mcc",
            name_ru: "Без коррекций",
            name_en: "No Corrections",
            desc_ru: "Перелёт без коррекции курса",
            desc_en: "Transit without MCC burns",
            unlocked: false,
        },
        Achievement {
            id: "cool_shield",
            name_ru: "Холодный щит",
            name_en: "Cool Shield",
            desc_ru: "Тепловой щит ≤ 60% при посадке",
            desc_en: "Heat shield ≤ 60% at splashdown",
            unlocked: false,
        },
        Achievement {
            id: "master_pilot",
            name_ru: "Мастер-пилот",
            name_en: "Master Pilot",
            desc_ru: "Все 5 достижений на Realistic",
            desc_en: "All 5 achievements on Realistic",
            unlocked: false,
        },
    ]
}

fn setup_achievements(mut tracker: ResMut<AchievementTracker>) {
    tracker.list = default_achievements();
}

fn reset_achievements(mut tracker: ResMut<AchievementTracker>) {
    tracker.list = default_achievements();
}

fn check_achievements(
    mut tracker: ResMut<AchievementTracker>,
    tli: Res<TliResult>,
    icps: Res<IcpsParams>,
    flyby: Res<FlybyResult>,
    outcome: Res<TransitOutcome>,
    reentry: Res<ReentryState>,
    diff: Res<Difficulty>,
) {
    let accuracy = tli.accuracy_pct(icps.target_delta_v_ms);
    let angle_err = (reentry.entry_angle_deg - 6.25_f32).abs();

    let results = [
        ("perfect_tli", accuracy >= 99.0),
        ("precision_entry", angle_err <= 0.1),
        ("close_flyby", flyby.perilune_km <= 6_600.0),
        ("no_mcc", outcome.mcc_corrections == 0),
        ("cool_shield", reentry.heat_pct <= 60.0),
    ];

    for (id, condition) in results {
        if let Some(ach) = tracker.list.iter_mut().find(|a| a.id == id) {
            if condition {
                ach.unlocked = true;
                info!("достижение разблокировано: {} ({})", ach.name_ru, id);
            }
        }
    }

    // Мастер-пилот: все 5 предыдущих + Realistic
    let first_five_unlocked = tracker
        .list
        .iter()
        .filter(|a| a.id != "master_pilot")
        .all(|a| a.unlocked);

    if let Some(master) = tracker.list.iter_mut().find(|a| a.id == "master_pilot") {
        if first_five_unlocked && matches!(*diff, Difficulty::Realistic) {
            master.unlocked = true;
            info!("достижение разблокировано: {} (master_pilot)", master.name_ru);
        }
    }

    let unlocked_count = tracker.list.iter().filter(|a| a.unlocked).count();
    info!(
        "achievements: {}/{} разблокировано",
        unlocked_count,
        tracker.list.len()
    );
}
