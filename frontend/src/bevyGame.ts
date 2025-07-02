import wasmUrl from '/wasm/game_demo.js?url'

export async function initBevyGame(): Promise<void> {
  // Vite serves the generated loader from the public directory. We first
  // resolve its URL at build time and then dynamically import it at runtime.
  const module = await import(/* @vite-ignore */ wasmUrl)

  const init = module.default || module.init
  if (typeof init === 'function') {
    await init()
  }
}
