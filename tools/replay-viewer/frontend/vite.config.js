import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

// https://vite.dev/config/
export default defineConfig({
  plugins: [svelte()],
  build: {
    // Build output goes to tools/replay-viewer/dist/ (served by axum)
    outDir: '../dist',
    emptyOutDir: true,
  },
  server: {
    proxy: {
      // Forward all /api/* requests to the axum backend during development
      '/api': {
        target: 'http://localhost:3030',
        changeOrigin: true,
      },
    },
  },
})
