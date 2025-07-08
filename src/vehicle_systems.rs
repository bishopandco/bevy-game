use bevy::prelude::*;
use avian3d::prelude::*;

/// Tuning parameters for the vehicle suspension system.
#[derive(Resource, Clone, Copy)]
pub struct SuspensionTuning {
    /// Spring stiffness coefficient (N/m).
    pub k: f32,
    /// Damping coefficient (N·s/m).
    pub c: f32,
    /// Longitudinal tire coefficient.
    pub mu_long: f32,
    /// Lateral tire coefficient.
    pub mu_lat: f32,
    /// Anti-roll bar stiffness.
    pub k_anti_roll: f32,
    /// Suspension rest length (m).
    pub rest_length: f32,
    /// Maximum additional travel beyond the rest length (m).
    pub max_travel: f32,
    /// Draw debug gizmos when true.
    pub gizmo: bool,
}

impl Default for SuspensionTuning {
    fn default() -> Self {
        Self {
            k: 2.0e5,
            c: 6.0e3,
            mu_long: 1.0,
            mu_lat: 1.0,
            k_anti_roll: 1.5e4,
            rest_length: 0.2,
            max_travel: 0.1,
            gizmo: false,
        }
    }
}

/// Marks the rigid body of the vehicle chassis.
#[derive(Component)]
pub struct Chassis {
    pub mass: f32,
}

/// Wheel using a ray cast suspension.
#[derive(Component)]
pub struct RaycastWheel {
    /// Mount point relative to the chassis.
    pub mount: Vec3,
    /// Radius in meters.
    pub radius: f32,
    /// True if on the front axle.
    pub is_front: bool,
    /// True if left side of the vehicle.
    pub is_left: bool,
    /// Compression amount from 0..=rest+travel.
    pub compression: f32,
    /// Previous compression for velocity calculation.
    pub prev_compression: f32,
    /// Contact position in world space.
    pub contact_point: Vec3,
    /// Contact normal in world space.
    pub contact_normal: Vec3,
    /// Whether the wheel is in contact with the ground.
    pub grounded: bool,
}

impl RaycastWheel {
    pub fn new(mount: Vec3, radius: f32, is_front: bool, is_left: bool) -> Self {
        Self {
            mount,
            radius,
            is_front,
            is_left,
            compression: 0.0,
            prev_compression: 0.0,
            contact_point: Vec3::ZERO,
            contact_normal: Vec3::Y,
            grounded: false,
        }
    }
}

/// Plugin installing the vehicle physics systems.
pub struct VehiclePhysicsPlugin;

impl Plugin for VehiclePhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SuspensionTuning::default())
            .add_systems(
                Update,
                (
                    raycast_wheels,
                    apply_suspension.after(raycast_wheels),
                    compute_tire_forces.after(apply_suspension),
                    apply_anti_roll.after(compute_tire_forces),
                ),
            );
    }
}

/// Casts suspension rays for all wheels and stores the hit information.
pub fn raycast_wheels(
    spatial: SpatialQuery,
    tuning: Res<SuspensionTuning>,
    chassis_q: Query<(&GlobalTransform, &Children), With<Chassis>>,
    mut wheels: Query<(&mut RaycastWheel, &mut Transform)>,
) {
    let ray_len = tuning.rest_length + tuning.max_travel;
    for (chassis_tf, children) in &chassis_q {
        for child in children.iter() {
            if let Ok((mut wheel, mut tf)) = wheels.get_mut(*child) {
                let origin = chassis_tf.transform_point(wheel.mount);
                let dir = chassis_tf.rotation * Vec3::NEG_Y;
                let result = spatial.cast_ray(
                    origin,
                    Dir3::new_unchecked(dir),
                    ray_len,
                    false,
                    &SpatialQueryFilter::default(),
                );
                if let Some(hit) = result {
                    wheel.grounded = true;
                    wheel.contact_point = origin + dir.normalize() * hit.distance;
                    wheel.contact_normal = hit.normal;
                    wheel.prev_compression = wheel.compression;
                    wheel.compression = tuning.rest_length - hit.distance;
                    tf.translation = wheel.mount - Vec3::Y * wheel.compression;
                } else {
                    wheel.grounded = false;
                    wheel.prev_compression = wheel.compression;
                    wheel.compression = 0.0;
                    tf.translation = wheel.mount - Vec3::Y * tuning.rest_length;
                }
            }
        }
    }
}

/// Applies spring and damping forces based on wheel compression.
pub fn apply_suspension(
    time: Res<Time>,
    tuning: Res<SuspensionTuning>,
    mut chassis_q: Query<(&mut LinearVelocity, &GlobalTransform, &Chassis)>,
    wheels: Query<&RaycastWheel>,
) {
    let dt = time.delta_secs();
    for (mut lv, chassis_tf, chassis) in &mut chassis_q {
        for wheel in wheels.iter() {
            if !wheel.grounded { continue; }
            let rel_vel = lv.0.dot(wheel.contact_normal);
            let spring = tuning.k * wheel.compression;
            let damper = -tuning.c * rel_vel;
            let mut force = spring + damper;
            if force < 0.0 { force = 0.0; }
            let impulse = wheel.contact_normal * force * dt;
            lv.0 += impulse / chassis.mass;
        }
    }
}

/// Computes tire friction forces and applies them as impulses.
pub fn compute_tire_forces(
    time: Res<Time>,
    tuning: Res<SuspensionTuning>,
    mut chassis_q: Query<(&mut LinearVelocity, &Chassis)>,
    wheels: Query<&RaycastWheel>,
) {
    let dt = time.delta_secs();
    for (mut lv, chassis) in &mut chassis_q {
        for wheel in wheels.iter() {
            if !wheel.grounded { continue; }
            let load = tuning.k * wheel.compression;
            let long_force = -tuning.mu_long * load;
            let lat_force = -tuning.mu_lat * load;
            let impulse = (wheel.contact_normal.cross(Vec3::Y) * lat_force)
                + (wheel.contact_normal * long_force);
            lv.0 += impulse * dt / chassis.mass;
        }
    }
}

/// Applies an anti-roll torque across each axle.
pub fn apply_anti_roll(
    tuning: Res<SuspensionTuning>,
    mut chassis_q: Query<&mut AngularVelocity, With<Chassis>>,
    wheels: Query<&RaycastWheel>,
) {
    let mut front_l = None;
    let mut front_r = None;
    let mut rear_l = None;
    let mut rear_r = None;
    for wheel in &wheels {
        match (wheel.is_front, wheel.is_left) {
            (true, true) => front_l = Some(wheel.compression),
            (true, false) => front_r = Some(wheel.compression),
            (false, true) => rear_l = Some(wheel.compression),
            (false, false) => rear_r = Some(wheel.compression),
        }
    }
    for mut av in &mut chassis_q {
        if let (Some(fl), Some(fr)) = (front_l, front_r) {
            let delta = fl - fr;
            av.0.z -= delta * tuning.k_anti_roll;
        }
        if let (Some(rl), Some(rr)) = (rear_l, rear_r) {
            let delta = rl - rr;
            av.0.z -= delta * tuning.k_anti_roll;
        }
        let mag = av.0.length();
        if mag > 100.0 {
            av.0 *= 100.0 / mag;
        }
    }
}

