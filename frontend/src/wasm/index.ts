import init, * as wasm from './pkg/game_demo.js'

let ready: Promise<typeof wasm> | null = null

export async function useWasm() {
  if (!ready) ready = init().then(() => wasm)
  return ready
}
