use bevy::prelude::*;
use avian3d::prelude::*;

use crate::vehicle::{Wheel, VehicleTuning};

fn apply_suspension(
    start: Vec3,
    vel: &mut LinearVelocity,
    spatial: &SpatialQuery,
    params: &SuspensionParams,
    radius: f32,
) {
    if let Some(hit) = spatial.cast_ray(start, Dir3::NEG_Y, params.max_travel + radius, false, &SpatialQueryFilter::default()) {
        let disp = params.max_travel + radius - hit.distance;
        let force = params.spring_k * disp - params.damping_c * vel.y;
        vel.y += force * 0.016;
    }
}

pub fn suspension_system(
    spatial: SpatialQuery,
    mut wheels: Query<(&mut LinearVelocity, &Transform, &Wheel)>,
    tuning: Res<VehicleTuning>,
) {
    for (mut vel, tf, wheel) in &mut wheels {
        apply_suspension(tf.translation + Vec3::Y * wheel.radius, &mut vel, &spatial, &tuning.suspension, wheel.radius);
    }
}
