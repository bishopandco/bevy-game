use bevy::prelude::*;
use bevy::render::camera::Viewport;
use crate::globals::GameParams;
use crate::input::Player;

#[derive(Component)]
struct MinimapCamera;

pub struct MinimapPlugin;
impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_minimap_camera)
            .add_system(minimap_follow_system);
    }
}

fn setup_minimap_camera(mut commands: Commands, windows: Res<Windows>, params: Res<GameParams>) {
    let window = windows.primary();
    let size = (window.width() * 0.2) as u32;
    let viewport = Viewport {
        physical_position: UVec2::new(window.physical_width() - size - 10, 10),
        physical_size: UVec2::new(size, size),
        ..Default::default()
    };
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                order: 1,
                viewport: Some(viewport),
                ..Default::default()
            },
            projection: OrthographicProjection {
                left: -params.mini_map_size,
                right: params.mini_map_size,
                bottom: -params.mini_map_size,
                top: params.mini_map_size,
                near: 0.1,
                far: 1000.0,
                ..Default::default()
            }.into(),

            transform: Transform::from_xyz(0.0, params.mini_map_height, 0.0)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            ..Default::default()
        },
        MinimapCamera,
    ));
}

fn minimap_follow_system(
    player_query: Query<&Transform, With<Player>>,
    mut mini_cam_query: Query<&mut Transform, With<MinimapCamera>>,
) {
    if let (Ok(player_tf), Ok(mut cam_tf)) = (player_query.get_single(), mini_cam_query.get_single_mut()) {
        cam_tf.translation.x = player_tf.translation.x;
        cam_tf.translation.z = player_tf.translation.z;
    }
}
