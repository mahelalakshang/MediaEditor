var _a, _b;
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
// In dev, proxy straight to the BFF (/api, /events) and to core-service's
// static file server (/media) — the same split nginx does in production,
// see docker-compose.yml. Keeping the split in dev too means the frontend
// code never has to know which backend actually serves what.
var bffUrl = (_a = process.env.VITE_BFF_URL) !== null && _a !== void 0 ? _a : 'http://localhost:3001';
var coreUrl = (_b = process.env.VITE_CORE_URL) !== null && _b !== void 0 ? _b : 'http://localhost:8080';
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
