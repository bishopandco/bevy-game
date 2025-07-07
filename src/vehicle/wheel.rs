use avian3d::prelude::*;
use bevy::math::primitives::Cylinder;
use bevy::prelude::*;

use super::chassis::Chassis;

/// A wheel entity.
#[derive(Component)]
pub struct Wheel {
    pub front: bool,
}

pub struct WheelPlugin;

impl Plugin for WheelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_vehicle_wheels);
    }
}

/// Spawn all four wheels for the vehicle.
fn spawn_vehicle_wheels(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chassis_q: Query<(Entity, &Transform), With<Chassis>>,
    wheel_q: Query<(), With<Wheel>>,
    params: Res<super::suspension::SuspensionParams>,
) {
    if !wheel_q.is_empty() {
        return;
    }
    let Ok((chassis, ch_tf)) = chassis_q.single() else {
        return;
    };
    let offs = [
        Vec3::new(-1.0, -0.75, 1.5),
        Vec3::new(1.0, -0.75, 1.5),
        Vec3::new(-1.0, -0.75, -1.5),
        Vec3::new(1.0, -0.75, -1.5),
    ];
    for (i, o) in offs.into_iter().enumerate() {
        let arm = super::suspension::spawn_arm(
            &mut commands,
            meshes.as_mut(),
            materials.as_mut(),
            chassis,
            ch_tf,
            o,
            &params,
        );
        let pos = ch_tf.translation + o;
        spawn_wheel(
            &mut commands,
            meshes.as_mut(),
            materials.as_mut(),
            arm,
            pos,
            i < 2,
        );
    }
}

/// Spawn a single wheel body under the given suspension arm.
pub fn spawn_wheel(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    arm: Entity,
    pos: Vec3,
    front: bool,
) -> Entity {
    let rot = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
    let mesh = meshes.add(Cylinder::new(0.75, 0.3));
    let material = materials.add(Color::srgb(0.1, 0.1, 0.1));
    let wheel = commands
        .spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            RigidBody::Dynamic,
            Collider::cylinder(0.75, 0.15),
            Mass(10.0),
            {
                let mut tf = Transform::from_translation(pos);
                tf.rotation = rot;
                tf
            },
            ExternalForce::default().with_persistence(false),
            ExternalTorque::default().with_persistence(false),
            Wheel { front },
        ))
        .id();
    commands.spawn(RevoluteJoint::new(arm, wheel).with_aligned_axis(Vec3::X));
    wheel
}
