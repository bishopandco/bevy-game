use crate::globals::GameParams;
use avian3d::prelude::*;
use bevy::{
    input::{keyboard::KeyCode, ButtonInput},
    prelude::*,
};

/// treat anything steeper than this as a wall (≈ 53 °)
const WALL_N_Y_THRESHOLD: f32 = 0.6;

/// snap / buffer constants
const RAY_OFFSET: f32 = 0.03;
const SNAP_EPS: f32 = 0.001;

/// misc
const GROUND_RAY_LEN: f32 = 1.0;
const FALL_RESET_Y: f32 = -10.0;
const RESPAWN_POS: Vec3 = Vec3::new(0.0, 3.0, 0.0);

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub vertical_vel: f32,
    pub yaw: f32,
    /// Half-extents derived from the car mesh at startup.
    pub half_extents: Vec3,
}

pub struct PlayerControlPlugin;
impl Plugin for PlayerControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (player_controller, fall_reset_system));
    }
}

fn player_controller(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    params: Res<GameParams>,
    mut spatial: SpatialQuery,
    mut q: Query<(Entity, &mut Transform, &mut Player)>,
) {
    for (entity, mut tf, mut plyr) in &mut q {
        let dt = time.delta_secs();

        /* ───────── throttle / brake ───────── */
        if keys.pressed(KeyCode::ArrowUp) {
            plyr.speed = (plyr.speed + params.acceleration * dt).min(params.max_speed);
        } else if keys.pressed(KeyCode::ArrowDown) {
            plyr.speed = (plyr.speed - params.brake_acceleration * dt).max(-params.max_speed);
        } else {
            let slowed = (plyr.speed.abs() - params.friction * dt).max(0.0);
            plyr.speed = plyr.speed.signum() * slowed;
        }

        /* ───────── yaw ────────────────────── */
        if keys.pressed(KeyCode::ArrowLeft) {
            plyr.yaw += params.yaw_rate * dt;
        }
        if keys.pressed(KeyCode::ArrowRight) {
            plyr.yaw -= params.yaw_rate * dt;
        }
        let yaw_rot = Quat::from_rotation_y(plyr.yaw);

        /* ───────── horizontal step & sweep ── */
        let forward = yaw_rot * Vec3::Z;
        let step_h = forward * plyr.speed * dt;
        let mut ground_n = Vec3::Y; // default up

        if step_h.length_squared() > f32::EPSILON {
            let dist = step_h.length();
            let dir3 = Dir3::new_unchecked(step_h / dist);

            // Inflate the player bounds a hair for the sweep.
            let ext = plyr.half_extents + Vec3::splat(RAY_OFFSET);
            let shape = Collider::cuboid(ext.x, ext.y, ext.z);


            let filter = SpatialQueryFilter::default().with_excluded_entities([entity]);

            if let Some(hit) = spatial.cast_shape(
                &shape,
                tf.translation,
                tf.rotation,
                dir3,
                &ShapeCastConfig::from_max_distance(dist),
                &filter,
            ) {
                // treat steep faces as walls
                if hit.normal1.y < WALL_N_Y_THRESHOLD {
                    // slide along the wall
                    let slide_dir =
                        (step_h - hit.normal1 * step_h.dot(hit.normal1)).normalize_or_zero();
                    let allowed = (hit.distance - RAY_OFFSET).max(0.0);
                    tf.translation += slide_dir * allowed;
                } else {
                    // gentle ramp – keep going
                    let allowed = (hit.distance - RAY_OFFSET).max(0.0);
                    tf.translation += dir3.as_vec3() * allowed;
                    ground_n = hit.normal1;
                }
            } else {
                tf.translation += step_h;
            }
        }

        /* ───────── gravity & vertical ─────── */
        plyr.vertical_vel -= params.gravity * dt;
        tf.translation.y += plyr.vertical_vel * dt;

        /* ───────── ground-snap raycast ────── */
        let half_height = plyr.half_extents.y;
        let ray_origin = tf.translation - Vec3::Y * (half_height - RAY_OFFSET);
        let filter = SpatialQueryFilter::default().with_excluded_entities([entity]);

        if let Some(hit) = spatial.cast_ray(ray_origin, Dir3::NEG_Y, GROUND_RAY_LEN, false, &filter)
        {
            if hit.distance < RAY_OFFSET + SNAP_EPS {
                tf.translation.y += RAY_OFFSET - hit.distance;
                plyr.vertical_vel = 0.0;
                ground_n = hit.normal;
            }
        }

        /* ───────── tilt to ground normal ──── */
        tf.rotation = Quat::from_rotation_arc(Vec3::Y, ground_n) * yaw_rot;
    }
}

fn fall_reset_system(mut q: Query<(&mut Transform, &mut Player)>) {
    for (mut tf, mut plyr) in &mut q {
        if tf.translation.y < FALL_RESET_Y {
            tf.translation = RESPAWN_POS;
            plyr.speed = 0.0;
            plyr.vertical_vel = 0.0;
        }
    }
}

// if let Some(hit) = spatial.cast_shape(
//     &shape,
//     tf.translation,
//     tf.rotation,
//     dir3,
//     &ShapeCastConfig::from_max_distance(dist),
//     &filter,
// ) {
