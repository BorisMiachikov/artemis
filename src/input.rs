use bevy::prelude::*;

use crate::config::TimeScale;
use crate::events::MissionEvent;
use crate::physics::rocket::{FlightPhase, Rocket, THROTTLE_MAX, THROTTLE_MIN};
use crate::states::MissionStage;

/// Скорость, с которой A/D смещают ручной офсет тангажа, °/с.
const PITCH_OFFSET_RATE_DEG_S: f32 = 3.0;
/// Скорость дросселя при удержании = / − , долей номинала в секунду.
const THROTTLE_RATE_PER_S: f32 = 0.30;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, handle_timescale_input).add_systems(
        Update,
        (
            handle_pitch_input,
            handle_throttle_input,
            handle_engine_cutoff,
        )
            .run_if(in_state(MissionStage::Launch)),
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

/// `=` — добавить тяги, `−` — убавить. Поддерживаем и обычные клавиши, и Numpad+/-.
/// Активно только в CoreOnly (после сброса SRB), как у реальной RS-25.
fn handle_throttle_input(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut rockets: Query<&mut Rocket>,
) {
    let dt = time.delta_secs();
    let mut delta = 0.0;
    if keys.pressed(KeyCode::Equal) || keys.pressed(KeyCode::NumpadAdd) {
        delta += THROTTLE_RATE_PER_S * dt;
    }
    if keys.pressed(KeyCode::Minus) || keys.pressed(KeyCode::NumpadSubtract) {
        delta -= THROTTLE_RATE_PER_S * dt;
    }
    if delta == 0.0 {
        return;
    }
    for mut rocket in &mut rockets {
        if rocket.phase != FlightPhase::CoreOnly {
            continue;
        }
        rocket.throttle_pct = (rocket.throttle_pct + delta).clamp(THROTTLE_MIN, THROTTLE_MAX);
    }
}

/// `X` — ручное выключение двигателей (manual MECO). Гасит все двигатели и
/// принудительно переходит в Orbit. Дальше игра сама покажет в TLI, хватило
/// ли скорости/высоты для нормального продолжения миссии.
fn handle_engine_cutoff(
    keys: Res<ButtonInput<KeyCode>>,
    mut rockets: Query<&mut Rocket>,
    mut events: MessageWriter<MissionEvent>,
    mut next_stage: ResMut<NextState<MissionStage>>,
) {
    if !keys.just_pressed(KeyCode::KeyX) {
        return;
    }
    let mut triggered = false;
    for mut rocket in &mut rockets {
        if rocket.phase != FlightPhase::Coast {
            rocket.phase = FlightPhase::Coast;
            rocket.thrust_kn = 0.0;
            triggered = true;
        }
    }
    if triggered {
        events.write(MissionEvent::Meco);
        next_stage.set(MissionStage::Orbit);
        info!("manual MECO triggered by player (X)");
    }
}
