use bevy::prelude::*;
use bevy::math::primitives::Sphere;

#[derive(Component)]
struct SkyDome;

pub struct SkyDomePlugin;

impl Plugin for SkyDomePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_sky_dome)
            .add_systems(Update, follow_camera);
    }
}

fn setup_sky_dome(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let texture: Handle<Image> = asset_server.load("starfield.png");
    let mesh = meshes.add(Mesh::from(Sphere { radius: 500.0 }));
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(texture.clone()),
        emissive_texture: Some(texture),
        unlit: true,
        double_sided: true,
        cull_mode: None,
        ..default()
    });
    commands
        .spawn(Mesh3d(mesh))
        .insert(MeshMaterial3d(material))
        .insert(Transform::from_scale(Vec3::splat(-1.0)))
        .insert(SkyDome);
}

fn follow_camera(
    cam_q: Query<&GlobalTransform, (With<Camera3d>, With<crate::camera::FollowCamera>)>,
    mut sky_q: Query<&mut Transform, With<SkyDome>>,
) {
    let Ok(cam_tf) = cam_q.single() else { return; };
    for mut tf in &mut sky_q {
        tf.translation = cam_tf.translation();
    }
}