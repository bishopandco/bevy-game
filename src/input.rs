use crate::globals::GameParams;
use avian3d::prelude::*;
use bevy::{
    input::{keyboard::KeyCode, ButtonInput},
    log::info,
    prelude::*,
};

const STEP_HEIGHT: f32 = 0.25;
const MAX_SLOPE_COS: f32 = 0.707;
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
        app.add_systems(
            Update,
            (
                player_input_system,
                player_move_system.after(player_input_system),
                player_orientation_system.after(player_move_system),
                fall_reset_system,
            ),
        );
    }
}

fn player_input_system(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    params: Res<GameParams>,
    mut q: Query<&mut Player>,
) {
    let dt = time.delta_secs();
    for mut plyr in &mut q {
        update_speed(&keys, &params, &mut plyr, dt);
        update_yaw(&keys, &params, &mut plyr, dt);
    }
}

fn player_move_system(
    time: Res<Time>,
    params: Res<GameParams>,
    spatial: SpatialQuery,
    mut q: Query<(Entity, &mut Transform, &mut Player)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut plyr) in &mut q {
        let col = Collider::cuboid(
            plyr.half_extents.x,
            plyr.half_extents.y,
            plyr.half_extents.z,
        );
        move_horizontal(&spatial, entity, &col, &mut tf, &mut plyr, dt);
        move_vertical(&spatial, &params, entity, &col, &mut tf, &mut plyr, dt);
    }
}

fn player_orientation_system(
    spatial: SpatialQuery,
    mut q: Query<(Entity, &mut Transform, &mut Player)>,
) {
    for (entity, mut tf, mut plyr) in &mut q {
        apply_ground_snap(&spatial, entity, &mut tf, &mut plyr);
        orient_to_ground(&spatial, entity, &mut tf, &plyr);
    }
}

fn update_speed(
    keys: &ButtonInput<KeyCode>,
    params: &GameParams,
    plyr: &mut Player,
    dt: f32,
) {
    if keys.pressed(KeyCode::ArrowUp) {
        plyr.speed = (plyr.speed + params.acceleration * dt).min(params.max_speed);
    } else if keys.pressed(KeyCode::Space) {
        plyr.speed = (plyr.speed - params.brake_acceleration * dt).max(-params.max_speed);
    } else {
        plyr.speed = plyr.speed.signum() * (plyr.speed.abs() - params.friction * dt).max(0.0);
    }
}

fn update_yaw(
    keys: &ButtonInput<KeyCode>,
    params: &GameParams,
    plyr: &mut Player,
    dt: f32,
) {
    if keys.pressed(KeyCode::ArrowLeft) {
        plyr.yaw += params.yaw_rate * dt;
    }
    if keys.pressed(KeyCode::ArrowRight) {
        plyr.yaw -= params.yaw_rate * dt;
    }
}

fn move_horizontal(
    spatial: &SpatialQuery,
    entity: Entity,
    col: &Collider,
    tf: &mut Transform,
    plyr: &mut Player,
    dt: f32,
) {
    let yaw_rot = Quat::from_rotation_y(plyr.yaw);
    let forward = yaw_rot * Vec3::Z;
    let mut remaining = forward * plyr.speed * dt;
    let filter = SpatialQueryFilter::default().with_excluded_entities([entity]);

    for _ in 0..3 {
        let dist = remaining.length();
        if dist < f32::EPSILON {
            break;
        }
        let dir = Dir3::new_unchecked(remaining / dist);
        match spatial.cast_shape(
            col,
            tf.translation,
            tf.rotation,
            dir,
            &ShapeCastConfig { max_distance: dist + SKIN, ..Default::default() },
            &filter,
        ) {
            Some(hit) => {
                tf.translation += dir.as_vec3() * (hit.distance - SKIN).max(0.0);
                if hit.normal1.y > MAX_SLOPE_COS {
                    plyr.grounded = true;
                    break;
                }
                bounce(plyr, &mut remaining, hit.normal1);
            }
            None => {
                tf.translation += remaining;
                break;
            }
        }
    }
}

fn bounce(plyr: &mut Player, remaining: &mut Vec3, normal: Vec3) {
    const BOUNCE: f32 = 0.1;
    plyr.speed = -plyr.speed * BOUNCE;
    *remaining = if BOUNCE > 0.0 {
        *remaining - 2.0 * remaining.dot(normal) * normal
    } else {
        Vec3::ZERO
    };
}

fn move_vertical(
    spatial: &SpatialQuery,
    params: &GameParams,
    entity: Entity,
    col: &Collider,
    tf: &mut Transform,
    plyr: &mut Player,
    dt: f32,
) {
    // re-check ground contact before applying gravity
    apply_ground_snap(spatial, entity, tf, plyr);

    if plyr.grounded {
        // player was grounded last frame, so don't apply gravity
        plyr.vertical_vel = 0.0;
    } else {
        // apply gravity when falling
        plyr.vertical_vel -= params.gravity * dt;
    }
    tf.translation.y += plyr.vertical_vel * dt;
    resolve_vertical_collision(spatial, entity, col, tf, plyr);
}

fn resolve_vertical_collision(
    spatial: &SpatialQuery,
    entity: Entity,
    col: &Collider,
    tf: &mut Transform,
    plyr: &mut Player,
) {
    let filter = SpatialQueryFilter::default().with_excluded_entities([entity]);
    if let Some(hit) = spatial.cast_shape(
        col,
        tf.translation + Vec3::Y * (plyr.half_extents.y + STEP_HEIGHT),
        tf.rotation,
        Dir3::NEG_Y,
        &ShapeCastConfig {
            compute_contact_on_penetration: true,
            max_distance: plyr.half_extents.y + STEP_HEIGHT + SKIN,
            ..Default::default()
        },
        &filter,
    ) {
        tf.translation.y = hit.point1.y + plyr.half_extents.y + SKIN;
        plyr.grounded = true;
        plyr.vertical_vel = 0.0;
    }
}

fn apply_ground_snap(
    spatial: &SpatialQuery,
    entity: Entity,
    tf: &mut Transform,
    plyr: &mut Player,
) {
    let filter = SpatialQueryFilter::default().with_excluded_entities([entity]);
    let grounded_now = spatial.cast_ray(tf.translation, Dir3::NEG_Y, plyr.half_extents.y + STEP_HEIGHT + SKIN, false, &filter).is_some();
    plyr.grounded = grounded_now;
}

fn orient_to_ground(
    spatial: &SpatialQuery,
    entity: Entity,
    tf: &mut Transform,
    plyr: &Player,
) {
    let filter = SpatialQueryFilter::default().with_excluded_entities([entity]);
    let ground_n = spatial
        .cast_ray(tf.translation, Dir3::NEG_Y, plyr.half_extents.y + STEP_HEIGHT + SKIN, false, &filter)
        .map(|h| h.normal)
        .unwrap_or(Vec3::Y);
    let yaw_rot = Quat::from_rotation_y(plyr.yaw);
    tf.rotation = Quat::from_rotation_arc(Vec3::Y, ground_n) * yaw_rot;
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
