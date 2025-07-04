use bevy::prelude::*;
use avian3d::prelude::*;

use crate::vehicle::{Wheel, VehicleTuning};

pub fn suspension_system(
    spatial: SpatialQuery,
    mut wheels: Query<(&mut LinearVelocity, &Transform, &Wheel)>,
    tuning: Res<VehicleTuning>,
) {
    for (mut vel, tf, wheel) in &mut wheels {
        let start = tf.translation + Vec3::Y * wheel.radius;
        if let Some(hit) = spatial.cast_ray(
            start,
            Dir3::NEG_Y,
            tuning.suspension.max_travel + wheel.radius,
            false,
            &SpatialQueryFilter::default(),
        ) {
            let disp = tuning.suspension.max_travel + wheel.radius - hit.distance;
            let spring = tuning.suspension.spring_k * disp;
            let damp = tuning.suspension.damping_c * vel.y;
            vel.y += (spring - damp) * 0.016;
        }
    }
}
