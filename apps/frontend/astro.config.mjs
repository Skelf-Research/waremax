import { defineConfig } from 'astro/config';
import vue from '@astrojs/vue';
import tailwind from '@astrojs/tailwind';

export default defineConfig({
  site: 'https://waremax.skelfresearch.com',
  integrations: [
    vue(),
    tailwind({
      applyBaseStyles: false,
    }),
  ],
  output: 'static',
});
