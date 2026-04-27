use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

#[derive(Resource, Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum CameraMode {
    Cockpit,
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
        .add_systems(Update, switch_camera_mode);
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
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

fn switch_camera_mode(keys: Res<ButtonInput<KeyCode>>, mut mode: ResMut<CameraMode>) {
    if keys.just_pressed(KeyCode::F1) {
        *mode = CameraMode::Cockpit;
    }
    if keys.just_pressed(KeyCode::F2) {
        *mode = CameraMode::Chase;
    }
    if keys.just_pressed(KeyCode::F3) {
        *mode = CameraMode::External;
    }
    if keys.just_pressed(KeyCode::F4) {
        *mode = CameraMode::Free;
    }
}
