import { defineConfig } from 'vite'
export default defineConfig({
  server: {
    host: '0.0.0.0',
    proxy: {
      '/api': 'https://rocket:8080',
    },
  },
});