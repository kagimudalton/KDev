import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import monaco from "@tomjs/vite-plugin-monaco-editor";

export default defineConfig({
  // Monaco is copied into the built app instead of being loaded from a CDN.
  // This is required for KDev's offline-first/Tauri desktop build.
  plugins: [react(), monaco({ local: true })],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: "localhost"
  },
  envPrefix: ["VITE_", "TAURI_"]
});
