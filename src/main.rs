use game_demo::camera::CameraPlugin;
use game_demo::globals::GameParams;
use game_demo::input::PlayerControlPlugin;
use game_demo::minimap::MiniMapPlugin;
use game_demo::world::WorldPlugin;
use game_demo::debug_ui::DebugUiPlugin;
use avian3d::prelude::*;
use bevy::prelude::*;


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
