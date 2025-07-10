use bevy::prelude::*;
use avian3d::prelude::*;

use crate::suspension::{WheelConfigs, WheelConfig, SuspensionConfig, Suspension, suspension_force_system};

/// Revolute joint used for wheel rolling.
#[derive(Component)]
pub struct RollRevolute(pub RevoluteJoint);

/// Revolute joint used for steering the wheel.
#[derive(Component)]
pub struct SteerRevolute(pub RevoluteJoint);

/// Plugin spawning a simple vehicle with independent suspension.
pub struct VehiclePlugin;

impl Plugin for VehiclePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(default_wheels())
            .insert_resource(SuspensionConfig {
                rest_len: 0.3,
                travel: 0.2,
                stiffness: 30000.0,
                damping: 4500.0,
            })
            .add_systems(Startup, spawn_vehicle)
            .add_systems(Update, (
                steering_input_system,
                drive_system,
                suspension_force_system,
                sync_visuals.after(suspension_force_system),
            ));
    }
}

fn default_wheels() -> WheelConfigs {
    WheelConfigs(vec![
        WheelConfig { pos: Vec3::new( 1.0, 0.0, 1.5), radius: 0.5, mass: 20.0, steer: true,  drive: false },
        WheelConfig { pos: Vec3::new(-1.0, 0.0, 1.5), radius: 0.5, mass: 20.0, steer: true,  drive: false },
        WheelConfig { pos: Vec3::new( 1.0, 0.0,-1.5), radius: 0.5, mass: 20.0, steer: false, drive: true  },
        WheelConfig { pos: Vec3::new(-1.0, 0.0,-1.5), radius: 0.5, mass: 20.0, steer: false, drive: true  },
    ])
}

#[allow(clippy::too_many_arguments)]
fn spawn_vehicle(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    wheels: Res<WheelConfigs>,
    sus_cfg: Res<SuspensionConfig>,
) {
    let chassis = commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::new(2.0, 0.5, 4.0))),
            material: materials.add(Color::srgb(0.2, 0.2, 0.8)),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(1.0, 0.25, 2.0))
        .id();

    for cfg in wheels.0.iter() {
        let wheel = commands
            .spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(Cylinder {
                    radius: cfg.radius,
                    half_height: 0.15,
                })),
                material: materials.add(Color::srgb(0.1, 0.1, 0.1)),
                transform: Transform::from_translation(cfg.pos),
                ..default()
            })
            .insert(RigidBody::Dynamic)
            .insert(Collider::cylinder(cfg.radius, 0.15))
            .insert(Suspension::default())
            .insert(*cfg)
            .id();

        commands.entity(wheel).insert(PrismaticJoint::new(
            chassis,
            Vec3::Y,
            -sus_cfg.travel,
            sus_cfg.travel,
        ));
        commands.entity(wheel).insert(RollRevolute(RevoluteJoint::new(chassis, Vec3::X)));
        if cfg.steer {
            commands.entity(wheel).insert(SteerRevolute(RevoluteJoint::new(chassis, Vec3::Y)));
        }
    }
}

fn steering_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut joints: Query<&mut SteerRevolute>,
) {
    let steer = if keys.pressed(KeyCode::KeyA) {
        0.5
    } else if keys.pressed(KeyCode::KeyD) {
        -0.5
    } else {
        0.0
    };

    for mut joint in &mut joints {
        joint.0.set_motor_target(steer);
    }
}

fn drive_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut joints: Query<(&mut RollRevolute, &WheelConfig)>,
) {
    let throttle = if keys.pressed(KeyCode::KeyW) {
        50.0
    } else if keys.pressed(KeyCode::KeyS) {
        -50.0
    } else {
        0.0
    };

    for (mut joint, cfg) in &mut joints {
        if cfg.drive {
            joint.0.add_motor_torque(throttle);
        }
    }
}

fn sync_visuals(
    wheels: Query<(&GlobalTransform, &Parent), With<Suspension>>,
    mut visuals: Query<&mut Transform, Without<RigidBody>>,
) {
    for (tf, parent) in &wheels {
        if let Ok(mut v_tf) = visuals.get_mut(parent.get()) {
            v_tf.translation = tf.translation();
            v_tf.rotation = tf.rotation();
        }
    }
}

