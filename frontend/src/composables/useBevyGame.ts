import { onMounted } from 'vue'
import { useWasm } from '../wasm'

export function useBevyGame() {
  onMounted(async () => {
    const wasm = await useWasm()
    if (typeof (wasm as any).start === 'function') {
      ;(wasm as any).start()
    }
  })
}
