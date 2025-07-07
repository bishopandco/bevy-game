use avian3d::prelude::*;
use bevy::prelude::*;

use super::chassis::Chassis;

/// Suspension tuning parameters.
#[derive(Resource)]
pub struct SuspensionParams { pub spring_k: f32, pub damping_c: f32, pub max_travel: f32 }

impl Default for SuspensionParams { fn default() -> Self { Self{ spring_k:40000.0, damping_c:4000.0, max_travel:0.75 } } }

#[derive(Component)]
pub struct SuspensionArm { pub chassis: Entity }

pub struct SuspensionPlugin;

impl Plugin for SuspensionPlugin { fn build(&self, app: &mut App) { app.insert_resource(SuspensionParams::default()).add_systems(Update, suspension_system); } }

pub fn spawn_arm(commands: &mut Commands, chassis: Entity, offset: Vec3, params: &SuspensionParams) -> Entity {
    let arm = commands.spawn((RigidBody::Dynamic, Mass(0.0), Transform::from_translation(offset), ExternalForce::default().with_persistence(false), SuspensionArm{chassis})).id();
    commands.spawn(PrismaticJoint::new(chassis,arm).with_local_anchor_1(offset).with_limits(-params.max_travel,0.0).with_compliance(1.0/params.spring_k).with_linear_velocity_damping(params.damping_c/params.spring_k));
    arm
}

pub fn suspension_system(params: Res<SuspensionParams>, mut arms: Query<(&Transform,&mut ExternalForce,&LinearVelocity),With<SuspensionArm>>, mut ch_q: Query<(&Transform,&mut ExternalForce,&LinearVelocity),With<Chassis>>) {
    let (ct,mut cf,cv)=ch_q.single_mut(); let axis=ct.rotation*Vec3::Y;
    for (tf,mut f,vel) in &mut arms {
        let (d,v)=((tf.translation-ct.translation).dot(axis),(vel.0-cv.0).dot(axis));
        let f=params.spring_k*-d-params.damping_c*v; f.apply_force(axis*f); cf.apply_force(-axis*f);
    }
}
