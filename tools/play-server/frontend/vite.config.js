import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

/**
 * M11-local Session 6 (`memory/m11-session-plan.md` §4, items 1-2).
 *
 * `$viewer` points at the replay viewer's component library **in place**. The
 * components there are props-based precisely so a second app can import them
 * without a copy (`docs/mtg-engine-replay-viewer.md` §"Import Mechanism"), and a
 * copy would fork on the next `StackObjectKind` variant. Promotion to a shared
 * `tools/ui-shared/` package is deferred — plan §8 R8.
 *
 * The alias resolves through `node:url` rather than a bare relative string so it
 * is absolute at resolve time; a relative alias target is resolved against the
 * *importing* file, which would break for imports from `src/lib/`.
 */
export default defineConfig({
  plugins: [svelte()],
  resolve: {
    alias: {
      $viewer: fileURLToPath(
        new URL('../../replay-viewer/frontend/src/lib', import.meta.url),
      ),
    },
  },
  build: {
    // tools/play-server/dist/ — the directory `build_router` mounts a ServeDir
    // on when it exists (`tools/play-server/src/main.rs`).
    outDir: '../dist',
    emptyOutDir: true,
  },
  server: {
    proxy: {
      // The play server's default bind address (`--host 127.0.0.1 --port 3040`).
      // 3040 and not 3030: the replay viewer owns 3030 and the two run side by
      // side.
      '/api': {
        target: 'http://127.0.0.1:3040',
        changeOrigin: true,
      },
    },
  },
})
