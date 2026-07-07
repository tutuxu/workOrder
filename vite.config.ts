import { defineConfig, loadEnv } from "vite";
import vue from "@vitejs/plugin-vue";
import {
  PREVIEW_PORT,
  TAURI_DEV_PORT,
  WEB_DEV_PORT,
} from "./config/ports";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "");
  const devPort = Number(env.VITE_DEV_PORT) || defaultDevPort(mode);
  const previewPort = Number(env.VITE_PREVIEW_PORT) || PREVIEW_PORT;
  const hmrPort = devPort + 1;

  return {
    plugins: [vue()],
    clearScreen: false,
    server: {
      port: devPort,
      strictPort: true,
      host: host || false,
      hmr: host
        ? {
            protocol: "ws",
            host,
            port: hmrPort,
          }
        : undefined,
      watch: {
        ignored: ["**/src-tauri/**"],
      },
    },
    preview: {
      port: previewPort,
      strictPort: true,
    },
    envPrefix: ["VITE_", "TAURI_"],
    build: {
      target:
        process.env.TAURI_ENV_PLATFORM === "windows" ? "chrome105" : "safari13",
      minify: !process.env.TAURI_ENV_DEBUG ? "esbuild" : false,
      sourcemap: !!process.env.TAURI_ENV_DEBUG,
    },
  };
});

function defaultDevPort(mode: string): number {
  if (mode === "tauri") return TAURI_DEV_PORT;
  return WEB_DEV_PORT;
}
