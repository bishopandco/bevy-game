use bevy::prelude::*;
use crate::globals::GameParams;
use crate::world::WorldPlugin;
use crate::input::PlayerControlPlugin;
use crate::camera::CameraPlugin;
use crate::minimap::MinimapPlugin;

mod globals;
mod world;
mod input;
mod camera;
mod minimap;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(GameParams::default())
        .add_plugins((WorldPlugin,PlayerControlPlugin, CameraPlugin, MinimapPlugin))
        .run();
}
