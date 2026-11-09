//! Exposed mechanical visualization driven by the raycast suspension state.
//! These parts are meshes, not additional rigid bodies or constraints.

use bevy::{asset::RenderAssetUsages, mesh::{Indices, PrimitiveTopology}, prelude::*};
use bevy::ecs::hierarchy::ChildSpawnerCommands;

use crate::vehicle::Vehicle;
use crate::vehicle_systems::{SuspensionTuning, WHEEL_MOUNTS, WHEEL_NAMES, WHEEL_RADIUS};

#[derive(Clone, Copy)]
enum PartKind {
    Wheel,
    HalfShaft,
    UpperArmFront,
    UpperArmRear,
    LowerArmFront,
    LowerArmRear,
    Knuckle,
    DamperBody,
    DamperRod,
    Spring,
    UpperSeat,
    LowerSeat,
    CenterOfMass,
}

#[derive(Component)]
pub(crate) struct SuspensionPart {
    wheel: usize,
    kind: PartKind,
}

pub(crate) fn spawn_vehicle_parts(
    commands: &mut Commands,
    vehicle: Entity,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let steel = materials.add(StandardMaterial {
        base_color: Color::srgb(0.28, 0.32, 0.36),
        metallic: 0.8,
        perceptual_roughness: 0.35,
        ..default()
    });
    let alloy = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.75, 0.8),
        metallic: 0.85,
        perceptual_roughness: 0.22,
        ..default()
    });
    let black = materials.add(Color::srgb(0.035, 0.045, 0.055));
    let rubber = materials.add(StandardMaterial {
        base_color: Color::srgb(0.04, 0.04, 0.045),
        perceptual_roughness: 0.95,
        ..default()
    });
    let orange = materials.add(Color::srgb(1.0, 0.35, 0.06));
    let yellow = materials.add(Color::srgb(1.0, 0.8, 0.05));
    let blue = materials.add(Color::srgb(0.1, 0.45, 1.0));
    let marker_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 1.0, 0.9),
        unlit: true,
        ..default()
    });

    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let axle = meshes.add(Cylinder { radius: 0.035, half_height: 0.5 });
    let arm = meshes.add(Cylinder { radius: 0.025, half_height: 0.5 });
    let body = meshes.add(Cylinder { radius: 0.055, half_height: 0.5 });
    let rod = meshes.add(Cylinder { radius: 0.018, half_height: 0.5 });
    let seat = meshes.add(Cylinder { radius: 0.13, half_height: 0.5 });
    let coil = meshes.add(coil_mesh());
    let tire = meshes.add(Cylinder { radius: WHEEL_RADIUS, half_height: 0.15 });
    let hub = meshes.add(Cylinder { radius: 0.25, half_height: 0.17 });
    let differential = meshes.add(Sphere { radius: 0.16 });
    let marker = meshes.add(Sphere { radius: 0.07 });

    commands.entity(vehicle).with_children(|parent| {
        // Two open frame rails, three crossmembers, and a low ballast tray.
        for side in [-1.0, 1.0] {
            fixed_part(parent, "Frame rail", &cube, &steel,
                Transform::from_xyz(side * 0.58, -0.12, 0.0).with_scale(Vec3::new(0.12, 0.15, 3.3)));
        }
        for z in [-1.5, 0.0, 1.5] {
            fixed_part(parent, "Crossmember", &cube, &steel,
                Transform::from_xyz(0.0, -0.12, z).with_scale(Vec3::new(1.25, 0.12, 0.12)));
        }
        fixed_part(parent, "Low ballast tray", &cube, &black,
            Transform::from_xyz(0.0, -0.24, 0.0).with_scale(Vec3::new(0.85, 0.12, 1.8)));
        fixed_part(parent, "Center driveshaft", &axle, &alloy,
            segment_transform(Vec3::new(0.0, -0.18, -1.5), Vec3::new(0.0, -0.18, 1.5)));
        for z in [-1.5, 1.5] {
            fixed_part(parent, "Differential", &differential, &steel, Transform::from_xyz(0.0, -0.18, z));
            for side in [-1.0, 1.0] {
                for offset in [-0.2, 0.2] {
                    fixed_part(parent, "Shock tower brace", &axle, &steel,
                        segment_transform(Vec3::new(side * 0.58, -0.12, z + offset), Vec3::new(side * 0.6, 0.18, z)));
                }
            }
        }
        animated_part(parent, "Center of mass", 0, PartKind::CenterOfMass, &marker, &marker_material);

        for (index, name) in WHEEL_NAMES.iter().enumerate() {
            // A wheel's tire, metal hub, and visible spoke share its spin.
            parent.spawn((
                Name::new(format!("{name} wheel")),
                Transform::default(),
                Visibility::default(),
                SuspensionPart { wheel: index, kind: PartKind::Wheel },
            )).with_children(|wheel| {
                let rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
                fixed_part(wheel, "Tire", &tire, &rubber, Transform::from_rotation(rotation));
                fixed_part(wheel, "Hub", &hub, &alloy, Transform::from_rotation(rotation));
                for side in [-1.0, 1.0] {
                    fixed_part(wheel, "Wheel spoke", &cube, &orange,
                        Transform::from_xyz(side * 0.175, 0.0, 0.0)
                            .with_scale(Vec3::new(0.02, 0.4, 0.055)));
                }
            });
            for (label, kind, mesh, material) in [
                ("Half-shaft", PartKind::HalfShaft, &axle, &alloy),
                ("Upper front control arm", PartKind::UpperArmFront, &arm, &orange),
                ("Upper rear control arm", PartKind::UpperArmRear, &arm, &orange),
                ("Lower front control arm", PartKind::LowerArmFront, &arm, &orange),
                ("Lower rear control arm", PartKind::LowerArmRear, &arm, &orange),
                ("Steering knuckle", PartKind::Knuckle, &body, &steel),
                ("Damper body", PartKind::DamperBody, &body, &black),
                ("Damper piston", PartKind::DamperRod, &rod, &alloy),
                ("Coil spring", PartKind::Spring, &coil, if index < 2 { &yellow } else { &blue }),
                ("Upper spring seat", PartKind::UpperSeat, &seat, &alloy),
                ("Lower spring seat", PartKind::LowerSeat, &seat, &alloy),
            ] {
                animated_part(parent, &format!("{name} {label}"), index, kind, mesh, material);
            }
        }
    });
}

fn fixed_part(
    parent: &mut ChildSpawnerCommands,
    name: &str,
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    transform: Transform,
) {
    parent.spawn((Name::new(name.to_owned()), Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()), transform));
}

fn animated_part(
    parent: &mut ChildSpawnerCommands,
    name: &str,
    wheel: usize,
    kind: PartKind,
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
) {
    parent.spawn((
        Name::new(name.to_owned()), Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()),
        Transform::default(), SuspensionPart { wheel, kind },
    ));
}

/// Unit-height cylinder/coil aligned between two points in chassis-local space.
fn segment_transform(a: Vec3, b: Vec3) -> Transform {
    let delta = b - a;
    let length = delta.length().max(0.001);
    let rotation = if delta.length_squared() > 1.0e-6 {
        Quat::from_rotation_arc(Vec3::Y, delta / length)
    } else { Quat::IDENTITY };
    Transform::from_translation((a + b) * 0.5)
        .with_rotation(rotation)
        .with_scale(Vec3::new(1.0, length, 1.0))
}

pub(crate) fn update_vehicle_parts(
    vehicles: Query<&Vehicle>,
    tuning: Res<SuspensionTuning>,
    mut parts: Query<(&ChildOf, &SuspensionPart, &mut Transform)>,
) {
    for (parent, part, mut transform) in &mut parts {
        let Ok(vehicle) = vehicles.get(parent.parent()) else { continue; };
        if matches!(part.kind, PartKind::CenterOfMass) {
            *transform = Transform::from_xyz(0.0, tuning.center_of_mass_height, 0.0);
            continue;
        }
        let mount = WHEEL_MOUNTS[part.wheel];
        let side = mount.x.signum();
        let state = vehicle.wheels[part.wheel];
        let length = if state.length > 0.0 { state.length } else { tuning.wheels[part.wheel].rest_length };
        let center = mount - Vec3::Y * length;
        let steering = if part.wheel < 2 { vehicle.steering } else { 0.0 };
        let steer = Quat::from_rotation_y(steering);
        let inner_hub = center - steer * Vec3::X * side * 0.18;
        let upper_joint = inner_hub + Vec3::Y * 0.14;
        let lower_joint = inner_hub - Vec3::Y * 0.14;
        let shock_top = Vec3::new(side * 0.6, 0.18, mount.z);
        let shock_bottom = center + Vec3::new(-side * 0.18, 0.02, 0.0);
        let shock_axis = (shock_bottom - shock_top).normalize();
        let shock_length = shock_top.distance(shock_bottom);
        let body_end = shock_top + shock_axis * (shock_length * 0.55).min(0.32);
        let spring_top = shock_top + shock_axis * 0.055;
        let spring_bottom = shock_bottom - shock_axis * 0.055;
        *transform = match part.kind {
            PartKind::Wheel => Transform::from_translation(center)
                .with_rotation(steer * Quat::from_rotation_x(state.spin)),
            PartKind::HalfShaft => segment_transform(Vec3::new(0.0, -0.18, mount.z), inner_hub),
            PartKind::UpperArmFront => segment_transform(Vec3::new(side * 0.58, -0.04, mount.z + 0.22), upper_joint),
            PartKind::UpperArmRear => segment_transform(Vec3::new(side * 0.58, -0.04, mount.z - 0.22), upper_joint),
            PartKind::LowerArmFront => segment_transform(Vec3::new(side * 0.58, -0.2, mount.z + 0.3), lower_joint),
            PartKind::LowerArmRear => segment_transform(Vec3::new(side * 0.58, -0.2, mount.z - 0.3), lower_joint),
            PartKind::Knuckle => segment_transform(lower_joint, upper_joint),
            PartKind::DamperBody => segment_transform(shock_top, body_end),
            PartKind::DamperRod => segment_transform(body_end, shock_bottom),
            PartKind::Spring => segment_transform(spring_top, spring_bottom),
            PartKind::UpperSeat => segment_transform(spring_top - shock_axis * 0.012, spring_top + shock_axis * 0.012),
            PartKind::LowerSeat => segment_transform(spring_bottom - shock_axis * 0.012, spring_bottom + shock_axis * 0.012),
            PartKind::CenterOfMass => unreachable!(),
        };
    }
}

/// Six-turn tubular helix, centered on Y and one unit high. Stretching Y
/// changes coil spacing with suspension travel while preserving coil radius.
fn coil_mesh() -> Mesh {
    const SEGMENTS: u32 = 96;
    const SIDES: u32 = 6;
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    for segment in 0..=SEGMENTS {
        let t = segment as f32 / SEGMENTS as f32;
        let angle = t * 6.0 * std::f32::consts::TAU;
        let radial = Vec3::new(angle.cos(), 0.0, angle.sin());
        let tangent = Vec3::new(-angle.sin(), 1.0 / (0.1 * 6.0 * std::f32::consts::TAU), angle.cos()).normalize();
        let binormal = tangent.cross(radial).normalize();
        let center = radial * 0.1 + Vec3::Y * (t - 0.5);
        for side in 0..SIDES {
            let phi = side as f32 / SIDES as f32 * std::f32::consts::TAU;
            let normal = radial * phi.cos() + binormal * phi.sin();
            positions.push((center + normal * 0.012).to_array());
            normals.push(normal.to_array());
        }
    }
    for segment in 0..SEGMENTS {
        for side in 0..SIDES {
            let a = segment * SIDES + side;
            let b = segment * SIDES + (side + 1) % SIDES;
            let c = a + SIDES;
            let d = b + SIDES;
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(indices))
}
