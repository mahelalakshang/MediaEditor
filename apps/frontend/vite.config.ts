import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// In dev, proxy straight to the BFF (/api, /events) and to core-service's
// static file server (/media) — the same split nginx does in production,
// see docker-compose.yml. Keeping the split in dev too means the frontend
// code never has to know which backend actually serves what.
const bffUrl = process.env.VITE_BFF_URL ?? 'http://localhost:3001';
const coreUrl = process.env.VITE_CORE_URL ?? 'http://localhost:8080';

export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      '/api': { target: bffUrl, changeOrigin: true },
      '/events': { target: bffUrl, changeOrigin: true },
      '/media': { target: coreUrl, changeOrigin: true },
    },
  },
});
