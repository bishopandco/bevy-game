use bevy::prelude::*;
use bevy::render::view::{Layer, RenderLayers};
use bevy_svg::prelude::*;

use crate::{hud::HUD_LAYER, input::Player};

/// Plugin that displays the player's weapon charge.
pub struct WeaponHudPlugin;

impl Plugin for WeaponHudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_weapon_hud)
            .add_systems(Update, (update_weapon_charge, position_weapon_charge));
    }
}

#[derive(Component)]
struct WeaponChargeMeter;

fn setup_weapon_hud(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    let win_size = window.unwrap().resolution.physical_size();

    let bar = asset_server.load("gradient.svg");

    commands.spawn((
        Svg2d(bar),
        Origin::Custom((0.0, 0.0)),
        Transform::from_xyz(
            -(win_size.x as f32) / 2.0 + 50.0,
            -(win_size.y as f32) / 2.0 + 60.0,
            0.0,
        ),
        RenderLayers::layer(HUD_LAYER as Layer),
        WeaponChargeMeter,
    ));
}

fn update_weapon_charge(
    players: Query<&Player>,
    mut q: Query<&mut Transform, With<WeaponChargeMeter>>,
) {
    let Ok(player) = players.single() else {
        return;
    };
    let mut ratio = player.weapon_energy.clamp(0.0, 1.0);
    if ratio > 1.0 {
        ratio = 1.0;
    }

    for mut tf in &mut q {
        tf.scale.x = ratio;
    }
}

fn position_weapon_charge(
    windows: Query<&Window>,
    mut q: Query<&mut Transform, With<WeaponChargeMeter>>,
) {
    let window = windows.single();
    let size = window.unwrap().resolution.physical_size();
    for mut tf in &mut q {
        tf.translation.x = -(size.x as f32 / 4.0) + 20.0;
        tf.translation.y = -(size.y as f32 / 4.0) + 150.0;
    }
}
