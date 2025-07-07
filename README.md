# Install Rust
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