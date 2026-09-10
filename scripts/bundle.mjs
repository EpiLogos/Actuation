// Regenerate the self-contained `bin/actuation` from the source module tree.
//
// Actuation's install contract (`.oi/product.json` `artifact.entry` and O:I's
// `surfaces.json` `source_install`) declares a single-file entry with no build
// step. The source CLI is split across cli/, contracts/ and detection/ for
// development, but the shipped `bin/actuation` must be one self-contained file
// so a single-file copy (O:I `suite update` staging, `oi install`, the npm
// `bin`) can run it with no sibling modules present.
//
// Determinism: esbuild is pinned to 0.24.0, output is a plain ESM bundle.
// Re-run and `git diff --stat bin/actuation` to confirm the committed bin
// matches this source.
import { spawnSync } from "node:child_process";

const esbuild = [
  "npx",
  "--yes",
  "esbuild@0.24.0",
  "scripts/entry.mjs",
  "--bundle",
  "--platform=node",
  "--format=esm",
  "--target=node22",
  "--banner:js=#!/usr/bin/env node",
  "--outfile=bin/actuation",
  "--log-level=warning",
];

const result = spawnSync(esbuild[0], esbuild.slice(1), {
  stdio: "inherit",
  cwd: new URL("..", import.meta.url).pathname,
});
process.exit(result.status ?? 1);
