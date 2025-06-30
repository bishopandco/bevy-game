use avian3d::prelude::*;
use bevy::prelude::*;
use game_demo::camera::CameraPlugin;
use game_demo::debug_ui::DebugUiPlugin;
use game_demo::hud::HudPlugin;
use game_demo::globals::GameParams;
use game_demo::input::PlayerControlPlugin;
use game_demo::weapons::WeaponPlugin;
use game_demo::minimap::MiniMapPlugin;
use game_demo::sky::SkyDomePlugin;
use game_demo::world::WorldPlugin;

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
            SkyDomePlugin,
            PlayerControlPlugin,
            WeaponPlugin,
            CameraPlugin,
            MiniMapPlugin,
            HudPlugin,
        ))
        .add_plugins(DebugUiPlugin)
        .run();
}
