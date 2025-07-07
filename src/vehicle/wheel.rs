use avian3d::prelude::*;
use bevy::prelude::*;

use super::chassis::Chassis;

/// A wheel entity.
#[derive(Component)]
pub struct Wheel { pub front: bool }

pub struct WheelPlugin;

impl Plugin for WheelPlugin { fn build(&self, app: &mut App) { app.add_systems(Startup, spawn_vehicle_wheels); } }

fn spawn_vehicle_wheels(mut commands: Commands, chassis_q: Query<Entity, With<Chassis>>, params: Res<super::suspension::SuspensionParams>) {
    let Ok(chassis) = chassis_q.get_single() else { return; };
    let offs = [Vec3::new(-1.0,-0.75, 1.5), Vec3::new(1.0,-0.75,1.5), Vec3::new(-1.0,-0.75,-1.5), Vec3::new(1.0,-0.75,-1.5)];
    for (i,o) in offs.into_iter().enumerate() { let arm = super::suspension::spawn_arm(&mut commands,chassis,o,&params); spawn_wheel(&mut commands,arm,i<2); }
}

pub fn spawn_wheel(commands: &mut Commands, arm: Entity, front: bool) -> Entity {
    let rot = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
    let wheel = commands.spawn((RigidBody::Dynamic, Collider::cylinder(0.75,0.15), Mass(10.0), Transform::from_rotation(rot), ExternalForce::default().with_persistence(false), ExternalTorque::default().with_persistence(false), Wheel{front})).id();
    commands.spawn(RevoluteJoint::new(arm,wheel).with_aligned_axis(Vec3::X));
    wheel
}
