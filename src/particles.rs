use bevy::prelude::*;
use rand::RngExt;

use crate::camera::PlayerVehicle;
use crate::physics::rocket::{FlightPhase, Rocket};
use crate::stages::tli::TliBurnState;
use crate::states::MissionStage;

const SPAWN_INTERVAL_S: f32 = 0.05;
const PARTICLE_LIFETIME_S: f32 = 0.55;
const PARTICLE_SPEED: f32 = 10.0;
const PARTICLE_SPREAD: f32 = 0.28;
const PARTICLE_RADIUS: f32 = 0.22;

pub fn plugin(app: &mut App) {
    app.init_resource::<ExhaustTimer>()
        .add_systems(Startup, setup_particle_materials)
        .add_systems(
            Update,
            (
                tick_exhaust_particles,
                spawn_launch_exhaust.run_if(in_state(MissionStage::Launch)),
                spawn_tli_exhaust.run_if(in_state(MissionStage::TLI)),
            ),
        );
}

#[derive(Resource, Default)]
struct ExhaustTimer(f32);

#[derive(Resource)]
pub struct ParticleMaterials {
    pub srb_mat: Handle<StandardMaterial>,   // оранжевый — SRB
    pub rs25_mat: Handle<StandardMaterial>,  // бело-синий — RS-25
    pub icps_mat: Handle<StandardMaterial>,  // синий — ICPS
    pub mesh: Handle<Mesh>,
}

#[derive(Component)]
pub struct ExhaustParticle {
    remaining: f32,
    total: f32,
    velocity: Vec3,
}

fn setup_particle_materials(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Sphere::new(1.0));
    let srb_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.55, 0.1),
        emissive: LinearRgba::rgb(5.0, 2.2, 0.4),
        unlit: true,
        ..default()
    });
    let rs25_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.85, 0.90, 1.0),
        emissive: LinearRgba::rgb(3.5, 4.0, 6.0),
        unlit: true,
        ..default()
    });
    let icps_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.7, 1.0),
        emissive: LinearRgba::rgb(1.8, 3.5, 7.0),
        unlit: true,
        ..default()
    });
    commands.insert_resource(ParticleMaterials { srb_mat, rs25_mat, icps_mat, mesh });
    info!("particles: материалы выхлопа инициализированы");
}

fn burst(
    commands: &mut Commands,
    mats: &ParticleMaterials,
    mat: Handle<StandardMaterial>,
    origin: Vec3,
    direction: Vec3,
) {
    let mut rng = rand::rng();
    for _ in 0..4 {
        let spread = Vec3::new(
            rng.random::<f32>() * 2.0 - 1.0,
            rng.random::<f32>() * 2.0 - 1.0,
            rng.random::<f32>() * 2.0 - 1.0,
        ) * PARTICLE_SPREAD;
        let vel = (direction + spread).normalize_or_zero()
            * (PARTICLE_SPEED + rng.random::<f32>() * 5.0);
        let lifetime = PARTICLE_LIFETIME_S + rng.random::<f32>() * 0.15;

        commands.spawn((
            Mesh3d(mats.mesh.clone()),
            MeshMaterial3d(mat.clone()),
            Transform::from_translation(origin).with_scale(Vec3::splat(PARTICLE_RADIUS)),
            ExhaustParticle {
                remaining: lifetime,
                total: lifetime,
                velocity: vel,
            },
        ));
    }
}

fn spawn_launch_exhaust(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<ExhaustTimer>,
    mats: Option<Res<ParticleMaterials>>,
    vehicle_q: Query<(&Transform, &Rocket), With<PlayerVehicle>>,
) {
    let Some(mats) = mats else { return };
    timer.0 -= time.delta_secs();
    if timer.0 > 0.0 {
        return;
    }
    timer.0 = SPAWN_INTERVAL_S;

    let Ok((tf, rocket)) = vehicle_q.single() else {
        return;
    };
    if matches!(rocket.phase, FlightPhase::Coast) {
        return;
    }

    let mat = match rocket.phase {
        FlightPhase::BoosterPhase => mats.srb_mat.clone(),
        FlightPhase::CoreOnly => mats.rs25_mat.clone(),
        _ => return,
    };
    // Нозиция: низ ракеты (ракета ориентирована вверх = local +Y).
    let nozzle = tf.translation - *tf.local_y() * 4.0;
    burst(&mut commands, &mats, mat, nozzle, -*tf.local_y());
}

fn spawn_tli_exhaust(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<ExhaustTimer>,
    mats: Option<Res<ParticleMaterials>>,
    burn_state: Res<TliBurnState>,
    vehicle_q: Query<&Transform, With<PlayerVehicle>>,
) {
    let Some(mats) = mats else { return };
    if !matches!(*burn_state, TliBurnState::Burning) {
        return;
    }
    timer.0 -= time.delta_secs();
    if timer.0 > 0.0 {
        return;
    }
    timer.0 = SPAWN_INTERVAL_S;

    let Ok(tf) = vehicle_q.single() else { return };
    // ICPS толкает вперёд (к Луне = -Z), выхлоп идёт назад = +forward.
    let nozzle = tf.translation - *tf.local_z() * 4.0;
    burst(&mut commands, &mats, mats.icps_mat.clone(), nozzle, *tf.local_z());
}

fn tick_exhaust_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut Transform, &mut ExhaustParticle)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut p) in &mut particles {
        p.remaining -= dt;
        if p.remaining <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        tf.translation += p.velocity * dt;
        let frac = (p.remaining / p.total).clamp(0.0, 1.0);
        tf.scale = Vec3::splat(frac * PARTICLE_RADIUS);
    }
}
