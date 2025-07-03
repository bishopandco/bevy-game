use crate::globals::GameParams;
use avian3d::prelude::*;
use bevy::{
    input::{keyboard::KeyCode, ButtonInput},
    log::info,
    prelude::*,
};

const STEP_HEIGHT: f32 = 0.25;
const MAX_SLOPE_COS: f32 = 0.707;
const SKIN: f32 = 0.1;
const SUBSTEPS: u32 = 4;
const FALL_RESET_Y: f32 = -100.0;
const RESPAWN_POS: Vec3 = Vec3::new(0.0, 1.5, 0.0);
const RESPAWN_YAW: f32 = 0.0;
const JUMP_IMPULSE: f32 = 5.0;

#[derive(Component, Default)]
pub struct Player {
    pub speed: f32,
    pub vertical_vel: f32,
    pub vertical_input: f32,
    pub yaw: f32,
    pub half_extents: Vec3,
    pub grounded: bool,
    pub fire_timer: f32,
    pub weapon_energy: f32,
}

pub struct PlayerControlPlugin;
impl Plugin for PlayerControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                player_input_system,
                player_move_system.after(player_input_system),
                fall_reset_system,
                player_orientation_system.after(player_move_system),
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
        update_vertical_input(&keys, &mut plyr);
    }
}

fn player_move_system(
    time: Res<Time>,
    params: Res<GameParams>,
    spatial: SpatialQuery,
    mut q: Query<(Entity, &mut Transform, &mut Player)>,
) {
    let dt = time.delta_secs() / SUBSTEPS as f32;
    for (entity, mut tf, mut plyr) in &mut q {
        let col = Collider::cuboid(
            plyr.half_extents.x,
            plyr.half_extents.y,
            plyr.half_extents.z,
        );
        for _ in 0..SUBSTEPS {
            move_horizontal(&spatial, &params, entity, &col, &mut tf, &mut plyr, dt);
            move_vertical(&spatial, &params, entity, &col, &mut tf, &mut plyr, dt);
        }
    }
}

fn player_orientation_system(
    spatial: SpatialQuery,
    params: Res<GameParams>,
    mut q: Query<(Entity, &mut Transform, &mut Player)>,
) {
    for (entity, mut tf, mut plyr) in &mut q {
        apply_ground_snap(&spatial, entity, &mut tf, &mut plyr);
        orient_to_ground(&spatial, &params, entity, &mut tf, &plyr);
    }
}

fn update_speed(keys: &ButtonInput<KeyCode>, params: &GameParams, plyr: &mut Player, dt: f32) {
    if keys.pressed(KeyCode::ArrowUp) {
        plyr.speed = (plyr.speed + params.acceleration * dt).min(params.max_speed);
    } else if keys.pressed(KeyCode::ArrowDown) {
        plyr.speed = (plyr.speed - params.brake_acceleration * dt).max(-params.max_speed);
    } else {
        plyr.speed = plyr.speed.signum() * (plyr.speed.abs() - params.friction * dt).max(0.0);
    }
}

fn update_yaw(keys: &ButtonInput<KeyCode>, params: &GameParams, plyr: &mut Player, dt: f32) {
    if keys.pressed(KeyCode::ArrowLeft) {
        plyr.yaw += params.yaw_rate * dt;
    }
    if keys.pressed(KeyCode::ArrowRight) {
        plyr.yaw -= params.yaw_rate * dt;
    }
}

fn update_vertical_input(keys: &ButtonInput<KeyCode>, plyr: &mut Player) {
    if keys.just_pressed(KeyCode::KeyW) {
        plyr.vertical_input = 1.0;
    } else if keys.just_pressed(KeyCode::KeyS) {
        plyr.vertical_input = -1.0;
    } else {
        plyr.vertical_input = 0.0;
    }
}

fn move_horizontal(
    spatial: &SpatialQuery,
    params: &GameParams,
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
            &ShapeCastConfig {
                compute_contact_on_penetration: true,
                max_distance: dist + SKIN,
                ..Default::default()
            },
            &filter,
        ) {
            Some(hit) => {
                tf.translation += dir.as_vec3() * (hit.distance - SKIN).max(0.0);
                if hit.normal1.y > MAX_SLOPE_COS {
                    plyr.grounded = true;
                }
                slide(&mut remaining, hit.normal1, plyr, params);
            }
            None => {
                tf.translation += remaining;
                break;
            }
        }
    }
}

fn slide(remaining: &mut Vec3, normal: Vec3, plyr: &mut Player, params: &GameParams) {
    let incoming = *remaining;
    // Project the movement onto the collision plane to keep momentum
    *remaining -= remaining.dot(normal) * normal;

    // reduce momentum based on collision
    let mut factor = 1.0 - params.collision_damping;
    if normal.y > 0.0 && normal.y < 1.0 {
        let slope = 1.0 - normal.y;
        let eased = slope.powf(params.slope_ease);
        factor *= 1.0 - eased * params.slope_damping;
    }
    factor = factor.clamp(0.0, 1.0);
    *remaining *= factor;
    plyr.speed *= factor;

    // add a small bounce based on collision angle
    let reflect_dir = (incoming - 2.0 * incoming.dot(normal) * normal).normalize_or_zero();
    let incident_angle = incoming.normalize_or_zero().dot(-normal).abs();
    let bounce = incoming.length() * params.bounce_factor * incident_angle;
    *remaining += reflect_dir * bounce;
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

    if plyr.vertical_input != 0.0 {
        plyr.vertical_vel = plyr.vertical_input * JUMP_IMPULSE;
        plyr.vertical_input = 0.0;
        plyr.grounded = false;
    }

    plyr.vertical_vel -= params.gravity * dt;
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
        if plyr.vertical_vel < 0.0 {
            plyr.speed *= 0.1;
        }
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
    let grounded_now = spatial
        .cast_ray(
            tf.translation,
            Dir3::NEG_Y,
            plyr.half_extents.y + STEP_HEIGHT + SKIN,
            false,
            &filter,
        )
        .is_some();
    plyr.grounded = grounded_now;
}

fn orient_to_ground(
    spatial: &SpatialQuery,
    params: &GameParams,
    entity: Entity,
    tf: &mut Transform,
    plyr: &Player,
) {
    let filter = SpatialQueryFilter::default().with_excluded_entities([entity]);
    let ground_n = spatial
        .cast_ray(
            tf.translation,
            Dir3::NEG_Y,
            plyr.half_extents.y + STEP_HEIGHT + SKIN,
            false,
            &filter,
        )
        .map(|h| h.normal)
        .unwrap_or(Vec3::Y);
    let yaw_rot = Quat::from_rotation_y(plyr.yaw);
    let target = Quat::from_rotation_arc(Vec3::Y, ground_n) * yaw_rot;
    tf.rotation = tf.rotation.slerp(target, params.ground_align_lerp);
}

fn fall_reset_system(mut q: Query<(&mut Transform, &mut Player)>) {
    for (mut tf, mut plyr) in &mut q {
        if tf.translation.y < FALL_RESET_Y {
            info!("respawn");
            tf.translation = RESPAWN_POS;
            plyr.speed = 0.0;
            plyr.vertical_vel = 0.0;
            plyr.vertical_input = 0.0;
            plyr.grounded = false;
            plyr.yaw = RESPAWN_YAW;
            plyr.fire_timer = 0.0;
            plyr.weapon_energy = 1.0;
        }
    }
}
