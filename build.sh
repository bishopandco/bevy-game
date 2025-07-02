#!/usr/bin/env bash
set -e

# Build the wasm package using wasm-pack
# The output goes under frontend/src/wasm/pkg so Vite can bundle it.
# We also copy the generated files into frontend/public/wasm for
# direct loading in the browser when running the dev server.
OUT_DIR="frontend/src/wasm/pkg"
PUBLIC_DIR="frontend/public/wasm"

wasm-pack build --release --target web --out-name game_demo \
  --out-dir "$OUT_DIR" -- --no-default-features

mkdir -p "$PUBLIC_DIR"
cp "$OUT_DIR"/game_demo.js "$OUT_DIR"/game_demo_bg.wasm "$PUBLIC_DIR"/

# Optimize the resulting wasm if wasm-opt is available
if command -v wasm-opt >/dev/null 2>&1; then
  wasm-opt -Os -o "$OUT_DIR/game_demo_bg.wasm" "$OUT_DIR/game_demo_bg.wasm"
  cp "$OUT_DIR/game_demo_bg.wasm" "$PUBLIC_DIR/game_demo_bg.wasm"
fi


