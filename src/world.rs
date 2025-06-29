use crate::globals::PLAYER_HALF_EXTENTS;
use crate::input::Player;
use avian3d::prelude::Collider;
use avian3d::prelude::{LinearVelocity, RigidBody};
use bevy::prelude::*;
use bevy::render::mesh::MeshAabb;
use bevy::render::primitives::Aabb;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_world);
    }
}

fn setup_world(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    _materials: Res<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let terrain_scene: Handle<Scene> = asset_server.load("models/terrain.glb#Scene0");
    commands
        .spawn(SceneRoot(terrain_scene))
        .insert(Transform::default())
        .insert(GlobalTransform::default())
        .insert(Collider::cuboid(100.0, 0.1, 100.0))
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

    let mesh_handle: Handle<Mesh> = asset_server.load("models/car.glb#Mesh0/Primitive0");

    let default_half_extents = Vec3::splat(0.5);

    let _half_extents = meshes
        .get(&mesh_handle)
        .and_then(|m| m.compute_aabb())
        .map(|aabb: Aabb| (aabb.max() - aabb.min()) * 0.5) // ← half the full size
        .unwrap_or(Vec3A::from(default_half_extents));

    let car_scene: Handle<Scene> = asset_server.load("models/car.glb#Scene0");
    commands
        .spawn(SceneRoot(car_scene))
        .insert(Transform::from_xyz(0.0, 1.5, 0.0))
        .insert(GlobalTransform::default())
        .insert(RigidBody::Kinematic)
        .insert(LinearVelocity::ZERO)
        .insert(Player {
            speed: 0.0,
            vertical_vel: 0.0,
            yaw: 0.0,
            half_extents: PLAYER_HALF_EXTENTS,
            grounded: false,
        });
}
