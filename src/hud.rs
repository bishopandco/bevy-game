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
            .add_systems(Update, (update_speedometer, position_speedometer));
    }
}

#[derive(Component)]
struct Speedometer;

fn setup_hud(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    windows: Query<&Window>,
) {
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

    let window = windows.single();
    let win_size = window.resolution.physical_size();

    let speedometer = asset_server.load("speedometer.svg");

    commands.spawn((
        Svg2d(speedometer),
        Origin::Custom((0.0, 0.0)),
        Transform::from_xyz(
            -(win_size.x as f32) / 2.0 + 10.0,
            -(win_size.y as f32) / 2.0 + 10.0,
            0.0,
        ),
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

fn position_speedometer(windows: Query<&Window>, mut q: Query<&mut Transform, With<Speedometer>>) {
    let window = windows.single();
    let size = window.resolution.physical_size();
    for mut tf in &mut q {
        tf.translation.x = -(size.x as f32) / 2.0 + 10.0;
        tf.translation.y = -(size.y as f32) / 2.0 + 10.0;
    }
}
