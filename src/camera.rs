use crate::input::Player;
use bevy::prelude::*;

#[derive(Component)]
struct FollowCamera {
    distance: f32,
    height: f32,
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (setup_camera, follow_camera_system));
    }
}

fn setup_camera(
    mut commands: Commands,
    player_q: Query<Entity, With<Player>>,
    cam_q: Query<(), With<FollowCamera>>,
) {
    if !cam_q.is_empty() {
        return;
    }

    if let Ok(_player) = player_q.single() {
        commands
            .spawn(Camera3d::default())
            .insert(Transform::from_xyz(0.0, 1.0, -10.0))
            .insert(FollowCamera {
                distance: 10.0,
                height: 1.0,
            });
    }
}

fn follow_camera_system(
    mut cam_q: Query<(&FollowCamera, &mut Transform)>,
    target_q: Query<&Transform, (With<Player>, Without<FollowCamera>)>,
) {
    let Ok(target_tf) = target_q.single() else {
        return;
    };

    for (follow, mut cam_tf) in &mut cam_q {
        let forward = target_tf.rotation * Vec3::Z;
        cam_tf.translation =
            target_tf.translation - forward * follow.distance + Vec3::Y * follow.height;

        cam_tf.look_at(target_tf.translation, Vec3::Y);
    }
}
