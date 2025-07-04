use bevy::prelude::*;
use avian3d::prelude::*;
use game_demo::vehicle_plugin::VehiclePlugin;
use game_demo::vehicle::{VehicleBundle, Vehicle};

#[test]
fn vehicle_climbs_ramp() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, PhysicsPlugins::default()));
    app.insert_resource(Gravity(Vec3::new(0.0, -9.81, 0.0)));
    app.add_plugins(VehiclePlugin);

    app.world.spawn((
        Collider::cuboid(20.0, 1.0, 20.0),
        Transform::from_xyz(0.0, -1.0, 0.0),
        GlobalTransform::default(),
        RigidBody::Static,
    ));
    app.world.spawn((
        Collider::cuboid(10.0, 0.5, 2.0),
        Transform {
            translation: Vec3::new(5.0, 0.0, 0.0),
            rotation: Quat::from_rotation_z(-0.523599),
            ..default()
        },
        GlobalTransform::default(),
        RigidBody::Static,
    ));

    app.world.spawn(VehicleBundle::default());

    for _ in 0..120 {
        app.update();
    }

    let tf = app
        .world
        .query::<&Transform, With<Vehicle>>()
        .single(&app.world);
    assert!(tf.translation.z > 0.0);
}
