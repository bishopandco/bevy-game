use bevy::prelude::*;
use avian3d::prelude::*;

use crate::vehicle::{Vehicle, Wheel};

pub fn vehicle_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut q: Query<&mut LinearVelocity, With<Vehicle>>,
) {
    let a = (keys.pressed(KeyCode::ArrowUp) as i32 - keys.pressed(KeyCode::ArrowDown) as i32) as f32 * 10.0;
    for mut vel in &mut q {
        vel.z += a;
        if keys.pressed(KeyCode::Space) { vel.x *= 0.5; vel.z *= 0.5; }
    }
}

pub fn wheel_steer_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut q: Query<&mut Transform, With<Wheel>>,
) {
    let steer = if keys.pressed(KeyCode::ArrowLeft) { 0.02 } else if keys.pressed(KeyCode::ArrowRight) { -0.02 } else { 0.0 };
    if steer != 0.0 {
        for mut tf in &mut q { tf.rotation *= Quat::from_rotation_y(steer); }
    }
}
