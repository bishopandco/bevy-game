use bevy::prelude::*;
use bevy::ecs::hierarchy::{ChildSpawnerCommands, ChildOf};
use bevy::math::primitives::Cylinder;
use avian3d::prelude::*;

use crate::globals::{GameParams, Controlled, InVehicle};
use crate::input::Player;

const STEP_HEIGHT: f32 = 0.25;
const MAX_SLOPE_COS: f32 = 0.707;
const SKIN: f32 = 0.1;
const SUBSTEPS: u32 = 4;
const FALL_RESET_Y: f32 = -100.0;
const RESPAWN_POS: Vec3 = Vec3::new(0.0, 1.5, 0.0);
const RESPAWN_YAW: f32 = 0.0;

#[derive(Component)]
pub struct Vehicle {
    pub speed: f32,
    pub vertical_vel: f32,
    pub yaw: f32,
    pub half_extents: Vec3,
    pub grounded: bool,
}

impl Default for Vehicle {
    fn default() -> Self {
        Self {
            speed: 0.0,
            vertical_vel: 0.0,
            yaw: 0.0,
            half_extents: Vec3::new(1.0, 0.5, 1.5),
            grounded: false,
        }
    }
}

#[derive(Component)]
pub struct Wheel {
    pub is_front: bool,
    pub radius: f32,
    pub rest_offset: Vec3,
    pub suspension: f32,
    pub rotation: f32,
}

pub struct VehiclePlugin;

impl Plugin for VehiclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_vehicle)
            .add_systems(
                Update,
                (
                    vehicle_toggle_system,
                    vehicle_input_system,
                    vehicle_move_system.after(vehicle_input_system),
                    vehicle_fall_reset_system,
                    vehicle_orientation_system.after(vehicle_move_system),
                    wheel_update_system.after(vehicle_orientation_system),
                    sync_player_to_vehicle_system,
                ),
            );
    }
}

const WHEEL_RADIUS: f32 = 0.5;
const WHEEL_WIDTH: f32 = 0.3;
const FRONT_AXLE_Z: f32 = 1.5;
const REAR_AXLE_Z: f32 = -1.5;
const AXLE_X: f32 = 1.0;
const SUSPENSION_TRAVEL: f32 = 0.2;
const ENTER_DISTANCE: f32 = 2.0;

fn spawn_vehicle(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let scene: Handle<Scene> = asset_server.load("models/car.glb#Scene0");
    let vehicle_defaults = Vehicle::default();
    let collider_size = vehicle_defaults.half_extents;
    let vehicle = commands
        .spawn(SceneRoot(scene))
        .insert(Transform::from_xyz(0.0, WHEEL_RADIUS + 0.5, 0.0))
        .insert(GlobalTransform::default())
        .insert(vehicle_defaults)
        .insert(RigidBody::Kinematic)
        .insert(LinearVelocity::ZERO)
        .insert(Collider::cuboid(
            collider_size.x,
            collider_size.y,
            collider_size.z,
        ))
        .id();

    let wheel_mesh = meshes.add(Mesh::from(Cylinder {
        radius: WHEEL_RADIUS,
        half_height: WHEEL_WIDTH * 0.5,
    }));
    let wheel_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.1, 0.1),
        ..default()
    });

    commands.entity(vehicle).with_children(|p| {
        spawn_wheel(
            p,
            wheel_mesh.clone(),
            wheel_material.clone(),
            Vec3::new(AXLE_X, -WHEEL_RADIUS, FRONT_AXLE_Z),
            true,
        );
        spawn_wheel(
            p,
            wheel_mesh.clone(),
            wheel_material.clone(),
            Vec3::new(-AXLE_X, -WHEEL_RADIUS, FRONT_AXLE_Z),
            true,
        );
        spawn_wheel(
            p,
            wheel_mesh.clone(),
            wheel_material.clone(),
            Vec3::new(AXLE_X, -WHEEL_RADIUS, REAR_AXLE_Z),
            false,
        );
        spawn_wheel(
            p,
            wheel_mesh,
            wheel_material,
            Vec3::new(-AXLE_X, -WHEEL_RADIUS, REAR_AXLE_Z),
            false,
        );
    });
}

fn spawn_wheel(
    parent: &mut ChildSpawnerCommands,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    offset: Vec3,
    is_front: bool,
) {
    parent
        .spawn(Mesh3d(mesh))
        .insert(MeshMaterial3d(material))
        .insert(Transform::from_translation(offset))
        .insert(Wheel {
            is_front,
            radius: WHEEL_RADIUS,
            rest_offset: offset,
            suspension: SUSPENSION_TRAVEL,
            rotation: 0.0,
        });
}

fn vehicle_input_system(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    params: Res<GameParams>,
    mut q: Query<&mut Vehicle, With<Controlled>>,
) {
    let dt = time.delta_secs();
    for mut vehicle in &mut q {
        if keys.pressed(KeyCode::KeyW) {
            vehicle.speed = (vehicle.speed + params.acceleration * dt).min(params.max_speed);
        } else if keys.pressed(KeyCode::KeyS) {
            vehicle.speed = (vehicle.speed - params.brake_acceleration * dt).max(-params.max_speed);
        } else {
            vehicle.speed = vehicle.speed.signum()
                * (vehicle.speed.abs() - params.friction * dt).max(0.0);
        }

        if keys.pressed(KeyCode::KeyA) {
            vehicle.yaw += params.yaw_rate * dt;
        }
        if keys.pressed(KeyCode::KeyD) {
            vehicle.yaw -= params.yaw_rate * dt;
        }
    }
}

fn vehicle_move_system(
    time: Res<Time>,
    params: Res<GameParams>,
    spatial: SpatialQuery,
    mut q: Query<(Entity, &mut Transform, &mut Vehicle), With<Controlled>>,
) {
    let dt = time.delta_secs() / SUBSTEPS as f32;
    for (entity, mut tf, mut vehicle) in &mut q {
        let col = Collider::cuboid(
            vehicle.half_extents.x,
            vehicle.half_extents.y,
            vehicle.half_extents.z,
        );
        for _ in 0..SUBSTEPS {
            move_vehicle_horizontal(&spatial, &params, entity, &col, &mut tf, &mut vehicle, dt);
            move_vehicle_vertical(&spatial, &params, entity, &col, &mut tf, &mut vehicle, dt);
        }
    }
}

fn vehicle_orientation_system(
    spatial: SpatialQuery,
    mut q: Query<(Entity, &mut Transform, &mut Vehicle), With<Controlled>>,
) {
    for (entity, mut tf, mut vehicle) in &mut q {
        apply_ground_snap(&spatial, entity, &mut tf, &mut vehicle);
        orient_to_ground(&spatial, entity, &mut tf, &vehicle);
    }
}

fn wheel_update_system(
    time: Res<Time>,
    spatial: SpatialQuery,
    vehicles: Query<(&Transform, &Vehicle), Without<Wheel>>,
    mut wheels: Query<(&ChildOf, &mut Transform, &mut Wheel), Without<Vehicle>>,
) {
    let dt = time.delta_secs();
    for (parent, mut tf, mut wheel) in &mut wheels {
        if let Ok((veh_tf, vehicle)) = vehicles.get(parent.parent()) {
            wheel.rotation += vehicle.speed * dt / wheel.radius;
            let steer = if wheel.is_front { vehicle.yaw } else { 0.0 };
            tf.rotation =
                Quat::from_rotation_y(steer)
                    * Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)
                    * Quat::from_rotation_x(wheel.rotation);

            let world_rest = veh_tf.transform_point(wheel.rest_offset);
            let filter = SpatialQueryFilter::default().with_excluded_entities([parent.parent()]);
            let y_off = spatial
                .cast_ray(
                    world_rest + Vec3::Y * wheel.suspension,
                    Dir3::NEG_Y,
                    wheel.suspension + wheel.radius + SKIN,
                    false,
                    &filter,
                )
                .map(|h| wheel.suspension + wheel.radius - h.distance)
                .unwrap_or(0.0)
                .clamp(0.0, wheel.suspension);
            tf.translation = wheel.rest_offset + Vec3::Y * y_off;
        }
    }
}

fn vehicle_toggle_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut players: Query<
        (
            Entity,
            &mut Transform,
            &Player,
            Option<&InVehicle>,
            Option<&Controlled>,
        ),
        (With<Player>, Without<Vehicle>),
    >,
    mut vehicles: Query<
        (Entity, &Transform, Option<&Controlled>),
        (With<Vehicle>, Without<Player>),
    >,
) {
    if !keys.just_pressed(KeyCode::KeyE) {
        return;
    }
    let (player_ent, mut player_tf, player_comp, in_vehicle, player_ctrl) = match players.single_mut() {
        Ok(v) => v,
        Err(_) => return,
    };

    if player_ctrl.is_some() {
        for (veh_ent, veh_tf, veh_ctrl) in &mut vehicles {
            if veh_ctrl.is_some() {
                continue;
            }
            if player_tf.translation.distance(veh_tf.translation) < ENTER_DISTANCE {
                commands
                    .entity(player_ent)
                    .remove::<Controlled>()
                    .remove::<RigidBody>()
                    .remove::<Collider>()
                    .remove::<LinearVelocity>()
                    .insert(InVehicle { vehicle: veh_ent })
                    .insert(Visibility::Hidden);
                commands.entity(veh_ent).insert(Controlled);
                player_tf.translation = veh_tf.translation;
                break;
            }
        }
    } else if let Some(occupy) = in_vehicle {
        if let Ok((veh_ent, veh_tf, _)) = vehicles.get(occupy.vehicle) {
            commands
                .entity(player_ent)
                .insert(Controlled)
                .insert(RigidBody::Kinematic)
                .insert(Collider::cuboid(
                    player_comp.half_extents.x,
                    player_comp.half_extents.y,
                    player_comp.half_extents.z,
                ))
                .insert(LinearVelocity::ZERO)
                .remove::<InVehicle>()
                .insert(Visibility::Visible);
            commands.entity(veh_ent).remove::<Controlled>();
            player_tf.translation = veh_tf.translation + Vec3::Y;
        }
    }
}

fn sync_player_to_vehicle_system(
    mut players: Query<(&mut Transform, &InVehicle), Without<Vehicle>>,
    vehicles: Query<&Transform, (With<Vehicle>, Without<Player>)>,
) {
    for (mut tf, iv) in &mut players {
        if let Ok(v_tf) = vehicles.get(iv.vehicle) {
            tf.translation = v_tf.translation;
        }
    }
}

fn move_vehicle_horizontal(
    spatial: &SpatialQuery,
    params: &GameParams,
    entity: Entity,
    col: &Collider,
    tf: &mut Transform,
    veh: &mut Vehicle,
    dt: f32,
) {
    let yaw_rot = Quat::from_rotation_y(veh.yaw);
    let forward = yaw_rot * Vec3::Z;
    let mut remaining = forward * veh.speed * dt;
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
                    veh.grounded = true;
                }
                slide(&mut remaining, hit.normal1, veh, params);
            }
            None => {
                tf.translation += remaining;
                break;
            }
        }
    }
}

fn slide(remaining: &mut Vec3, normal: Vec3, veh: &mut Vehicle, params: &GameParams) {
    let incoming = *remaining;
    *remaining -= remaining.dot(normal) * normal;

    let mut factor = 1.0 - params.collision_damping;
    if normal.y > 0.0 && normal.y < 1.0 {
        let slope = 1.0 - normal.y;
        let eased = slope.powf(params.slope_ease);
        factor *= 1.0 - eased * params.slope_damping;
    }
    factor = factor.clamp(0.0, 1.0);
    *remaining *= factor;
    veh.speed *= factor;

    let reflect_dir = (incoming - 2.0 * incoming.dot(normal) * normal).normalize_or_zero();
    let incident_angle = incoming.normalize_or_zero().dot(-normal).abs();
    let bounce = incoming.length() * params.bounce_factor * incident_angle;
    *remaining += reflect_dir * bounce;
}

fn move_vehicle_vertical(
    spatial: &SpatialQuery,
    params: &GameParams,
    entity: Entity,
    col: &Collider,
    tf: &mut Transform,
    veh: &mut Vehicle,
    dt: f32,
) {
    apply_ground_snap(spatial, entity, tf, veh);

    if veh.grounded {
        veh.vertical_vel = 0.0;
    } else {
        veh.vertical_vel -= params.gravity * dt;
    }
    tf.translation.y += veh.vertical_vel * dt;
    resolve_vertical_collision(spatial, entity, col, tf, veh);
}

fn resolve_vertical_collision(
    spatial: &SpatialQuery,
    entity: Entity,
    col: &Collider,
    tf: &mut Transform,
    veh: &mut Vehicle,
) {
    let filter = SpatialQueryFilter::default().with_excluded_entities([entity]);
    if let Some(hit) = spatial.cast_shape(
        col,
        tf.translation + Vec3::Y * (veh.half_extents.y + STEP_HEIGHT),
        tf.rotation,
        Dir3::NEG_Y,
        &ShapeCastConfig {
            compute_contact_on_penetration: true,
            max_distance: veh.half_extents.y + STEP_HEIGHT + SKIN,
            ..Default::default()
        },
        &filter,
    ) {
        tf.translation.y = hit.point1.y + veh.half_extents.y + SKIN;
        veh.grounded = true;
        veh.vertical_vel = 0.0;
    }
}

fn apply_ground_snap(
    spatial: &SpatialQuery,
    entity: Entity,
    tf: &mut Transform,
    veh: &mut Vehicle,
) {
    let filter = SpatialQueryFilter::default().with_excluded_entities([entity]);
    let grounded_now = spatial
        .cast_ray(
            tf.translation,
            Dir3::NEG_Y,
            veh.half_extents.y + STEP_HEIGHT + SKIN,
            false,
            &filter,
        )
        .is_some();
    veh.grounded = grounded_now;
}

fn orient_to_ground(spatial: &SpatialQuery, entity: Entity, tf: &mut Transform, veh: &Vehicle) {
    let filter = SpatialQueryFilter::default().with_excluded_entities([entity]);
    let ground_n = spatial
        .cast_ray(
            tf.translation,
            Dir3::NEG_Y,
            veh.half_extents.y + STEP_HEIGHT + SKIN,
            false,
            &filter,
        )
        .map(|h| h.normal)
        .unwrap_or(Vec3::Y);
    let yaw_rot = Quat::from_rotation_y(veh.yaw);
    let target = Quat::from_rotation_arc(Vec3::Y, ground_n) * yaw_rot;
    const ROT_SMOOTH: f32 = 0.2;
    tf.rotation = tf.rotation.slerp(target, ROT_SMOOTH);
}

fn vehicle_fall_reset_system(mut q: Query<(&mut Transform, &mut Vehicle)>) {
    for (mut tf, mut veh) in &mut q {
        if tf.translation.y < FALL_RESET_Y {
            tf.translation = RESPAWN_POS;
            veh.speed = 0.0;
            veh.vertical_vel = 0.0;
            veh.grounded = false;
            veh.yaw = RESPAWN_YAW;
        }
    }
}
