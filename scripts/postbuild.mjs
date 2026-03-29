import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const pkgRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const projectRoot = process.cwd();

const extractBin =
  process.platform === "win32"
    ? join(pkgRoot, "native", "target", "release", "arm-next-extract.exe")
    : join(pkgRoot, "native", "target", "release", "arm-next-extract");

if (existsSync(extractBin)) {
  const input = join(projectRoot, ".next", "server", "app");
  const output = join(projectRoot, "public", "ai-md");
  const r = spawnSync(extractBin, [input, output], {
    stdio: "inherit",
    cwd: projectRoot,
  });
  if (r.status !== 0) {
    console.warn(
      "postbuild: arm-next-extract exited non-zero (skipping failure for CI without HTML output)",
    );
  }
} else {
  console.warn(
    "postbuild: arm-next-extract not built; in the arm-next package run `npm run build:extract` with Rust, or skip static extract.",
  );
}
