import type { NextConfig } from "next";
import { createRequire } from "node:module";
import path from "node:path";
import { withArmNext } from "arm-next/next/with-arm-next";

const require = createRequire(import.meta.url);

const nextDir = path.dirname(require.resolve("next/package.json"));
const turbopackRoot = path.resolve(nextDir, "..", "..");

const nextConfig: NextConfig = {
  turbopack: {
    root: turbopackRoot,
  },
};

export default withArmNext(nextConfig);
