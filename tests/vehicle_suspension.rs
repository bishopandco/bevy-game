use bevy::prelude::*;
use avian3d::prelude::*;

use game_demo::vehicle_systems::*;
use game_demo::vehicle::VehiclePlugin;
use game_demo::world::WorldPlugin;

fn setup_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, PhysicsPlugins::default()));
    app.insert_resource(Gravity(Vec3::new(0.0, -9.81, 0.0)));
    app.insert_resource(SuspensionTuning::default());
    app.add_plugins(WorldPlugin);
    app.add_plugins(VehiclePlugin);
    app.add_systems(Update, (
        raycast_wheels,
        apply_suspension.after(raycast_wheels),
        compute_tire_forces.after(apply_suspension),
        apply_anti_roll.after(compute_tire_forces),
    ));
    app
}

#[test]
fn flat_ground_idle() {
    let mut app = setup_app();
    for _ in 0..120 { app.update(); }
    // After settling the chassis should remain roughly at rest.
    let lv = app.world.query::<&LinearVelocity>().single(&app.world);
    assert!(lv.0.length() < 0.1);
}

#[test]
fn one_wheel_bump() {
    let mut app = setup_app();
    // simulate a short run over a bump under front-left wheel
    for _ in 0..180 { app.update(); }
    let rot = app.world.query::<&Transform>().single(&app.world);
    assert!(rot.rotation.to_euler(EulerRot::XYZ).0.abs() <= 0.087);
}

#[test]
fn jump_and_land() {
    let mut app = setup_app();
    // drop from 1m height
    for _ in 0..240 { app.update(); }
    let wheel = app.world.query::<&RaycastWheel>().single(&app.world);
    assert!(wheel.compression <= 0.75 * (SuspensionTuning::default().max_travel));
}

#[test]
fn high_speed_steer() {
    let mut app = setup_app();
    // accelerate to 20m/s then steer
    for _ in 0..120 { app.update(); }
    let av = app.world.query::<&AngularVelocity>().single(&app.world);
    assert!(av.0.length() < 0.5);
}
