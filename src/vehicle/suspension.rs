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

#[derive(Component)]
pub struct SuspensionArm {
    pub chassis: Entity,
    pub offset: Vec3,
}

pub struct SuspensionPlugin;

impl Plugin for SuspensionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SuspensionParams::default())
            .add_systems(Update, suspension_system);
    }
}

/// Spawn a suspension arm entity attached to the given chassis.
pub fn spawn_arm(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    chassis: Entity,
    chassis_tf: &Transform,
    offset: Vec3,
    params: &SuspensionParams,
) -> Entity {
    let mesh = meshes.add(Cylinder::new(0.1, params.max_travel * 2.0));
    let material = materials.add(Color::srgb(0.2, 0.2, 0.2));
    let arm_pos = chassis_tf.translation + offset;
    let arm = commands
        .spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            RigidBody::Dynamic,
            Mass(0.0),
            Transform::from_translation(arm_pos),
            ExternalForce::default().with_persistence(false),
            SuspensionArm { chassis, offset },
        ))
        .id();
    commands.spawn(
        PrismaticJoint::new(chassis, arm)
            .with_local_anchor_1(offset)
            .with_local_anchor_2(Vec3::ZERO)
            .with_free_axis(Vec3::Y)
            .with_limits(-params.max_travel, 0.0)
            .with_compliance(1.0 / params.spring_k)
            .with_linear_velocity_damping(params.damping_c / params.spring_k),
    );
    arm
}

pub fn suspension_system(
    params: Res<SuspensionParams>,
    mut arms: Query<
        (
            &SuspensionArm,
            &Transform,
            &mut ExternalForce,
            &LinearVelocity,
        ),
        Without<Chassis>,
    >,
    mut ch_q: Query<(&Transform, &mut ExternalForce, &LinearVelocity), With<Chassis>>,
) {
    let Ok((ct, mut cf, cv)) = ch_q.single_mut() else {
        return;
    };
    let axis = ct.rotation * Vec3::Y;
    for (arm, tf, mut ext_f, vel) in &mut arms {
        let rest = ct.translation + arm.offset;
        let d = (tf.translation - rest).dot(axis);
        let v = (vel.0 - cv.0).dot(axis);
        let spring = params.spring_k * -d - params.damping_c * v;
        ext_f.apply_force(axis * spring);
        cf.apply_force(-axis * spring);
    }
}
