use bevy::prelude::*;
use avian3d::prelude::*;

use crate::vehicle::{Vehicle, Wheel};

pub fn vehicle_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut q: Query<&mut LinearVelocity, With<Vehicle>>,
) {
    for mut vel in &mut q {
        if keys.pressed(KeyCode::ArrowUp) {
            vel.z += 10.0;
        }
        if keys.pressed(KeyCode::ArrowDown) {
            vel.z -= 10.0;
        }
        if keys.pressed(KeyCode::Space) {
            vel.x *= 0.5;
            vel.z *= 0.5;
        }
    }
}

pub fn wheel_steer_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut q: Query<&mut Transform, With<Wheel>>,
) {
    for mut tf in &mut q {
        if keys.pressed(KeyCode::ArrowLeft) {
            tf.rotation *= Quat::from_rotation_y(0.02);
        }
        if keys.pressed(KeyCode::ArrowRight) {
            tf.rotation *= Quat::from_rotation_y(-0.02);
        }
    }
}
