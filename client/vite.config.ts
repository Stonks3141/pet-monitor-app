import { sveltekit } from '@sveltejs/kit/vite';
import type { UserConfig } from 'vite';

const config: UserConfig = {
  plugins: [sveltekit()],
  server: {
    proxy: {
      '/api': 'http://127.0.0.1:8080',
      '^/stream.mp4$': 'http://127.0.0.1:8080',
    },
  },
};

export default config;
