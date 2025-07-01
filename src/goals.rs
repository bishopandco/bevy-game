use bevy::prelude::*;

#[derive(Component)]
pub struct StartGoal;

#[derive(Component)]
pub struct FinishGoal;

pub struct GoalsPlugin;

impl Plugin for GoalsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_goals);
    }
}

fn setup_goals(mut commands: Commands, asset_server: Res<AssetServer>) {
    let start_scene: Handle<Scene> = asset_server.load("models/start.glb#Scene0");
    commands
        .spawn(SceneRoot(start_scene))
        .insert(StartGoal)
        .insert(Transform::from_xyz(0.0, 0.0, 0.0))
        .insert(GlobalTransform::default());

    let finish_scene: Handle<Scene> = asset_server.load("models/finish.glb#Scene0");
    commands
        .spawn(SceneRoot(finish_scene))
        .insert(FinishGoal)
        .insert(Transform::from_xyz(0.0, 0.0, 50.0))
        .insert(GlobalTransform::default());
}
