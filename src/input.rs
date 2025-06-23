use bevy::prelude::*;
use bevy::prelude::Update;
use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use crate::globals::GameParams;
use bevy::{math::Dir3};
use bevy_rapier3d::control::KinematicCharacterController;


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
    time: Res<Time>,
    params: Res<GameParams>,
    keys: Res<ButtonInput<KeyCode>>,
    mut q: Query<(
        &mut KinematicCharacterController,
        &mut Transform,
        &mut Player,
    )>,
) {
    let dt = time.delta_secs();

    for (mut controller, mut tf, mut player) in &mut q {
        if keys.pressed(KeyCode::ArrowUp) {
            player.speed = (player.speed + params.acceleration * dt).min(params.max_speed);
        } else if keys.pressed(KeyCode::ArrowDown) {
            player.speed = (player.speed - params.brake_deceleration * dt).max(-params.max_speed);
        } else {
            player.speed = player
                .speed
                .signum()
                * (player.speed.abs() - params.friction * dt).max(0.0);
        }

        if keys.pressed(KeyCode::ArrowLeft) {
            tf.rotate_axis(Dir3::Y, params.rotation_speed * dt);
        } else if keys.pressed(KeyCode::ArrowRight) {
            tf.rotate_axis(Dir3::Y, -params.rotation_speed * dt);
        }

        let forward = tf.rotation * Vec3::Z;
        controller.translation = Some(forward * player.speed * dt);
    }
}
