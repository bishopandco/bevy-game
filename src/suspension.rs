use bevy::prelude::*;
use avian3d::prelude::*;

/// Configuration for a wheel of the vehicle.
#[derive(Component, Clone, Copy, Debug)]
pub struct WheelConfig {
    pub pos: Vec3,
    pub radius: f32,
    pub mass: f32,
    pub steer: bool,
    pub drive: bool,
}

/// Collection of wheel configurations used when spawning the vehicle.
#[derive(Resource, Clone)]
pub struct WheelConfigs(pub Vec<WheelConfig>);

/// Parameters controlling the suspension behaviour.
#[derive(Resource, Clone, Copy, Debug)]
pub struct SuspensionConfig {
    pub rest_len: f32,
    pub travel: f32,
    pub stiffness: f32,
    pub damping: f32,
}

/// Runtime suspension state for a wheel.
#[derive(Component, Debug)]
pub struct Suspension {
    pub compression: f32,
    pub contact_point: Vec3,
    pub contact_normal: Vec3,
    pub grounded: bool,
    pub limit_counter: u8,
}

impl Default for Suspension {
    fn default() -> Self {
        Self {
            compression: 0.0,
            contact_point: Vec3::ZERO,
            contact_normal: Vec3::Y,
            grounded: false,
            limit_counter: 0,
        }
    }
}

/// Applies spring and damper forces based on the current wheel compression.
#[allow(clippy::type_complexity)]
pub fn suspension_force_system(
    spatial: SpatialQuery,
    cfg: Res<SuspensionConfig>,
    mut wheels: Query<(
        &GlobalTransform,
        &mut RigidBody,
        &Parent,
        &mut Suspension,
    )>,
    mut chassis: Query<&mut RigidBody>,
) {
    let ray_len = cfg.rest_len + cfg.travel;
    for (tf, mut wheel_rb, parent, mut sus) in &mut wheels {
        let Ok(mut chassis_rb) = chassis.get_mut(parent.get()) else { continue; };
        let origin = tf.translation();
        let dir = Vec3::NEG_Y;
        let result = spatial.cast_ray(
            origin,
            Dir3::new_unchecked(dir),
            ray_len,
            false,
            &SpatialQueryFilter::default(),
        );
        if let Some(hit) = result {
            let compression = (cfg.rest_len - hit.distance).clamp(0.0, cfg.travel);
            let rel_vel = wheel_rb.linear_velocity().dot(Vec3::Y)
                - chassis_rb.linear_velocity().dot(Vec3::Y);
            let force_mag = cfg.stiffness * compression - cfg.damping * rel_vel;
            let force = Vec3::Y * force_mag;
            wheel_rb.apply_force(force, ForceMode::Force);
            chassis_rb.apply_force(-force, ForceMode::Force);
            sus.compression = compression;
            sus.contact_point = origin + dir * hit.distance;
            sus.contact_normal = hit.normal;
            sus.grounded = true;
            if compression.abs() >= cfg.travel {
                sus.limit_counter += 1;
                if sus.limit_counter > 10 {
                    warn!("suspension travel exceeded");
                    sus.limit_counter = 0;
                }
            } else {
                sus.limit_counter = 0;
            }
        } else {
            sus.grounded = false;
            sus.compression = 0.0;
            sus.limit_counter = 0;
        }
    }
}

