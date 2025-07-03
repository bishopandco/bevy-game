use bevy::prelude::*;

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
        app.add_systems(Startup, spawn_target).add_systems(
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
        .insert(Target::new(100));
}

const LASER_DAMAGE: i32 = 5;

fn laser_hit_system(
    mut commands: Commands,
    mut lasers: Query<(&mut Transform, &mut Laser), Without<Target>>,
    mut targets: Query<(Entity, &Transform, &mut Target), Without<Laser>>,
) {
    for (mut laser_tf, mut laser) in &mut lasers {
        for (target_entity, target_tf, mut target) in &mut targets {
            let dist = laser_tf.translation.distance(target_tf.translation);
            if dist < 1.0 {
                if target.hp <= LASER_DAMAGE {
                    commands.entity(target_entity).despawn();
                } else {
                    target.hp -= LASER_DAMAGE;
                }
                let normal = (laser_tf.translation - target_tf.translation).normalize();
                laser.velocity = (laser.velocity - 2.0 * laser.velocity.dot(normal) * normal)
                    * crate::weapons::LASER_BOUNCE_DECAY;
                laser_tf.translation = target_tf.translation + normal;
                break;
            }
        }
    }
}
