use bevy::prelude::*;

pub const PLAYER_HALF_EXTENTS: Vec3 = Vec3::new(5.0, 1.0, 10.0);

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
    pub cam_rot_lerp: f32,
    pub look_ahead: f32,
    pub mini_map_size: f32,
    pub mini_map_height: f32,
    pub brake_acceleration: f32,
    pub gravity: f32,
    pub yaw_rate: f32,
    pub fire_rate: f32,
    pub collision_damping: f32,
    pub slope_damping: f32,
    pub slope_ease: f32,
    pub bounce_factor: f32,
    pub socket_url: String,
}

impl Default for GameParams {
    fn default() -> Self {
        Self {
            max_speed: 40.0,
            acceleration: 10.0,
            brake_deceleration: 150.0,
            brake_acceleration: 15.0,
            friction: 0.7,
            rotation_speed: std::f32::consts::PI,
            cam_distance: 10.0,
            cam_height: 1.9,
            cam_lerp: 0.65,
            cam_rot_lerp: 0.45,
            look_ahead: 2.0,
            mini_map_size: 300.0,
            mini_map_height: 400.0,
            gravity: 9.81,
            yaw_rate: std::f32::consts::PI / 2.0,
            fire_rate: 50.0,
            collision_damping: 0.4,
            slope_damping: 0.2,
            slope_ease: 0.5,
            bounce_factor: 0.05,
            socket_url: "wss://535rf3b3kk.execute-api.us-east-1.amazonaws.com/$default".to_string(),
        }
    }
}
