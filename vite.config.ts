import { defineConfig } from "vitest/config";
import { fileURLToPath, URL } from "node:url";

export default defineConfig({
  root: "apps/demo",
  resolve: {
    alias: [
      {
        find: /^@gamestruments\/runtime$/,
        replacement: fileURLToPath(
          new URL("./packages/runtime/src/index.ts", import.meta.url),
        ),
      },
      {
        find: /^@strudel\/core$/,
        replacement: fileURLToPath(
          new URL("./packages/studio/strudel-core-shim.mjs", import.meta.url),
        ),
      },
    ],
  },
  build: {
    outDir: "../../dist/demo",
    emptyOutDir: true,
  },
  test: {
    root: fileURLToPath(new URL(".", import.meta.url)),
    include: ["tests/**/*.test.ts"],
    server: {
      deps: {
        inline: [/@strudel\/mini/],
      },
    },
  },
});
