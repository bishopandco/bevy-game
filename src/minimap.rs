use bevy::prelude::*;
use bevy::render::camera::Viewport;
use bevy::window::PrimaryWindow;          // <-- new
use crate::globals::GameParams;
use crate::input::Player;

#[derive(Component)]
struct MinimapCamera;

pub struct MinimapPlugin;

impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_minimap_camera)
            .add_systems(Update, minimap_follow_system);
    }
}

fn setup_minimap_camera(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,  // <-- was Res<Windows>
    params: Res<GameParams>,
) {
    let window = windows.single();                 // primary window only
    let win_size = window.resolution.physical_size();

    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                viewport: Some(Viewport {
                    physical_position: UVec2::new(
                        (win_size.x as f32 - params.mini_map_size) as u32 - 10,
                        10),
                    physical_size: UVec2::splat(params.mini_map_size as u32),
                    ..default()
                }),
                ..default()
            },
            transform: Transform::from_xyz(0.0, params.mini_map_height, 0.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        MinimapCamera,
    ));
}

fn minimap_follow_system(
    player_q: Query<&Transform, (With<Player>, Without<MinimapCamera>)>,
    mut cam_q: Query<&mut Transform, (With<MinimapCamera>, Without<Player>)>,
) {
    let Ok(player_tf) = player_q.get_single() else { return };
    let Ok(mut cam_tf) = cam_q.get_single_mut() else { return };
    cam_tf.translation.x = player_tf.translation.x;
    cam_tf.translation.z = player_tf.translation.z;
    cam_tf.look_at(player_tf.translation, Vec3::Y);
}