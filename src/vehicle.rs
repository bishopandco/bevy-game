use bevy::prelude::*;
use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::math::primitives::Cylinder;
use rand::Rng;

use crate::globals::GameParams;

#[derive(Component, Default)]
pub struct Vehicle {
    pub speed: f32,
    pub yaw: f32,
}

#[derive(Component)]
pub struct Wheel {
    pub is_front: bool,
    pub radius: f32,
    pub rest_offset: Vec3,
    pub suspension: f32,
    pub phase: f32,
    pub rotation: f32,
}

pub struct VehiclePlugin;

impl Plugin for VehiclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_vehicle)
            .add_systems(
                Update,
                (
                    vehicle_input_system,
                    vehicle_move_system.after(vehicle_input_system),
                    wheel_update_system.after(vehicle_move_system),
                ),
            );
    }
}

const WHEEL_RADIUS: f32 = 0.5;
const WHEEL_WIDTH: f32 = 0.3;
const FRONT_AXLE_Z: f32 = 1.5;
const REAR_AXLE_Z: f32 = -1.5;
const AXLE_X: f32 = 1.0;
const SUSPENSION_TRAVEL: f32 = 0.2;

fn spawn_vehicle(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let scene: Handle<Scene> = asset_server.load("models/car.glb#Scene0");
    let vehicle = commands
        .spawn(SceneRoot(scene))
        .insert(Transform::from_xyz(0.0, WHEEL_RADIUS + 0.5, 0.0))
        .insert(GlobalTransform::default())
        .insert(Vehicle::default())
        .id();

    let wheel_mesh = meshes.add(Mesh::from(Cylinder {
        radius: WHEEL_RADIUS,
        half_height: WHEEL_WIDTH * 0.5,
    }));
    let wheel_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.1, 0.1),
        ..default()
    });

    commands.entity(vehicle).with_children(|p| {
        spawn_wheel(
            p,
            wheel_mesh.clone(),
            wheel_material.clone(),
            Vec3::new(AXLE_X, -WHEEL_RADIUS, FRONT_AXLE_Z),
            true,
        );
        spawn_wheel(
            p,
            wheel_mesh.clone(),
            wheel_material.clone(),
            Vec3::new(-AXLE_X, -WHEEL_RADIUS, FRONT_AXLE_Z),
            true,
        );
        spawn_wheel(
            p,
            wheel_mesh.clone(),
            wheel_material.clone(),
            Vec3::new(AXLE_X, -WHEEL_RADIUS, REAR_AXLE_Z),
            false,
        );
        spawn_wheel(
            p,
            wheel_mesh,
            wheel_material,
            Vec3::new(-AXLE_X, -WHEEL_RADIUS, REAR_AXLE_Z),
            false,
        );
    });
}

fn spawn_wheel(
    parent: &mut ChildSpawnerCommands,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    offset: Vec3,
    is_front: bool,
) {
    parent
        .spawn(Mesh3d(mesh))
        .insert(MeshMaterial3d(material))
        .insert(Transform::from_translation(offset))
        .insert(Wheel {
            is_front,
            radius: WHEEL_RADIUS,
            rest_offset: offset,
            suspension: SUSPENSION_TRAVEL,
            phase: rand::thread_rng().gen::<f32>() * std::f32::consts::TAU,
            rotation: 0.0,
        });
}

fn vehicle_input_system(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    params: Res<GameParams>,
    mut q: Query<&mut Vehicle>,
) {
    let dt = time.delta_secs();
    for mut vehicle in &mut q {
        if keys.pressed(KeyCode::KeyW) {
            vehicle.speed = (vehicle.speed + params.acceleration * dt).min(params.max_speed);
        } else if keys.pressed(KeyCode::KeyS) {
            vehicle.speed = (vehicle.speed - params.brake_acceleration * dt).max(-params.max_speed);
        } else {
            vehicle.speed = vehicle.speed.signum()
                * (vehicle.speed.abs() - params.friction * dt).max(0.0);
        }

        if keys.pressed(KeyCode::KeyA) {
            vehicle.yaw += params.yaw_rate * dt;
        }
        if keys.pressed(KeyCode::KeyD) {
            vehicle.yaw -= params.yaw_rate * dt;
        }
    }
}

fn vehicle_move_system(time: Res<Time>, mut q: Query<(&mut Transform, &Vehicle)>) {
    let dt = time.delta_secs();
    for (mut tf, vehicle) in &mut q {
        let yaw_rot = Quat::from_rotation_y(vehicle.yaw);
        tf.rotation = yaw_rot;
        let forward = yaw_rot * Vec3::Z;
        tf.translation += forward * vehicle.speed * dt;
    }
}

fn wheel_update_system(
    time: Res<Time>,
    vehicles: Query<&Vehicle>,
    mut wheels: Query<(&Parent, &mut Transform, &mut Wheel)>,
) {
    let dt = time.delta_secs();
    let elapsed = time.elapsed_seconds_f32();
    for (parent, mut tf, mut wheel) in &mut wheels {
        if let Ok(vehicle) = vehicles.get(parent.get()) {
            wheel.rotation += vehicle.speed * dt / wheel.radius;
            let steer = if wheel.is_front { vehicle.yaw } else { 0.0 };
            tf.rotation = Quat::from_rotation_y(steer) * Quat::from_rotation_x(wheel.rotation);
            let y_off = (elapsed + wheel.phase).sin() * wheel.suspension;
            tf.translation = wheel.rest_offset + Vec3::Y * y_off;
        }
    }
}
