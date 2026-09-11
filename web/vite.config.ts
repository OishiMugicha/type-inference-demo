import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vite';

export default defineConfig({
  root: fileURLToPath(new URL('.', import.meta.url)),
  base: process.env.BASE_PATH || '/',
  worker: { format: 'es' },
  build: { assetsInlineLimit: 0 },
});
