use bevy::prelude::*;

use crate::config::{EARTH_RADIUS_KM, GM_EARTH};

/// Гравитационное ускорение на высоте `altitude_m` над поверхностью Земли,
/// двух-тельная модель: g(r) = GM/r².
pub fn gravity_at_altitude(altitude_m: f32) -> f32 {
    let r = EARTH_RADIUS_KM as f32 * 1000.0 + altitude_m.max(0.0);
    let gm = GM_EARTH as f32;
    gm / (r * r)
}

pub fn plugin(_app: &mut App) {
    // Phase 3: только helper выше. Полная орбитальная механика (LEO, 2-body Земля/Луна) —
    // Phase 4.
}
