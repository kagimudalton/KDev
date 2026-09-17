import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  // Monaco workers are imported directly by the application and bundled by
  // Vite. This avoids CDN loading and avoids the incompatible third-party
  // Monaco Vite plugin that previously blocked npm dependency resolution.
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: "localhost"
  },
  envPrefix: ["VITE_", "TAURI_"]
});
