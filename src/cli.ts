#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const pkgRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const postbuild = join(pkgRoot, "scripts", "postbuild.mjs");

const [cmd, ...args] = process.argv.slice(2);

if (cmd === "postbuild" || cmd === undefined) {
  const r = spawnSync(process.execPath, [postbuild, ...args], {
    stdio: "inherit",
    cwd: process.cwd(),
  });
  process.exit(r.status ?? 1);
} else {
  console.error(`arm-next: unknown command "${cmd}". Use: arm-next postbuild`);
  process.exit(1);
}
