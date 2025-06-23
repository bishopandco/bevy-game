use bevy::prelude::*;
use bevy::prelude::{Cuboid, Plane3d};

use crate::input::Player;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_world);
    }
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(100.0, 100.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.8, 0.2),
            ..Default::default()
        })),
        Transform::IDENTITY,
    ));

    // ── terrain glb ──
    let terrain = asset_server.load("models/terrain.glb");
    commands.spawn((
        SceneRoot(terrain),
        Transform::IDENTITY,
    ));

    // ── ambient light ──
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.5,
        affects_lightmapped_meshes: true,
    });

    // ── directional "sun" ──
    commands.spawn((
        DirectionalLight {
            illuminance: 3_000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(5.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // ── player cube ──
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.8, 0.2),
            ..Default::default()
        })),
        Transform::from_xyz(0.0, 1.0, 0.0),
        Player { speed: 0.0 },
    ));
}