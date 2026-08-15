import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { visualizer } from "rollup-plugin-visualizer";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async ({ mode }) => ({
  base: mode === 'web' || mode === 'webos' ? process.env.VITE_BASE_PATH || '/' : '/',
  define: {
    'import.meta.env.VITE_FLUXA_TARGET': JSON.stringify(mode === 'web' || mode === 'webos' ? mode : 'desktop'),
  },
  plugins: [
    react(),
    ...(process.env.ANALYZE_BUNDLE
      ? [visualizer({ filename: "dist/bundle-stats.html", gzipSize: true, brotliSize: true })]
      : []),
  ],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  envPrefix: ["VITE_", "TAURI_ENV_*"],
  optimizeDeps: {
    entries: ["index.html"],
  },
  build: {
    outDir: mode === 'webos' ? 'dist-webos' : mode === 'web' ? 'dist-web' : 'dist',
    chunkSizeWarningLimit: 500,
    target:
      mode === 'webos'
        ? 'chrome87'
        : process.env.TAURI_ENV_PLATFORM === "windows"
        ? "chrome105"
        : "safari15",
    minify: !process.env.TAURI_ENV_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
}));
