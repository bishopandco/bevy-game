use crate::camera::CameraPlugin;
use crate::globals::GameParams;
use crate::input::PlayerControlPlugin;
use crate::minimap::MiniMapPlugin;
use crate::world::WorldPlugin;
use crate::debug_ui::DebugUiPlugin;
use avian3d::prelude::*;
use bevy::prelude::*;

mod camera;
mod debug_ui;
mod globals;
mod input;
mod minimap;
mod world;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
        ))
        .insert_resource(Gravity(Vec3::new(0.0, -9.81, 0.0)))
        .insert_resource(GameParams::default())
        .add_plugins((
            WorldPlugin,
            PlayerControlPlugin,
            CameraPlugin,
            MiniMapPlugin,
        ))
        .add_plugins(DebugUiPlugin)
        .run();
}
