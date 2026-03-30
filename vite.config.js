import { defineConfig } from 'vite';

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [],
  root: 'src',
  build: {
    outDir: '../dist',
    emptyOutDir: true,
  },
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
}));
