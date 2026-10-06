import assert from "node:assert/strict";
import { appendFileSync, chmodSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import { checkFacultyCoherence, install, verify, writeFacultyConfig } from "../tools/epi-distribution.mjs";

// The distribution installed under a root that is not any product's tool root,
// loaded by the real Prime loader the way the launcher loads it (explicit -e
// with --no-extensions). No model call, fake extension API or substituted QL
// receipt is involved. The research binary and faculty configuration named in
// the binding are placeholders here: the binding extension records them in the
// environment and never executes them.
const here = dirname(fileURLToPath(import.meta.url));
const manifest = resolve(here, "../manifest.json");
const temporary = mkdtempSync(join(tmpdir(), "prime-epi-"));
const checks = [];
const REVISION = "6a81fc441e4dda477f4de3a7ebd59c368cb28f37", STALE = "8eff719b235703c65911f2461e4a7dd5872a32ad";
const instrument = join(temporary, "owner-instrument");
writeFileSync(instrument, `#!/bin/sh\ncat >/dev/null\nprintf '{"error":"operation is required","owner_revision":"${REVISION}","schema":"actuation.ql-owner-operation/v1"}'\n`);
chmodSync(instrument, 0o755);
const facultyFile = (name, owner) => { const file = join(temporary, name); writeFileSync(file, JSON.stringify({ schema: "actuation.prime-faculty/v1", owner: { process: { program: instrument, args: [], cwd: temporary, environment: {}, timeout_ms: 5000, output_limit: 1048576 }, ...owner }, harmonic_enabled: false })); return file; };
const facultyConfig = facultyFile("faculty.json", { revision: REVISION });

const commandsFor = (extension, environment) => {
  const result = spawnSync(process.env.PRIME_AGENT_BIN || "prime-agent", [
    "--mode", "rpc", "--offline", "--no-session", "--no-context-files", "--no-themes",
    "--no-prompt-templates", "--no-skills", "--no-extensions", "-e", extension,
  ], { cwd: temporary, input: '{"id":"c","type":"get_commands"}\n', encoding: "utf8", timeout: 20000,
    env: { ...process.env, ACTUATION_RESEARCH_INSTALLATION: "", ...environment } });
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stderr);
  const row = result.stdout.split("\n").filter(Boolean).map(line => JSON.parse(line)).find(r => r.id === "c");
  assert.equal(row?.success, true, "ordinary Prime entry must remain available");
  return row.data.commands.map(command => command.name).filter(name => name.startsWith("ql-")).sort();
};
const record = (name, extra) => checks.push({ name, ...extra });

try {
  assert.deepEqual(verify(), [], "the manifest's digests hold");
  record("manifest-verifies", {});

  const root = join(temporary, "tools");
  const installed = install({ researchBin: process.execPath, facultyConfig, root });
  const binding = readFileSync(installed.binding, "utf8");
  assert(!binding.includes(".workcell"), "no Workcell path in the binding");
  assert(installed.installed.startsWith(root), "installed under the chosen root");
  assert.deepEqual(installed.faculty, { probed: true, revision: REVISION, program: instrument });
  record("faculty-coherence-probed", { revision: REVISION });
  const environment = { EPI_LOGOS_TOOLS_ROOT: root };
  const bindingFile = JSON.parse(binding);

  // Loaded through the default location: no ACTUATION_RESEARCH_INSTALLATION.
  const loaded = commandsFor(installed.extension, environment);
  assert.deepEqual(loaded, ["ql-mode", "ql-status"]);
  record("installed-root-loads", { commands: loaded, binding: installed.binding });

  // The previous behaviour is gone: an absent binding is no longer a silent
  // success. Prime hides the throw, so the proof here is that nothing loads;
  // the launcher's own refusal is tested in Rust.
  const empty = join(temporary, "empty-root");
  assert.deepEqual(commandsFor(installed.extension, { EPI_LOGOS_TOOLS_ROOT: empty }), []);
  record("absent-binding-loads-nothing", {});

  // No node_modules is installed beside the owner extension, and none is needed:
  // Prime supplies `typebox` to extensions itself. (The previous install linked
  // Pi's copy into the owner root; that link was never required.)
  assert.equal(readdirSync(bindingFile.owner_extension.root).includes("node_modules"), false);
  record("loads-with-no-node_modules", {});

  // One changed byte in the owner source is refused by the closure check.
  const again = install({ researchBin: process.execPath, facultyConfig, root: join(temporary, "tools2") });
  appendFileSync(join(JSON.parse(readFileSync(again.binding, "utf8")).owner_extension.root, "extensions/ql-agent.ts"), "\n");
  assert.deepEqual(commandsFor(again.extension, { EPI_LOGOS_TOOLS_ROOT: join(temporary, "tools2") }), []);
  record("changed-owner-source-loads-nothing", {});

  // A Workcell tool root is refused outright.
  assert.throws(() => install({ researchBin: process.execPath, facultyConfig,
    root: join(temporary, ".workcell/tools") }), /Workcell/);
  record("workcell-root-refused", {});

  // A configuration pinning another revision than the instrument reports is refused at install, naming both
  // (it would otherwise file no native faculty receipt: "owner response correlation/revision mismatch").
  assert.throws(() => install({ researchBin: process.execPath, facultyConfig: facultyFile("faculty.stale.json", { revision: STALE }), root: join(temporary, "tools3") }),
    new RegExp(`pins owner revision ${STALE} but ${instrument.replace(/[/.]/g, "\\$&")} reports ${REVISION}`));
  assert.throws(() => checkFacultyCoherence(manifest), /schema/);
  assert.throws(() => checkFacultyCoherence(facultyFile("faculty.norev.json", { revision: "x" })), /40-hex/);
  assert.deepEqual(checkFacultyCoherence(facultyFile("faculty.cli.json", { revision: STALE, native_cli: true })).probed, false);
  record("incoherent-faculty-refused", {});

  // The generator pins the configuration to what the instrument reports, so a rebuilt instrument is followed, not contradicted.
  const generated = writeFacultyConfig({ instrument, sourceRoot: temporary, evidenceRoot: join(temporary, "evidence"), out: join(temporary, "generated", "faculty.json") });
  assert.equal(generated.revision, REVISION);
  assert.equal(JSON.parse(readFileSync(generated.out, "utf8")).owner.revision, REVISION);
  assert.equal(generated.coherence.probed, true);
  record("faculty-config-generated-coherent", { revision: REVISION });

  process.stdout.write(JSON.stringify({ schema: "actuation.prime-epi-install-proof/v1", checks, model_calls: 0 }, null, 1) + "\n");
} finally {
  rmSync(temporary, { recursive: true, force: true });
}
