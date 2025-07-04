use bevy::prelude::*;
use avian3d::prelude::*;

#[derive(Resource, Clone, Copy)]
pub struct SuspensionParams {
    pub spring_k: f32,
    pub damping_c: f32,
    pub max_travel: f32,
}

impl Default for SuspensionParams {
    fn default() -> Self {
        Self { spring_k: 20.0, damping_c: 5.0, max_travel: 0.75 }
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

#[derive(Component)]
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
            collider: Collider::cuboid(1.0, 0.5, 1.5),
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
            wheel: Wheel { offset: Vec3::ZERO, radius: 0.75, width: 0.3, steer: false, drive: false },
            rb: RigidBody::Dynamic,
            collider: Collider::cylinder(0.75, 0.15),
            transform: Transform::from_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2)),
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
            .spawn((RigidBody::Dynamic, Transform::from_translation(pos + offset)))
            .id();

        let wheel = commands
            .spawn(WheelBundle {
                wheel: Wheel {
                    offset,
                    steer: offset.z > 0.0,
                    drive: true,
                    ..Default::default()
                },
                transform: Transform {
                    translation: pos + offset,
                    rotation: Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2),
                    ..default()
                },
                ..Default::default()
            })
            .id();

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
