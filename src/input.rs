use bevy::prelude::*;
use crate::globals::GameParams;

#[derive(Component)]
pub struct Player {
    pub speed: f32,
}

pub struct PlayerControlPlugin;

impl Plugin for PlayerControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(player_movement_system);
    }
}


fn player_movement_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    params: Res<GameParams>,
    mut query: Query<(&mut Transform, &mut Player)>,
) {
    for (mut transform, mut player) in query.iter_mut() {
        let dt = time.delta_seconds();


        if keyboard.pressed(KeyCode::Left) || keyboard.pressed(KeyCode::A) {
            transform.rotate_y(params.rotation_speed * dt);
        }
        if keyboard.pressed(KeyCode::Right) || keyboard.pressed(KeyCode::D) {
            transform.rotate_y(-params.rotation_speed * dt);
        }

        if keyboard.pressed(KeyCode::Up) || keyboard.pressed(KeyCode::W) {
            player.speed -= params.acceleration * dt;
            if player.speed < -params.max_speed {
                player.speed = -params.max_speed;
            }
        } else if keyboard.pressed(KeyCode::Down) || keyboard.pressed(KeyCode::S) {
            player.speed += params.brake_deceleration * dt;
            if player.speed > params.max_speed {
                player.speed = params.max_speed;
            }
        } else {
            if player.speed > 0.0 {
                player.speed -= params.friction * dt;
                if player.speed < 0.0 {
                    player.speed = 0.0;
                }
            } else if player.speed < 0.0 {
                player.speed += params.friction * dt;
                if player.speed > 0.0 {
                    player.speed = 0.0;
                }
            }
        }

        if player.speed.abs() < 0.001 {
            player.speed = 0.0;
        }
        let direction = transform.rotation * Vec3::Z;
        transform.translation += direction * player.speed * dt;
    }
}
