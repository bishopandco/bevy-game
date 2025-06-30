use bevy::prelude::*;
use bevy::render::view::{Layer, RenderLayers};
use bevy_svg::prelude::*;

/// All HUD elements are drawn on this render layer.
pub const HUD_LAYER: u8 = 1;

/// Plugin that sets up the heads-up display.
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SvgPlugin).add_systems(Startup, setup_hud);
    }
}

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

    let mask = asset_server.load("mask.svg");
    let gradient = asset_server.load("gradient.svg");

    // Gradient underneath.
    commands.spawn((
        Svg2d(gradient),
        Origin::Center,
        Transform::from_xyz(0.0, 0.0, 0.0),
        RenderLayers::layer(HUD_LAYER as Layer),
    ));

    // Outline on top.
    commands.spawn((
        Svg2d(mask),
        Origin::Center,
        Transform::from_xyz(0.0, 0.0, 1.0),
        RenderLayers::layer(HUD_LAYER as Layer),
    ));
}
