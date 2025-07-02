#!/usr/bin/env bash
set -e

# Compile the game for the wasm32 target
cargo build --release --target wasm32-unknown-unknown

# Generate JS bindings for the web target
wasm-bindgen ./target/wasm32-unknown-unknown/release/game_demo.wasm \
  --out-dir web --out-name game_demo --target web

# Optimize the resulting wasm
wasm-opt -Os -o web/game_demo_opt.wasm web/game_demo_bg.wasm

# Copy the build output into the frontend so it can be served by Vite
mkdir -p frontend/public/wasm
cp web/game_demo.js frontend/public/wasm/
cp web/game_demo_opt.wasm frontend/public/wasm/

