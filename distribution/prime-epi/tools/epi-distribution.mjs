#!/usr/bin/env node
// Verify and install the Prime Epi distribution without any product's private
// tool root.
//
//   epi-distribution.mjs verify
//   epi-distribution.mjs seal          (re-pin the manifest's digests after an edit)
//   epi-distribution.mjs install --research-bin ABS --faculty-config ABS [--root DIR]
//
// `verify` recomputes every digest the manifest pins. `install` copies the
// binding extension and the shared QL owner extension (from ../pi-ql-native)
// into a content-addressed directory under the tools root and writes the
// binding file `prime-epi/current.json` that the
// extension and the `actuation-epi-prime` launcher both verify. It never writes
// under a `.workcell` path and refuses a root that is one.
import { createHash } from "node:crypto";
import { cpSync, existsSync, mkdirSync, readFileSync, readdirSync, realpathSync, renameSync,
  rmSync, statSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, isAbsolute, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const distribution = resolve(here, "..");
const piPackage = resolve(distribution, "../pi-ql-native");
const manifest = JSON.parse(readFileSync(join(distribution, "manifest.json"), "utf8"));
const sha = bytes => createHash("sha256").update(bytes).digest("hex");

function treeDigest(root) {
  const rows = [];
  const walk = dir => {
    for (const entry of readdirSync(dir, { withFileTypes: true }).sort((a, b) => a.name < b.name ? -1 : 1)) {
      const path = join(dir, entry.name);
      if (entry.isDirectory()) walk(path);
      else if (relative(root, path) !== "PROVENANCE.json") rows.push([relative(root, path), sha(readFileSync(path))]);
    }
  };
  walk(root);
  rows.sort((a, b) => a[0] < b[0] ? -1 : 1);
  return sha(rows.map(([name, digest]) => `${name} ${digest}`).join("\n"));
}

export function verify() {
  const problems = [];
  for (const [file, expected] of Object.entries(manifest.files)) {
    const path = join(distribution, file);
    if (!existsSync(path)) problems.push(`${file}: missing`);
    else if (sha(readFileSync(path)) !== expected.sha256) problems.push(`${file}: digest differs from the manifest`);
  }
  const tree = treeDigest(piPackage);
  if (tree !== manifest.shared_owner_extension.tree_sha256) problems.push("pi-ql-native tree digest differs from the manifest");
  const provenance = JSON.parse(readFileSync(join(piPackage, "PROVENANCE.json"), "utf8"));
  if (provenance.tree_sha256 !== tree) problems.push("pi-ql-native PROVENANCE.json tree digest differs from its files");
  return problems;
}

export function seal() {
  for (const file of Object.keys(manifest.files)) manifest.files[file] = { sha256: sha(readFileSync(join(distribution, file))) };
  manifest.shared_owner_extension.tree_sha256 = treeDigest(piPackage);
  writeFileSync(join(distribution, "manifest.json"), JSON.stringify(manifest, null, 1) + "\n");
}

function toolsRoot(explicit) {
  const root = explicit || process.env.EPI_LOGOS_TOOLS_ROOT
    || join(process.env.AIKIT_HOME || join(homedir(), ".aikit"), "tools");
  if (!isAbsolute(root)) throw new Error("the tools root must be an absolute path");
  if (root.split("/").includes(".workcell")) throw new Error("the tools root must not be a Workcell path");
  return root;
}

export function install({ researchBin, facultyConfig, root }) {
  const problems = verify();
  if (problems.length) throw new Error(`distribution does not verify: ${problems.join("; ")}`);
  for (const [name, path] of [["--research-bin", researchBin], ["--faculty-config", facultyConfig]]) {
    if (!path || !isAbsolute(path) || !existsSync(path) || !statSync(path).isFile()) {
      throw new Error(`${name} must be an absolute path to an existing file`);
    }
  }
  const base = join(toolsRoot(root), "prime-epi");
  const id = sha(JSON.stringify({ manifest: manifest.files, owner: manifest.shared_owner_extension.tree_sha256 })).slice(0, 16);
  const target = join(base, id);
  const staging = join(base, `.staging-${process.pid}`);
  rmSync(staging, { recursive: true, force: true });
  mkdirSync(join(staging, "extensions"), { recursive: true });
  cpSync(join(distribution, "extensions/ql-faculty-bindings.ts"), join(staging, "extensions/ql-faculty-bindings.ts"));
  const ownerSource = join(piPackage, "skills/ql-agent-reading");
  const owner = join(staging, "owner");
  for (const file of ["extensions/ql-agent.ts", "extensions/ql-event-context.ts",
    "ql-agent-contracts-v1.schema.json", "provenance.json", "package.json"]) {
    mkdirSync(dirname(join(owner, file)), { recursive: true });
    cpSync(join(ownerSource, file), join(owner, file));
  }
  rmSync(target, { recursive: true, force: true });
  renameSync(staging, target);
  const sourceFiles = {};
  for (const file of ["extensions/ql-agent.ts", "extensions/ql-event-context.ts",
    "ql-agent-contracts-v1.schema.json", "provenance.json", "package.json"]) {
    sourceFiles[file] = sha(readFileSync(join(target, "owner", file)));
  }
  const binding = {
    schema: "actuation.prime-faculty-installation/v1",
    binary: researchBin,
    configuration: facultyConfig,
    owner_extension: {
      root: join(target, "owner"),
      module: join(target, "owner/extensions/ql-agent.ts"),
      sha256: sourceFiles["extensions/ql-agent.ts"],
      source_files: sourceFiles,
    },
    distribution: { name: manifest.name, version: manifest.version, id,
      upstream_prime: manifest.upstream.release },
  };
  const current = join(base, "current.json");
  writeFileSync(`${current}.tmp`, JSON.stringify(binding, null, 2) + "\n");
  renameSync(`${current}.tmp`, current);
  return { installed: target, binding: current, extension: join(target, "extensions/ql-faculty-bindings.ts"), id };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [command, ...rest] = process.argv.slice(2);
  const option = name => { const i = rest.indexOf(name); return i >= 0 ? rest[i + 1] : undefined; };
  try {
    if (command === "verify") {
      const problems = verify();
      process.stdout.write(JSON.stringify({ ok: problems.length === 0, problems, files: Object.keys(manifest.files).length }) + "\n");
      process.exit(problems.length ? 1 : 0);
    } else if (command === "seal") {
      seal();
      process.stdout.write(JSON.stringify({ sealed: Object.keys(manifest.files).length }) + "\n");
    } else if (command === "install") {
      process.stdout.write(JSON.stringify(install({ researchBin: option("--research-bin"),
        facultyConfig: option("--faculty-config"), root: option("--root") }), null, 2) + "\n");
    } else {
      process.stderr.write("usage: epi-distribution.mjs verify | seal | install --research-bin ABS --faculty-config ABS [--root DIR]\n");
      process.exit(2);
    }
  } catch (error) {
    process.stderr.write(`epi-distribution: ${error.message}\n`);
    process.exit(1);
  }
}
