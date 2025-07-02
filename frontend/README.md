# Vue 3 + TypeScript + Vite

This template should help get you started developing with Vue 3 and TypeScript in Vite. The template uses Vue 3 `<script setup>` SFCs, check out the [script setup docs](https://v3.vuejs.org/api/sfc-script-setup.html#sfc-script-setup) to learn more.

Learn more about the recommended Project Setup and IDE Support in the [Vue Docs TypeScript Guide](https://vuejs.org/guide/typescript/overview.html#project-setup).

## Running the game

After running `../../build.sh` to generate the wasm bindings, start the development server:

```bash
npm install
npm run dev
```

The build places the loader and wasm binary in `src/wasm/pkg` so Vite can bundle
them automatically.

