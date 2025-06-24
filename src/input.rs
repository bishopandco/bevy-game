use crate::globals::GameParams;
use bevy::input::{keyboard::KeyCode, ButtonInput};
use bevy::prelude::*;
use bevy_rapier3d::prelude::{KinematicCharacterController, KinematicCharacterControllerOutput};

const FALL_THRESHOLD: f32 = -10.0;
const RESPAWN_POS: Vec3 = Vec3::new(0.0, 3.0, 0.0);

const HORIZONTAL_BOUNCE_DAMPING: f32 = 0.9;
const VERTICAL_BOUNCE_DAMPING: f32 = 0.1;

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub vertical_vel: f32,
}

pub struct PlayerControlPlugin;
impl Plugin for PlayerControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                player_movement_system,
                apply_collision_response_system.after(player_movement_system),
                fall_reset_system,
            ),
        );
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
        Option<&KinematicCharacterControllerOutput>,
    )>,
) {
    let dt = time.delta_secs();
    for (mut controller, mut transform, mut player, out) in &mut q {
        if keys.pressed(KeyCode::ArrowUp) {
            player.speed = (player.speed + params.acceleration * dt).min(params.max_speed);
        } else if keys.pressed(KeyCode::ArrowDown) {
            player.speed = (player.speed - params.brake_acceleration * dt).max(-params.max_speed);
        } else {
            player.speed = player.speed.signum() * (player.speed.abs() - params.friction * dt).max(0.0);
        }

        if keys.pressed(KeyCode::ArrowLeft) {
            transform.rotate_y(params.rotation_speed * dt);
        } else if keys.pressed(KeyCode::ArrowRight) {
            transform.rotate_y(-params.rotation_speed * dt);
        }

        player.vertical_vel -= params.gravity * dt;

        if let Some(out) = out {
            if out.grounded {
                player.vertical_vel = 0.0;
            }
        }

        let forward = transform.rotation * Vec3::Z;
        let horiz = forward * player.speed * dt;
        let vertical = Vec3::Y * player.vertical_vel * dt;
        controller.translation = Some(horiz + vertical);
    }
}

fn apply_collision_response_system(
    mut q: Query<(&mut Player, Option<&KinematicCharacterControllerOutput>)>,
) {
    for (mut plyr, out_opt) in &mut q {
        let Some(out) = out_opt else { continue };
        if !out.collisions.is_empty() {
            plyr.speed *= HORIZONTAL_BOUNCE_DAMPING;
            if plyr.vertical_vel < 0.0 {
                plyr.vertical_vel = -(plyr.vertical_vel * VERTICAL_BOUNCE_DAMPING);
            }
        }
    }
}

fn fall_reset_system(mut q: Query<(&mut Transform, &mut Player)>) {
    for (mut tf, mut player) in &mut q {
        if tf.translation.y < FALL_THRESHOLD {
            tf.translation = RESPAWN_POS;
            player.speed = 0.0;
            player.vertical_vel = 0.0;
        }
    }
}
