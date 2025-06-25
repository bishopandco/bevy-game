use crate::globals::{GameParams, PLAYER_HALF_EXTENTS};
use avian3d::prelude::*;
use bevy::input::{keyboard::KeyCode, ButtonInput};
use bevy::prelude::*;

const FALL_THRESHOLD: f32 = -10.0;
const RESPAWN_POS: Vec3 = Vec3::new(0.0, 3.0, 0.0);

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub vertical_vel: f32,
    pub yaw: f32,
}

pub struct PlayerControlPlugin;
impl Plugin for PlayerControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (player_movement_system, fall_reset_system));
    }
}

pub fn player_movement_system(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    params: Res<GameParams>,
    mut spatial: SpatialQuery,
    // grab the entity ID so we can exclude it from queries
    mut q: Query<(Entity, &mut Transform, &mut Player)>,
) {
    const HALF_HEIGHT: f32 = 0.5;
    const RAY_OFFSET: f32 = 0.02;
    const SNAP_EPS: f32 = 0.005;

    for (entity, mut tf, mut plyr) in &mut q {
        let dt = time.delta_secs();

        // ---- input-driven speed ----
        if keys.pressed(KeyCode::ArrowUp) {
            plyr.speed = (plyr.speed + params.acceleration * dt).min(params.max_speed);
        } else if keys.pressed(KeyCode::ArrowDown) {
            plyr.speed = (plyr.speed - params.brake_acceleration * dt).max(-params.max_speed);
        } else {
            let slowed = (plyr.speed.abs() - params.friction * dt).max(0.0);
            plyr.speed = plyr.speed.signum() * slowed;
        }

        // ---- yaw ----
        if keys.pressed(KeyCode::ArrowLeft) {
            plyr.yaw += params.yaw_rate * dt;
        }
        if keys.pressed(KeyCode::ArrowRight) {
            plyr.yaw -= params.yaw_rate * dt;
        }

        // ---- gravity ----
        plyr.vertical_vel -= params.gravity * dt;

        // ---- ground snap ----
        let ray_start = tf.translation + Vec3::Y * (-HALF_HEIGHT + RAY_OFFSET);
        let max_dist = RAY_OFFSET + HALF_HEIGHT; // just past the feet
        let filter = SpatialQueryFilter::default().with_excluded_entities([entity]);

        let ground_hit = spatial.cast_ray(ray_start, Dir3::NEG_Y, max_dist + 0.5, false, &filter);

        if let Some(hit) = ground_hit {
            if hit.distance < max_dist - SNAP_EPS {
                tf.translation.y += max_dist - hit.distance;
                plyr.vertical_vel = 0.0;
            }
        }

        // ---- horizontal movement & slope handling ----
        let yaw_rot = Quat::from_axis_angle(Vec3::Y, plyr.yaw);
        let forward_world = yaw_rot * Vec3::Z;

        let ground_n = ground_hit.map(|h| h.normal).unwrap_or(Vec3::Y);
        let forward_ground =
            (forward_world - ground_n * forward_world.dot(ground_n)).normalize_or_zero();

        let step_h = forward_ground * plyr.speed * dt;
        if step_h.length_squared() > f32::EPSILON {
            let dist = step_h.length();
            let dir_vec = step_h / dist;
            let dir = Dir3::new_unchecked(dir_vec);

            let shape = Collider::cuboid(
                PLAYER_HALF_EXTENTS.x,
                PLAYER_HALF_EXTENTS.y,
                PLAYER_HALF_EXTENTS.z,
            );
            let filter = SpatialQueryFilter::default().with_excluded_entities([entity]);

            if let Some(hit) = spatial.cast_shape(
                &shape,
                tf.translation,
                tf.rotation,
                dir,
                &ShapeCastConfig::from_max_distance(dist),
                &filter,
            ) {
                // stop just shy of the wall
                let allowed = (hit.distance - 0.01).max(0.0);
                tf.translation += dir_vec * allowed;
                plyr.speed = 0.0; // zero momentum on impact
            } else {
                tf.translation += step_h;
            }
        }

        // ---- vertical position integration ----
        tf.translation.y += plyr.vertical_vel * dt;

        // ---- align to ground normal ----
        tf.rotation = Quat::from_rotation_arc(Vec3::Y, ground_n) * yaw_rot;
    }
}

fn fall_reset_system(mut q: Query<(&mut Transform, &mut Player)>) {
    for (mut tf, mut plyr) in &mut q {
        if tf.translation.y < FALL_THRESHOLD {
            tf.translation = RESPAWN_POS;
            plyr.speed = 0.0;
            plyr.vertical_vel = 0.0;
        }
    }
}
