import { defineConfig } from "tsup";

export default defineConfig({
  entry: {
    index: "src/index.ts",
    proxy: "src/proxy.ts",
    "routes/markdown": "src/routes/markdown.ts",
    "routes/markdown-edge": "src/routes/markdown-edge.ts",
    "next/with-arm-next": "src/next/with-arm-next.ts",
    cli: "src/cli.ts",
  },
  format: ["esm"],
  dts: true,
  splitting: false,
  sourcemap: true,
  clean: true,
  treeshake: true,
  external: ["next", "react", "react-dom", "arm-next/wasm"],
  outDir: "dist",
});
