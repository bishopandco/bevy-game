use avian3d::prelude::*;
use bevy::prelude::*;

use crate::globals::{Controlled, InVehicle};
use crate::input::Player;
use crate::vehicle_systems::{SuspensionTuning, WheelState, CHASSIS_MASS};
use crate::vehicle_visuals::{spawn_vehicle_parts, update_vehicle_parts};

#[derive(Component, Default)]
pub struct Vehicle {
    /// Measured chassis velocity along its forward axis (m/s).
    pub speed: f32,
    pub steering: f32,
    pub wheels: [WheelState; 4],
}

pub struct VehiclePlugin;

impl Plugin for VehiclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_vehicle)
            .add_systems(Update, (vehicle_toggle_system, sync_player_to_vehicle_system).chain())
            .add_systems(Update, update_vehicle_parts);
    }
}

const ENTER_DISTANCE: f32 = 3.0;

fn spawn_vehicle(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    tuning: Res<SuspensionTuning>,
) {
    // Visible mechanical parts are children of the one dynamic chassis body.
    // They have no colliders and cannot interfere with suspension ground probes.
    let collider = Collider::cuboid(1.6, 0.5, 3.2);
    let inertia = AngularInertia::from_shape(&collider, CHASSIS_MASS);
    let vehicle = commands.spawn((
        Name::new("Independent suspension chassis"),
        Transform::from_xyz(2.0, 1.2, 0.0),
        Visibility::default(),
        Vehicle::default(),
        RigidBody::Dynamic,
        collider,
        Mass(CHASSIS_MASS),
        CenterOfMass::new(0.0, tuning.center_of_mass_height, 0.0),
        inertia,
        Friction::ZERO,
        Restitution::ZERO,
        LinearVelocity::ZERO,
        AngularVelocity::ZERO,
    )).id();

    spawn_vehicle_parts(&mut commands, vehicle, &mut meshes, &mut materials);
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
