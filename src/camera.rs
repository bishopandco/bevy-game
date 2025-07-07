use crate::globals::GameParams;
use crate::input::Player;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;

#[derive(Component, Default)]
pub struct FollowCamera;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (setup_camera, follow_camera_system));
    }
}

fn setup_camera(
    mut commands: Commands,
    player_q: Query<&GlobalTransform, With<Player>>,
    cam_q: Query<(), With<FollowCamera>>,
    params: Res<GameParams>,
) {
    if !cam_q.is_empty() {
        return;
    }

    if let Ok(player_tf) = player_q.single() {
        let player_tf = player_tf.compute_transform();
        let forward = player_tf.rotation * Vec3::Z;
        let cam_pos =
            player_tf.translation - forward * params.cam_distance + Vec3::Y * params.cam_height;
        let up = player_tf.rotation * Vec3::Y;
        commands
            .spawn(Camera3d::default())
            .insert(RenderLayers::layer(0))
            .insert(Transform::from_translation(cam_pos).looking_at(player_tf.translation, up))
            .insert(FollowCamera::default());
    }
}

fn follow_camera_system(
    params: Res<GameParams>,
    mut cam_q: Query<&mut Transform, With<FollowCamera>>,
    target_q: Query<&GlobalTransform, (With<Player>, Without<FollowCamera>)>,
) {
    let Ok(target_tf) = target_q.single() else {
        return;
    };

    let target_tf = target_tf.compute_transform();

    for mut cam_tf in &mut cam_q {
        let forward = target_tf.rotation * Vec3::Z;
        let target_pos =
            target_tf.translation - forward * params.cam_distance + Vec3::Y * params.cam_height;
        cam_tf.translation = cam_tf.translation.lerp(target_pos, params.cam_lerp);

        let look_at_pos = target_tf.translation + forward * params.look_ahead;
        let target_up = target_tf.rotation * Vec3::Y;
        let mut target_tf_cam = Transform::from_translation(cam_tf.translation);
        target_tf_cam.look_at(look_at_pos, target_up);
        cam_tf.rotation = cam_tf.rotation.slerp(target_tf_cam.rotation, params.cam_rot_lerp);
    }
}
