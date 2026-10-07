# Install Rust
Requires Rust 1.95 or newer for Bevy 0.19.1. The game uses compatible stable
releases of Avian 0.7, bevy_egui 0.42, and bevy_svg 0.19, with resolved versions
recorded in `Cargo.lock`.

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
  - `W`/`S`: accelerate and brake
  - `A`/`D`: steer
  - `E`: enter or exit a nearby vehicle

# Roadmap 
- [ ] Compile to WASM
- [ ] Add Tests
- [ ] Add Splash and Game UI
- [ ] Levels manager
See `SUSPENSION_TUNING.md` for suspension parameters.

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
