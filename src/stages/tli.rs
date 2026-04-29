use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;

use crate::assets::GameAssets;
use crate::camera::PlayerVehicle;
use crate::config::{G0, IcpsParams, TimeScale, TliResult};
use crate::events::MissionEvent;
use crate::states::MissionStage;

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TliBurnState {
    #[default]
    Idle,
    Burning,
    Completed,
}

/// Локальная масса связки во время горения. Отдельный ресурс, чтобы не мешать
/// `IcpsParams` (который read-only справочник параметров).
#[derive(Resource, Clone, Copy, Debug)]
pub struct IcpsBurn {
    pub mass_kg: f32,
    pub elapsed_s: f32,
}

impl Default for IcpsBurn {
    fn default() -> Self {
        Self {
            mass_kg: 0.0,
            elapsed_s: 0.0,
        }
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<TliBurnState>()
        .init_resource::<IcpsBurn>()
        .add_systems(OnEnter(MissionStage::TLI), setup_tli_scene)
        .add_systems(
            Update,
            (handle_burn_input, tick_burn).run_if(in_state(MissionStage::TLI)),
        );
}

fn setup_tli_scene(
    mut commands: Commands,
    assets: Res<GameAssets>,
    icps: Res<IcpsParams>,
    mut burn_state: ResMut<TliBurnState>,
    mut burn: ResMut<IcpsBurn>,
    mut tli: ResMut<TliResult>,
) {
    *burn_state = TliBurnState::Idle;
    *burn = IcpsBurn {
        mass_kg: icps.stack_mass_kg,
        elapsed_s: 0.0,
    };
    *tli = TliResult::default();

    // Освещение и Земля «уходят за корму». Орион в кадре по центру.
    commands.spawn((
        DirectionalLight {
            illuminance: 28_000.0,
            color: Color::srgb(1.0, 0.95, 0.88),
            ..default()
        },
        Transform::from_xyz(40.0, 20.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
        DespawnOnExit(MissionStage::TLI),
    ));

    // Земля сзади — даёт ощущение убегания от планеты при burn.
    commands.spawn((
        SceneRoot(assets.earth.clone()),
        Transform::from_xyz(0.0, 0.0, -120.0).with_scale(Vec3::splat(40.0)),
        DespawnOnExit(MissionStage::TLI),
    ));

    // Orion + ICPS — в центре сцены.
    commands.spawn((
        SceneRoot(assets.orion.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(4.0)),
        PlayerVehicle,
        DespawnOnExit(MissionStage::TLI),
    ));

    info!("stages/tli: scene ready, ожидаем команду на burn");
}

fn handle_burn_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut burn_state: ResMut<TliBurnState>,
    mut events: MessageWriter<MissionEvent>,
) {
    if *burn_state == TliBurnState::Idle && keys.just_pressed(KeyCode::Space) {
        *burn_state = TliBurnState::Burning;
        events.write(MissionEvent::TliBurnStart);
        info!("stages/tli: TLI burn START");
    }
}

#[allow(clippy::too_many_arguments)] // система собирает burn-state, физику и переход стадии — всё нужно
fn tick_burn(
    time: Res<Time>,
    timescale: Res<TimeScale>,
    icps: Res<IcpsParams>,
    mut burn_state: ResMut<TliBurnState>,
    mut burn: ResMut<IcpsBurn>,
    mut tli: ResMut<TliResult>,
    mut events: MessageWriter<MissionEvent>,
    mut next_stage: ResMut<NextState<MissionStage>>,
) {
    if *burn_state != TliBurnState::Burning {
        return;
    }

    let dt = time.delta_secs() * timescale.multiplier;
    if dt <= 0.0 {
        return;
    }

    // Mass flow: ṁ = F / (Isp · g0). Расход топлива и проверка минимальной массы.
    let mass_flow = (icps.thrust_kn * 1000.0) / (icps.isp_s * G0);
    let burned = (mass_flow * dt).min((burn.mass_kg - icps.stack_dry_mass_kg).max(0.0));
    burn.mass_kg -= burned;
    burn.elapsed_s += dt;

    // Δv от этого шага по формуле Циолковского: Δv = Isp · g0 · ln(m₀ / m₁).
    if burned > 0.0 {
        let m0 = burn.mass_kg + burned;
        let m1 = burn.mass_kg.max(icps.stack_dry_mass_kg);
        let dv_step = icps.isp_s * G0 * (m0 / m1).ln();
        tli.delta_v_ms += dv_step;
    }

    let burn_target_reached = burn.elapsed_s >= icps.target_burn_duration_s;
    let dv_target_reached = tli.delta_v_ms >= icps.target_delta_v_ms;
    let propellant_dry = burn.mass_kg <= icps.stack_dry_mass_kg + 0.5;

    if burn_target_reached || dv_target_reached || propellant_dry {
        *burn_state = TliBurnState::Completed;
        tli.burn_duration_s = burn.elapsed_s;
        tli.completed = true;
        events.write(MissionEvent::TliBurnEnd);
        info!(
            "stages/tli: TLI burn END — Δv={:.0} m/s, t={:.1}s, точность={:.1}%",
            tli.delta_v_ms,
            tli.burn_duration_s,
            tli.accuracy_pct(icps.target_delta_v_ms),
        );
        next_stage.set(MissionStage::Transit);
    }
}
