use bevy::prelude::*;
use crate::globals::GameParams;
use crate::world::WorldPlugin;
use crate::input::PlayerControlPlugin;
use crate::camera::CameraPlugin;
use crate::minimap::MiniMapPlugin;
use bevy_rapier3d::prelude::*;

mod globals;
mod world;
mod input;
mod camera;
mod minimap;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(
            (RapierPhysicsPlugin::<NoUserData>::default(), RapierDebugRenderPlugin::default())
        )
        .insert_resource(GameParams::default())
        .add_plugins((WorldPlugin, PlayerControlPlugin, CameraPlugin, MiniMapPlugin))
        .run();
}
