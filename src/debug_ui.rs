use bevy::prelude::*;
use bevy::input::{keyboard::KeyCode, ButtonInput};
use bevy_egui::{egui, EguiContexts, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass};

use crate::globals::GameParams;
use crate::input::Player;
use crate::vehicle::Vehicle;
use crate::vehicle_systems::{SuspensionTuning, VehicleReset, WHEEL_NAMES};

#[derive(Message, Default, Debug)]
pub struct RespawnEvent;

pub struct DebugUiPlugin;

impl Plugin for DebugUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            // The HUD camera owns the primary context; never attach it to the minimap.
            .insert_resource(EguiGlobalSettings {
                auto_create_primary_context: false,
                ..default()
            })
            .insert_resource(DebugUiState::default())
            .add_message::<RespawnEvent>()
            .add_systems(EguiPrimaryContextPass, (debug_ui, vehicle_tuning_ui))
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
    mut respawn_writer: MessageWriter<RespawnEvent>,
    state: Res<DebugUiState>,
) -> Result {
    if !state.visible {
        return Ok(());
    }
    let ctx = ctxs.ctx_mut()?;
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
    Ok(())
}

fn handle_respawn(
    mut ev: MessageReader<RespawnEvent>,
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

fn vehicle_tuning_ui(
    mut ctxs: EguiContexts,
    mut tuning: ResMut<SuspensionTuning>,
    vehicles: Query<(Entity, &Vehicle)>,
    mut reset: MessageWriter<VehicleReset>,
    state: Res<DebugUiState>,
) -> Result {
    if !state.visible { return Ok(()); }
    egui::Window::new("Vehicle suspension")
        .default_pos(egui::pos2(420.0, 20.0))
        .default_width(350.0)
        .show(ctxs.ctx_mut()?, |ui| {
            ui.label("W/S: drive/reverse · A/D: steer · Space: brake · E: enter/exit");
            ui.horizontal(|ui| {
                if ui.button("Reset vehicle").clicked() { reset.write(VehicleReset); }
                if ui.button("Default tuning").clicked() { *tuning = SuspensionTuning::default(); }
            });
            ui.checkbox(&mut tuning.debug_draw, "Show suspension probes and loads");
            egui::ScrollArea::vertical().max_height(600.0).show(ui, |ui| {
                ui.collapsing("Drive and steering", |ui| {
                    ui.add(egui::Slider::new(&mut tuning.engine_force, 0.0..=20_000.0).text("Drive force (N)"));
                    ui.add(egui::Slider::new(&mut tuning.brake_force, 0.0..=30_000.0).text("Brake force (N)"));
                    ui.add(egui::Slider::new(&mut tuning.rolling_resistance, 0.0..=0.1).text("Rolling resistance"));
                    ui.add(egui::Slider::new(&mut tuning.max_speed, 1.0..=100.0).text("Drive cutoff (m/s)"));
                    ui.add(egui::Slider::new(&mut tuning.steering_angle, 0.0..=0.8).text("Steer limit (rad)"));
                    ui.add(egui::Slider::new(&mut tuning.steering_rate, 0.1..=5.0).text("Steer rate (rad/s)"));
                });
                if ui.button("Copy front-left settings to all wheels").clicked() {
                    let settings = tuning.wheels[0];
                    tuning.wheels.fill(settings);
                }
                for (index, name) in WHEEL_NAMES.iter().enumerate() {
                    ui.collapsing(*name, |ui| {
                        let wheel = &mut tuning.wheels[index];
                        ui.add(egui::Slider::new(&mut wheel.spring_rate, 5_000.0..=100_000.0).text("Spring (N/m)"));
                        ui.add(egui::Slider::new(&mut wheel.compression_damping, 0.0..=10_000.0).text("Bump damping (Ns/m)"));
                        ui.add(egui::Slider::new(&mut wheel.rebound_damping, 0.0..=10_000.0).text("Rebound damping (Ns/m)"));
                        ui.add(egui::Slider::new(&mut wheel.rest_length, 0.1..=0.8).text("Rest length (m)"));
                        let max_compression = wheel.rest_length - 0.02;
                        wheel.compression_travel = wheel.compression_travel.min(max_compression);
                        ui.add(egui::Slider::new(&mut wheel.compression_travel, 0.01..=max_compression).text("Bump travel (m)"));
                        ui.add(egui::Slider::new(&mut wheel.droop_travel, 0.0..=0.5).text("Droop travel (m)"));
                        ui.add(egui::Slider::new(&mut wheel.bump_stop_rate, 0.0..=300_000.0).text("Bump stop (N/m)"));
                        ui.add(egui::Slider::new(&mut wheel.max_force, 2_000.0..=40_000.0).text("Force limit (N)"));
                        ui.add(egui::Slider::new(&mut wheel.friction, 0.0..=2.5).text("Tire friction"));
                        ui.add(egui::Slider::new(&mut wheel.lateral_stiffness, 0.0..=10_000.0).text("Side grip (Ns/m)"));
                    });
                }
                for (entity, vehicle) in &vehicles {
                    ui.separator();
                    ui.label(format!("Vehicle {entity:?}: {:.1} m/s", vehicle.speed));
                    for (index, wheel) in vehicle.wheels.iter().enumerate() {
                        ui.label(format!("{}: {} · {:.3} m compression · {:.0} N load",
                            WHEEL_NAMES[index], if wheel.grounded { "contact" } else { "airborne" },
                            wheel.compression, wheel.load));
                    }
                }
            });
        });
    Ok(())
}
