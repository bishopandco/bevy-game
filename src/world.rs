use crate::input::Player;
use crate::globals::Controlled;
use avian3d::prelude::{Collider, ColliderConstructor, ColliderConstructorHierarchy};
use avian3d::prelude::{DebugRender, Friction, LinearVelocity, RigidBody};
use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};

/// Select before startup; track assets and spawning remain available for later.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub enum WorldTerrain {
    #[default]
    RollingHills,
    Track,
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldTerrain>()
            .add_systems(Startup, setup_world)
            .add_systems(Update, draw_terrain_grid);
    }
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    terrain_mode: Res<WorldTerrain>,
) {
    match *terrain_mode {
        WorldTerrain::RollingHills => spawn_rolling_hills(&mut commands, &mut meshes, &mut materials),
        WorldTerrain::Track => spawn_track(&mut commands, &asset_server),
    }

    commands.insert_resource(GlobalAmbientLight {
        brightness: if *terrain_mode == WorldTerrain::RollingHills { 150.0 } else { 1000.0 },
        ..default()
    });
    // Low-angle sunlight makes hill contours readable while driving.
    commands.spawn((
        DirectionalLight {
            illuminance: 20_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.4, -0.65, 0.0)),
    ));

    let mesh = meshes.add(Cuboid::new(0.25, 0.25, 0.25));
    commands
        .spawn(Mesh3d(mesh))
        .insert(MeshMaterial3d(materials.add(Color::srgb(0.2, 0.8, 0.2))))
        .insert(Transform::from_xyz(0.0, 3.0, 0.0))
        .insert(RigidBody::Kinematic)
        .insert(Collider::cuboid(0.25, 0.25, 0.25))
        .insert(LinearVelocity::ZERO)
        .insert(Player {
            speed: 0.0,
            vertical_vel: 0.0,
            yaw: 0.0,
            half_extents: Vec3::splat(0.25),
            grounded: false,
            fire_timer: 0.0,
            weapon_energy: 1.0,
        })
        .insert(Controlled);
}

/// Original track, kept intact and loaded only when Track mode is selected.
fn spawn_track(commands: &mut Commands, asset_server: &AssetServer) {
    let terrain: Handle<WorldAsset> = asset_server.load("models/terrain.glb#Scene0");
    commands
        .spawn(WorldAssetRoot(terrain))
        .insert(Transform::from_xyz(0.0, 0.0, 0.0))
        .insert(GlobalTransform::default())
        .insert(ColliderConstructorHierarchy::new(
            ColliderConstructor::TrimeshFromMesh,
        ))
        .insert(RigidBody::Static);
}

const TERRAIN_CHUNKS: u32 = 16;
const CHUNK_SIZE: f32 = 128.0;
const CHUNK_CELLS: u32 = 32;
const GRID_SPACING: f32 = CHUNK_SIZE / CHUNK_CELLS as f32;

/// Smooth deterministic hills, with a level pad around the starting positions.
fn terrain_height(x: f32, z: f32) -> f32 {
    let hills = 10.0 * (x * 0.012).sin() * (z * 0.009).cos()
        + 5.0 * ((x + z) * 0.022).sin()
        + 3.0 * ((x - z) * 0.04).cos();
    let t = ((Vec2::new(x, z).length() - 20.0) / 40.0).clamp(0.0, 1.0);
    hills * t * t * (3.0 - 2.0 * t)
}

fn spawn_rolling_hills(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.92,
        ..default()
    });
    spawn_navigation_markers(commands, meshes, materials);
    let half_size = TERRAIN_CHUNKS as f32 * CHUNK_SIZE * 0.5;
    for chunk_z in 0..TERRAIN_CHUNKS {
        for chunk_x in 0..TERRAIN_CHUNKS {
            let origin = Vec3::new(
                chunk_x as f32 * CHUNK_SIZE - half_size,
                0.0,
                chunk_z as f32 * CHUNK_SIZE - half_size,
            );
            let mut vertices = Vec::new();
            let mut normals = Vec::new();
            let mut colors = Vec::new();
            for z in 0..=CHUNK_CELLS {
                for x in 0..=CHUNK_CELLS {
                    let local_x = x as f32 * GRID_SPACING;
                    let local_z = z as f32 * GRID_SPACING;
                    let world_x = origin.x + local_x;
                    let world_z = origin.z + local_z;
                    let height = terrain_height(world_x, world_z);
                    vertices.push(Vec3::new(local_x, height, local_z));
                    // World-space gradients keep lighting continuous at chunk seams.
                    let dx = terrain_height(world_x + 0.5, world_z) - terrain_height(world_x - 0.5, world_z);
                    let dz = terrain_height(world_x, world_z + 0.5) - terrain_height(world_x, world_z - 0.5);
                    normals.push(Vec3::new(-dx, 1.0, -dz).normalize().to_array());
                    // Subtle concrete mottling in world coordinates keeps chunk seams invisible.
                    let shade = 0.44
                        + 0.025 * (world_x * 0.71).sin() * (world_z * 0.63).sin()
                        + 0.015 * (height / 18.0).clamp(-1.0, 1.0);
                    colors.push([shade, shade * 1.01, shade * 1.02, 1.0]);
                }
            }
            let mut triangles = Vec::new();
            let row = CHUNK_CELLS + 1;
            for z in 0..CHUNK_CELLS {
                for x in 0..CHUNK_CELLS {
                    let a = z * row + x;
                    // Counterclockwise when viewed from above: normals face up.
                    triangles.push([a, a + row, a + 1]);
                    triangles.push([a + 1, a + row, a + row + 1]);
                }
            }
            // Render and collision use exactly the same grid and triangulation.
            let positions: Vec<[f32; 3]> = vertices.iter().map(|v| v.to_array()).collect();
            let indices = triangles.iter().flatten().copied().collect();
            let collider = Collider::trimesh(vertices, triangles);
            let mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
                .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
                .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
                .with_inserted_indices(Indices::U32(indices));
            commands.spawn((
                Name::new(format!("Rolling hills {chunk_x},{chunk_z}")),
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(origin),
                RigidBody::Static,
                collider,
                Friction::new(1.3).with_static_coefficient(1.5),
                // Avoid drawing half a million triangle edges every frame.
                DebugRender::none(),
            ));
        }
    }
}

/// Interpolate the actual terrain triangles rather than floating above/below
/// their analytic height function between the four-metre mesh samples.
fn terrain_surface_height(x: f32, z: f32) -> f32 {
    let cell_x = (x / GRID_SPACING).floor() * GRID_SPACING;
    let cell_z = (z / GRID_SPACING).floor() * GRID_SPACING;
    let u = (x - cell_x) / GRID_SPACING;
    let v = (z - cell_z) / GRID_SPACING;
    let h00 = terrain_height(cell_x, cell_z);
    let h10 = terrain_height(cell_x + GRID_SPACING, cell_z);
    let h01 = terrain_height(cell_x, cell_z + GRID_SPACING);
    let h11 = terrain_height(cell_x + GRID_SPACING, cell_z + GRID_SPACING);
    if u + v <= 1.0 {
        h00 + (h10 - h00) * u + (h01 - h00) * v
    } else {
        h11 + (h01 - h11) * (1.0 - u) + (h10 - h11) * (1.0 - v)
    }
}

fn draw_terrain_grid(
    terrain: Res<WorldTerrain>,
    controlled: Query<&Transform, With<Controlled>>,
    mut gizmos: Gizmos,
) {
    if *terrain != WorldTerrain::RollingHills { return; }
    let Ok(transform) = controlled.single() else { return; };
    const LINE_SPACING: f32 = 20.0;
    const RADIUS: f32 = 200.0;
    let center = Vec2::new(transform.translation.x, transform.translation.z);
    let anchor = (center / LINE_SPACING).floor() * LINE_SPACING;
    let bound = TERRAIN_CHUNKS as f32 * CHUNK_SIZE * 0.5;
    // World-anchored coordinates remain fixed as the visible grid follows the driver.
    for direction in 0..2 {
        for line in -10..=10 {
            let fixed = (if direction == 0 { anchor.x } else { anchor.y })
                + line as f32 * LINE_SPACING;
            if fixed.abs() > bound { continue; }
            let major = (fixed.round() as i32).rem_euclid(100) == 0;
            let rgb = if fixed.abs() < 0.1 {
                if direction == 0 { [0.2, 0.7, 1.0] } else { [1.0, 0.3, 0.2] }
            } else if major { [0.95, 0.8, 0.4] } else { [0.15, 0.18, 0.2] };
            let along = if direction == 0 { anchor.y } else { anchor.x };
            for segment in 0..100 {
                let start = along - RADIUS + segment as f32 * GRID_SPACING;
                let end = start + GRID_SPACING;
                if start < -bound || end > bound { continue; }
                let a = if direction == 0 { Vec2::new(fixed, start) } else { Vec2::new(start, fixed) };
                let b = if direction == 0 { Vec2::new(fixed, end) } else { Vec2::new(end, fixed) };
                let distance = ((a + b) * 0.5 - center).length();
                let fade = (1.0 - distance / RADIUS).clamp(0.0, 1.0);
                if fade <= 0.0 { continue; }
                let alpha = fade * if major { 0.9 } else { 0.6 };
                gizmos.line(
                    Vec3::new(a.x, terrain_surface_height(a.x, a.y) + 0.06, a.y),
                    Vec3::new(b.x, terrain_surface_height(b.x, b.y) + 0.06, b.y),
                    Color::srgba(rgb[0], rgb[1], rgb[2], alpha),
                );
            }
        }
    }
}

fn spawn_navigation_markers(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let post = meshes.add(Cuboid::new(0.6, 8.0, 0.6));
    let cap = meshes.add(Cuboid::new(1.5, 0.5, 1.5));
    let dark = materials.add(Color::srgb(0.08, 0.1, 0.12));
    for (name, x, z, color) in [
        ("North 100 m", 0.0, 100.0, Color::srgb(0.1, 0.8, 1.0)),
        ("South 100 m", 0.0, -100.0, Color::srgb(0.2, 0.4, 1.0)),
        ("East 100 m", 100.0, 0.0, Color::srgb(1.0, 0.5, 0.1)),
        ("West 100 m", -100.0, 0.0, Color::srgb(1.0, 0.15, 0.15)),
    ] {
        let ground = terrain_surface_height(x, z);
        let beacon = materials.add(StandardMaterial {
            base_color: color,
            unlit: true,
            ..default()
        });
        // Visual reference posts have no collision and do not obstruct driving.
        commands.spawn((
            Name::new(name),
            Mesh3d(post.clone()),
            MeshMaterial3d(dark.clone()),
            Transform::from_xyz(x, ground + 4.0, z),
        ));
        commands.spawn((
            Mesh3d(cap.clone()),
            MeshMaterial3d(beacon),
            Transform::from_xyz(x, ground + 8.0, z),
        ));
    }
}
