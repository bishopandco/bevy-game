use avian3d::prelude::*;
use bevy::prelude::*;

use crate::globals::{Controlled, InVehicle};
use crate::input::Player;
use crate::vehicle_systems::{SuspensionTuning, WheelState, CHASSIS_MASS, WHEEL_MOUNTS, WHEEL_RADIUS};

#[derive(Component, Default)]
pub struct Vehicle {
    /// Measured chassis velocity along its forward axis (m/s).
    pub speed: f32,
    pub steering: f32,
    pub wheels: [WheelState; 4],
}

#[derive(Component)]
struct WheelVisual { index: usize }

pub struct VehiclePlugin;

impl Plugin for VehiclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_vehicle)
            .add_systems(Update, (vehicle_toggle_system, sync_player_to_vehicle_system).chain())
            .add_systems(Update, update_wheel_visuals);
    }
}

const ENTER_DISTANCE: f32 = 3.0;

fn spawn_vehicle(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    tuning: Res<SuspensionTuning>,
) {
    // The GLTF is visual only. A convex primitive is the sole chassis collider;
    // wheel visuals cannot collide with the body or intercept ground probes.
    let collider = Collider::cuboid(1.6, 0.5, 3.2);
    let inertia = AngularInertia::from_shape(&collider, CHASSIS_MASS);
    let vehicle = commands.spawn((
        WorldAssetRoot(asset_server.load("models/car.glb#Scene0")),
        Transform::from_xyz(2.0, 1.2, 0.0),
        Vehicle::default(),
        RigidBody::Dynamic,
        collider,
        Mass(CHASSIS_MASS),
        inertia,
        Friction::ZERO,
        Restitution::ZERO,
        LinearVelocity::ZERO,
        AngularVelocity::ZERO,
    )).id();

    let wheel_mesh = meshes.add(Cylinder { radius: WHEEL_RADIUS, half_height: 0.15 });
    let wheel_material = materials.add(Color::srgb(0.1, 0.1, 0.1));
    commands.entity(vehicle).with_children(|parent| {
        for index in 0..4 {
            parent.spawn((
                Mesh3d(wheel_mesh.clone()),
                MeshMaterial3d(wheel_material.clone()),
                Transform::from_translation(WHEEL_MOUNTS[index] - Vec3::Y * tuning.wheels[index].rest_length)
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
                WheelVisual { index },
            ));
        }
    });
}

fn update_wheel_visuals(
    vehicles: Query<&Vehicle>,
    mut wheels: Query<(&ChildOf, &WheelVisual, &mut Transform)>,
) {
    for (parent, visual, mut transform) in &mut wheels {
        let Ok(vehicle) = vehicles.get(parent.parent()) else { continue; };
        let wheel = vehicle.wheels[visual.index];
        transform.translation = WHEEL_MOUNTS[visual.index] - Vec3::Y * wheel.length;
        let steer = if visual.index < 2 { vehicle.steering } else { 0.0 };
        transform.rotation = Quat::from_rotation_y(steer)
            * Quat::from_rotation_x(wheel.spin)
            * Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
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
                    .insert(Visibility::Hidden)
                    .insert(Sensor);
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
                .insert(Visibility::Visible)
                .remove::<Sensor>();
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
