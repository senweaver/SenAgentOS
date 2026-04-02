import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "path";

export default defineConfig({
  base: "/",
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  build: {
    outDir: "dist",
  },
  server: {
    port: 5173,
    proxy: {
      "/api": {
        target: "http://localhost:42617",
        changeOrigin: true,
      },
      "/ws": {
        target: "ws://localhost:42617",
        ws: true,
        changeOrigin: true,
      },
      "/health": {
        target: "http://localhost:42617",
        changeOrigin: true,
      },
      "/pair": {
        target: "http://localhost:42617",
        changeOrigin: true,
      },
    },
  },
});
