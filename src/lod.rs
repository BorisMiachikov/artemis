use bevy::prelude::*;

use crate::camera::GameCamera;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_lod_materials)
        .add_systems(PostUpdate, tick_lod);
}

/// Материалы и меш для lo-poly сфер — инициализируются один раз в Startup.
#[derive(Resource)]
pub struct LodMaterials {
    pub earth_mat: Handle<StandardMaterial>,
    pub moon_mat: Handle<StandardMaterial>,
    pub lo_mesh: Handle<Mesh>,
}

/// Прикрепляется к hi-poly entity. Переключает отображение на lo-poly сферу,
/// когда объект занимает менее `min_apparent` от расстояния до камеры.
#[derive(Component)]
pub struct DistanceLod {
    pub lo_entity: Entity,
    /// Порог: scale.x / dist. Ниже — показываем lo-poly.
    pub min_apparent: f32,
}

/// Маркер lo-poly сферы — пара для `DistanceLod`.
#[derive(Component)]
pub struct LodSphere;

fn setup_lod_materials(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let lo_mesh = meshes.add(Sphere::new(1.0));
    let earth_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.35, 0.65),
        emissive: LinearRgba::rgb(0.04, 0.10, 0.20),
        unlit: true,
        ..default()
    });
    let moon_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.62, 0.60, 0.57),
        unlit: true,
        ..default()
    });
    commands.insert_resource(LodMaterials { earth_mat, moon_mat, lo_mesh });
    info!("lod: материалы LOD инициализированы");
}

fn tick_lod(
    cam_q: Query<&GlobalTransform, With<GameCamera>>,
    mut hi_q: Query<(&Transform, &DistanceLod, &mut Visibility), Without<LodSphere>>,
    mut lo_q: Query<(&mut Transform, &mut Visibility), With<LodSphere>>,
) {
    let Ok(cam_tf) = cam_q.single() else { return };
    let cam_pos = cam_tf.translation();

    for (hi_tf, lod, mut hi_vis) in &mut hi_q {
        let dist = cam_pos.distance(hi_tf.translation).max(1.0);
        let apparent = hi_tf.scale.x / dist;
        let show_hi = apparent >= lod.min_apparent;

        *hi_vis = if show_hi { Visibility::Inherited } else { Visibility::Hidden };

        if let Ok((mut lo_tf, mut lo_vis)) = lo_q.get_mut(lod.lo_entity) {
            *lo_tf = *hi_tf;
            *lo_vis = if show_hi { Visibility::Hidden } else { Visibility::Inherited };
        }
    }
}
