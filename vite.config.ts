import { defineConfig } from 'vite';
import { minify } from 'html-minifier-terser';

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
  ],
});
