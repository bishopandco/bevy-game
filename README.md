# Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

# Compile and run
```bash
  cargo run --package game_demo --bin game_demo
```

## Build for the web
The `build.sh` script compiles the game to WebAssembly and copies the results
into the Vue application. The build requires the `wasm-bindgen-cli`,
`bindgen-cli`, and `wasm-opt` tools on your PATH. It produces `game_demo.js`
and `game_demo_bg.wasm` in `frontend/public/wasm` for Vite to load.

```bash
./build.sh
```

The generated files are placed in `frontend/public/wasm` so that they are served
by Vite.

# Roadmap
- [x] Compile to WASM
- [ ] Add Tests
- [ ] Add Splash and Game UI
- [ ] Levels manager
