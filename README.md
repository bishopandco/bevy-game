# Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

# Compile and run
```bash
  cargo run --package game_demo --bin game_demo
```

## Build for the web
The `build.sh` script uses `wasm-pack` to compile the game and place the
generated bindings under `frontend/src/wasm/pkg`. These files are also copied
to `frontend/public/wasm` so the Vue dev server can load them directly.
Ensure that `wasm-pack` and `wasm-opt` are on your `PATH`.

```bash
./build.sh
```

The generated files are imported directly from the source tree so Vite can bundle
them automatically.

# Roadmap
- [x] Compile to WASM
- [ ] Add Tests
- [ ] Add Splash and Game UI
- [ ] Levels manager
