import { defineConfig } from "vitest/config";

export default defineConfig({
  build: {
    outDir: "dist",
    sourcemap: true,
    target: "es2022"
  },
  server: {
    port: 5173,
    strictPort: true
  },
  preview: {
    port: 4173,
    strictPort: true
  },
  test: {
    environment: "node",
    include: ["src/**/*.test.ts"]
  }
});
