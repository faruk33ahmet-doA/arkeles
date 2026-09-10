import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "node:path";

// Tauri, webview'i sabit bir port ve host'ta bekler.
// Anayasa madde 13.4: dış istek yok — strictPort ile sürpriz port kayması engellenir.
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: { "@": path.resolve(__dirname, "src") },
  },
  // Tauri geliştirme sunucusu ayarları
  clearScreen: false,
  server: {
    host: host || "127.0.0.1",
    port: 1420,
    strictPort: true,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  // Anayasa madde 34.1: JS bundle bütçesi < 250 KB gzip.
  // Bütçe aşımını görünür kılmak için uyarı limiti düşük tutulur.
  build: {
    target: "es2022",
    chunkSizeWarningLimit: 250,
    sourcemap: false,
  },
});
