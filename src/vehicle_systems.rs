use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{globals::Controlled, input::Player, vehicle::Vehicle};

pub const CHASSIS_MASS: f32 = 800.0;
pub const WHEEL_RADIUS: f32 = 0.5;
pub const WHEEL_NAMES: [&str; 4] = ["Front left", "Front right", "Rear left", "Rear right"];
pub const WHEEL_MOUNTS: [Vec3; 4] = [
    Vec3::new(1.0, -0.15, 1.5),
    Vec3::new(-1.0, -0.15, 1.5),
    Vec3::new(1.0, -0.15, -1.5),
    Vec3::new(-1.0, -0.15, -1.5),
];

/// Each corner has its own spring, damper, travel limits, and tire settings.
#[derive(Clone, Copy)]
pub struct WheelTuning {
    pub spring_rate: f32,
    pub compression_damping: f32,
    pub rebound_damping: f32,
    pub rest_length: f32,
    pub compression_travel: f32,
    pub droop_travel: f32,
    pub bump_stop_rate: f32,
    pub max_force: f32,
    pub friction: f32,
    pub lateral_stiffness: f32,
}

impl Default for WheelTuning {
    fn default() -> Self {
        Self {
            spring_rate: 35_000.0,
            compression_damping: 3_000.0,
            rebound_damping: 4_000.0,
            rest_length: 0.35,
            compression_travel: 0.25,
            droop_travel: 0.15,
            bump_stop_rate: 100_000.0,
            max_force: 20_000.0,
            friction: 1.1,
            lateral_stiffness: 2_500.0,
        }
    }
}

#[derive(Resource, Clone)]
pub struct SuspensionTuning {
    pub wheels: [WheelTuning; 4],
    pub engine_force: f32,
    pub brake_force: f32,
    pub rolling_resistance: f32,
    pub max_speed: f32,
    pub steering_angle: f32,
    pub steering_rate: f32,
    pub center_of_mass_height: f32,
    pub debug_draw: bool,
}

impl Default for SuspensionTuning {
    fn default() -> Self {
        Self {
            wheels: [WheelTuning::default(); 4],
            engine_force: 6_000.0,
            brake_force: 12_000.0,
            rolling_resistance: 0.015,
            max_speed: 40.0,
            steering_angle: 0.5,
            steering_rate: 2.0,
            center_of_mass_height: -0.35,
            debug_draw: false,
        }
    }
}

/// Contact data belongs to one corner of one vehicle, never a global wheel list.
#[derive(Clone, Copy, Default)]
pub struct WheelState {
    pub grounded: bool,
    pub length: f32,
    pub compression: f32,
    pub load: f32,
    pub contact: Vec3,
    pub normal: Vec3,
    pub spin: f32,
}

pub struct VehiclePhysicsPlugin;

#[derive(Message)]
pub struct VehicleReset;

impl Plugin for VehiclePhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SuspensionTuning>()
            .add_message::<VehicleReset>()
            .add_systems(FixedPostUpdate, reset_vehicles.before(PhysicsSystems::Prepare))
            .add_systems(FixedPostUpdate, sync_center_of_mass.before(PhysicsSystems::Prepare))
            .add_systems(
                PhysicsSchedule,
                simulate_vehicles
                    .after(PhysicsStepSystems::NarrowPhase)
                    .before(PhysicsStepSystems::Solver),
            )
            .add_systems(Update, draw_suspension);
    }
}

fn sync_center_of_mass(
    tuning: Res<SuspensionTuning>,
    mut bodies: Query<&mut CenterOfMass, With<Vehicle>>,
) {
    for mut center in &mut bodies {
        let target = Vec3::new(0.0, tuning.center_of_mass_height, 0.0);
        if center.0 != target { center.0 = target; }
    }
}

fn reset_vehicles(
    mut messages: MessageReader<VehicleReset>,
    mut vehicles: Query<(&mut Position, &mut Rotation, &mut LinearVelocity, &mut AngularVelocity, &mut Vehicle)>,
) {
    if messages.read().next().is_none() { return; }
    messages.clear();
    for (mut position, mut rotation, mut linear, mut angular, mut vehicle) in &mut vehicles {
        position.y += 1.5;
        // Preserve heading, but remove roll and pitch before placing the wheels down.
        let heading = rotation.0 * Vec3::Z;
        rotation.0 = Quat::from_rotation_y(heading.x.atan2(heading.z));
        linear.0 = Vec3::ZERO;
        angular.0 = Vec3::ZERO;
        *vehicle = Vehicle::default();
    }
}

/// Avian integrates the force and its torque once per fixed physics step.
/// Wheel queries use the physical pose, not a render-frame GlobalTransform.
fn simulate_vehicles(
    time: Res<Time<Physics>>,
    keys: Res<ButtonInput<KeyCode>>,
    tuning: Res<SuspensionTuning>,
    spatial: SpatialQuery,
    players: Query<Entity, With<Player>>,
    mut vehicles: Query<(Entity, Forces, &mut Vehicle, Has<Controlled>)>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 { return; }

    for (entity, mut body, mut vehicle, controlled) in &mut vehicles {
        let position = body.position();
        let rotation = body.rotation();
        let up = rotation * Vec3::Y;
        let forward = rotation * Vec3::Z;
        vehicle.speed = body.linear_velocity().dot(forward);

        let throttle = if controlled {
            keys.pressed(KeyCode::KeyW) as u8 as f32 - keys.pressed(KeyCode::KeyS) as u8 as f32
        } else { 0.0 };
        let brake = controlled && keys.pressed(KeyCode::Space);
        let steer_input = if controlled {
            keys.pressed(KeyCode::KeyA) as u8 as f32 - keys.pressed(KeyCode::KeyD) as u8 as f32
        } else { 0.0 };
        let steer_target = steer_input * tuning.steering_angle;
        let steer_step = tuning.steering_rate * dt;
        let current_steering = vehicle.steering;
        vehicle.steering = current_steering
            + (steer_target - current_steering).clamp(-steer_step, steer_step);

        // Only the chassis has a collider; the GLTF hierarchy is visual only.
        // Exclude the chassis and player colliders from suspension ground probes.
        let filter = SpatialQueryFilter::default()
            .with_excluded_entities(std::iter::once(entity).chain(players.iter()));

        for index in 0..4 {
            let settings = tuning.wheels[index];
            let min_length = (settings.rest_length - settings.compression_travel).max(0.02);
            let max_length = settings.rest_length + settings.droop_travel;
            let mount = position + rotation * WHEEL_MOUNTS[index];
            let steer = if index < 2 { vehicle.steering } else { 0.0 };
            let wheel_forward = rotation * (Quat::from_rotation_y(steer) * Vec3::Z);
            let wheel = &mut vehicle.wheels[index];
            wheel.grounded = false;
            wheel.load = 0.0;
            wheel.compression = 0.0;
            wheel.length = max_length;

            let Some(hit) = spatial.cast_ray(
                mount, Dir3::new_unchecked(-up), max_length + WHEEL_RADIUS, true, &filter,
            ) else { continue; };
            // Reject surfaces facing away from the suspension (walls/ceilings).
            let alignment = hit.normal.dot(up);
            if alignment < 0.2 { continue; }

            let raw_length = hit.distance - WHEEL_RADIUS;
            wheel.length = raw_length.clamp(min_length, max_length);
            wheel.compression = (settings.rest_length - wheel.length).max(0.0);
            wheel.contact = mount - up * hit.distance;
            wheel.normal = hit.normal;
            wheel.grounded = true;

            // Point velocity includes angular motion. No compression differences
            // are used, so landing or changing travel cannot create derivative spikes.
            let velocity = body.velocity_at_point(wheel.contact);
            let compression_speed = -velocity.dot(up);
            let damping = if compression_speed >= 0.0 {
                settings.compression_damping
            } else { settings.rebound_damping };
            let bump = (min_length - raw_length).max(0.0) * settings.bump_stop_rate;
            let support = (settings.spring_rate * wheel.compression
                + damping * compression_speed + bump).clamp(0.0, settings.max_force);
            wheel.load = support * alignment;
            body.apply_force_at_point(up * support, wheel.contact);

            // Tire axes lie in the road plane. All tire forces share a friction
            // circle, limited by this corner's actual normal load.
            let tangent = (wheel_forward - hit.normal * wheel_forward.dot(hit.normal))
                .normalize_or_zero();
            let lateral = hit.normal.cross(tangent).normalize_or_zero();
            let longitudinal_speed = velocity.dot(tangent);
            let lateral_speed = velocity.dot(lateral);
            let effective_mass = CHASSIS_MASS / 4.0;
            let stop_force = -longitudinal_speed * effective_mass / dt;
            let rolling = tuning.rolling_resistance * wheel.load;
            let resistance = if brake { tuning.brake_force / 4.0 + rolling } else { rolling };
            let braking_force = stop_force.clamp(-resistance, resistance);
            let can_drive = longitudinal_speed.abs() < tuning.max_speed
                || longitudinal_speed * throttle <= 0.0;
            let drive = if !brake && can_drive { throttle * tuning.engine_force / 4.0 } else { 0.0 };
            // Bound velocity correction per step to reduce low-speed oscillation.
            let side_force = -lateral_speed * settings.lateral_stiffness
                .min(effective_mass * 0.5 / dt);
            let tire_force = tangent * (drive + braking_force) + lateral * side_force;
            body.apply_force_at_point(
                tire_force.clamp_length_max(settings.friction * wheel.load), wheel.contact,
            );
            wheel.spin = (wheel.spin + longitudinal_speed * dt / WHEEL_RADIUS)
                .rem_euclid(std::f32::consts::TAU);
        }
    }
}

fn draw_suspension(
    tuning: Res<SuspensionTuning>,
    vehicles: Query<(&GlobalTransform, &Vehicle)>,
    mut gizmos: Gizmos,
) {
    if !tuning.debug_draw { return; }
    for (transform, vehicle) in &vehicles {
        for (index, wheel) in vehicle.wheels.iter().enumerate() {
            let mount = transform.transform_point(WHEEL_MOUNTS[index]);
            let end = mount - transform.rotation() * Vec3::Y * (wheel.length + WHEEL_RADIUS);
            let color = if wheel.grounded { Color::srgb(0.2, 1.0, 0.2) } else { Color::srgb(1.0, 0.2, 0.2) };
            gizmos.line(mount, end, color);
            if wheel.grounded {
                gizmos.line(wheel.contact, wheel.contact + wheel.normal * wheel.load / 10_000.0, Color::srgb(1.0, 1.0, 0.0));
            }
        }
    }
}
