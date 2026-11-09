# Install Rust
Requires Rust 1.97.1 or newer for Bevy 0.20.1. The game uses Avian 0.8 and
bevy_egui 0.43, with resolved versions recorded in `Cargo.lock`.

The published `bevy_svg` release still targets Bevy 0.19. SVG rendering uses
[the upstream Bevy 0.20 port](https://github.com/Weasy666/bevy_svg/commit/d4a646e66aaefa1efc5e43a7a4d0b19281e3168e),
pinned to commit `d4a646e66aaefa1efc5e43a7a4d0b19281e3168e`. This is
unreleased dependency code; replace the Git pin when a compatible release is
published.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

# Compile and run
```bash
  cargo run --package game_demo --bin game_demo
```

## Controls

- **Player**
  - `ArrowUp`/`ArrowDown`: accelerate and brake
  - `ArrowLeft`/`ArrowRight`: steer
- **Vehicle**
  - `W`/`S`: drive forward or reverse
  - `A`/`D`: steer the front wheels
  - `Space`: brake all four wheels
  - `E`: enter or exit a nearby vehicle

## Terrain

The active world is a 2,048 m square of rolling hills, with a flat starting area.
Ground meshes and colliders share a 4 m grid, split into 128 m chunks.
The original track asset and spawn code are preserved. To restore the track,
change `WorldTerrain::RollingHills` to `WorldTerrain::Track` in `src/main.rs`.
Track mode also restores the start and finish markers; hills mode hides them.
Terrain selection applies at startup.

Hills mode draws a terrain-following grid every 20 m, with gold lines every
100 m. The X axis is red and the Z axis is blue. The grid fades within 200 m
of the driver. Four colored reference posts sit 100 m from the origin:
north cyan, south blue, east orange, and west red. Posts are visual only.
Low-angle sunlight and reduced ambient fill emphasize the slopes.

# Roadmap 
- [ ] Compile to WASM
- [ ] Add Tests
- [ ] Add Splash and Game UI
- [ ] Levels manager
Open the **Vehicle suspension** debug window for live per-wheel tuning and telemetry.
See [SUSPENSION_TUNING.md](SUSPENSION_TUNING.md) for parameters and model limits.
The vehicle uses an exposed procedural frame, axles, control arms, and coil-over
shocks instead of the placeholder car model. The cyan sphere marks the lowered
center of mass, which is adjustable in the suspension window.

## API and frontend dependencies

Use Node.js 22.12 or newer. The root and `frontend/` directories each have their
own `package.json` and `package-lock.json`; run `npm ci` in each directory to
install the recorded dependencies.

The frontend uses Vue 3.5, Vite 8, and vue-tsc 3.3. TypeScript is kept on the
latest 6.0 patch because vue-tsc uses the JavaScript compiler API that
[TypeScript 7 does not yet provide](https://devblogs.microsoft.com/typescript/announcing-typescript-7-0/).

SST is updated from v3 to v4. Before deploying an existing stage, follow the
[SST v4 migration guide](https://sst.dev/docs/migrate-from-v3/): review `sst diff`,
then migrate the full stage with `sst refresh` (no `--target`) before deploying.
Repeat for each stage and use `--dev` for stages previously deployed with
`sst dev`. This dependency update does not perform the infrastructure migration.
