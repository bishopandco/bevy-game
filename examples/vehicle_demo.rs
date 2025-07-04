use bevy::prelude::*;
use avian3d::prelude::*;

use game_demo::vehicle_plugin::VehiclePlugin;
use game_demo::vehicle::spawn_vehicle;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .insert_resource(Gravity(Vec3::new(0.0, -9.81, 0.0)))
        .add_plugins(VehiclePlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(DirectionalLight::default());
    let ground = meshes.add(Cuboid::new(50.0, 1.0, 50.0));
    commands
        .spawn(Mesh3d(ground))
        .insert(MeshMaterial3d(materials.add(Color::srgb(0.3, 0.7, 0.3))))
        .insert(Transform::from_xyz(0.0, -1.0, 0.0))
        .insert(RigidBody::Static)
        .insert(Collider::cuboid(50.0, 1.0, 50.0));
    spawn_vehicle(&mut commands, Vec3::new(0.0, 2.0, 0.0));
}
