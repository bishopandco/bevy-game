use bevy::prelude::*;
use avian3d::prelude::*;

use game_demo::input::{player_controller, Player};
use game_demo::globals::GameParams;

#[test]
fn player_falls_onto_ground() {
    // ----- build the app -----
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, PhysicsPlugins::default()));
    app.insert_resource(Gravity(Vec3::new(0.0, -9.81, 0.0)));
    app.insert_resource(GameParams::default());
    app.add_systems(Update, player_controller);

    // ----- set up the scene (borrow world only in this block) -----
    {
        let mut world = app.world_mut();

        // ground plane
        world.spawn((
            Collider::cuboid(5.0, 0.5, 5.0),
            Transform::from_xyz(0.0, -0.5, 0.0),
            GlobalTransform::default(),
            RigidBody::Static,
        ));

        // player above the ground
        world.spawn((
            Collider::cuboid(0.5, 0.5, 0.5),
            Transform::from_xyz(0.0, 5.0, 0.0),
            GlobalTransform::default(),
            RigidBody::Kinematic,      // or Dynamic if you want gravity to handle everything
            LinearVelocity::ZERO,       // start at rest
            Player {
                half_extents: Vec3::splat(0.5),
                ..Default::default()
            },
        ));
    } // <-- world borrow ends here

    // ----- simulate ~1 s @ 60 Hz -----
    for _ in 0..60 {
        app.update();
    }

    // ----- assertions -----
    {
        let mut world = app.world_mut();
        let (tf, player) = world
            .query::<(&Transform, &Player)>()
            .single(&mut world)
            .expect("exactly one Player in the world");

        assert!(
            tf.translation.y >= 0.0,
            "player fell through the ground: {}",
            tf.translation.y
        );
        assert!(
            player.grounded,
            "player should be grounded after falling"
        );
    } // world borrow drops here and all refs die with it
}
