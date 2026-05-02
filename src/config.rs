#![allow(dead_code)] // часть полей и значений включается только в Phase 4+

use bevy::prelude::*;

pub const G0: f32 = 9.80665;

pub const RS25_ISP_S: f32 = 453.0;
pub const SRB_ISP_S: f32 = 269.0;

pub const GM_EARTH: f64 = 3.986e14;
pub const GM_MOON: f64 = 4.9e12;

pub const EARTH_RADIUS_KM: f64 = 6_371.0;
pub const MOON_RADIUS_KM: f64 = 1_737.4;

/// 1 единица сцены = 100 м реального пространства. Перевод высоты в Y-координату:
/// `transform.translation.y = altitude_m / WORLD_SCALE`. На высоте 200 км рендер
/// имеет y ≈ 2000 — комфортно для f32.
pub const WORLD_SCALE: f32 = 100.0;

/// Высота, ниже которой считаем что орбита нестабильна и надо разрушаться.
pub const ORBITAL_INSERTION_ALT_M: f32 = 150_000.0;
/// Минимальная горизонтальная скорость для устойчивой LEO, м/с (~v_circ для 200 км).
pub const ORBITAL_INSERTION_VEL_MS: f32 = 7_600.0;
/// Нижняя граница «нестабильной» зоны: выше = unstable orbit, ниже = баллистика.
pub const UNSTABLE_ORBIT_ALT_M: f32 = 80_000.0;
pub const UNSTABLE_ORBIT_VEL_MS: f32 = 6_500.0;

/// Окно декея unstable‑орбиты, секунды (линейно от 80 км до 150 км).
pub const ORBIT_DECAY_MIN_S: f32 = 300.0;
pub const ORBIT_DECAY_MAX_S: f32 = 1_800.0;

/// Штраф к `TransitOutcome::trajectory_error` при выходе на нестабильную орбиту.
pub const UNSTABLE_ORBIT_TRAJ_PENALTY: f32 = 0.20;

/// Аэродинамическое сопротивление: тупой носовой конус SLS Core Stage.
pub const ROCKET_DRAG_CD: f32 = 1.0;
pub const ROCKET_DRAG_AREA_M2: f32 = 55.0;

/// Параметры SLS Block 1 (Artemis II), упрощённые.
/// Источники: NASA SLS fact sheet, Artemis II Mission Profile.
#[derive(Resource, Clone, Copy, Debug)]
pub struct SlsParams {
    /// Стартовая масса (ракета + топливо + полезная нагрузка), кг.
    pub mass_total_kg: f32,
    /// Сухая масса первой ступени (без SRB и без топлива core stage), кг.
    pub mass_core_dry_kg: f32,
    /// Масса связки из двух SRB вместе с топливом, кг.
    pub mass_srb_total_kg: f32,
    /// Полная тяга при старте: 4×RS-25 + 2×SRB, кН.
    pub thrust_total_kn: f32,
    /// Тяга только RS-25 (4 двигателя), после SRB sep, кН.
    pub thrust_rs25_only_kn: f32,
    /// Удельный импульс RS-25 (вакуумный) в секундах.
    pub rs25_isp_s: f32,
    /// Удельный импульс SRB в секундах.
    pub srb_isp_s: f32,
    /// Длительность горения SRB до сброса, секунды (T+~126).
    pub srb_burn_duration_s: f32,
    /// Время отсечки главных двигателей (MECO), секунды (T+~510 = 8:30).
    pub meco_target_t_plus_s: f32,
    /// Целевой апоцентр после MECO, км.
    pub target_apoapsis_km: f32,
}

impl Default for SlsParams {
    fn default() -> Self {
        Self {
            mass_total_kg: 2_608_000.0,
            mass_core_dry_kg: 99_000.0,
            mass_srb_total_kg: 1_274_000.0,
            thrust_total_kn: 39_144.0,
            thrust_rs25_only_kn: 7_440.0,
            rs25_isp_s: RS25_ISP_S,
            srb_isp_s: SRB_ISP_S,
            srb_burn_duration_s: 126.0,
            meco_target_t_plus_s: 510.0,
            target_apoapsis_km: 200.0,
        }
    }
}

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Difficulty {
    Story,
    #[default]
    Realistic,
}

/// Окна допусков и игровые поблажки на уровне сложности.
#[derive(Clone, Copy, Debug)]
pub struct DifficultyParams {
    /// Допустимое отклонение `pitch_deg` от командного значения, градусы.
    pub pitch_tolerance_deg: f32,
    /// Сколько секунд отклонение должно быть выше допуска, прежде чем триггерится Abort.
    pub pitch_abort_grace_s: f32,
    /// Половина окна TLI burn (для Phase 4), секунды.
    pub tli_burn_window_sec: f32,
    /// Допустимое отклонение угла входа в атмосферу (Phase 6), градусы.
    pub reentry_angle_window_deg: f32,
}

impl Difficulty {
    pub fn params(self) -> DifficultyParams {
        match self {
            Difficulty::Story => DifficultyParams {
                pitch_tolerance_deg: 15.0,
                pitch_abort_grace_s: 3.0,
                tli_burn_window_sec: 60.0,
                reentry_angle_window_deg: 1.5,
            },
            Difficulty::Realistic => DifficultyParams {
                pitch_tolerance_deg: 5.0,
                pitch_abort_grace_s: 1.0,
                tli_burn_window_sec: 20.0,
                reentry_angle_window_deg: 0.5,
            },
        }
    }
}

/// Игровое ускорение времени. Применяется только к физике миссии (расход топлива,
/// высота, T+ таймер) — UI/ввод остаются в реальном времени.
#[derive(Resource, Clone, Copy, Debug)]
pub struct TimeScale {
    pub multiplier: f32,
}

impl Default for TimeScale {
    fn default() -> Self {
        Self { multiplier: 1.0 }
    }
}

impl TimeScale {
    const STEPS: [f32; 3] = [1.0, 5.0, 20.0];

    fn current_index(self) -> usize {
        Self::STEPS
            .iter()
            .position(|&v| (v - self.multiplier).abs() < 0.01)
            .unwrap_or(0)
    }

    pub fn cycle_up(&mut self) {
        let idx = self.current_index();
        let next = (idx + 1).min(Self::STEPS.len() - 1);
        self.multiplier = Self::STEPS[next];
    }

    pub fn cycle_down(&mut self) {
        let idx = self.current_index();
        let next = idx.saturating_sub(1);
        self.multiplier = Self::STEPS[next];
    }
}

/// Параметры разгонного блока ICPS (Interim Cryogenic Propulsion Stage), упрощённые.
/// Источник: NASA ICPS / Delta IV Upper Stage spec.
#[derive(Resource, Clone, Copy, Debug)]
pub struct IcpsParams {
    /// Тяга RL10B-2, кН.
    pub thrust_kn: f32,
    /// Удельный импульс RL10B-2 (вакуумный), секунды.
    pub isp_s: f32,
    /// Масса связки «ICPS + Orion» перед TLI burn, кг.
    pub stack_mass_kg: f32,
    /// Сухая масса ICPS + Orion после полного выгорания топлива, кг.
    pub stack_dry_mass_kg: f32,
    /// Целевая длительность горения TLI, секунды (≈ 18 минут — реальный Artemis II).
    pub target_burn_duration_s: f32,
    /// Целевой Δv для TLI, м/с (≈ 3 050 для миссии Artemis II).
    pub target_delta_v_ms: f32,
}

impl Default for IcpsParams {
    fn default() -> Self {
        Self {
            thrust_kn: 110.1,
            isp_s: 462.0,
            stack_mass_kg: 56_000.0,
            stack_dry_mass_kg: 28_800.0,
            target_burn_duration_s: 1_080.0,
            target_delta_v_ms: 3_050.0,
        }
    }
}

/// Итог TLI burn: накопленный Δv и длительность. Сохраняется автосейвом и читается в Phase 5.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct TliResult {
    pub delta_v_ms: f32,
    pub burn_duration_s: f32,
    pub completed: bool,
}

impl TliResult {
    /// Точность относительно целевого Δv, в процентах. 100% при попадании, 0% при полном промахе.
    pub fn accuracy_pct(&self, target_delta_v_ms: f32) -> f32 {
        if target_delta_v_ms <= 0.0 {
            return 0.0;
        }
        let err = (self.delta_v_ms - target_delta_v_ms).abs() / target_delta_v_ms;
        (1.0 - err).clamp(0.0, 1.0) * 100.0
    }
}

/// Итог этапа Transit: насколько скорректирована траектория к Луне.
/// Записывается в transit.rs, читается в lunar_flyby.rs.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct TransitOutcome {
    /// Ошибка траектории после MCC [0..1], 0 = идеальная. Влияет на расстояние перицентра.
    pub trajectory_error: f32,
    /// Количество использованных коррекций курса.
    pub mcc_corrections: u8,
}

/// Итог этапа LunarFlyby: достигнутое расстояние перицентра.
/// Читается экраном победы в ui/mission.rs.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct FlybyResult {
    /// Расчётное расстояние перицентра, км.
    pub perilune_km: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accuracy_pct_perfect() {
        let r = TliResult { delta_v_ms: 3_050.0, burn_duration_s: 1_080.0, completed: true };
        assert_eq!(r.accuracy_pct(3_050.0), 100.0);
    }

    #[test]
    fn accuracy_pct_half_error() {
        let r = TliResult { delta_v_ms: 4_575.0, burn_duration_s: 0.0, completed: false };
        assert_eq!(r.accuracy_pct(3_050.0), 50.0);
    }

    #[test]
    fn accuracy_pct_clamps_at_zero() {
        let r = TliResult { delta_v_ms: 0.0, burn_duration_s: 0.0, completed: false };
        assert_eq!(r.accuracy_pct(3_050.0), 0.0);
    }

    #[test]
    fn accuracy_pct_zero_target() {
        let r = TliResult { delta_v_ms: 1_000.0, burn_duration_s: 0.0, completed: false };
        assert_eq!(r.accuracy_pct(0.0), 0.0);
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Difficulty>()
        .init_resource::<SlsParams>()
        .init_resource::<IcpsParams>()
        .init_resource::<TliResult>()
        .init_resource::<TimeScale>()
        .init_resource::<TransitOutcome>()
        .init_resource::<FlybyResult>();
}
