import { defineConfig } from "vite";
import { fileURLToPath, URL } from "node:url";
import react from "@vitejs/plugin-react";

export default defineConfig({
  // Monaco workers are imported directly by the application and bundled by
  // Vite. This avoids CDN loading and avoids the incompatible third-party
  // Monaco Vite plugin that previously blocked npm dependency resolution.
  plugins: [react()],
  resolve: {
    alias: {
      "monaco-editor/esm": fileURLToPath(new URL("./node_modules/monaco-editor/esm", import.meta.url))
    }
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: "localhost"
  },
  envPrefix: ["VITE_", "TAURI_"]
});
