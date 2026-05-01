//! Общее окружение стартового комплекса для MainMenu, Prelaunch и Launch:
//! бетонная плита, трава, подъездная полоса для Crawler'а, дальние холмы
//! и редкая хвойная посадка для глубины кадра.

use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use rand::RngExt;

use crate::states::MissionStage;

/// Цвет неба (светло-голубой, слегка выцветший флоридский день).
pub const SKY_COLOR: Color = Color::srgb(0.55, 0.70, 0.84);

/// Спавнит весь набор «земного» окружения и помечает его `DespawnOnExit(state)`.
/// Вызывается из `setup_scene` стадий MainMenu / Prelaunch / Launch.
pub fn spawn_environment(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    state: MissionStage,
) {
    let grass_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.32, 0.42, 0.20),
        perceptual_roughness: 0.95,
        ..default()
    });
    let pad_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.55, 0.54, 0.50),
        perceptual_roughness: 0.85,
        ..default()
    });
    let road_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.40, 0.38, 0.34),
        perceptual_roughness: 0.92,
        ..default()
    });
    let hill_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.32, 0.16),
        perceptual_roughness: 0.98,
        ..default()
    });
    let tree_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.10, 0.22, 0.10),
        perceptual_roughness: 1.0,
        ..default()
    });

    // Травянистая поверхность — большая, чтобы не было обрыва при подъёме.
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(1500.0, 1500.0))),
        MeshMaterial3d(grass_mat),
        Transform::from_xyz(0.0, -0.6, 0.0),
        DespawnOnExit(state.clone()),
    ));

    // Бетонная плита у стартового стола.
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(40.0, 40.0))),
        MeshMaterial3d(pad_mat),
        Transform::from_xyz(0.0, -0.55, 0.0),
        DespawnOnExit(state.clone()),
    ));

    // Подъездная полоса (Crawlerway) — узкий длинный прямоугольник.
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(8.0, 0.05, 240.0))),
        MeshMaterial3d(road_mat),
        Transform::from_xyz(0.0, -0.5, -120.0),
        DespawnOnExit(state.clone()),
    ));

    // Дальние холмы — 6 крупных сплющенных эллипсоидов по периметру.
    let hill_mesh = meshes.add(Sphere::new(80.0));
    let hill_positions = [
        Vec3::new(-380.0, -10.0, -260.0),
        Vec3::new(360.0, -10.0, -300.0),
        Vec3::new(-420.0, -10.0, 80.0),
        Vec3::new(440.0, -10.0, 60.0),
        Vec3::new(-200.0, -10.0, -480.0),
        Vec3::new(220.0, -10.0, -460.0),
    ];
    for pos in hill_positions {
        commands.spawn((
            Mesh3d(hill_mesh.clone()),
            MeshMaterial3d(hill_mat.clone()),
            Transform::from_translation(pos).with_scale(Vec3::new(1.4, 0.35, 1.4)),
            DespawnOnExit(state.clone()),
        ));
    }

    // Редкая хвойная посадка для глубины — 40 «ёлок».
    let tree_mesh = meshes.add(Cone {
        radius: 2.0,
        height: 8.0,
    });
    let mut rng = rand::rng();
    for _ in 0..40 {
        // Радиус 60..220, угол 0..tau, исключая узкий сектор перед стартовым столом.
        let theta = rng.random::<f32>() * std::f32::consts::TAU;
        let r = 60.0 + rng.random::<f32>() * 160.0;
        let x = r * theta.cos();
        let z = r * theta.sin();
        // Не ставить деревья прямо на crawlerway (полоса z<0 ширины 8).
        if z < 0.0 && x.abs() < 6.0 {
            continue;
        }
        let scale = 0.7 + rng.random::<f32>() * 0.7;
        commands.spawn((
            Mesh3d(tree_mesh.clone()),
            MeshMaterial3d(tree_mat.clone()),
            Transform::from_xyz(x, -0.5 + 4.0 * scale, z).with_scale(Vec3::splat(scale)),
            DespawnOnExit(state.clone()),
        ));
    }
}
