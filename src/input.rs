use bevy::prelude::*;

use crate::config::{TimeScale, WarpLevel, WARP_LEVELS};
use crate::events::MissionEvent;
use crate::physics::rocket::{FlightPhase, Rocket, THROTTLE_MAX, THROTTLE_MIN};
use crate::states::MissionStage;

/// Событие: игрок нажал Q (sign=+1.0) или E (sign=−1.0) для коррекции курса MCC.
#[derive(Message, Debug, Clone, Copy)]
pub struct MccCommand(pub f64);

/// Скорость, с которой A/D смещают ручной офсет тангажа, °/с.
const PITCH_OFFSET_RATE_DEG_S: f32 = 3.0;
/// Скорость дросселя при удержании = / − , долей номинала в секунду.
const THROTTLE_RATE_PER_S: f32 = 0.30;

pub fn plugin(app: &mut App) {
    app.add_message::<MccCommand>()
        .add_systems(Update, handle_timescale_input)
        .add_systems(
            Update,
            (
                handle_pitch_input,
                handle_throttle_input,
                handle_engine_cutoff,
            )
                .run_if(in_state(MissionStage::Launch)),
        )
        .add_systems(
            Update,
            (handle_transit_input,).run_if(in_state(MissionStage::Transit)),
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
/// эмитит [`MissionEvent::Meco`]. Решение, переходить ли в Orbit, принимает
/// арбитр `evaluate_meco_outcome` в `physics::rocket` — он же определяет
/// устойчивая орбита, нестабильная или ракета остаётся в Launch на падение.
fn handle_engine_cutoff(
    keys: Res<ButtonInput<KeyCode>>,
    mut rockets: Query<&mut Rocket>,
    mut events: MessageWriter<MissionEvent>,
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
        info!("manual MECO triggered by player (X)");
    }
}

/// Transit-ввод: `,`/`.` — warp уровень; `Q`/`E` — MCC коррекция курса.
fn handle_transit_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut warp: ResMut<WarpLevel>,
    mut timescale: ResMut<TimeScale>,
    mut mcc_events: MessageWriter<MccCommand>,
) {
    if keys.just_pressed(KeyCode::Period) {
        warp.increase();
        timescale.multiplier = warp.multiplier();
        info!("warp → ×{}", WARP_LEVELS[warp.0]);
    }
    if keys.just_pressed(KeyCode::Comma) {
        warp.decrease();
        timescale.multiplier = warp.multiplier();
        info!("warp → ×{}", WARP_LEVELS[warp.0]);
    }
    if keys.just_pressed(KeyCode::KeyQ) {
        mcc_events.write(MccCommand(1.0));
    }
    if keys.just_pressed(KeyCode::KeyE) {
        mcc_events.write(MccCommand(-1.0));
    }
}
