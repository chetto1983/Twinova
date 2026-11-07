import { paraglideVitePlugin } from "@inlang/paraglide-js";
import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Tauri expects a fixed port and must see Rust compile errors in the terminal.
export default defineConfig({
  plugins: [
    paraglideVitePlugin({
      project: "./project.inlang",
      outdir: "./src/paraglide",
      emitTsDeclarations: true,
      // As in chetto1983/ModbusTool: virtual modules loaded two runtimes on Windows, physical ones load one.
      outputStructure: "message-modules",
    }),
    svelte(),
  ],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  // three.js alone is ~600 kB; a desktop app loads it from disk, so the web-oriented 500 kB warning does not apply.
  build: { target: "es2022", chunkSizeWarningLimit: 800 },
  test: {
    include: ["src/**/*.test.ts"],
    environment: "happy-dom",
    coverage: {
      include: ["src/lib/**/*.ts"],
      exclude: ["src/lib/**/*.test.ts"],
      // CLAUDE.md coverage floor.
      thresholds: { lines: 85, statements: 85, functions: 85, branches: 85 },
    },
  },
});
