use bevy::prelude::*;
use bevy::render::view::{Layer, RenderLayers};
use bevy_svg::prelude::*;

use crate::{globals::GameParams, input::Player};

/// All HUD elements are drawn on this render layer.
pub const HUD_LAYER: u8 = 1;

/// Plugin that sets up the heads-up display.
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SvgPlugin)
            .add_systems(Startup, setup_hud)
            .add_systems(Update, update_speedometer);
    }
}

#[derive(Component)]
struct Speedometer;

fn setup_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 2D camera for the HUD overlay. Clear color is disabled so the 3d scene
    // remains visible.
    commands.spawn((
        Camera2d,
        Camera {
            order: 2,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        RenderLayers::layer(HUD_LAYER as Layer),
    ));

    let speedometer = asset_server.load("speedometer.svg");

    commands.spawn((
        Svg2d(speedometer),
        Origin::Custom((0.0, 0.5)),
        Transform::from_xyz(0.0, 0.0, 0.0),
        RenderLayers::layer(HUD_LAYER as Layer),
        Speedometer,
    ));
}

fn update_speedometer(
    params: Res<GameParams>,
    players: Query<&Player>,
    mut q: Query<&mut Transform, With<Speedometer>>,
) {
    let Ok(player) = players.get_single() else {
        return;
    };
    let mut speed_ratio = player.speed.abs() / params.max_speed.max(f32::EPSILON);
    if speed_ratio > 1.0 {
        speed_ratio = 1.0;
    }

    for mut tf in &mut q {
        tf.scale.x = speed_ratio;
    }
}
