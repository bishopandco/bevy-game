use bevy::prelude::*;
use avian3d::prelude::*;
use std::f32::consts::FRAC_PI_2;

#[derive(Resource, Clone, Copy)]
pub struct SuspensionParams {
    pub spring_k: f32,
    pub damping_c: f32,
    pub max_travel: f32,
}

impl Default for SuspensionParams {
    fn default() -> Self {
        // A slightly stiffer spring helps keep the chassis from bottoming out
        // under its own weight.
        Self { spring_k: 800.0, damping_c: 40.0, max_travel: 0.75 }
    }
}

#[derive(Resource, Clone, Copy)]
pub struct VehicleTuning {
    pub com_offset: Vec3,
    pub suspension: SuspensionParams,
}

impl Default for VehicleTuning {
    fn default() -> Self {
        Self { com_offset: Vec3::new(0.0, -0.3, 0.0), suspension: SuspensionParams::default() }
    }
}

#[derive(Component)]
pub struct Vehicle {
    pub drive_mode: DriveMode,
}

#[derive(Component, Default)]
pub struct Wheel {
    pub offset: Vec3,
    pub radius: f32,
    pub width: f32,
    pub steer: bool,
    pub drive: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DriveMode {
    Fwd,
    Rwd,
    Awd,
}

#[derive(Bundle)]
pub struct VehicleBundle {
    pub vehicle: Vehicle,
    pub rb: RigidBody,
    pub collider: Collider,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for VehicleBundle {
    fn default() -> Self {
        Self {
            vehicle: Vehicle { drive_mode: DriveMode::Awd },
            rb: RigidBody::Dynamic,
            // A slightly smaller collider keeps the overall mass low so the
            // suspension can properly support the chassis.
            collider: Collider::cuboid(0.8, 0.4, 1.2),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
        }
    }
}

#[derive(Bundle)]
pub struct WheelBundle {
    pub wheel: Wheel,
    pub rb: RigidBody,
    pub collider: Collider,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for WheelBundle {
    fn default() -> Self {
        Self {
            wheel: Wheel { offset: Vec3::ZERO, radius: 0.25, width: 0.3, steer: false, drive: false },
            rb: RigidBody::Dynamic,
            collider: Collider::cylinder(0.25, 0.15),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
        }
    }
}

pub fn spawn_vehicle(
    commands: &mut Commands,
    pos: Vec3,
    tuning: &VehicleTuning,
) -> Entity {
    let vehicle = commands
        .spawn(VehicleBundle {
            transform: Transform::from_translation(pos),
            ..Default::default()
        })
        .id();

    let wheel_offsets = [
        Vec3::new(1.0, 0.0, 1.0),
        Vec3::new(-1.0, 0.0, 1.0),
        Vec3::new(1.0, 0.0, -1.0),
        Vec3::new(-1.0, 0.0, -1.0),
    ];

    for offset in wheel_offsets {
        let axle = commands
            .spawn((RigidBody::Dynamic, Transform::from_translation(pos + offset), GlobalTransform::default()))
            .id();
        commands.entity(vehicle).add_child(axle);

        let wheel = commands
            .spawn(WheelBundle {
                wheel: Wheel {
                    offset,
                    steer: offset.z > 0.0,
                    drive: true,
                    ..Default::default()
                },
                // Rotate the wheel so its axis aligns correctly with the
                // vehicle's forward motion. Without this the wheels appear
                // flattened against the ground.
                transform: Transform::from_translation(pos + offset)
                    .with_rotation(Quat::from_rotation_z(-FRAC_PI_2)),
                ..Default::default()
            })
            .id();

        commands.entity(vehicle).add_child(wheel);


        commands.spawn(
            PrismaticJoint::new(vehicle, axle)
                .with_local_anchor_1(offset)
                .with_free_axis(Vec3::Y)
                .with_limits(-tuning.suspension.max_travel, 0.0),
        );
        commands.spawn(
            RevoluteJoint::new(axle, wheel)
                .with_aligned_axis(Vec3::X),
        );
    }

    vehicle
}
