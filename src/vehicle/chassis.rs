use super::controls::DriveCmd;
use crate::input::Player;
use avian3d::prelude::*;
use bevy::prelude::*;

/// Marker for the vehicle chassis.
#[derive(Component)]
pub struct Chassis;

pub struct ChassisPlugin;

impl Plugin for ChassisPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_chassis);
    }
}

/// Spawn the vehicle chassis body and attach it to the player.
pub fn spawn_chassis(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut players: Query<(Entity, &Transform, &mut Player)>,
) {
    let Ok((p_ent, p_tf, mut player)) = players.single_mut() else {
        return;
    };
    if player.vehicle.is_some() {
        return;
    }
    let mesh = meshes.add(Cuboid::new(1.0, 0.5, 2.0));
    let material = materials.add(Color::srgb(0.3, 0.3, 0.35));
    let chassis = commands
        .spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
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
    commands.entity(chassis).add_child(p_ent);
    commands
        .entity(p_ent)
        .insert(Transform::from_xyz(0.0, 1.0, 0.0))
        .remove::<RigidBody>()
        .remove::<Collider>();
    player.vehicle = Some(chassis);
}

// Driving forces are applied directly to the wheels now.
