export async function initBevyGame(): Promise<void> {
  // Use @vite-ignore so Vite doesn't try to process the public file
  const module = await import(/* @vite-ignore */ '/wasm/game_demo.js');
  if (module.default) {
    await module.default();
  }
}
