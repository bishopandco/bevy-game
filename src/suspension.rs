use bevy::prelude::*;
use avian3d::prelude::*;

use crate::vehicle::{Vehicle, Wheel, VehicleTuning, SuspensionParams};
use bevy::prelude::Parent;

fn apply_suspension(
    start: Vec3,
    wheel_vel: &mut LinearVelocity,
    chassis_vel: &mut LinearVelocity,
    spatial: &SpatialQuery,
    params: &SuspensionParams,
    radius: f32,
) {
    if let Some(hit) = spatial.cast_ray(start, Dir3::NEG_Y, params.max_travel + radius, false, &SpatialQueryFilter::default()) {
        let disp = params.max_travel + radius - hit.distance;
        let force = params.spring_k * disp - params.damping_c * wheel_vel.y;
        wheel_vel.y += force * 0.016;
        chassis_vel.y += force * 0.016;
    }
}

pub fn suspension_system(
    spatial: SpatialQuery,
    mut wheels: Query<(&mut LinearVelocity, &GlobalTransform, &Wheel, &Parent)>,
    mut vehicles: Query<&mut LinearVelocity, With<Vehicle>>,
    tuning: Res<VehicleTuning>,
) {
    for (mut vel, tf, wheel, parent) in &mut wheels {
        if let Ok(mut car_vel) = vehicles.get_mut(parent.get()) {
            apply_suspension(tf.translation() + Vec3::Y * wheel.radius, &mut vel, &mut car_vel, &spatial, &tuning.suspension, wheel.radius);
        }
    }
}
