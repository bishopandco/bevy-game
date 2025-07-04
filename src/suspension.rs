use bevy::prelude::*;
use avian3d::prelude::*;

use crate::vehicle::{SuspensionParams, Vehicle, VehicleTuning, Wheel};
use bevy::prelude::ChildOf;

fn calc_suspension(
    start: Vec3,
    vel: &mut LinearVelocity,
    spatial: &SpatialQuery,
    params: &SuspensionParams,
    radius: f32,
) -> Option<f32> {
    spatial.cast_ray(start, Dir3::NEG_Y, params.max_travel + radius, false, &SpatialQueryFilter::default()).map(|hit| {
        let disp = params.max_travel + radius - hit.distance;
        let force = params.spring_k * disp - params.damping_c * vel.y;
        vel.y += force * 0.016;
        force
    })
}

pub fn suspension_system(
    spatial: SpatialQuery,
    mut set: ParamSet<(
        Query<(&mut LinearVelocity, &GlobalTransform, &Wheel, &ChildOf<Vehicle>)>,
        Query<&mut LinearVelocity, With<Vehicle>>,
    )>,
    tuning: Res<VehicleTuning>,
) {
    let mut forces = Vec::new();
    for (mut vel, tf, wheel, parent) in &mut set.p0() {
        if let Some(force) = calc_suspension(
            tf.translation() + Vec3::Y * wheel.radius,
            &mut vel,
            &spatial,
            &tuning.suspension,
            wheel.radius,
        ) {
            forces.push((parent.get(), force));
        }
    }

    for (vehicle_entity, force) in forces {
        if let Ok(mut car_vel) = set.p1().get_mut(vehicle_entity) {
            car_vel.y += force * 0.016;
        }
    }
}
