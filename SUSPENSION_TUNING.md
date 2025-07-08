# Suspension Tuning Guide

The `SuspensionTuning` resource controls the behaviour of the vehicle suspension
system. Values are expressed in SI units.

| Field | Description |
|-------|-------------|
| `k` | Spring stiffness in newtons per metre. Higher values produce a stiffer ride. |
| `c` | Damping coefficient in newton‑seconds per metre. Tune to remove oscillations. |
| `mu_long` | Longitudinal tyre friction coefficient controlling acceleration and braking traction. |
| `mu_lat` | Lateral tyre friction coefficient controlling cornering grip. |
| `k_anti_roll` | Torque applied to resist body roll. Increase to keep the chassis level. |
| `rest_length` | Suspension rest length when uncompressed. |
| `max_travel` | Maximum extension beyond the rest length. |
| `gizmo` | When true, suspension rays and forces are drawn for debugging. |

Tweak these values at runtime by modifying the `SuspensionTuning` resource. The
anti–roll coefficient is applied per axle using the difference in wheel
compression. Excessive spring or damping can lead to numerical instability, so
values are clamped internally.
