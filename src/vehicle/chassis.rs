use avian3d::prelude::*;
use bevy::prelude::*;

/// Marker for the vehicle chassis.
#[derive(Component)]
pub struct Chassis;

pub struct ChassisPlugin;

impl Plugin for ChassisPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_chassis);
    }
}

/// Spawn the vehicle chassis body.
pub fn spawn_chassis(mut commands: Commands) {
    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 0.5, 2.0),
        Mass(160.0),
        CenterOfMass::new(0.0, -0.3, 0.0),
        AngularInertia::new(Vec3::splat(50.0)),
        Transform::from_xyz(0.0, 1.5, 0.0),
        ExternalForce::default().with_persistence(false),
        Chassis,
    ));
}
