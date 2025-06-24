use bevy::prelude::*;

#[derive(Resource)]
pub struct GameParams {
    pub max_speed: f32,
    pub acceleration: f32,
    pub brake_deceleration: f32,
    pub friction: f32,
    pub rotation_speed: f32,
    pub cam_distance: f32,
    pub cam_height: f32,
    pub cam_lerp: f32,
    pub look_ahead: f32,
    pub mini_map_size: f32,
    pub mini_map_height: f32,
    pub brake_acceleration: f32,
    pub gravity: f32,
    pub yaw_rate: f32,
}

impl Default for GameParams {
    fn default() -> Self {
        Self {
            max_speed: 100.0,
            acceleration: 25.0,
            brake_deceleration: 10.0,
            friction: 4.0,
            rotation_speed: std::f32::consts::PI,
            cam_distance: 5.0,
            cam_height: 4.0,
            cam_lerp: 0.1,
            look_ahead: 6.0,
            mini_map_size: 40.0,
            mini_map_height: 120.0,
            brake_acceleration: 20.0,
            gravity: 9.81,
            yaw_rate: std::f32::consts::PI / 2.0, // 90 degrees per second
        }
    }
}
