use bevy::prelude::*;
use avian3d::prelude::{ColliderConstructor, ColliderConstructorHierarchy, RigidBody, LinearVelocity, AngularVelocity};
use bevy::ecs::system::ChildBuilder;
use bevy::math::primitives::Cylinder;
use rand::Rng;

use crate::globals::{GameParams, Controlled, InVehicle};
use crate::input::Player;

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
                    vehicle_toggle_system,
                    vehicle_input_system,
                    vehicle_move_system.after(vehicle_input_system),
                    wheel_update_system.after(vehicle_move_system),
                    sync_player_to_vehicle_system,
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
const ENTER_DISTANCE: f32 = 2.0;

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
        .insert(RigidBody::Dynamic)
        .insert(ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh))
        .insert(LinearVelocity::ZERO)
        .insert(AngularVelocity::ZERO)
        .insert(crate::vehicle_systems::Chassis { mass: 800.0 })
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
    parent: &mut ChildBuilder,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    offset: Vec3,
    is_front: bool,
) {
    parent
        .spawn(Mesh3d(mesh))
        .insert(MeshMaterial3d(material))
        .insert(Transform::from_translation(offset))
        .insert(crate::vehicle_systems::RaycastWheel::new(
            offset,
            WHEEL_RADIUS,
            is_front,
            offset.x < 0.0,
        ))
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
    mut q: Query<&mut Vehicle, With<Controlled>>,
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

fn vehicle_move_system(
    time: Res<Time>,
    mut q: Query<(&mut Transform, &Vehicle), With<Controlled>>,
) {
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
    mut wheels: Query<(&ChildOf, &mut Transform, &mut Wheel)>,
) {
    let dt = time.delta_secs();
    let elapsed = time.elapsed_secs();
    for (parent, mut tf, mut wheel) in &mut wheels {
        if let Ok(vehicle) = vehicles.get(parent.parent()) {
            wheel.rotation += vehicle.speed * dt / wheel.radius;
            let steer = if wheel.is_front { vehicle.yaw } else { 0.0 };
            // keep wheel upright while allowing steering and rolling
            tf.rotation =
                Quat::from_rotation_y(steer)
                    * Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)
                    * Quat::from_rotation_x(wheel.rotation);
            let y_off = (elapsed + wheel.phase).sin() * wheel.suspension;
            tf.translation = wheel.rest_offset + Vec3::Y * y_off;
        }
    }
}

fn vehicle_toggle_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut players: Query<
        (Entity, &mut Transform, Option<&InVehicle>, Option<&Controlled>),
        (With<Player>, Without<Vehicle>),
    >,
    mut vehicles: Query<
        (Entity, &Transform, Option<&Controlled>),
        (With<Vehicle>, Without<Player>),
    >,
) {
    if !keys.just_pressed(KeyCode::KeyE) {
        return;
    }
    let (player_ent, mut player_tf, in_vehicle, player_ctrl) = match players.single_mut() {
        Ok(v) => v,
        Err(_) => return,
    };

    if player_ctrl.is_some() {
        for (veh_ent, veh_tf, veh_ctrl) in &mut vehicles {
            if veh_ctrl.is_some() {
                continue;
            }
            if player_tf.translation.distance(veh_tf.translation) < ENTER_DISTANCE {
                commands.entity(player_ent)
                    .remove::<Controlled>()
                    .insert(InVehicle { vehicle: veh_ent })
                    .insert(Visibility::Hidden);
                commands.entity(veh_ent).insert(Controlled);
                player_tf.translation = veh_tf.translation;
                break;
            }
        }
    } else if let Some(occupy) = in_vehicle {
        if let Ok((veh_ent, veh_tf, _)) = vehicles.get(occupy.vehicle) {
            commands.entity(player_ent)
                .insert(Controlled)
                .remove::<InVehicle>()
                .insert(Visibility::Visible);
            commands.entity(veh_ent).remove::<Controlled>();
            player_tf.translation = veh_tf.translation + Vec3::Y;
        }
    }
}

fn sync_player_to_vehicle_system(
    mut players: Query<(&mut Transform, &InVehicle), Without<Vehicle>>,
    vehicles: Query<&Transform, (With<Vehicle>, Without<Player>)>,
) {
    for (mut tf, iv) in &mut players {
        if let Ok(v_tf) = vehicles.get(iv.vehicle) {
            tf.translation = v_tf.translation;
        }
    }
}
