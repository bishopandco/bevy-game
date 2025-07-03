use crate::input::Player;
use avian3d::prelude::{Collider, ColliderConstructor, ColliderConstructorHierarchy};
use avian3d::prelude::{LinearVelocity, RigidBody};
use bevy::prelude::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_world);
    }
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let terrain: Handle<Scene> = asset_server.load("models/terrain.glb#Scene0");
    commands
        .spawn(SceneRoot(terrain))
        .insert(Transform::from_xyz(0.0, 0.0, 0.0))
        .insert(GlobalTransform::default())
        .insert(ColliderConstructorHierarchy::new(
            ColliderConstructor::TrimeshFromMesh,
        ))
        .insert(RigidBody::Static);

    // commands.insert_resource(AmbientLight {
    //     brightness: 0.25,
    //     ..default()
    // });

    let mesh = meshes.add(Cuboid::new(0.5, 0.5, 0.5));
    let wheel_mesh = meshes.add(Cuboid::new(0.2, 0.2, 0.2));
    let wheel_mat = materials.add(Color::srgb(0.2, 0.2, 0.2));
    let player = Player {
        speed: 0.0,
        vertical_vel: 0.0,
        vertical_input: 0.0,
        yaw: 0.0,
        prev_yaw: 0.0,
        half_extents: Vec3::splat(0.5),
        grounded: false,
        fire_timer: 0.0,
        weapon_energy: 1.0,
    };
    let offsets = crate::input::wheel_offsets(&player);
    commands
        .spawn(Mesh3d(mesh))
        .insert(MeshMaterial3d(materials.add(Color::srgb(0.2, 0.8, 0.2))))
        .insert(Transform::from_xyz(0.0, 3.0, 0.0))
        .insert(RigidBody::Kinematic)
        .insert(Collider::cuboid(0.5, 0.5, 0.5))
        .insert(LinearVelocity::ZERO)
        .insert(player)
        .with_children(|parent| {
            for offset in offsets {
                parent
                    .spawn(Mesh3d(wheel_mesh.clone()))
                    .insert(MeshMaterial3d(wheel_mat.clone()))
                    .insert(Transform::from_translation(offset))
                    .insert(crate::input::Wheel { offset });
            }

            // Headlights - slightly yellow directional lights
            let head_color = Color::srgb(1.0, 1.0, 0.8);
            let front_z = 0.5 + 0.1;
            parent
                .spawn(DirectionalLight {
                    color: head_color,
                    illuminance: 1000.0,
                    ..default()
                })
                .insert(
                    Transform::from_translation(Vec3::new(0.3, 0.0, front_z))
                        .looking_at(Vec3::new(0.3, 0.0, front_z + 1.0), Vec3::Y),
                );
            parent
                .spawn(DirectionalLight {
                    color: head_color,
                    illuminance: 20.0,
                    ..default()
                })
                .insert(
                    Transform::from_translation(Vec3::new(-0.3, 0.0, front_z))
                        .looking_at(Vec3::new(-0.3, 0.0, front_z + 1.0), Vec3::Y),
                );

            // Tail lights - red point lights
            let back_z = -0.5 - 0.1;
            let tail_color = Color::srgb(1.0, 0.0, 0.0);
            parent
                .spawn(PointLight {
                    intensity: 100.0,
                    range: 5.0,
                    color: tail_color,
                    ..default()
                })
                .insert(Transform::from_translation(Vec3::new(0.3, 0.0, back_z)));
            parent
                .spawn(PointLight {
                    intensity: 100.0,
                    range: 5.0,
                    color: tail_color,
                    ..default()
                })
                .insert(Transform::from_translation(Vec3::new(-0.3, 0.0, back_z)));
        });
}
