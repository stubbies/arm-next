/**
 * wasm-pack always writes wasm/bundler/.gitignore with `*`. npm-packlist honors that
 * while packing and drops every file under wasm/bundler from the tarball.
 */
import { existsSync, unlinkSync } from "fs";
import { dirname, join } from "path";
import { fileURLToPath } from "url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const gitignorePath = join(root, "wasm/bundler/.gitignore");

if (existsSync(gitignorePath)) {
  unlinkSync(gitignorePath);
}
