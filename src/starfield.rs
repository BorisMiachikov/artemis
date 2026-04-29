use bevy::prelude::*;
use rand::RngExt;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_stars);
}

#[derive(Component)]
struct Star;

fn spawn_stars(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Sphere::new(0.06));
    let mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        emissive: LinearRgba::rgb(3.0, 3.2, 4.0),
        unlit: true,
        ..default()
    });

    let mut rng = rand::rng();
    for _ in 0..2000 {
        let theta: f32 = rng.random::<f32>() * std::f32::consts::TAU;
        let cos_phi: f32 = rng.random::<f32>() * 2.0 - 1.0;
        let sin_phi = (1.0_f32 - cos_phi * cos_phi).sqrt();
        let r = 850.0_f32 + rng.random::<f32>() * 150.0;

        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(mat.clone()),
            Transform::from_xyz(
                r * sin_phi * theta.cos(),
                r * sin_phi * theta.sin(),
                r * cos_phi,
            ),
            Star,
        ));
    }

    info!("starfield: 2000 звёзд размещено");
}
