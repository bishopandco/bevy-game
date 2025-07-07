use avian3d::prelude::*;
use bevy::math::primitives::Cylinder;
use bevy::prelude::*;

use super::chassis::Chassis;

/// Suspension tuning parameters.
#[derive(Resource)]
pub struct SuspensionParams {
    pub spring_k: f32,
    pub damping_c: f32,
    pub max_travel: f32,
}

impl Default for SuspensionParams {
    fn default() -> Self {
        Self {
            spring_k: 40_000.0,
            damping_c: 4_000.0,
            max_travel: 0.75,
        }
    }
}

/// Connection point between a wheel and the chassis.
#[derive(Component)]
pub struct WheelHub {
    pub chassis: Entity,
    pub offset: Vec3,
    pub front: bool,
}

pub struct SuspensionPlugin;

impl Plugin for SuspensionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SuspensionParams::default())
            .add_systems(Update, suspension_system);
    }
}

/// Spawn a suspension arm entity attached to the given chassis.
pub fn spawn_hub(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    chassis: Entity,
    chassis_tf: &Transform,
    offset: Vec3,
    front: bool,
    params: &SuspensionParams,
) -> Entity {
    const SPREAD: f32 = 0.5;
    let hub_pos = chassis_tf.translation + offset;
    let hub = commands
        .spawn((
            Mesh3d(meshes.add(Cylinder::new(0.05, SPREAD))),
            MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.2))),
            RigidBody::Dynamic,
            Mass(0.0),
            Transform::from_translation(hub_pos),
            ExternalForce::default().with_persistence(false),
            WheelHub {
                chassis,
                offset,
                front,
            },
        ))
        .id();

    for side in [-0.5, 0.5] {
        let anchor = Vec3::new(side * SPREAD, 0.0, 0.0);
        commands.spawn(
            PrismaticJoint::new(chassis, hub)
                .with_local_anchor_1(offset + anchor)
                .with_local_anchor_2(anchor)
                .with_free_axis(Vec3::Y)
                .with_limits(-params.max_travel, 0.0)
                .with_compliance(1.0 / params.spring_k)
                .with_linear_velocity_damping(params.damping_c / params.spring_k),
        );
    }

    hub
}

pub fn suspension_system(
    params: Res<SuspensionParams>,
    mut hubs: Query<(&WheelHub, &Transform, &mut ExternalForce, &LinearVelocity), Without<Chassis>>,
    mut ch_q: Query<(&Transform, &mut ExternalForce, &LinearVelocity), With<Chassis>>,
) {
    let Ok((ct, mut cf, cv)) = ch_q.single_mut() else {
        return;
    };
    let axis = ct.rotation * Vec3::Y;
    for (hub, tf, mut ext_f, vel) in &mut hubs {
        let rest = ct.translation + hub.offset;
        let d = (tf.translation - rest).dot(axis);
        let v = (vel.0 - cv.0).dot(axis);
        let spring = params.spring_k * -d - params.damping_c * v;
        ext_f.apply_force(axis * spring);
        cf.apply_force(-axis * spring);
    }
}
