export async function initBevyGame(): Promise<void> {
  const module = await import('../public/wasm/game_demo.js');
  if (module.default) {
    await module.default();
  }
}
