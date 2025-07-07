use bevy::prelude::*;

#[derive(Component)]
pub struct Chassis {
    pub mass: f32,
    pub com_offset: Vec3,
    pub pitch_roll_smoothing: f32,
}

#[derive(Component)]
pub struct Wheel {
    pub parent: Entity,
    pub local_pos: Vec3,
    pub radius: f32,
    pub rest_length: f32,
    pub spring_k: f32,
    pub damper_c: f32,
    pub anti_roll_group: u8,
}

#[derive(Component, Default)]
pub struct SuspensionState {
    pub compression: f32,
    pub compression_vel: f32,
    pub contact: bool,
}

#[derive(serde::Deserialize, bevy::asset::Asset, bevy::reflect::TypePath, Clone)]
pub struct VehicleConfig {
    pub chassis: ChassisConfig,
    pub wheels: Vec<WheelConfig>,
    pub anti_roll_stiffness: [f32; 2],
}

#[derive(serde::Deserialize, Clone)]
pub struct ChassisConfig {
    pub mass: f32,
    pub com_offset: [f32; 3],
    pub pitch_roll_smoothing: f32,
}

#[derive(serde::Deserialize, Clone)]
pub struct WheelConfig {
    pub local_pos: [f32; 3],
    pub radius: f32,
    pub rest_length: f32,
    pub spring_k: f32,
    pub damper_c: f32,
    pub anti_roll_group: u8,
}
