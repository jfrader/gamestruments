import { fileURLToPath, URL } from "node:url";
import { defineConfig } from "vite";

export default defineConfig({
  resolve: {
    alias: [
      {
        find: /^@strudel\/core$/,
        replacement: fileURLToPath(
          new URL("./packages/studio/strudel-core-shim.mjs", import.meta.url),
        ),
      },
    ],
  },
  build: {
    lib: {
      entry: {
        index: fileURLToPath(
          new URL("./packages/studio/src/index.ts", import.meta.url),
        ),
        manifest: fileURLToPath(
          new URL(
            "./packages/studio/src/generation-manifest.ts",
            import.meta.url,
          ),
        ),
        cli: fileURLToPath(
          new URL("./packages/studio/src/generate-cli.ts", import.meta.url),
        ),
      },
      formats: ["es"],
      fileName: (_format, entryName) => `${entryName}.js`,
    },
    outDir: "packages/studio/dist",
    emptyOutDir: true,
    rollupOptions: {
      external: [/^node:/, /^@gamestruments\/runtime(?:\/.*)?$/],
    },
  },
});
