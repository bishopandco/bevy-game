use bevy::prelude::*;
use bevy::render::camera::Viewport;
use bevy::render::view::RenderLayers;
use crate::globals::GameParams;

pub struct MiniMapPlugin;

impl Plugin for MiniMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_minimap_camera);
    }
}

fn setup_minimap_camera(
    mut commands: Commands,
    windows: Query<&Window>,
    params: Res<GameParams>,
) {
    let window = windows.single();
    let win_size = window.unwrap().resolution.physical_size();
    let mut cam = commands.spawn(Camera3d::default());

    cam.insert(Camera {
        order: 1,
        viewport: Some(Viewport {
            physical_position: UVec2::new(10, 10),
            physical_size: UVec2::new(win_size.x / 5, win_size.y / 5),
            ..default()
        }),
        ..default()
    })
        .insert(
            Transform::from_xyz(0.0, params.cam_height * 2.0, 0.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
        )
        .insert(RenderLayers::layer(0));
}