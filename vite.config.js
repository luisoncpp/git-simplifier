import { defineConfig } from "vite";

export default defineConfig({
  root: "ui",
  server: {
    port: 5173,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**"] }
  },
  build: {
    outDir: "../dist",
    emptyOutDir: true
  }
});
