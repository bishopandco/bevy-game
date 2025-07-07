use bevy::prelude::*;
use bevy::math::primitives::Cylinder;
use avian3d::prelude::{Collider, ExternalForce, RigidBody};

use crate::components::{Chassis, SuspensionState, VehicleConfig, Wheel};
use crate::vehicle::Vehicle;

#[derive(Resource)]
pub struct VehicleConfigHandle(pub Handle<VehicleConfig>);

#[derive(Resource)]
pub struct AntiRollBar(pub [f32; 2]);

pub fn load_vehicle_config(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load::<VehicleConfig>("vehicle.ron");
    commands.insert_resource(VehicleConfigHandle(handle));
}

pub fn spawn_vehicle_from_config(
    mut commands: Commands,
    config_handle: Res<VehicleConfigHandle>,
    configs: Res<Assets<VehicleConfig>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }
    let Some(cfg) = configs.get(&config_handle.0) else { return; };

    commands.insert_resource(AntiRollBar(cfg.anti_roll_stiffness));

    let chassis = commands
        .spawn((
            Transform::default(),
            GlobalTransform::default(),
            RigidBody::Dynamic,
            ExternalForce { persistent: false, ..Default::default() },
            Collider::cuboid(1.0, 0.5, 2.0),
            Vehicle::default(),
            Chassis {
                mass: cfg.chassis.mass,
                com_offset: Vec3::new(
                    cfg.chassis.com_offset[0],
                    cfg.chassis.com_offset[1],
                    cfg.chassis.com_offset[2],
                ),
                pitch_roll_smoothing: cfg.chassis.pitch_roll_smoothing,
            },
        ))
        .id();

    let wheel_mesh = meshes.add(Mesh::from(Cylinder {
        radius: cfg.wheels[0].radius,
        half_height: 0.2,
    }));
    let wheel_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.1, 0.1),
        ..default()
    });

    for wc in &cfg.wheels {
        let local_pos = Vec3::new(wc.local_pos[0], wc.local_pos[1], wc.local_pos[2]);
        commands
            .spawn((
                Mesh3d(wheel_mesh.clone()),
                MeshMaterial3d(wheel_material.clone()),
                Transform::default(),
                GlobalTransform::default(),
                RigidBody::Kinematic,
                Collider::cylinder(wc.radius, 0.2),
                Wheel {
                    parent: chassis,
                    local_pos,
                    radius: wc.radius,
                    rest_length: wc.rest_length,
                    spring_k: wc.spring_k,
                    damper_c: wc.damper_c,
                    anti_roll_group: wc.anti_roll_group,
                },
                SuspensionState::default(),
            ));
    }

    *done = true;
}
