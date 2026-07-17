import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  server: {
    // Keep this paired with src-tauri/tauri.conf.json's build.devUrl.
    // 1420 is Tauri's common default and is frequently claimed by another
    // desktop project during development.
    port: 1430,
    strictPort: true,
    watch: {
      // Local embedding downloads live under src-tauri during development.
      // They must not trigger frontend hot reloads while a model initializes.
      ignored: ["**/src-tauri/.fastembed_cache/**"],
    },
  },
});
