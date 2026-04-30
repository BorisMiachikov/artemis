use bevy::prelude::*;

use crate::config::{FlybyResult, IcpsParams, TliResult};
use crate::stages::reentry::ReentryState;
use crate::states::MissionStage;
use crate::ui::hud::MissionTime;

#[derive(Clone, Debug)]
pub struct PhaseRecord {
    pub name_ru: &'static str,
    pub name_en: &'static str,
    pub time_s: u64,
    pub detail: String,
}

impl PhaseRecord {
    pub fn fmt_time(&self) -> String {
        let h = self.time_s / 3600;
        let m = (self.time_s / 60) % 60;
        let s = self.time_s % 60;
        if h > 0 {
            format!("T+{h}:{m:02}:{s:02}")
        } else {
            format!("T+{m:02}:{s:02}")
        }
    }
}

#[derive(Resource, Default)]
pub struct FlightRecord {
    pub phases: Vec<PhaseRecord>,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<FlightRecord>()
        .add_systems(OnEnter(MissionStage::Prelaunch), reset_record)
        .add_systems(OnEnter(MissionStage::Launch), record_launch)
        .add_systems(OnEnter(MissionStage::Orbit), record_orbit)
        .add_systems(OnEnter(MissionStage::Transit), record_transit)
        .add_systems(OnEnter(MissionStage::LunarFlyby), record_flyby)
        .add_systems(OnEnter(MissionStage::Reentry), record_reentry)
        .add_systems(OnEnter(MissionStage::Splashdown), record_splashdown);
}

fn reset_record(mut record: ResMut<FlightRecord>) {
    record.phases.clear();
}

fn record_launch(mut record: ResMut<FlightRecord>, t: Res<MissionTime>) {
    record.phases.push(PhaseRecord {
        name_ru: "Старт",
        name_en: "Launch",
        time_s: t.elapsed.as_secs(),
        detail: String::new(),
    });
}

fn record_orbit(mut record: ResMut<FlightRecord>, t: Res<MissionTime>) {
    record.phases.push(PhaseRecord {
        name_ru: "Орбита",
        name_en: "Orbit",
        time_s: t.elapsed.as_secs(),
        detail: String::new(),
    });
}

fn record_transit(
    mut record: ResMut<FlightRecord>,
    t: Res<MissionTime>,
    tli: Res<TliResult>,
    icps: Res<IcpsParams>,
) {
    let acc = tli.accuracy_pct(icps.target_delta_v_ms);
    record.phases.push(PhaseRecord {
        name_ru: "Транзит к Луне",
        name_en: "Trans-Lunar Transit",
        time_s: t.elapsed.as_secs(),
        detail: format!("TLI {acc:.1}%"),
    });
}

fn record_flyby(
    mut record: ResMut<FlightRecord>,
    t: Res<MissionTime>,
    flyby: Res<FlybyResult>,
) {
    record.phases.push(PhaseRecord {
        name_ru: "Облёт Луны",
        name_en: "Lunar Flyby",
        time_s: t.elapsed.as_secs(),
        detail: format!("{:.0} km", flyby.perilune_km),
    });
}

fn record_reentry(
    mut record: ResMut<FlightRecord>,
    t: Res<MissionTime>,
    reentry: Res<ReentryState>,
) {
    record.phases.push(PhaseRecord {
        name_ru: "Вход в атмосферу",
        name_en: "Reentry",
        time_s: t.elapsed.as_secs(),
        detail: format!("{:.2}°", reentry.entry_angle_deg),
    });
}

fn record_splashdown(
    mut record: ResMut<FlightRecord>,
    t: Res<MissionTime>,
    reentry: Res<ReentryState>,
) {
    record.phases.push(PhaseRecord {
        name_ru: "Приводнение",
        name_en: "Splashdown",
        time_s: t.elapsed.as_secs(),
        detail: format!("{:.0}%", reentry.heat_pct),
    });
}
