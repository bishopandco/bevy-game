use bevy::prelude::*;

use avian3d::prelude::{
    ColliderConstructor,
    ColliderConstructorHierarchy,
    Dir3,
    RigidBody,
    SpatialQuery,
    SpatialQueryFilter,
};

use crate::hp_text::{HpText, HpTextPlugin};
use crate::weapons::Laser;

#[derive(Component)]
pub struct Target {
    hp: i32,
}

impl Target {
    pub fn new(hp: i32) -> Self {
        Self { hp }
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
        .insert(Transform::from_xyz(0.0, 0.0, 5.0))
        .insert(GlobalTransform::default())
        .insert(ColliderConstructorHierarchy::new(
            ColliderConstructor::TrimeshFromMesh,
        ))
        .insert(RigidBody::Static)
        .insert(Target::new(100));
    info!("spawned target with hp 100");
}

const LASER_DAMAGE: i32 = 5;

fn laser_hit_system(
    mut commands: Commands,
    spatial: SpatialQuery,
    mut lasers: Query<(Entity, &mut Transform, &mut Laser), Without<Target>>,
    mut targets: Query<(Entity, &mut Target)>,
    asset_server: Res<AssetServer>,
) {
    for (laser_entity, mut laser_tf, mut laser) in &mut lasers {
        let start = laser.prev_position;
        let dir_vec = laser_tf.translation - start;
        let dist = dir_vec.length();
        if dist <= f32::EPSILON {
            laser.prev_position = laser_tf.translation;
            continue;
        }
        let dir = Dir3::new_unchecked(dir_vec / dist);
        let filter = SpatialQueryFilter::default().with_excluded_entities([laser_entity]);
        if let Some(hit) = spatial.cast_ray(start, dir, dist, true, &filter) {
            if let Ok((target_entity, mut target)) = targets.get_mut(hit.entity) {
                let normal = hit.normal;
                let hit_pos = start + dir.as_vec3() * hit.distance;
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
            }
        }
        laser.prev_position = laser_tf.translation;
    }
}
