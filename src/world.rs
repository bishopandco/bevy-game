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
    // ── ground plane ──
    let ground_mesh = meshes.add(Plane3d::default().mesh().size(100.0, 100.0));
    commands
        .spawn(Mesh3d(ground_mesh))
        .insert(MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.8, 0.2),
            ..default()
        })))
        .insert(Transform::IDENTITY);

    // ── terrain glb ──
    let terrain: Handle<Scene> = asset_server.load("models/terrain.glb#Scene0");
    commands
        .spawn(SceneRoot(terrain))
        .insert(Transform::IDENTITY);

    // ── ambient light ──
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.5,
        affects_lightmapped_meshes: true,
    });

    // ── directional "sun" ──
    commands
        .spawn(DirectionalLight {
            illuminance: 3_000.0,
            shadows_enabled: true,
            ..default()
        })
        .insert(Transform::from_xyz(5.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y));

    // ── player cube ──
    let player_mesh = meshes.add(Cuboid::default());
    commands
        .spawn(Mesh3d(player_mesh))
        .insert(MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.8, 0.2),
            ..default()
        })))
        .insert(Transform::from_xyz(0.0, 1.0, 0.0))
        .insert(Player { speed: 0.0 });
}
