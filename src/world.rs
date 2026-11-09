use crate::input::Player;
use crate::globals::Controlled;
use avian3d::prelude::{Collider, ColliderConstructor, ColliderConstructorHierarchy};
use avian3d::prelude::{DebugRender, LinearVelocity, RigidBody};
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
            .add_systems(Startup, setup_world);
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
        brightness: 1000.0,
        ..default()
    });
    // Low-angle sunlight makes hill contours readable while driving.
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.65, -0.4, 0.0)),
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
        perceptual_roughness: 0.95,
        ..default()
    });
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
                    let shade = (height / 36.0 + 0.5).clamp(0.0, 1.0);
                    colors.push([0.13 + shade * 0.12, 0.25 + shade * 0.15, 0.07 + shade * 0.07, 1.0]);
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
                // Avoid drawing half a million triangle edges every frame.
                DebugRender::none(),
            ));
        }
    }
}
