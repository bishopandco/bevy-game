use bevy::prelude::*;
use bevy::render::camera::Viewport;
use bevy::render::view::RenderLayers;
use bevy::window::PrimaryWindow;
use crate::globals::GameParams;
use crate::input::Player;

#[derive(Component)]
struct MinimapCamera;

pub struct MiniMapPlugin;

impl Plugin for MiniMapPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_minimap_camera)
            .add_systems(Update, minimap_follow_system);
    }
}

fn setup_minimap_camera(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    params: Res<GameParams>,
) {
    let window = windows.single();
    let win_size = window.unwrap().resolution.physical_size();
    let mut cam = commands.spawn(Camera3d::default());

    let size = params.mini_map_size as u32;
    cam.insert(Camera {
        order: 1,
        viewport: Some(Viewport {
            physical_position: UVec2::new(win_size.x - size - 10, 10),
            physical_size: UVec2::splat(size),
            ..default()
        }),
        ..default()
    })
    .insert(
        Transform::from_xyz(0.0, params.mini_map_height, 0.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
    )
    .insert(MinimapCamera)
    .insert(RenderLayers::layer(0));
}

fn minimap_follow_system(
    player_q: Query<&Transform, (With<Player>, Without<MinimapCamera>)>,
    mut cam_q: Query<&mut Transform, With<MinimapCamera>>,
) {
    let Ok(player_tf) = player_q.single() else { return; };
    let Ok(mut cam_tf) = cam_q.single_mut() else { return; };
    cam_tf.translation.x = player_tf.translation.x;
    cam_tf.translation.z = player_tf.translation.z;
    cam_tf.look_at(player_tf.translation, Vec3::Y);
}
