use std::cmp::Ordering;
use crate::globals::GameParams;
use bevy::input::{keyboard::KeyCode, ButtonInput};
use bevy::prelude::*;
use bevy_rapier3d::control::CharacterCollision;
use bevy_rapier3d::prelude::{
    KinematicCharacterController,
    KinematicCharacterControllerOutput,
};

const FALL_THRESHOLD: f32 = -10.0;
const RESPAWN_POS: Vec3 = Vec3::new(0.0, 3.0, 0.0);

const HORIZONTAL_BOUNCE_DAMPING: f32 = 0.1;
const VERTICAL_BOUNCE_DAMPING: f32 = 0.1;

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub vertical_vel: f32,
    pub yaw: f32,
}

pub struct PlayerControlPlugin;
impl Plugin for PlayerControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                player_movement_system,
                slope_adjust_system.after(player_movement_system),
                collision_response_system.after(slope_adjust_system),
                fall_reset_system,
            ),
        );
    }
}

fn player_movement_system(
    time: Res<Time>,
    params: Res<GameParams>,
    keys: Res<ButtonInput<KeyCode>>,
    mut q: Query<(&mut Transform, &mut Player, &mut KinematicCharacterController)>,
) {
    let dt = time.delta_secs();

    for (mut tf, mut plyr, mut ctrl) in &mut q {
        if keys.pressed(KeyCode::ArrowUp) {
            plyr.speed = (plyr.speed + params.acceleration * dt).min(params.max_speed);
        } else if keys.pressed(KeyCode::ArrowDown) {
            plyr.speed = (plyr.speed - params.brake_acceleration * dt).max(-params.max_speed);
        } else {
            plyr.speed =
                plyr.speed.signum() * (plyr.speed.abs() - params.friction * dt).max(0.0);
        }

        if keys.pressed(KeyCode::ArrowLeft) {
            plyr.yaw += params.rotation_speed * dt;
        } else if keys.pressed(KeyCode::ArrowRight) {
            plyr.yaw -= params.rotation_speed * dt;
        }
        plyr.vertical_vel -= params.gravity * dt;

        tf.rotation = Quat::from_axis_angle(Vec3::Y, plyr.yaw);

        let forward = tf.rotation * Vec3::Z;
        let horiz = forward * plyr.speed * dt;
        let vertical = Vec3::Y * plyr.vertical_vel * dt;

        ctrl.translation = Some(horiz + vertical);
    }
}

fn slope_adjust_system(
    mut q: Query<(
        &mut Transform,
        &mut Player,
        Option<&KinematicCharacterControllerOutput>,
    )>,
    params: Res<GameParams>,
) {

    fn find_ground_normal(collisions: &[CharacterCollision]) -> Vec3 {
        collisions
            .iter()
            .filter_map(|c| {
                c.hit
                    .details
                    .as_ref()
                    .map(|d| d.normal2)
                    .map(|v| Vec3::new(v.x, v.y, v.z))
            })
            .filter(|n| n.y > 0.1)
            .max_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(Ordering::Equal))
            .unwrap_or(Vec3::Y)               
            .normalize()
    }

    for (mut tf, mut plyr, out_opt) in &mut q {
        let Some(out) = out_opt else { continue };
        if !out.grounded {
            continue;
        }

        // Pick the steepest *up-facing* collision normal.
        let ground_normal = find_ground_normal(&out.collisions);

        // ─ tilt player to match ramp ─
        let tilt = Quat::from_rotation_arc(Vec3::Y, ground_normal);
        tf.rotation = tilt * Quat::from_axis_angle(Vec3::Y, plyr.yaw);


        let slope_dot = ground_normal.y;
        let uphill_cutoff = 0.7;

        let accel_scale = if slope_dot >= uphill_cutoff {
            1.0
        } else {
            let t = (uphill_cutoff - slope_dot) / uphill_cutoff;
            1.0 - t * 0.7
        };




        plyr.speed = plyr
            .speed
            .clamp(-params.max_speed * accel_scale, params.max_speed * accel_scale);
    }
}

// 3. Bump + bounce
fn collision_response_system(
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

        if out.grounded {
            plyr.vertical_vel = 0.0;
        }
    }
}

// 4. Yeet-guard
fn fall_reset_system(mut q: Query<(&mut Transform, &mut Player)>) {
    for (mut tf, mut plyr) in &mut q {
        if tf.translation.y < FALL_THRESHOLD {
            tf.translation = RESPAWN_POS;
            plyr.speed = 0.0;
            plyr.vertical_vel = 0.0;
        }
    }
}
