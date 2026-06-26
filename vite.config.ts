import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [vue()],

  // Tauri expects the dev server on a fixed port.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },

  // Vite options tailored for Tauri development and only activated in our build
  build: {
    target: "esnext",
    outDir: "dist",
    assetsDir: ".",
  },
}));
