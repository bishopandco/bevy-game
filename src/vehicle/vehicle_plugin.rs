use bevy::prelude::*;

use super::{
    chassis::ChassisPlugin,
    controls::VehicleControlsPlugin,
    suspension::SuspensionPlugin,
    wheel::WheelPlugin,
};

/// Plugin grouping all vehicle related plugins.
pub struct VehiclePlugin;

impl Plugin for VehiclePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            VehicleControlsPlugin,
            ChassisPlugin,
            SuspensionPlugin,
            WheelPlugin,
        ));
    }
}
