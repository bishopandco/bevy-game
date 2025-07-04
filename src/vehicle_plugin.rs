use bevy::prelude::*;
use avian3d::prelude::*;

use crate::vehicle::{VehicleTuning};
use crate::controls::{vehicle_input_system, wheel_steer_system};
use crate::suspension::suspension_system;

pub struct VehiclePlugin;

impl Plugin for VehiclePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(VehicleTuning::default())
            .add_plugins((
                VehiclePhysicsPlugin,
                SuspensionPlugin,
                WheelSteeringPlugin,
            ));
    }
}

pub struct VehiclePhysicsPlugin;
impl Plugin for VehiclePhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, vehicle_input_system);
    }
}

pub struct SuspensionPlugin;
impl Plugin for SuspensionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, suspension_system);
    }
}

pub struct WheelSteeringPlugin;
impl Plugin for WheelSteeringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, wheel_steer_system);
    }
}
