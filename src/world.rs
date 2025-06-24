use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
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
    // ── terrain ──
    let terrain: Handle<Scene> = asset_server.load("models/terrain.glb#Scene0");

    commands
        .spawn(SceneRoot(terrain))
        .insert(Transform::from_xyz(0.0, 0.0, 0.0))
        .insert(GlobalTransform::default())
        .insert(RigidBody::Fixed)
        .insert(AsyncSceneCollider::default());

    // ── lighting ──
    commands.insert_resource(AmbientLight { brightness: 0.5, ..default() });
    commands
        .spawn(DirectionalLight { illuminance: 3_000.0, shadows_enabled: true, ..default() })
        .insert(Transform::from_xyz(5.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y));

    // ── player ──
    let mesh = meshes.add(Cuboid::default());
    commands
        .spawn(Mesh3d(mesh))
        .insert(MeshMaterial3d(materials.add(Color::srgb(0.2, 0.8, 0.2))))
        .insert(Transform::from_xyz(0.0, 3.0, 0.0))
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::cuboid(0.5, 0.5, 0.5))
        .insert(KinematicCharacterController::default())
        .insert(Player { speed: 0.0, vertical_vel: 0.0 });
}
