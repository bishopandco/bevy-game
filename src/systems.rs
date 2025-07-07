use bevy::prelude::*;
use avian3d::prelude::{Collider, Dir3, ExternalForce, ShapeCastConfig, SpatialQuery, SpatialQueryFilter};

use crate::components::{Chassis, SuspensionState, Wheel};
use crate::setup_vehicle::AntiRollBar;

pub fn drive_suspension(
    time: Res<Time>,
    spatial: SpatialQuery,
    mut chassis_q: Query<(&Transform, &mut ExternalForce, Entity, &Chassis)>,
    mut wheels: Query<(&Wheel, &mut SuspensionState)>,
    anti_roll: Option<Res<AntiRollBar>>,
) {
    let dt = time.delta_secs();
    let Ok((chassis_tf, mut force, chassis_ent, chassis)) = chassis_q.single_mut() else { return; };

    struct AxleData {
        left: Option<(Vec3, f32)>,
        right: Option<(Vec3, f32)>,
    }
    let mut axles = [
        AxleData {
            left: None,
            right: None,
        },
        AxleData {
            left: None,
            right: None,
        },
    ];

    for (wheel, mut state) in &mut wheels {
        if wheel.parent != chassis_ent { continue; }
        let origin = chassis_tf.translation + chassis_tf.rotation * wheel.local_pos;
        let down = -(chassis_tf.rotation * Vec3::Y);
        let dir = Dir3::new_unchecked(down);
        let shape = Collider::ball(wheel.radius);
        let config = ShapeCastConfig { max_distance: wheel.rest_length + wheel.radius, ..Default::default() };
        let filter = SpatialQueryFilter::default();

        let hit = spatial.cast_shape(&shape, origin, Quat::IDENTITY, dir, &config, &filter);
        let (compression, point) = if let Some(h) = hit {
            let comp = wheel.rest_length + wheel.radius - h.distance;
            (comp.clamp(0.0, wheel.rest_length), h.point1)
        } else {
            (0.0, origin + down * (wheel.rest_length))
        };
        state.compression_vel = (compression - state.compression) / dt;
        state.compression = compression;
        state.contact = hit.is_some();

        let spring_force = wheel.spring_k * state.compression;
        let damper_force = wheel.damper_c * state.compression_vel;
        let f = -(down * (spring_force + damper_force));
        force.apply_force_at_point(f, point, chassis_tf.translation + chassis.com_offset);

        let index = wheel.anti_roll_group as usize;
        if wheel.local_pos.x >= 0.0 {
            axles[index].left = Some((point, state.compression));
        } else {
            axles[index].right = Some((point, state.compression));
        }
    }

    if let Some(ar) = anti_roll {
        for (i, axle) in axles.into_iter().enumerate() {
            if let (Some((lp, lc)), Some((rp, rc))) = (axle.left, axle.right) {
                let diff = lc - rc;
                let roll_force = diff * ar.0[i];
                let down = -(chassis_tf.rotation * Vec3::Y);
                force.apply_force_at_point(-down * roll_force, lp, chassis_tf.translation + chassis.com_offset);
                force.apply_force_at_point(down * roll_force, rp, chassis_tf.translation + chassis.com_offset);
            }
        }
    }
}

pub fn update_chassis_pose(
    mut chassis_q: Query<(&mut Transform, &Chassis, Entity)>,
    wheels: Query<(&Wheel, &SuspensionState)>,
) {
    let Ok((mut tf, chassis, ent)) = chassis_q.single_mut() else { return; };
    let mut contacts = Vec::new();
    let mut hubs = Vec::new();

    for (wheel, state) in &wheels {
        if wheel.parent != ent { continue; }
        let base = tf.translation + tf.rotation * wheel.local_pos;
        let down = tf.rotation * Vec3::NEG_Y;
        let hub = base - down * state.compression;
        let contact = hub - down * wheel.radius;
        hubs.push(hub);
        contacts.push(contact);
    }
    if contacts.len() >= 3 {
        let n = (contacts[1] - contacts[0]).cross(contacts[2] - contacts[0]).normalize();
        let target = Quat::from_rotation_arc(Vec3::Y, n);
        tf.rotation = tf.rotation.slerp(target, chassis.pitch_roll_smoothing);
    }
    if !hubs.is_empty() {
        let mut avg = Vec3::ZERO;
        for p in hubs { avg += p; }
        avg /= hubs.len() as f32;
        tf.translation = avg + chassis.com_offset;
    }
}

pub fn sync_wheel_meshes(
    chassis_q: Query<(Entity, &Transform), With<Chassis>>,
    mut wheels: Query<(&Wheel, &SuspensionState, &mut Transform), Without<Chassis>>,
) {
    let Ok((chassis_ent, chassis_tf)) = chassis_q.single() else { return; };
    for (wheel, state, mut tf) in &mut wheels {
        if wheel.parent != chassis_ent { continue; }
        let base = chassis_tf.translation + chassis_tf.rotation * wheel.local_pos;
        let down = chassis_tf.rotation * Vec3::NEG_Y;
        tf.translation = base - down * state.compression;
        let forward = chassis_tf.rotation * Vec3::Z;
        let right = chassis_tf.rotation * Vec3::X;
        tf.rotation = Quat::from_rotation_arc(Vec3::X, right) * Quat::from_rotation_arc(Vec3::Z, forward);
    }
}
