import { defineConfig, loadEnv } from 'vite';
import react from '@vitejs/plugin-react';
import { visualizer } from 'rollup-plugin-visualizer';

const host = process.env.TAURI_DEV_HOST;
const sharedPath = new URL('../../shared', import.meta.url).pathname;

export default defineConfig(async ({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '');
  return {
    base: mode === 'web' || mode === 'webos' ? env.VITE_BASE_PATH || '/' : '/',
    resolve: {
      alias: [
        ...(mode === 'web' || mode === 'webos'
          ? []
          : [{ find: /^.*\/platform\/web\/libass$/, replacement: new URL('./src/platform/web/libass.stub.ts', import.meta.url).pathname }]),
      ],
    },
    define: {
      'import.meta.env.VITE_FLUXA_TARGET': JSON.stringify(mode === 'web' || mode === 'webos' ? mode : 'desktop'),
      'import.meta.env.VITE_TRAKT_CLIENT_ID': JSON.stringify(env.VITE_TRAKT_CLIENT_ID || env.FLUXA_TRAKT_CLIENT_ID || ''),
      'import.meta.env.VITE_SIMKL_CLIENT_ID': JSON.stringify(env.VITE_SIMKL_CLIENT_ID || env.FLUXA_SIMKL_CLIENT_ID || ''),
      'import.meta.env.VITE_ANILIST_CLIENT_ID': JSON.stringify(env.VITE_ANILIST_CLIENT_ID || env.FLUXA_ANILIST_CLIENT_ID || ''),
      'import.meta.env.VITE_NUVIO_SUPABASE_URL': JSON.stringify(env.VITE_NUVIO_SUPABASE_URL || env.FLUXA_NUVIO_SUPABASE_URL || ''),
      'import.meta.env.VITE_NUVIO_SUPABASE_KEY': JSON.stringify(env.VITE_NUVIO_SUPABASE_KEY || env.FLUXA_NUVIO_SUPABASE_KEY || ''),
    },
    plugins: [
      react(),
      ...(process.env.ANALYZE_BUNDLE ? [visualizer({ filename: 'dist/bundle-stats.html', gzipSize: true, brotliSize: true })] : []),
    ],
    clearScreen: false,
    server: {
      fs: {
        allow: [sharedPath],
      },
      port: 1420,
      strictPort: true,
      host: host || false,
      hmr: host
        ? {
            protocol: 'ws',
            host,
            port: 1421,
          }
        : undefined,
      watch: {
        ignored: ['**/src-tauri/**'],
      },
    },
    envPrefix: ['VITE_', 'TAURI_ENV_*'],
    optimizeDeps: {
      entries: ['index.html'],
    },
    build: {
      outDir: mode === 'webos' ? 'dist-webos' : mode === 'web' ? 'dist-web' : 'dist',
      chunkSizeWarningLimit: 500,
      target: mode === 'webos' ? 'chrome87' : process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari15',
      minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false,
      sourcemap: !!process.env.TAURI_ENV_DEBUG,
    },
  };
});
