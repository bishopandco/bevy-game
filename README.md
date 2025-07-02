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
into the Vue application.

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
