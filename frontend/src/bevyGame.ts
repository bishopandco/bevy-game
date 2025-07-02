export async function initBevyGame(): Promise<void> {
  const module = await import('/wasm/game_demo.js');
  if (module.default) {
    await module.default();
  }
}
