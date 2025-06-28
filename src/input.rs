use crate::globals::GameParams;
use avian3d::prelude::*;
use bevy::{
    input::{keyboard::KeyCode, ButtonInput},
    log::info,
    prelude::*,
};

const STEP_HEIGHT: f32 = 0.35;
const MAX_SLOPE_COS: f32 = 0.707; // 45 °
const SKIN: f32 = 0.03;
const FALL_RESET_Y: f32 = -10.0;
const RESPAWN_POS: Vec3 = Vec3::new(0.0, 1.5, 0.0);

#[derive(Component, Default)]
pub struct Player {
    pub speed: f32,
    pub vertical_vel: f32,
    pub yaw: f32,
    pub half_extents: Vec3,
    pub grounded: bool,
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
    spatial: SpatialQuery,
    mut q: Query<(Entity, &mut Transform, &mut Player)>,
) {
    for (entity, mut tf, mut plyr) in &mut q {
        let dt = time.delta_secs();
        plyr.grounded = false;

        /* input ------------------------------------------------------------ */
        if keys.pressed(KeyCode::ArrowUp) {
            plyr.speed = (plyr.speed + params.acceleration * dt).min(params.max_speed);
        } else if keys.pressed(KeyCode::Space) {
            plyr.speed = (plyr.speed - params.brake_acceleration * dt).max(-params.max_speed);
        } else {
            let decel = (plyr.speed.abs() - params.friction * dt).max(0.0);
            plyr.speed = plyr.speed.signum() * decel;
        }
        if keys.pressed(KeyCode::ArrowLeft) {
            plyr.yaw += params.yaw_rate * dt;
        }
        if keys.pressed(KeyCode::ArrowRight) {
            plyr.yaw -= params.yaw_rate * dt;
        }

        /* collider + filtering -------------------------------------------- */
        let yaw_rot = Quat::from_rotation_y(plyr.yaw);
        let forward = yaw_rot * Vec3::Z;
        let col = Collider::cuboid(
            plyr.half_extents.x,
            plyr.half_extents.y,
            plyr.half_extents.z,
        );
        let filter = SpatialQueryFilter::default().with_excluded_entities([entity]);

        /* horizontal sweep / slide / step --------------------------------- */
        let mut remaining = forward * plyr.speed * dt;
        for _ in 0..3 {
            let dist = remaining.length();
            if dist < f32::EPSILON {
                break;
            }
            let dir = Dir3::new_unchecked(remaining / dist);

            if let Some(hit) = spatial.cast_shape(
                &col,
                tf.translation,
                tf.rotation,
                dir,
                &ShapeCastConfig {
                    max_distance: dist + SKIN,
                    ..Default::default()
                },
                &filter,
            ) {
                // move right up to the contact
                tf.translation += dir.as_vec3() * (hit.distance - SKIN).max(0.0);

                // ── collision response ─────────────────────────────────────────────
                // gentle slope? treat as ground; otherwise stop (or bounce)
                if hit.normal1.y > MAX_SLOPE_COS {
                    plyr.grounded = true;
                    remaining = Vec3::ZERO;
                    // optional: zero horizontal speed when you *land* on a slope
                    // plyr.speed = 0.0;
                } else {
                    // hit a wall -> kill speed and/or bounce
                    const BOUNCE: f32 = 0.0; // 0 = stop, 1 = full elastic bounce
                    plyr.speed = -plyr.speed * BOUNCE;

                    // reflect the remaining vector for a bounce, or zero it out to stop
                    if BOUNCE > 0.0 {
                        let refl = remaining - 2.0 * remaining.dot(hit.normal1) * hit.normal1;
                        remaining = refl * BOUNCE;
                    } else {
                        remaining = Vec3::ZERO;
                    }
                }
            } else {
                tf.translation += remaining;
                break;
            }
        }

        /* gravity --------------------------------------------------------- */
        if !plyr.grounded {
            plyr.vertical_vel -= params.gravity * dt;
        } else {
            plyr.vertical_vel = 0.0;
        }
        tf.translation.y += plyr.vertical_vel * dt;

        /* ground snap ------------------------------------------------------ */

        let cfg = ShapeCastConfig {
            compute_contact_on_penetration: true,
            max_distance: 100.0,
            ..Default::default()
        };

        if let Some(hit) = spatial.cast_shape(
            &col,
            tf.translation + Vec3::Y * (plyr.half_extents.y + STEP_HEIGHT),
            tf.rotation,
            Dir3::NEG_Y,
            &cfg,
            &filter,
        ) {
            // stick to the ground
            tf.translation.y = hit.point1.y + plyr.half_extents.y + SKIN;
            plyr.grounded = true;
            plyr.vertical_vel = 0.0; // ← stop the downward build-up
        }

        /* tilt to ground --------------------------------------------------- */
        let ground_n = if plyr.grounded {
            spatial
                .cast_ray(
                    tf.translation,
                    Dir3::NEG_Y,
                    plyr.half_extents.y + STEP_HEIGHT + SKIN,
                    false,
                    &filter,
                )
                .map(|h| h.normal)
                .unwrap_or(Vec3::Y)
        } else {
            Vec3::Y
        };
        tf.rotation = Quat::from_rotation_arc(Vec3::Y, ground_n) * yaw_rot;
    }
}

fn fall_reset_system(mut q: Query<(&mut Transform, &mut Player)>) {
    for (mut tf, mut plyr) in &mut q {
        if tf.translation.y < FALL_RESET_Y {
            info!("respawn");
            tf.translation = RESPAWN_POS;
            plyr.speed = 0.0;
            plyr.vertical_vel = 0.0;
            plyr.grounded = false;
        }
    }
}
