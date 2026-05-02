use bevy::prelude::*;

use crate::config::{EARTH_RADIUS_KM, GM_EARTH};

/// Гравитационное ускорение на высоте `altitude_m` над поверхностью Земли,
/// двух-тельная модель: g(r) = GM/r².
pub fn gravity_at_altitude(altitude_m: f32) -> f32 {
    let r = EARTH_RADIUS_KM as f32 * 1000.0 + altitude_m.max(0.0);
    let gm = GM_EARTH as f32;
    gm / (r * r)
}

/// Плотность воздуха по экспоненциальной модели ISA, кг/м³.
/// На уровне моря 1.225, шкальная высота 8 000 м. Ниже нуля — насыщаем
/// значением у поверхности (тонкая защита для случая «ракета ещё не оторвалась»).
pub fn air_density(h_m: f32) -> f32 {
    const SEA_LEVEL: f32 = 1.225;
    const SCALE_H_M: f32 = 8_000.0;
    if h_m <= 0.0 {
        SEA_LEVEL
    } else {
        SEA_LEVEL * (-h_m / SCALE_H_M).exp()
    }
}

pub fn plugin(_app: &mut App) {
    // Phase 3: только helper выше. Полная орбитальная механика (LEO, 2-body Земля/Луна) —
    // Phase 4.
}
