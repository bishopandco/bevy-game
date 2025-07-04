use bevy::prelude::*;
use bevy::input::{keyboard::KeyCode, ButtonInput};
use bevy_egui::{egui, EguiContextPass, EguiContexts, EguiPlugin};

use crate::globals::GameParams;
use crate::input::Player;

#[derive(Event, Default, Debug)]
pub struct RespawnEvent;

pub struct DebugUiPlugin;

impl Plugin for DebugUiPlugin {
    fn build(&self, app: &mut App) {
        // Only add once in your whole app; drop this if EguiPlugin is in main.rs already.
        app.add_plugins(EguiPlugin::default())
            .insert_resource(DebugUiState::default())
            .add_event::<RespawnEvent>()
            .add_systems(EguiContextPass, debug_ui)
            .add_systems(Update, (handle_respawn, toggle_debug_ui));
    }
}

#[derive(Resource)]
pub struct DebugUiState {
    pub visible: bool,
}

impl Default for DebugUiState {
    fn default() -> Self {
        Self { visible: true }
    }
}

fn debug_ui(
    mut ctxs: EguiContexts,
    mut params: ResMut<GameParams>,
    players: Query<(&Player, &Transform)>,
    time: Res<Time>,
    mut respawn_writer: EventWriter<RespawnEvent>,
    state: Res<DebugUiState>,
) {
    if !state.visible {
        return;
    }
    let ctx = ctxs.ctx_mut();
    egui::Window::new("GameParams").show(ctx, |ui| {
        ui.label(format!("FPS: {:.0}", 1.0 / time.delta_secs()));
        if ui.button("Respawn").clicked() {
            respawn_writer.write(RespawnEvent);
        }
        macro_rules! slider {
            ($field:ident, $range:expr) => {
                ui.add(egui::Slider::new(&mut params.$field, $range).text(stringify!($field)));
            };
        }

        slider!(max_speed, 0.0..=150.0);
        slider!(acceleration, 0.0..=150.0);
        slider!(brake_deceleration, 0.0..=150.0);
        slider!(brake_acceleration, 0.0..=150.0);
        slider!(friction, 0.0..=50.0);
        slider!(rotation_speed, 0.0..=std::f32::consts::TAU);
        slider!(cam_distance, 0.0..=20.0);
        slider!(cam_height, 0.0..=20.0);
        slider!(cam_lerp, 0.0..=1.0);
        slider!(cam_rot_lerp, 0.0..=1.0);
        slider!(look_ahead, 0.0..=20.0);
        slider!(mini_map_size, 0.0..=100.0);
        slider!(mini_map_height, 0.0..=200.0);
        slider!(gravity, 0.0..=30.0);
        slider!(yaw_rate, 0.0..=std::f32::consts::TAU);
        slider!(fire_rate, 0.1..=20.0);
        slider!(collision_damping, 0.0..=1.0);
        slider!(slope_damping, 0.0..=1.0);
        slider!(slope_ease, 0.1..=2.0);
        slider!(bounce_factor, 0.0..=1.0);
    });

    egui::Window::new("Player Stats").show(ctx, |ui| {
        for (i, (p, tf)) in players.iter().enumerate() {
            ui.heading(format!("Player {i}"));
            ui.label(format!("speed        : {:>6.2}", p.speed));
            ui.label(format!("vertical_vel : {:>6.2}", p.vertical_vel));
            ui.label(format!("yaw (rad)    : {:>6.2}", p.yaw));
            ui.label(format!("weapon_energy: {:>6.2}", p.weapon_energy));
            ui.label(format!("pos          : {:.1?}", tf.translation));
            ui.separator();
        }
    });
}

fn handle_respawn(
    mut ev: EventReader<RespawnEvent>,
    mut players: Query<(&mut Transform, &mut Player)>,
) {
    if ev.is_empty() {
        return;
    }
    ev.clear();
    for (mut tf, mut plyr) in &mut players {
        tf.translation = Vec3::new(0.0, 3.0, 0.0);
        plyr.speed = 0.0;
        plyr.vertical_vel = 0.0;
        plyr.fire_timer = 0.0;
        plyr.weapon_energy = 1.0;
    }
}

fn toggle_debug_ui(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DebugUiState>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        state.visible = !state.visible;
    }
}
