import { defineConfig } from 'vite';
import { minify } from 'html-minifier-terser';
import { copyFileSync, existsSync } from 'fs';
import { resolve } from 'path';

const plainScripts = ['astro-glue.js', 'data.js', 'components.js'];

export default defineConfig({
  root: 'web',
  publicDir: '../public',
  build: {
    outDir: '../dist',
    emptyOutDir: true,
    minify: true,
  },
  plugins: [
    {
      name: 'html-minify',
      apply: 'build',
      async transformIndexHtml(html) {
        return minify(html, {
          collapseWhitespace: true,
          removeComments: true,
          minifyCSS: true,
          minifyJS: true,
        });
      },
    },
    {
      name: 'copy-plain-scripts',
      apply: 'build',
      closeBundle() {
        const webDir = resolve(__dirname, 'web');
        const distDir = resolve(__dirname, 'dist');
        for (const file of plainScripts) {
          const src = resolve(webDir, file);
          if (existsSync(src)) {
            copyFileSync(src, resolve(distDir, file));
          }
        }
      },
    },
  ],
});
