use crate::globals::GameParams;
use avian3d::prelude::*;
use bevy::{
    input::{keyboard::KeyCode, ButtonInput},
    prelude::*,
};
use bevy::prelude::ChildOf;

const WHEEL_RAY_LENGTH: f32 = 1.0;
const SUSPENSION_SMOOTH: f32 = 0.4;
const LAUNCH_SPEED: f32 = 20.0;

#[derive(Component, Default)]
pub struct Player {
    pub speed: f32,
    pub vertical_vel: f32,
    pub yaw: f32,
    pub half_extents: Vec3,
    pub grounded: bool,
    pub fire_timer: f32,
    pub weapon_energy: f32,
}

#[derive(Component)]
pub struct Wheel {
    pub offset: Vec3,
}

pub fn wheel_offsets(plyr: &Player) -> [Vec3; 4] {
    let ext = plyr.half_extents;
    [
        Vec3::new(ext.x, -ext.y, ext.z),
        Vec3::new(ext.x, -ext.y, -ext.z),
        Vec3::new(-ext.x, -ext.y, ext.z),
        Vec3::new(-ext.x, -ext.y, -ext.z),
    ]
}

fn wheel_hits(
    spatial: &SpatialQuery,
    entity: Entity,
    tf: &Transform,
    plyr: &Player,
) -> Vec<(Vec3, Vec3)> {
    let filter = SpatialQueryFilter::default().with_excluded_entities([entity]);
    let mut out = Vec::new();
    for offset in wheel_offsets(plyr) {
        let world_pos = tf.translation + tf.rotation.mul_vec3(offset);
        if let Some(hit) = spatial.cast_ray(
            world_pos,
            Dir3::NEG_Y,
            WHEEL_RAY_LENGTH + plyr.half_extents.y,
            false,
            &filter,
        ) {
            let point = world_pos + Dir3::NEG_Y.as_vec3() * hit.distance;
            out.push((point, hit.normal));
        }
    }
    out
}

pub struct PlayerControlPlugin;
impl Plugin for PlayerControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                player_input_system,
                car_movement_system.after(player_input_system),
                wheel_suspension_system.after(car_movement_system),
            ),
        );
    }
}

fn player_input_system(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    params: Res<GameParams>,
    mut players: Query<&mut Player>,
) {
    let dt = time.delta_secs();
    for mut plyr in &mut players {
        if keys.pressed(KeyCode::ArrowUp) {
            plyr.speed += params.acceleration * dt;
        } else if keys.pressed(KeyCode::ArrowDown) {
            if plyr.speed > 0.0 {
                plyr.speed -= params.brake_deceleration * dt;
            } else {
                plyr.speed -= params.brake_acceleration * dt;
            }
        } else {
            let sign = plyr.speed.signum();
            plyr.speed -= sign * params.friction * dt;
            if plyr.speed.signum() != sign {
                plyr.speed = 0.0;
            }
        }
        plyr.speed = plyr.speed.clamp(-params.max_speed, params.max_speed);

        if keys.pressed(KeyCode::ArrowLeft) {
            plyr.yaw += params.yaw_rate * dt;
        }
        if keys.pressed(KeyCode::ArrowRight) {
            plyr.yaw -= params.yaw_rate * dt;
        }
    }
}

fn car_movement_system(
    time: Res<Time>,
    params: Res<GameParams>,
    spatial: SpatialQuery,
    mut q: Query<(Entity, &mut Transform, &mut Player)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut plyr) in &mut q {
        let yaw_rot = Quat::from_rotation_y(plyr.yaw);
        let forward = yaw_rot * Vec3::Z;
        tf.translation += forward * plyr.speed * dt;

        let hits = wheel_hits(&spatial, entity, &tf, &plyr);
        if hits.is_empty() {
            plyr.grounded = false;
        } else {
            let avg_y = hits.iter().map(|(p, _)| p.y).sum::<f32>() / hits.len() as f32;
            let ground_n = hits
                .iter()
                .map(|(_, n)| *n)
                .fold(Vec3::ZERO, |a, b| a + b)
                .normalize_or_zero();
            let contact = hits.iter().any(|(p, _)| {
                (tf.translation.y - plyr.half_extents.y) - p.y <= WHEEL_RAY_LENGTH
            });
            if contact && plyr.speed.abs() < LAUNCH_SPEED {
                plyr.grounded = true;
                let target_y = avg_y + plyr.half_extents.y;
                tf.translation.y = tf.translation.y.lerp(target_y, 0.5);
                plyr.vertical_vel = 0.0;
                let target_rot = Quat::from_rotation_arc(Vec3::Y, ground_n) * yaw_rot;
                tf.rotation = tf.rotation.slerp(target_rot, 0.2);
            } else {
                plyr.grounded = false;
            }
        }

        if !plyr.grounded {
            plyr.vertical_vel -= params.gravity * dt;
            tf.translation.y += plyr.vertical_vel * dt;
            tf.rotation = Quat::from_rotation_y(plyr.yaw);
        }
    }
}

fn wheel_suspension_system(
    spatial: SpatialQuery,
    player_q: Query<(&Transform, &Player), Without<Wheel>>,
    mut wheels: Query<(&ChildOf, &mut Transform, &Wheel), Without<Player>>,
) {
    for (child_of, mut tf, wheel) in &mut wheels {
        if let Ok((player_tf, plyr)) = player_q.get(child_of.parent()) {
            let world_pos = player_tf.translation + player_tf.rotation.mul_vec3(wheel.offset);
            let filter = SpatialQueryFilter::default().with_excluded_entities([child_of.parent()]);
            let target = if let Some(hit) = spatial.cast_ray(
                world_pos,
                Dir3::NEG_Y,
                WHEEL_RAY_LENGTH,
                false,
                &filter,
            ) {
                wheel.offset + Vec3::Y * (-hit.distance)
            } else {
                wheel.offset - Vec3::Y * WHEEL_RAY_LENGTH
            };
            tf.translation = tf.translation.lerp(target, SUSPENSION_SMOOTH);
        }
    }
}
