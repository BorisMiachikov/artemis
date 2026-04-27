#![allow(dead_code)] // используется с Фазы 3 (физика)

use bevy::prelude::*;

pub const G0: f32 = 9.80665;

pub const RS25_ISP_S: f32 = 453.0;
pub const SRB_ISP_S: f32 = 269.0;

pub const GM_EARTH: f64 = 3.986e14;
pub const GM_MOON: f64 = 4.9e12;

pub const EARTH_RADIUS_KM: f64 = 6_371.0;
pub const MOON_RADIUS_KM: f64 = 1_737.4;

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Difficulty {
    Story,
    #[default]
    Realistic,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Difficulty>();
}
