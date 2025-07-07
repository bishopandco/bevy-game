use avian3d::prelude::*;
use bevy::prelude::*;
use crate::input::Player;
use super::controls::DriveCmd;

/// Marker for the vehicle chassis.
#[derive(Component)]
pub struct Chassis;

pub struct ChassisPlugin;

impl Plugin for ChassisPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_chassis)
            .add_systems(Update, drive_chassis_system);
    }
}

/// Spawn the vehicle chassis body and attach it to the player.
pub fn spawn_chassis(
    mut commands: Commands,
    mut players: Query<(Entity, &Transform, &mut Player)>,
) {
    let Ok((p_ent, p_tf, mut player)) = players.single_mut() else { return; };
    let chassis = commands
        .spawn((
            RigidBody::Dynamic,
            Collider::cuboid(1.0, 0.5, 2.0),
            Mass(160.0),
            CenterOfMass::new(0.0, -0.3, 0.0),
            AngularInertia::new(Vec3::splat(50.0)),
            Transform::from_translation(p_tf.translation),
            ExternalForce::default().with_persistence(false),
            Chassis,
        ))
        .id();
    commands.entity(chassis).push_children(&[p_ent]);
    commands.entity(p_ent).insert(Transform::from_xyz(0.0, 1.0, 0.0));
    player.vehicle = Some(chassis);
}

fn drive_chassis_system(
    cmd: Res<DriveCmd>,
    mut q: Query<(&Transform, &mut ExternalForce), With<Chassis>>,
) {
    let Ok((tf, mut force)) = q.single_mut() else { return; };
    let forward = tf.rotation * Vec3::Z;
    force.apply_force(forward * cmd.throttle * 2_000.0);
}
