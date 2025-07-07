use bevy::prelude::*;

/// Selected drive train mode.
#[derive(Resource, Clone, Copy, PartialEq, Eq)]
pub enum DriveMode { Fwd, Rwd, Awd }

/// Commanded throttle, steering and handbrake state.
#[derive(Resource, Default)]
pub struct DriveCmd {
    pub throttle: f32,
    pub steer: f32,
    pub handbrake: bool,
}

/// Handles keyboard input and updates [`DriveCmd`].
pub struct VehicleControlsPlugin;

impl Plugin for VehicleControlsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DriveCmd::default())
            .insert_resource(DriveMode::Awd)
            .add_systems(Update, read_input);
    }
}

fn read_input(keys: Res<ButtonInput<KeyCode>>, mut cmd: ResMut<DriveCmd>) {
    let t = keys.pressed(KeyCode::ArrowUp) as i8 - keys.pressed(KeyCode::ArrowDown) as i8;
    let s = keys.pressed(KeyCode::ArrowRight) as i8 - keys.pressed(KeyCode::ArrowLeft) as i8;
    cmd.throttle = t as f32;
    cmd.steer = s as f32;
    cmd.handbrake = keys.pressed(KeyCode::Space);
}
