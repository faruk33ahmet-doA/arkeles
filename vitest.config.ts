import { defineConfig } from "vitest/config";
import path from "node:path";

/*
 * Anayasa madde 39.4: "Saf mantık Vitest ile test edilir.
 * E2E test MVP'de yoktur — tek kişilik geliştirmede maliyeti faydasını aşar."
 *
 * Test koşum ortamı Sprint 0'da kurulur çünkü madde 39.1 gereği sonradan
 * kurulan test altyapısı teknik borçtur.
 */
export default defineConfig({
  resolve: {
    alias: { "@": path.resolve(__dirname, "src") },
  },
  test: {
    environment: "node",
    include: ["src/**/*.test.ts"],
  },
});
