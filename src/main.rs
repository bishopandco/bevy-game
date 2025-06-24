use crate::camera::CameraPlugin;
use crate::globals::GameParams;
use crate::input::PlayerControlPlugin;
use crate::minimap::MiniMapPlugin;
use crate::world::WorldPlugin;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

mod camera;
mod globals;
mod input;
mod minimap;
mod world;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((RapierPhysicsPlugin::<NoUserData>::default(), RapierDebugRenderPlugin::default()))
        .add_systems(Startup, |mut commands: Commands, mut conf_q: Query<&mut RapierConfiguration>| {
            if let Ok(mut conf) = conf_q.get_single_mut() {
                // plugin already spawned one—just mutate it
                conf.gravity = Vec3::new(0.0, -9.81, 0.0);
            } else {
                // no config yet? spawn our own
                commands.spawn(RapierConfiguration {
                    gravity: Vec3::new(0.0, -9.81, 0.0),
                    physics_pipeline_active: true,
                    query_pipeline_active: true,
                    scaled_shape_subdivision: 0_u32,
                    force_update_from_transform_changes: true,
                });
            }
        })
        .insert_resource(GameParams::default())
        .add_plugins((WorldPlugin, PlayerControlPlugin, CameraPlugin, MiniMapPlugin))
        .run();
}
