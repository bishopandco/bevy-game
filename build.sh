#!/usr/bin/env bash
set -e

# Build the wasm package using wasm-pack
wasm-pack build --release --target web --out-dir frontend/src/wasm/pkg -- --no-default-features

# Optimize the resulting wasm if wasm-opt is available
if command -v wasm-opt >/dev/null 2>&1; then
  wasm-opt -Os -o frontend/src/wasm/pkg/game_demo_bg.wasm frontend/src/wasm/pkg/game_demo_bg.wasm
fi


