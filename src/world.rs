use crate::input::Player;
use crate::vehicle::spawn_vehicle;
use avian3d::prelude::{Collider, ColliderConstructor, ColliderConstructorHierarchy};
use avian3d::prelude::RigidBody;
use bevy::prelude::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_world);
    }
}

fn setup_world(
    mut commands: Commands,
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

    let vehicle = spawn_vehicle(&mut commands, Vec3::new(0.0, 3.0, 0.0));
    commands.entity(vehicle).insert(Player {
        half_extents: Vec3::splat(0.25),
        weapon_energy: 1.0,
        ..default()
    });
}
