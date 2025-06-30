use bevy::prelude::*;

use crate::{input::Player, globals::GameParams};

pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (player_fire_system, laser_movement_system));
    }
}

#[derive(Component)]
struct Laser {
    velocity: Vec3,
    life: f32,
}

const LASER_SPEED: f32 = 100.0;
const LASER_LIFETIME: f32 = 2.0;

fn player_fire_system(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    params: Res<GameParams>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut players: Query<(&Transform, &mut Player)>,
) {
    let dt = time.delta_secs();
    for (tf, mut plyr) in &mut players {
        if plyr.fire_timer > 0.0 {
            plyr.fire_timer -= dt;
        }
        if keys.pressed(KeyCode::Space) && plyr.fire_timer <= 0.0 {
            let forward = tf.rotation * Vec3::Z;
            let pos = tf.translation + forward * (plyr.half_extents.z + 0.6);
            let mesh = meshes.add(Cuboid::new(0.05, 0.05, 0.3));
            let material = materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.0, 0.0),
                emissive: Color::srgb(5.0, 0.0, 0.0),
                ..default()
            });
            commands
                .spawn(Mesh3d(mesh))
                .insert(MeshMaterial3d(material))
                .insert(PointLight {
                    intensity: 1500.0,
                    range: 6.0,
                    color: Color::RED,
                    ..default()
                })
                .insert(Transform::from_translation(pos).looking_at(pos + forward, Vec3::Y))
                .insert(Laser {
                    velocity: forward * LASER_SPEED,
                    life: LASER_LIFETIME,
                });
            plyr.fire_timer = 1.0 / params.fire_rate.max(f32::EPSILON);
        }
    }
}

fn laser_movement_system(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Transform, &mut Laser)>,
) {
    let dt = time.delta_secs();
    for (e, mut tf, mut laser) in &mut q {
        tf.translation += laser.velocity * dt;
        laser.life -= dt;
        if laser.life <= 0.0 {
            commands.entity(e).despawn();
        }
    }
}
