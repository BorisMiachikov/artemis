use bevy::prelude::*;

use crate::config::TimeScale;
use crate::physics::rocket::Rocket;
use crate::states::MissionStage;

/// Скорость, с которой A/D смещают ручной офсет тангажа, °/с.
const PITCH_OFFSET_RATE_DEG_S: f32 = 3.0;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, handle_timescale_input).add_systems(
        Update,
        handle_pitch_input.run_if(in_state(MissionStage::Launch)),
    );
}

/// `[` — замедлить, `]` — ускорить. Шкала циклит между 1×/5×/20×.
fn handle_timescale_input(keys: Res<ButtonInput<KeyCode>>, mut scale: ResMut<TimeScale>) {
    if keys.just_pressed(KeyCode::BracketLeft) {
        scale.cycle_down();
    }
    if keys.just_pressed(KeyCode::BracketRight) {
        scale.cycle_up();
    }
}

/// Управление тангажом ракеты в Launch: A/D смещают `pitch_offset_deg`,
/// который суммируется с автопрограммой gravity-turn.
fn handle_pitch_input(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut rockets: Query<&mut Rocket>,
) {
    let dt = time.delta_secs();
    let mut delta = 0.0;
    if keys.pressed(KeyCode::KeyA) {
        delta += PITCH_OFFSET_RATE_DEG_S * dt;
    }
    if keys.pressed(KeyCode::KeyD) {
        delta -= PITCH_OFFSET_RATE_DEG_S * dt;
    }
    if delta == 0.0 {
        return;
    }
    for mut rocket in &mut rockets {
        rocket.pitch_offset_deg = (rocket.pitch_offset_deg + delta).clamp(-30.0, 30.0);
    }
}
