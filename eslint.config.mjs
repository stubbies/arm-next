import { defineConfig, globalIgnores } from "eslint/config";
import nextTs from "eslint-config-next/typescript";

export default defineConfig([
  ...nextTs,
  globalIgnores([
    "dist/**",
    "examples/**",
    "node_modules/**",
    "wasm/**",
    "native/target/**",
    ".next/**",
  ]),
  {
    files: ["src/**/*.ts", "tsup.config.ts"],
    rules: {
      "@typescript-eslint/no-require-imports": "off",
    },
  },
]);
