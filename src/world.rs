use crate::input::Player;
use avian3d::prelude::{
    ColliderConstructor, ColliderConstructorHierarchy, LinearVelocity, RigidBody,
};
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    /* ── terrain ─────────────────────────────────────── */
    let terrain_scene: Handle<Scene> = asset_server.load("models/terrain.glb#Scene0");
    commands
        .spawn(SceneRoot(terrain_scene))
        .insert(Transform::default())
        .insert(GlobalTransform::default())
        .insert(ColliderConstructorHierarchy::new(
            ColliderConstructor::TrimeshFromMesh,
        ))
        .insert(RigidBody::Static);

    /* ── lighting ────────────────────────────────────── */
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

    /* ── player car ──────────────────────────────────── */
    let mesh_handle: Handle<Mesh> = asset_server.load("models/car.glb#Mesh0/Primitive0");

    // Fallback size until mesh finishes loading
    let default_half_extents = Vec3::splat(0.5);

    // Compute half-extents manually (Aabb::half_extents was removed in 0.16)
    let half_extents = meshes
        .get(&mesh_handle)
        .and_then(|m| m.compute_aabb())
        .map(|aabb: Aabb| (aabb.max() - aabb.min()) * 0.5) // ← half the full size
        .unwrap_or(Vec3A::from(default_half_extents));

    let car_scene: Handle<Scene> = asset_server.load("models/car.glb#Scene0");
    commands
        .spawn(SceneRoot(car_scene))
        .insert(Transform::from_xyz(0.0, 3.0, 0.0))
        .insert(GlobalTransform::default())
        // Use the real mesh for collision
        .insert(ColliderConstructorHierarchy::new(
            ColliderConstructor::TrimeshFromMesh,
        ))
        .insert(RigidBody::Kinematic)
        .insert(LinearVelocity::ZERO)
        .insert(Player {
            speed: 0.0,
            vertical_vel: 0.0,
            yaw: 0.0,
            half_extents: Vec3::from(half_extents),
        });
}
