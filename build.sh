cargo build --release --target wasm32-unknown-unknown           &&
wasm-bindgen ./target/wasm32-unknown-unknown/release/game_demo.wasm \
  --out-dir web --out-name game_demo --target web               &&
wasm-opt -Os -o web/game_demo_opt.wasm web/game_demo_bg.wasm