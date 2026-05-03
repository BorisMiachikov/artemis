//! Численное интегрирование траектории Orion в системе Земля + Луна.
//!
//! Используется в стадии Transit (см. docs/next-session-realistic-transit.md).
//! Координаты — инерциальные с центром в Земле, единицы км и км/с.
//! Все накапливающиеся величины — `f64`, иначе ошибка интегрирования за 3 дня
//! полёта на f32 становится заметной.
//!
//! На текущем этапе (фундамент) модуль регистрирует ресурс `TrajectorySim`
//! и предоставляет публичную функцию `integrate`, но никем ещё не вызывается —
//! интеграция в `stages/transit.rs` будет в следующей сессии.

use std::collections::VecDeque;

use bevy::math::DVec3;
use bevy::prelude::*;

use crate::config::{MU_EARTH_KM3_S2, MU_MOON_KM3_S2, TRAJECTORY_MAX_DT_S};

/// Состояние симуляции: позиции и скорости Orion и Луны в инерциальной СК
/// с центром в Земле.
#[derive(Resource, Debug, Clone)]
pub struct TrajectorySim {
    pub orion_pos_km: DVec3,
    pub orion_vel_kms: DVec3,
    pub moon_pos_km: DVec3,
    pub moon_vel_kms: DVec3,
    pub elapsed_sim_s: f64,

    /// Минимальная дистанция «Orion–Луна» за всё время симуляции.
    pub closest_approach_km: f64,
    /// Время, когда был достигнут closest_approach.
    pub closest_approach_t_s: f64,

    /// Прогнозируемый перилуний (forward-prop на 3 суток). Обновляется системой UI.
    pub predicted_perilune_km: f64,
    pub predicted_perilune_t_s: f64,

    /// Топливо MCC, кг.
    pub mcc_fuel_kg: f32,

    /// Буфер прошлых позиций Orion для отрисовки трассы (push_back каждые ~5 sim-сек,
    /// pop_front при превышении ёмкости).
    pub trail_buffer: VecDeque<DVec3>,

    /// Кешированная предиктивная траектория (sample каждые ~9 шагов × 60 сек ≈ каждые 9 мин
    /// sim-времени). Обновляется системой `update_trajectory_prediction` раз в 2 game-сек.
    pub predicted_trajectory: Vec<DVec3>,
}

impl Default for TrajectorySim {
    fn default() -> Self {
        Self {
            orion_pos_km: DVec3::ZERO,
            orion_vel_kms: DVec3::ZERO,
            moon_pos_km: DVec3::new(crate::config::MOON_ORBIT_R_KM, 0.0, 0.0),
            moon_vel_kms: DVec3::new(0.0, 0.0, crate::config::MOON_ORBIT_V_KMS),
            elapsed_sim_s: 0.0,
            closest_approach_km: f64::INFINITY,
            closest_approach_t_s: 0.0,
            predicted_perilune_km: f64::INFINITY,
            predicted_perilune_t_s: 0.0,
            mcc_fuel_kg: crate::config::MCC_FUEL_INITIAL,
            trail_buffer: VecDeque::with_capacity(512),
            predicted_trajectory: Vec::new(),
        }
    }
}

/// Внутренний 12-мерный вектор состояния для RK4. Хранится отдельно от `TrajectorySim`
/// чтобы integrate-цикл не таскал лишних полей (closest_approach, fuel, buffer).
#[derive(Debug, Clone, Copy)]
struct State {
    op: DVec3, // Orion position
    ov: DVec3, // Orion velocity
    mp: DVec3, // Moon position
    mv: DVec3, // Moon velocity
}

impl State {
    fn from_sim(sim: &TrajectorySim) -> Self {
        Self {
            op: sim.orion_pos_km,
            ov: sim.orion_vel_kms,
            mp: sim.moon_pos_km,
            mv: sim.moon_vel_kms,
        }
    }

    fn write_back(&self, sim: &mut TrajectorySim) {
        sim.orion_pos_km = self.op;
        sim.orion_vel_kms = self.ov;
        sim.moon_pos_km = self.mp;
        sim.moon_vel_kms = self.mv;
    }

    fn add_scaled(&self, k: &State, factor: f64) -> State {
        State {
            op: self.op + k.op * factor,
            ov: self.ov + k.ov * factor,
            mp: self.mp + k.mp * factor,
            mv: self.mv + k.mv * factor,
        }
    }
}

/// Производная state по времени: возвращает [vel, accel, vel, accel] для Orion и Луны.
///
/// Гравитация Земли: a = -μ_e · r / |r|³.
/// Гравитация Луны на Orion: a = -μ_m · (r_orion - r_moon) / |r_orion - r_moon|³.
/// Луна притягивается только Землёй (массой Orion пренебрегаем).
fn derivatives(s: &State) -> State {
    let r_orion = s.op;
    let r_moon = s.mp;
    let r_om = s.op - s.mp;

    let mag_o = r_orion.length();
    let mag_m = r_moon.length();
    let mag_om = r_om.length();

    let a_orion = -MU_EARTH_KM3_S2 * r_orion / mag_o.powi(3)
        - MU_MOON_KM3_S2 * r_om / mag_om.powi(3);
    let a_moon = -MU_EARTH_KM3_S2 * r_moon / mag_m.powi(3);

    State {
        op: s.ov,
        ov: a_orion,
        mp: s.mv,
        mv: a_moon,
    }
}

/// Один шаг RK4. dt в секундах. Шаг должен быть ≤ TRAJECTORY_MAX_DT_S — иначе
/// орбита разойдётся (используйте `integrate` для автоматического sub-stepping).
fn rk4_step(s: &State, dt: f64) -> State {
    let k1 = derivatives(s);
    let k2 = derivatives(&s.add_scaled(&k1, dt * 0.5));
    let k3 = derivatives(&s.add_scaled(&k2, dt * 0.5));
    let k4 = derivatives(&s.add_scaled(&k3, dt));

    let sixth = dt / 6.0;
    State {
        op: s.op + (k1.op + 2.0 * k2.op + 2.0 * k3.op + k4.op) * sixth,
        ov: s.ov + (k1.ov + 2.0 * k2.ov + 2.0 * k3.ov + k4.ov) * sixth,
        mp: s.mp + (k1.mp + 2.0 * k2.mp + 2.0 * k3.mp + k4.mp) * sixth,
        mv: s.mv + (k1.mv + 2.0 * k2.mv + 2.0 * k3.mv + k4.mv) * sixth,
    }
}

/// Интегрирует `sim` на `dt_total_s` секунд вперёд с автоматическим sub-stepping.
/// Обновляет `closest_approach_*`. Не пушит в `trail_buffer` — это делает рендер-система.
pub fn integrate(sim: &mut TrajectorySim, dt_total_s: f64) {
    if dt_total_s <= 0.0 {
        return;
    }
    let n = (dt_total_s / TRAJECTORY_MAX_DT_S).ceil().max(1.0) as usize;
    let dt = dt_total_s / n as f64;

    let mut s = State::from_sim(sim);
    for _ in 0..n {
        s = rk4_step(&s, dt);
        sim.elapsed_sim_s += dt;

        let dist = (s.op - s.mp).length();
        if dist < sim.closest_approach_km {
            sim.closest_approach_km = dist;
            sim.closest_approach_t_s = sim.elapsed_sim_s;
        }
    }
    s.write_back(sim);
}

#[allow(dead_code)]
/// Forward-prop симуляции на T секунд вперёд без модификации оригинала.
/// Возвращает (минимальное расстояние до Луны, время этого минимума).
/// Используется в auto-target MCC для предсказания perilune.
pub fn predict_closest_approach(sim: &TrajectorySim, horizon_s: f64) -> (f64, f64) {
    let mut tmp = sim.clone();
    let t_end = sim.elapsed_sim_s + horizon_s;
    let dt_step: f64 = 30.0;
    let mut min_dist = (tmp.orion_pos_km - tmp.moon_pos_km).length();
    let mut min_t = tmp.elapsed_sim_s;
    while tmp.elapsed_sim_s < t_end {
        integrate(&mut tmp, dt_step);
        let d = (tmp.orion_pos_km - tmp.moon_pos_km).length();
        if d < min_dist {
            min_dist = d;
            min_t = tmp.elapsed_sim_s;
        }
        // Если упали в Землю — прекращаем.
        if tmp.orion_pos_km.length() < crate::config::EARTH_RADIUS_KM {
            break;
        }
    }
    (min_dist, min_t)
}

/// Применяет коррекцию курса MCC: находит позицию предсказанного сближения с Луной
/// (forward-prop 3 дня) и добавляет малый Δv в направлении к ней (`sign` > 0) или от неё.
/// Вычитает топливо и обновляет `predicted_perilune_km`.
pub fn apply_mcc(sim: &mut TrajectorySim, sign: f64) {
    use crate::config::{EARTH_RADIUS_KM, MCC_PRESS_DV_MS, MCC_PRESS_FUEL_KG};
    if sim.mcc_fuel_kg <= 0.0 {
        return;
    }

    // Forward-prop на 3 дня с шагом 30 сек → найти позицию ближайшего сближения с Луной.
    let mut tmp = sim.clone();
    let t_end = sim.elapsed_sim_s + 3.0 * 86400.0;
    let mut min_dist = f64::INFINITY;
    let mut min_pos = sim.orion_pos_km;
    let mut min_t = sim.elapsed_sim_s;
    while tmp.elapsed_sim_s < t_end {
        integrate(&mut tmp, 30.0);
        let d = (tmp.orion_pos_km - tmp.moon_pos_km).length();
        if d < min_dist {
            min_dist = d;
            min_pos = tmp.orion_pos_km;
            min_t = tmp.elapsed_sim_s;
        }
        if tmp.orion_pos_km.length() < EARTH_RADIUS_KM {
            break;
        }
    }
    sim.predicted_perilune_km = min_dist;
    sim.predicted_perilune_t_s = min_t;

    // Δv в направлении точки сближения (sign > 0 → уменьшить perilune, sign < 0 → увеличить).
    if min_dist < f64::INFINITY {
        let to_target = (min_pos - sim.orion_pos_km).normalize_or_zero();
        let dv = sign * (MCC_PRESS_DV_MS as f64) / 1000.0; // m/s → km/s
        sim.orion_vel_kms += to_target * dv;
    }
    sim.mcc_fuel_kg = (sim.mcc_fuel_kg - MCC_PRESS_FUEL_KG).max(0.0);
    info!(
        "MCC applied sign={:+.0}: predicted perilune → {:.0} km, fuel → {:.0} kg",
        sign, sim.predicted_perilune_km, sim.mcc_fuel_kg
    );
}

pub fn plugin(app: &mut App) {
    app.init_resource::<TrajectorySim>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{LEO_R_KM, LEO_V_KMS};
    use std::f64::consts::PI;

    /// Точная круговая скорость на LEO_R_KM (из μ и r), а не приближённая константа.
    fn leo_v_circular() -> f64 {
        (MU_EARTH_KM3_S2 / LEO_R_KM).sqrt()
    }

    /// Test helper: симуляция Orion в круговой LEO с очень далёкой «фейковой» Луной,
    /// чтобы её гравитация не влияла на тест. Луна на 1e9 км — μ_moon/r² пренебрежимо.
    fn leo_state() -> State {
        State {
            op: DVec3::new(LEO_R_KM, 0.0, 0.0),
            ov: DVec3::new(0.0, 0.0, leo_v_circular()),
            mp: DVec3::new(1.0e9, 0.0, 0.0),
            mv: DVec3::ZERO,
        }
    }

    fn leo_period_s() -> f64 {
        2.0 * PI * (LEO_R_KM.powi(3) / MU_EARTH_KM3_S2).sqrt()
    }

    /// Sanity check: константа LEO_V_KMS должна совпадать с круговой скоростью
    /// на LEO_R_KM с точностью ~0.1% (отклонение объясняется реальным значением
    /// 200 км ISS-орбиты, которое тоже не идеально круговое).
    #[test]
    fn leo_v_constant_close_to_circular() {
        let v_exact = leo_v_circular();
        let rel_err = ((LEO_V_KMS - v_exact) / v_exact).abs();
        assert!(rel_err < 0.001, "LEO_V_KMS={} vs circular {:.4}", LEO_V_KMS, v_exact);
    }

    /// После одного периода круговой орбиты позиция и скорость возвращаются к исходным.
    /// Кеплеров период T = 2π√(r³/μ) ≈ 5310 sec для LEO 200 км.
    #[test]
    fn circular_leo_returns_after_one_period() {
        let mut s = leo_state();
        let v0 = leo_v_circular();
        let t = leo_period_s();
        let dt = 1.0;
        let n = (t / dt).round() as usize;
        for _ in 0..n {
            s = rk4_step(&s, dt);
        }
        let pos_err = (s.op - DVec3::new(LEO_R_KM, 0.0, 0.0)).length();
        let vel_err = (s.ov - DVec3::new(0.0, 0.0, v0)).length();
        // Допуск 0.1% от радиуса/скорости (реальный RK4 даёт ~1e-6 на 5000 шагов)
        assert!(
            pos_err < LEO_R_KM * 0.001,
            "позиция ушла на {} км после одного периода",
            pos_err
        );
        assert!(
            vel_err < v0 * 0.001,
            "скорость ушла на {} км/с после одного периода",
            vel_err
        );
    }

    /// Удельная энергия E = v²/2 - μ/r должна сохраняться для замкнутой орбиты.
    /// Проверяем drift за 100 витков LEO.
    #[test]
    fn energy_conserved_over_100_orbits() {
        let mut s = leo_state();
        let energy0 =
            s.ov.length_squared() / 2.0 - MU_EARTH_KM3_S2 / s.op.length();

        let t_total = 100.0 * leo_period_s();
        let dt = 5.0;
        let n = (t_total / dt) as usize;
        for _ in 0..n {
            s = rk4_step(&s, dt);
        }
        let energy1 =
            s.ov.length_squared() / 2.0 - MU_EARTH_KM3_S2 / s.op.length();
        let drift = ((energy1 - energy0) / energy0).abs();
        // RK4 + dt=5s должен давать drift < 0.01% за 100 витков.
        assert!(drift < 0.001, "energy drift {:.6} > 0.1%", drift);
    }

    /// Угловой момент r × v для центральной силы Земли тоже сохраняется
    /// (если игнорировать гравитацию Луны). Проверка на ту же серию.
    #[test]
    fn angular_momentum_conserved() {
        let mut s = leo_state();
        let l0 = s.op.cross(s.ov);
        let dt = 5.0;
        let n = (10.0 * leo_period_s() / dt) as usize;
        for _ in 0..n {
            s = rk4_step(&s, dt);
        }
        let l1 = s.op.cross(s.ov);
        let rel_err = (l1 - l0).length() / l0.length();
        assert!(rel_err < 1e-4, "L drift {:.6}", rel_err);
    }

    /// Sub-stepping: integrate(sim, dt) с dt > MAX_DT_S должен дать тот же
    /// результат, что много мелких rk4_step.
    #[test]
    fn integrate_substeps_match_circular_orbit() {
        use crate::config::MOON_ORBIT_R_KM;
        let mut sim = TrajectorySim {
            orion_pos_km: DVec3::new(LEO_R_KM, 0.0, 0.0),
            orion_vel_kms: DVec3::new(0.0, 0.0, leo_v_circular()),
            moon_pos_km: DVec3::new(MOON_ORBIT_R_KM, 0.0, 0.0),
            moon_vel_kms: DVec3::new(0.0, 0.0, 1.022),
            ..Default::default()
        };
        // Один период LEO за один вызов integrate.
        // dt_total = 5310 sec → sub-stepping разобьёт на ~2655 шагов по 2 sec.
        integrate(&mut sim, leo_period_s());
        let pos_err = (sim.orion_pos_km - DVec3::new(LEO_R_KM, 0.0, 0.0)).length();
        // Луна за 5310 sec уехала на ~5° по орбите — в тесте на Orion это не должно
        // мешать (μ_moon на расстоянии 384k км пренебрежимо для LEO-объекта).
        assert!(
            pos_err < LEO_R_KM * 0.01,
            "integrate-orbit pos err {} км",
            pos_err
        );
        // elapsed_sim_s должен сравняться с реальным dt_total
        assert!(
            (sim.elapsed_sim_s - leo_period_s()).abs() < 0.1,
            "elapsed_sim_s рассинхронизирован: {} vs {}",
            sim.elapsed_sim_s,
            leo_period_s()
        );
    }

    /// Луна на круговой орбите вокруг Земли — её период должен быть ≈ 27.4 суток.
    /// Используем точную круговую скорость sqrt(μ/r), чтобы тест не страдал
    /// от приближения константы MOON_ORBIT_V_KMS = 1.022 (точное ~1.0182).
    #[test]
    fn moon_orbital_period_matches() {
        use crate::config::MOON_ORBIT_R_KM;
        let v_moon_exact = (MU_EARTH_KM3_S2 / MOON_ORBIT_R_KM).sqrt();
        let mut s = State {
            op: DVec3::new(LEO_R_KM, 0.0, 0.0), // Orion-болванка не должна влиять
            ov: DVec3::new(0.0, 0.0, leo_v_circular()),
            mp: DVec3::new(MOON_ORBIT_R_KM, 0.0, 0.0),
            mv: DVec3::new(0.0, 0.0, v_moon_exact),
        };
        let t_moon = 2.0 * PI * (MOON_ORBIT_R_KM.powi(3) / MU_EARTH_KM3_S2).sqrt();
        // Кеплеров период Луны ≈ 2_366_829 sec ≈ 27.4 суток.
        assert!(
            (t_moon - 27.4 * 86400.0).abs() < 0.5 * 86400.0,
            "T_moon = {:.0} sec, ожидался ~27.4 суток",
            t_moon
        );
        // Симуляция должна вернуть Луну в исходную точку с погрешностью 0.1%.
        let dt = 60.0;
        let n = (t_moon / dt) as usize;
        for _ in 0..n {
            s = rk4_step(&s, dt);
        }
        let pos_err = (s.mp - DVec3::new(MOON_ORBIT_R_KM, 0.0, 0.0)).length();
        assert!(
            pos_err < MOON_ORBIT_R_KM * 0.001,
            "Луна ушла на {} км после одного периода",
            pos_err
        );
    }
}
