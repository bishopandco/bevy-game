use bevy::prelude::*;

use avian3d::prelude::{ColliderConstructor, ColliderConstructorHierarchy, RigidBody};

use crate::hp_text::{HpText, HpTextPlugin};
use crate::weapons::Laser;

#[derive(Component)]
pub struct Target {
    hp: i32,
    half_extents: Vec3,
}

impl Target {
    pub fn new(hp: i32, half_extents: Vec3) -> Self {
        Self { hp, half_extents }
    }
}

pub struct TargetsPlugin;

impl Plugin for TargetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HpTextPlugin)
            .add_systems(Startup, spawn_target)
            .add_systems(
                Update,
                laser_hit_system.after(crate::weapons::laser_movement_system),
            );
    }
}

fn spawn_target(mut commands: Commands, asset_server: Res<AssetServer>) {
    let scene: Handle<Scene> = asset_server.load("models/targets.glb#Scene0");
    commands
        .spawn(SceneRoot(scene))
        .insert(Transform::from_xyz(0.0, 0.0, 0.0))
        .insert(GlobalTransform::default())
        .insert(ColliderConstructorHierarchy::new(
            ColliderConstructor::TrimeshFromMesh,
        ))
        .insert(RigidBody::Static)
        .insert(Target::new(100, Vec3::new(1.0, 1.0, 1.0)));
    info!("spawned target with hp 100");
}

const LASER_DAMAGE: i32 = 5;

fn laser_hit_system(
    mut commands: Commands,
    mut lasers: Query<(&mut Transform, &mut Laser), Without<Target>>,
    mut targets: Query<(Entity, &Transform, &mut Target), Without<Laser>>,
    asset_server: Res<AssetServer>,
) {
    for (mut laser_tf, mut laser) in &mut lasers {
        for (target_entity, target_tf, mut target) in &mut targets {
            if target.hp <= 0 {
                continue;
            }
            info!(
                "checking laser at {:?} against target at {:?}",
                laser_tf.translation, target_tf.translation
            );
            let start = laser.prev_position;
            let end = laser.projected_position;
            if let Some(hit_pos) = segment_intersects_aabb(
                start,
                end,
                target_tf.translation,
                target.half_extents,
            ) {
                let normal = (hit_pos - target_tf.translation).normalize_or_zero();
                let new_hp = (target.hp - LASER_DAMAGE).max(0);
                target.hp = new_hp;
                if new_hp == 0 {
                    info!("despawning target {:?}", target_entity);
                    commands.entity(target_entity).despawn();
                }
                info!("hit target {:?}, new hp {}", target_entity, new_hp);

                let font: Handle<Font> = asset_server.load("fonts/Arial.ttf");
                let text_pos = hit_pos + normal * 0.1;
                commands.spawn((
                    Text2d::new(format!("{} HP", new_hp)),
                    TextFont {
                        font,
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor::WHITE,
                    TextLayout::default(),
                    Transform::from_translation(text_pos),
                    HpText::new(1.0),
                ));
                info!("spawned hp text at {:?}", text_pos);

                laser.velocity =
                    (laser.velocity - 2.0 * laser.velocity.dot(normal) * normal)
                        * crate::weapons::LASER_BOUNCE_DECAY;
                laser_tf.translation = hit_pos;
                break;
            }
        }
        laser.prev_position = laser_tf.translation;
    }
}

fn segment_intersects_aabb(
    start: Vec3,
    end: Vec3,
    center: Vec3,
    half_extents: Vec3,
) -> Option<Vec3> {
    let dir = end - start;
    let axes = [
        (start.x, dir.x, center.x - half_extents.x, center.x + half_extents.x),
        (start.y, dir.y, center.y - half_extents.y, center.y + half_extents.y),
        (start.z, dir.z, center.z - half_extents.z, center.z + half_extents.z),
    ];
    let mut tmin = 0.0;
    let mut tmax = 1.0;
    for (s, d, min, max) in axes {
        if d.abs() < f32::EPSILON {
            if s < min || s > max {
                return None;
            }
        } else {
            let inv_d = 1.0 / d;
            let mut t1 = (min - s) * inv_d;
            let mut t2 = (max - s) * inv_d;
            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
            }
            tmin = tmin.max(t1);
            tmax = tmax.min(t2);
            if tmin > tmax {
                return None;
            }
        }
    }
    Some(start + dir * tmin)
}
