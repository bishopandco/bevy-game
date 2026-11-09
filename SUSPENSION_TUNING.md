# Vehicle suspension

The vehicle is an 800 kg dynamic Avian chassis with a cuboid collider and
four independent raycast wheels. The GLTF and wheel meshes are visual only.
Avian integrates wheel forces and their torques at its fixed physics timestep.
No vehicle system writes the chassis transform during normal driving.

## Controls

- E: enter or exit a nearby vehicle.
- W / S: forward or reverse drive force.
- A / D: turn the front wheels relative to the chassis.
- Space: brake all four wheels. Player weapons do not fire while driving.
- Escape: toggle the debug windows.

Open **Vehicle suspension** to change tuning live. Expand each corner for
independent settings. **Copy front-left settings to all wheels** provides a
shared baseline; corners remain independently simulated. **Default tuning**
restores the defaults. **Reset vehicle** lifts and rights all vehicles in place,
clears their velocities, and retains tuning. Runtime tuning is not saved on exit;
change the defaults in src/vehicle_systems.rs to preserve a setup.

## Per-wheel settings

| Setting | Unit | Behavior |
| --- | --- | --- |
| Spring | N/m | Supporting force per metre of compression below rest length. |
| Bump damping | Ns/m | Resists suspension compression. |
| Rebound damping | Ns/m | Resists extension. |
| Rest length | m | Unloaded mount-to-wheel-center spring length. |
| Bump travel | m | Compression allowed below rest length before the bump stop acts. |
| Droop travel | m | Extension allowed above rest length. |
| Bump stop | N/m | Extra progressive support beyond the compression limit. |
| Force limit | N | Caps total spring, damping, and bump-stop force at this corner. |
| Tire friction | coefficient | Combined acceleration, braking, and cornering grip, relative to wheel load. |
| Side grip | Ns/m | Force opposing sideways contact velocity, within the friction limit. |

Default springs are 35,000 N/m per corner. With equal loading on level ground,
800 kg at 9.81 m/s² gives roughly 0.056 m of static compression per wheel.
Start with equal settings. Change one parameter at a time. Increase spring rate
for less sag; adjust bump and rebound damping to control oscillation. Excessive
stiffness or damping can still cause instability; defaults need in-game evaluation.
There is no artificial anti-roll torque or force linking the two sides.

## Drive and steering

Drive and brake force settings are totals, distributed equally across the four
wheels. Steering angle is the front wheel angle in radians; steering rate limits
how quickly it changes. Drive cutoff stops adding power above the selected speed;
it does not forcibly clamp velocity. Rolling resistance opposes forward/reverse
motion. Tire forces act in the road plane and share a load-limited friction circle.
Airborne wheels supply neither suspension support nor traction.

## Telemetry and limits

Each corner reports contact, compression, and normal load. Enable the debug drawing
to show green contact probes, red airborne probes, and yellow normal-load lines.
Wheel meshes follow measured suspension length and contact speed, rather than a
sine-wave animation.

This is a raycast vehicle model for static terrain. Wheels have no separate rigid
bodies, unsprung mass, rotational inertia, or detailed tire-slip model. A single
probe can miss narrow edges and does not model tire sidewall/curb contact. Moving
platform velocity and equal/opposite forces on dynamic ground are not modeled.
The body collider still handles chassis impacts and rollover contact. No runtime
or handling validation has been performed for this replacement.
