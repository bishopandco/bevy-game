use bevy::prelude::*;
use avian3d::prelude::*;

use game_demo::input::{player_controller, Player};
use game_demo::globals::GameParams;

#[test]
fn player_falls_onto_ground() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, PhysicsPlugins::default()));
    app.insert_resource(Gravity(Vec3::new(0.0, -9.81, 0.0)));
    app.insert_resource(GameParams::default());
    app.add_systems(Update, player_controller);

    // simple ground collider
    app.world.spawn((
        Collider::cuboid(5.0, 0.5, 5.0),
        Transform::from_xyz(0.0, -0.5, 0.0),
        GlobalTransform::default(),
        RigidBody::Static,
    ));

    // player above the ground
    app.world.spawn((
        Collider::cuboid(0.5, 0.5, 0.5),
        Transform::from_xyz(0.0, 5.0, 0.0),
        GlobalTransform::default(),
        RigidBody::Kinematic,
        LinearVelocity::ZERO,
        Player {
            half_extents: Vec3::splat(0.5),
            ..Default::default()
        },
    ));

    // simulate several frames
    for _ in 0..60 {
        app.update();
    }

    let (tf, player) = app.world.query::<(&Transform, &Player)>().single(&app.world);
    assert!(tf.translation.y >= 0.0, "player fell through the ground: {}", tf.translation.y);
    assert!(player.grounded, "player should be grounded after falling");
}

