use bevy::prelude::*;

use crate::config::{Difficulty, G0, SlsParams, TimeScale, WORLD_SCALE};
use crate::events::MissionEvent;
use crate::physics::orbital;
use crate::states::MissionStage;
use crate::ui::hud::MissionTime;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlightPhase {
    BoosterPhase,
    CoreOnly,
    Coast,
}

#[derive(Component, Clone, Debug)]
pub struct Rocket {
    pub thrust_kn: f32,
    pub fuel_kg: f32,
    pub fuel_initial_kg: f32,
    pub mass_kg: f32,
    pub phase: FlightPhase,
    /// Итоговый угол тяги к горизонту: 90° = вертикально вверх, 0° = горизонтально.
    pub pitch_command_deg: f32,
    /// Игровое смещение тангажа от автопрограммы (накапливается через A/D).
    pub pitch_offset_deg: f32,
    /// Сколько секунд подряд `|pitch_deg − pitch_command_deg|` превышает допуск Difficulty.
    pub pitch_overshoot_s: f32,
    /// Дроссель RS-25 в долях номинала. Реальный диапазон 0.67–1.09 (67–109%).
    /// Меняется только в фазе CoreOnly — SRB неуправляемы, во время BoosterPhase
    /// держится в 1.0 и UI блокирует кнопки.
    pub throttle_pct: f32,
}

/// Минимум и максимум дросселя для RS-25 (от номинала).
pub const THROTTLE_MIN: f32 = 0.67;
pub const THROTTLE_MAX: f32 = 1.09;

impl Rocket {
    /// Эффективный удельный импульс смеси SRB+RS-25 либо чистых RS-25.
    pub fn current_isp_s(&self, params: &SlsParams) -> f32 {
        match self.phase {
            FlightPhase::BoosterPhase => {
                let srb_thrust = (params.thrust_total_kn - params.thrust_rs25_only_kn).max(1.0);
                let rs25_thrust = params.thrust_rs25_only_kn;
                let total = (srb_thrust + rs25_thrust).max(1.0);
                (srb_thrust * params.srb_isp_s + rs25_thrust * params.rs25_isp_s) / total
            }
            FlightPhase::CoreOnly => params.rs25_isp_s,
            FlightPhase::Coast => 0.0,
        }
    }
}

#[derive(Component, Clone, Debug, Default)]
pub struct FlightDynamics {
    pub altitude_m: f32,
    pub vertical_speed_ms: f32,
    pub horizontal_speed_ms: f32,
    pub speed_ms: f32,
    pub pitch_deg: f32,
    pub g_load: f32,
}

/// Стартовые компоненты ракеты — спавнит [`stages::launch`] вместе со SceneRoot.
pub fn initial_components(params: &SlsParams) -> (Rocket, FlightDynamics) {
    let fuel_total = params.mass_total_kg - params.mass_core_dry_kg;
    (
        Rocket {
            thrust_kn: params.thrust_total_kn,
            fuel_kg: fuel_total,
            fuel_initial_kg: fuel_total,
            mass_kg: params.mass_total_kg,
            phase: FlightPhase::BoosterPhase,
            pitch_command_deg: 90.0,
            pitch_offset_deg: 0.0,
            pitch_overshoot_s: 0.0,
            throttle_pct: 1.0,
        },
        FlightDynamics {
            pitch_deg: 90.0,
            ..default()
        },
    )
}

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_pitch_command_profile,
            apply_pitch_command,
            tick_rocket_physics,
            check_srb_separation,
            check_meco,
            check_pitch_abort,
            sync_rocket_transform,
        )
            .chain()
            .run_if(in_state(MissionStage::Launch)),
    );
}

/// Программа тангажа Artemis II: вертикально до T+10, плавный gravity-turn до T+150,
/// далее ~25° (близко к горизонту, для разгона на орбиту). Ввод игрока через
/// `pitch_offset_deg` суммируется поверх программы.
fn update_pitch_command_profile(mission_time: Res<MissionTime>, mut rockets: Query<&mut Rocket>) {
    let t = mission_time.elapsed.as_secs_f32();
    for mut rocket in &mut rockets {
        let base = if t < 10.0 {
            90.0
        } else if t < 150.0 {
            90.0 - 65.0 * (t - 10.0) / 140.0
        } else {
            25.0
        };
        rocket.pitch_command_deg = (base + rocket.pitch_offset_deg).clamp(0.0, 90.0);
    }
}

/// Плавно подгоняет реальный pitch к командному, скорость поворота зависит от Difficulty.
fn apply_pitch_command(
    time: Res<Time>,
    timescale: Res<TimeScale>,
    diff: Res<Difficulty>,
    mut q: Query<(&Rocket, &mut FlightDynamics)>,
) {
    let dt = time.delta_secs() * timescale.multiplier;
    if dt <= 0.0 {
        return;
    }
    let rate_deg_s = match *diff {
        Difficulty::Story => 10.0,
        Difficulty::Realistic => 5.0,
    };
    let max_step = rate_deg_s * dt;
    for (rocket, mut dynamics) in &mut q {
        let target = rocket.pitch_command_deg;
        let delta = (target - dynamics.pitch_deg).clamp(-max_step, max_step);
        dynamics.pitch_deg += delta;
    }
}

/// Главная физическая система: применить тягу, гравитацию, расход топлива; проинтегрировать
/// `vertical_speed`, `horizontal_speed`, `altitude`, `g_load`.
fn tick_rocket_physics(
    time: Res<Time>,
    timescale: Res<TimeScale>,
    params: Res<SlsParams>,
    mut q: Query<(&mut Rocket, &mut FlightDynamics)>,
) {
    let dt = time.delta_secs() * timescale.multiplier;
    if dt <= 0.0 {
        return;
    }
    for (mut rocket, mut dynamics) in &mut q {
        let g = orbital::gravity_at_altitude(dynamics.altitude_m);
        let pitch_rad = dynamics.pitch_deg.to_radians();
        let isp = rocket.current_isp_s(&params);

        // Эффективная тяга = базовая × throttle. Throttle меняется только в
        // CoreOnly (см. handle_throttle_input), так что на BoosterPhase здесь
        // всегда 1.0 и SRB+RS-25 идут на полную.
        let base_thrust_n = rocket.thrust_kn * 1000.0;
        let effective_thrust_n = base_thrust_n * rocket.throttle_pct;

        // Расход топлива: ṁ = F / (Isp · g0). При Coast или пустом баке — нулевой.
        let mass_flow = if rocket.phase == FlightPhase::Coast || rocket.fuel_kg <= 0.0 || isp <= 0.0
        {
            0.0
        } else {
            effective_thrust_n / (isp * G0)
        };
        let burned = (mass_flow * dt).min(rocket.fuel_kg);
        rocket.fuel_kg -= burned;
        rocket.mass_kg = (rocket.mass_kg - burned).max(params.mass_core_dry_kg);
        if rocket.fuel_kg <= 0.0 {
            rocket.thrust_kn = 0.0;
        }

        // Ускорение тяги по тангажу (горизонталь = 0°, зенит = 90°).
        let a_thrust = effective_thrust_n / rocket.mass_kg.max(1.0);
        let a_vertical = a_thrust * pitch_rad.sin() - g;
        let a_horizontal = a_thrust * pitch_rad.cos();

        // Эйлеровская интеграция.
        dynamics.vertical_speed_ms += a_vertical * dt;
        dynamics.horizontal_speed_ms += a_horizontal * dt;
        dynamics.altitude_m += dynamics.vertical_speed_ms * dt;
        if dynamics.altitude_m < 0.0 {
            dynamics.altitude_m = 0.0;
            dynamics.vertical_speed_ms = dynamics.vertical_speed_ms.max(0.0);
        }
        dynamics.speed_ms = dynamics.vertical_speed_ms.hypot(dynamics.horizontal_speed_ms);

        // g-нагрузка: ощущаемое ускорение в кабине = тяга/масса (ньютоновская невесомость
        // в свободном падении даёт 0; тяга добавляет вес).
        dynamics.g_load = a_thrust / G0;
    }
}

fn check_srb_separation(
    mission_time: Res<MissionTime>,
    params: Res<SlsParams>,
    mut events: MessageWriter<MissionEvent>,
    mut q: Query<&mut Rocket>,
) {
    let t = mission_time.elapsed.as_secs_f32();
    for mut rocket in &mut q {
        if rocket.phase == FlightPhase::BoosterPhase && t >= params.srb_burn_duration_s {
            rocket.phase = FlightPhase::CoreOnly;
            rocket.thrust_kn = params.thrust_rs25_only_kn;
            // Сбрасываем «сухие» SRB: 15% массы боковушек — каркас, 85% выгорело.
            let srb_dry = params.mass_srb_total_kg * 0.15;
            rocket.mass_kg = (rocket.mass_kg - srb_dry).max(params.mass_core_dry_kg);
            events.write(MissionEvent::SrbSep);
            info!(
                "physics: SRB sep @ T+{:.1}s, mass={:.0} kg, fuel={:.0} kg",
                t, rocket.mass_kg, rocket.fuel_kg
            );
        }
    }
}

fn check_meco(
    mission_time: Res<MissionTime>,
    params: Res<SlsParams>,
    mut events: MessageWriter<MissionEvent>,
    mut next_stage: ResMut<NextState<MissionStage>>,
    mut q: Query<&mut Rocket>,
) {
    let t = mission_time.elapsed.as_secs_f32();
    for mut rocket in &mut q {
        if rocket.phase == FlightPhase::CoreOnly && t >= params.meco_target_t_plus_s {
            rocket.phase = FlightPhase::Coast;
            rocket.thrust_kn = 0.0;
            events.write(MissionEvent::Meco);
            info!(
                "physics: MECO @ T+{:.1}s, fuel={:.0} kg, mass={:.0} kg",
                t, rocket.fuel_kg, rocket.mass_kg
            );
            next_stage.set(MissionStage::Orbit);
        }
    }
}

fn check_pitch_abort(
    time: Res<Time>,
    timescale: Res<TimeScale>,
    diff: Res<Difficulty>,
    mut events: MessageWriter<MissionEvent>,
    mut q: Query<(&mut Rocket, &FlightDynamics)>,
) {
    let dp = diff.params();
    let dt = time.delta_secs() * timescale.multiplier;
    let mut abort_fired = false;
    for (mut rocket, dynamics) in &mut q {
        if rocket.phase == FlightPhase::Coast {
            continue;
        }
        let deviation = (dynamics.pitch_deg - rocket.pitch_command_deg).abs();
        if deviation > dp.pitch_tolerance_deg {
            rocket.pitch_overshoot_s += dt;
            if rocket.pitch_overshoot_s >= dp.pitch_abort_grace_s {
                abort_fired = true;
            }
        } else {
            rocket.pitch_overshoot_s = 0.0;
        }
    }
    if abort_fired {
        events.write(MissionEvent::Abort("pitch deviation".into()));
        warn!("physics: ABORT — отклонение тангажа выше допуска Difficulty");
    }
}

/// Переносит высоту/тангаж в Transform ракеты для рендера.
fn sync_rocket_transform(mut q: Query<(&FlightDynamics, &mut Transform), With<Rocket>>) {
    for (dynamics, mut transform) in &mut q {
        transform.translation.y = dynamics.altitude_m / WORLD_SCALE;
        let tilt = (90.0 - dynamics.pitch_deg).to_radians();
        transform.rotation = Quat::from_rotation_z(-tilt);
    }
}
