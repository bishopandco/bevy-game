use bevy::prelude::*;

use avian3d::prelude::{Collider, ShapeCastConfig, SpatialQuery, SpatialQueryFilter};

use crate::{globals::GameParams, input::Player};

pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, player_fire_system)
            .add_systems(Update, laser_movement_system);
    }
}

#[derive(Component)]
pub struct Laser {
    pub(crate) velocity: Vec3,
    pub(crate) prev_position: Vec3,
    life: f32,
    material: Handle<StandardMaterial>,
}

pub const LASER_SPEED: f32 = 100.0;
pub const LASER_LIFETIME: f32 = 0.5; // seconds
pub const LASER_LIGHT_INTENSITY: f32 = 1500.0;
pub const LASER_BOUNCE_DECAY: f32 = 0.67;

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
    let recharge_rate = dt / 3.0;
    let fire_cost = 1.0 / (params.fire_rate * 1.0);
    for (tf, mut plyr) in &mut players {
        if plyr.fire_timer > 0.0 {
            plyr.fire_timer -= dt;
        }
        plyr.weapon_energy = (plyr.weapon_energy + recharge_rate).min(1.0);
        if keys.pressed(KeyCode::Space) && plyr.fire_timer <= 0.0 {
            if plyr.weapon_energy < fire_cost {
                continue;
            }
            let forward = tf.rotation * Vec3::Z;
            let pos = tf.translation + forward * (plyr.half_extents.z + 0.6);
            let mesh = meshes.add(Cuboid::new(0.05, 0.05, 0.3));
            let material = materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.0, 0.0),
                emissive: LinearRgba::from(Color::srgb(5.0, 0.0, 0.0)),
                ..default()
            });
            commands
                .spawn(Mesh3d(mesh))
                .insert(MeshMaterial3d(material.clone()))
                .insert(PointLight {
                    intensity: LASER_LIGHT_INTENSITY,
                    range: 6.0,
                    color: Color::srgb(5.0, 0.0, 0.0),
                    ..default()
                })
                .insert(Transform::from_translation(pos).looking_at(pos + forward, Vec3::Y))
                .insert(Laser {
                    velocity: forward * LASER_SPEED,
                    prev_position: pos,
                    life: LASER_LIFETIME,
                    material,
                });
            plyr.fire_timer = 1.0 / params.fire_rate.max(f32::EPSILON);
            plyr.weapon_energy -= fire_cost;
        }
    }
}

pub fn laser_movement_system(
    time: Res<Time>,
    spatial: SpatialQuery,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut q: Query<(Entity, &mut Transform, &mut Laser, &mut PointLight)>,
) {
    let dt = time.delta_secs();
    let col = Collider::cuboid(0.025, 0.025, 0.15);
    for (e, mut tf, mut laser, mut light) in &mut q {
        let start_pos = tf.translation;
        let mut remaining = laser.velocity * dt;
        let filter = SpatialQueryFilter::default().with_excluded_entities([e]);
        for _ in 0..2 {
            let dist = remaining.length();
            if dist <= f32::EPSILON {
                break;
            }
            let dir = Dir3::new_unchecked(remaining / dist);
            match spatial.cast_shape(
                &col,
                tf.translation,
                tf.rotation,
                dir,
                &ShapeCastConfig {
                    max_distance: dist,
                    ..Default::default()
                },
                &filter,
            ) {
                Some(hit) => {
                    tf.translation += dir.as_vec3() * hit.distance.max(0.0);
                    let normal = hit.normal1;
                    laser.velocity =
                        (laser.velocity - 2.0 * laser.velocity.dot(normal) * normal)
                            * LASER_BOUNCE_DECAY;
                    remaining = laser.velocity * ((dist - hit.distance) / dist);
                }
                None => {
                    tf.translation += remaining;
                    break;
                }
            }
        }

        if laser.velocity.length_squared() > 0.0 {
            let look_at_pos = tf.translation + laser.velocity;
            tf.look_at(look_at_pos, Vec3::Y);
        }

        laser.life -= dt;
        let ratio = (laser.life / LASER_LIFETIME).clamp(0.0, 1.0);
        light.intensity = LASER_LIGHT_INTENSITY * ratio;
        if let Some(mat) = materials.get_mut(&laser.material) {
            mat.emissive = LinearRgba::from(Color::srgb(5.0 * ratio, 0.0, 0.0));
        }
        if laser.life <= 0.0 {
            commands.entity(e).despawn();
        }

        laser.prev_position = start_pos;
    }
}
