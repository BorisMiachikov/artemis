use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::post_process::bloom::{Bloom, BloomCompositeMode, BloomPrefilter};
use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

/// Маркер на основной управляемой сущности (корабль / ракета) в текущем этапе.
/// Используется системами камеры для режимов Cockpit и Chase.
#[derive(Component)]
pub struct PlayerVehicle;

#[derive(Resource, Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum CameraMode {
    Cockpit,
    /// Кокпит, взгляд вдоль `vehicle.up()` (в зенит ракеты).
    CockpitUp,
    /// Кокпит, взгляд против `vehicle.up()` (в надир ракеты).
    CockpitDown,
    Chase,
    #[default]
    External,
    Free,
}

#[derive(Component)]
pub struct GameCamera;

pub fn plugin(app: &mut App) {
    app.add_plugins(PanOrbitCameraPlugin)
        .init_resource::<CameraMode>()
        .add_systems(Startup, setup_camera)
        .add_systems(
            Update,
            (switch_camera_mode, manage_pan_orbit, follow_vehicle_camera).chain(),
        );
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Tonemapping::TonyMcMapface,
        DebandDither::Enabled,
        Bloom {
            intensity: 0.30,
            low_frequency_boost: 0.55,
            low_frequency_boost_curvature: 0.50,
            high_pass_frequency: 1.0,
            prefilter: BloomPrefilter {
                threshold: 0.4,
                threshold_softness: 0.2,
            },
            composite_mode: BloomCompositeMode::EnergyConserving,
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 50.0, 200.0))
            .looking_at(Vec3::new(0.0, 50.0, 0.0), Vec3::Y),
        PanOrbitCamera::default(),
        AmbientLight {
            color: Color::srgb(0.18, 0.20, 0.28),
            brightness: 80.0,
            ..default()
        },
        GameCamera,
    ));
}

/// Цифровые хоткеи камеры (1‑6). Ctrl+1..8 для дебаг‑прыжков по стейтам игнорируем
/// здесь, чтобы не конфликтовать с `ui::debug::handle_stage_jump`.
fn switch_camera_mode(keys: Res<ButtonInput<KeyCode>>, mut mode: ResMut<CameraMode>) {
    if keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight) {
        return;
    }
    if keys.just_pressed(KeyCode::Digit1) {
        *mode = CameraMode::External;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        *mode = CameraMode::CockpitUp;
    }
    if keys.just_pressed(KeyCode::Digit3) {
        *mode = CameraMode::CockpitDown;
    }
    if keys.just_pressed(KeyCode::Digit4) {
        *mode = CameraMode::Cockpit;
    }
    if keys.just_pressed(KeyCode::Digit5) {
        *mode = CameraMode::Chase;
    }
    if keys.just_pressed(KeyCode::Digit6) {
        *mode = CameraMode::Free;
    }
}

/// При переключении режима добавляет или убирает `PanOrbitCamera` с камеры.
/// External/Free — PanOrbit активен; Cockpit/Chase — убран, трансформ управляется вручную.
fn manage_pan_orbit(
    mode: Res<CameraMode>,
    mut commands: Commands,
    cameras: Query<(Entity, Option<&PanOrbitCamera>), With<GameCamera>>,
) {
    if !mode.is_changed() {
        return;
    }
    let Ok((entity, pan_orbit)) = cameras.single() else {
        return;
    };
    match *mode {
        CameraMode::External | CameraMode::Free => {
            if pan_orbit.is_none() {
                commands.entity(entity).insert(PanOrbitCamera::default());
            }
        }
        CameraMode::Cockpit
        | CameraMode::CockpitUp
        | CameraMode::CockpitDown
        | CameraMode::Chase => {
            if pan_orbit.is_some() {
                commands.entity(entity).remove::<PanOrbitCamera>();
            }
        }
    }
}

/// Следит за `PlayerVehicle` в режимах Cockpit/Chase. В остальных режимах не активна.
fn follow_vehicle_camera(
    mode: Res<CameraMode>,
    vehicles: Query<&Transform, (With<PlayerVehicle>, Without<GameCamera>)>,
    mut cameras: Query<&mut Transform, With<GameCamera>>,
) {
    if !matches!(
        *mode,
        CameraMode::Cockpit | CameraMode::CockpitUp | CameraMode::CockpitDown | CameraMode::Chase
    ) {
        return;
    }
    let Ok(vehicle_tr) = vehicles.single() else {
        return;
    };
    let Ok(mut cam_tr) = cameras.single_mut() else {
        return;
    };

    let pos = vehicle_tr.translation;
    let fwd = vehicle_tr.forward();
    let up = vehicle_tr.up();
    let right = vehicle_tr.right();

    match *mode {
        CameraMode::Cockpit => {
            let cockpit_pos = pos + up * 2.0 + fwd * 1.0;
            let look_target = pos + fwd * 50.0;
            *cam_tr = Transform::from_translation(cockpit_pos).looking_at(look_target, up);
        }
        CameraMode::CockpitUp => {
            // Камера сбоку от корпуса, смотрит вдоль оси ракеты в зенит.
            let cam_pos = pos + right * 5.0 + up * 1.5;
            let look_target = pos + up * 40.0;
            *cam_tr = Transform::from_translation(cam_pos).looking_at(look_target, fwd);
        }
        CameraMode::CockpitDown => {
            // Камера сбоку, смотрит вдоль оси ракеты в надир (на сопла).
            let cam_pos = pos + right * 5.0 + up * 1.5;
            let look_target = pos - up * 40.0;
            *cam_tr = Transform::from_translation(cam_pos).looking_at(look_target, fwd);
        }
        CameraMode::Chase => {
            let chase_pos = pos - fwd * 20.0 + up * 5.0;
            *cam_tr = Transform::from_translation(chase_pos).looking_at(pos, Vec3::Y);
        }
        _ => {}
    }
}
