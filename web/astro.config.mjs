import { defineConfig } from 'astro/config';
import svelte from '@astrojs/svelte';

export default defineConfig({
  base: '/ulang',
  trailingSlash: 'always',
  output: 'static',
  integrations: [svelte()],
});
