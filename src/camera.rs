use bevy::prelude::*;
use bevy::core_pipeline::prelude::Camera3d;

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
    }
}

fn setup_camera(mut commands: Commands) {

    commands.spawn(
        Camera3d::default(),
    );
}