use bevy::prelude::*;
use bevy::prelude::Update;
use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use crate::globals::GameParams;
use bevy::{math::Dir3};


#[derive(Component)]
pub struct Player {
    pub speed: f32,
}

pub struct PlayerControlPlugin;

impl Plugin for PlayerControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, player_movement_system);
    }
}


fn player_movement_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut q: Query<(&mut Transform, &mut Player)>,
    params: Res<GameParams>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (mut tf, mut player) in &mut q {
        if keys.pressed(KeyCode::ArrowUp) {
            player.speed =
                (player.speed + params.acceleration * dt).clamp(-params.max_speed, params.max_speed);
        } else if keys.pressed(KeyCode::ArrowDown) {
            player.speed =
                (player.speed - params.brake_deceleration * dt).clamp(-params.max_speed, params.max_speed);
        } else {
            player.speed = player.speed.signum()
                * (player.speed.abs() - params.friction * dt).max(0.0);
        }

        if keys.pressed(KeyCode::ArrowLeft) {
            tf.rotate_axis(Dir3::Y,  params.rotation_speed * dt);
        } else if keys.pressed(KeyCode::ArrowRight) {
            tf.rotate_axis(Dir3::Y, -params.rotation_speed * dt);
        }

        let forward = tf.rotation * Vec3::Z;
        tf.translation += forward * player.speed * dt;
    }
}
