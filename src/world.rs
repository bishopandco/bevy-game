use bevy::prelude::*;

use crate::globals::GameParams;
use crate::input::Player;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_world);
    }
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // ── ground plane ──
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 100.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });

    // ── terrain glb ──
    let terrain = asset_server.load("models/terrain.glb");
    commands.spawn(SceneBundle {
        scene: terrain,
        transform: Transform::IDENTITY,
        ..Default::default()
    });

    // ── ambient light ──
    commands.insert_resource(AmbientLight {
        brightness: 0.5,
        color: Color::WHITE,
    });

    // ── directional “sun” ──
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 3_000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(5.0, 10.0, 5.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    // ── player cube ──
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.2, 0.2).into()),
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            ..Default::default()
        })
        .insert(Player { speed: 0.0 });
}
