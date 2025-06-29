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

    commands.insert_resource(AmbientLight {
        brightness: 0.5,
        ..default()
    });
    commands
        .spawn(DirectionalLight {
            illuminance: 3_000.0,
            shadows_enabled: true,
            ..default()
        })
        .insert(Transform::from_xyz(5.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y));

    let mesh = meshes.add(Cuboid::new(0.25, 0.25, 0.25));
    commands
        .spawn(Mesh3d(mesh))
        .insert(MeshMaterial3d(materials.add(Color::srgb(0.2, 0.8, 0.2))))
        .insert(Transform::from_xyz(0.0, 3.0, 0.0))
        .insert(RigidBody::Kinematic)
        .insert(Collider::cuboid(0.25, 0.25, 0.25))
        .insert(LinearVelocity::ZERO)
        .insert(Player {
            speed: 0.0,
            vertical_vel: 0.0,
            yaw: 0.0,
            half_extents: Vec3::splat(0.25),
            grounded: false,
        });
}
