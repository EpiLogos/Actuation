import assert from "node:assert/strict";
import { mkdtemp, mkdir, readFile, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, resolve, join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

// Real Prime loader and installed owner material. No agent inference, fake
// extension API or substituted QL receipts are involved in these checks.
const [installationPath, outputPath] = process.argv.slice(2);
assert(installationPath && outputPath);
const installation = JSON.parse(await readFile(installationPath, "utf8"));
const extension = resolve(dirname(fileURLToPath(import.meta.url)), "../extensions/ql-faculty-bindings.ts");
const temporary = await mkdtemp(join(tmpdir(), "actuation-ql-bindings-"));
const checks = [];
const execute = (name, bindingPath, expected) => {
  const result = spawnSync(process.env.PRIME_AGENT_BIN || "prime-agent", [
    "--mode", "rpc", "--offline", "--no-session", "--no-context-files",
    "--no-themes", "--no-prompt-templates", "--no-skills", "--no-extensions", "-e", extension,
  ], { cwd: temporary, env: { ...process.env, ACTUATION_RESEARCH_INSTALLATION: bindingPath },
    input: '{"id":"bindings","type":"get_commands"}\n', encoding: "utf8",
    timeout: 15000, maxBuffer: 1024 * 1024 });
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stderr);
  const rows = result.stdout.split("\n").filter(Boolean).map(line => JSON.parse(line));
  const response = rows.find(row => row.id === "bindings" && row.type === "response");
  assert.equal(response?.success, true, "ordinary Prime entry must remain available");
  assert.equal(response.data.commands.some(command => command.name === "ql-mode"), expected);
  checks.push({ name, ordinary_entry: true, ql_extension_loaded: expected,
    exit_code: result.status, stderr: result.stderr });
};
try {
  execute("valid-installed-source", resolve(installationPath), true);
  execute("absent-installation", join(temporary, "absent.json"), false);
  for (const member of Object.keys(installation.owner_extension.source_files)) {
    const omitted = structuredClone(installation);
    delete omitted.owner_extension.source_files[member];
    const omissionPath = join(temporary, `omission-${checks.length}.json`);
    await writeFile(omissionPath, JSON.stringify(omitted));
    execute(`omitted-owner-member:${member}`, omissionPath, false);
    const directory = join(temporary, `drift-${checks.length}`);
    await mkdir(directory);
    const binding = structuredClone(installation);
    binding.owner_extension.root = directory;
    const entry = installation.owner_extension.module.slice(installation.owner_extension.root.length + 1);
    binding.owner_extension.module = resolve(directory, entry);
    for (const file of Object.keys(binding.owner_extension.source_files)) {
      const target = resolve(directory, file);
      await mkdir(dirname(target), { recursive: true });
      const bytes = await readFile(resolve(installation.owner_extension.root, file));
      await writeFile(target, bytes);
    }
    await mkdir(join(directory, "node_modules"));
    await symlink(resolve(installation.owner_extension.root, "node_modules/typebox"),
      join(directory, "node_modules/typebox"), "dir");
    const path = join(directory, "binding.json");
    await writeFile(path, JSON.stringify(binding));
    // The identical relocated bundle must load before the single changed
    // byte proves Source refusal. Missing runtime dependencies cannot pass.
    execute(`valid-relocated-owner:${member}`, path, true);
    const changed = resolve(directory, member);
    await writeFile(changed, Buffer.concat([await readFile(changed), Buffer.from("\n")]));
    execute(`changed-owner-member:${member}`, path, false);
  }
  const outside = structuredClone(installation);
  outside.owner_extension.source_files["../outside"] = "a".repeat(64);
  const outsidePath = join(temporary, "outside.json");
  await writeFile(outsidePath, JSON.stringify(outside));
  execute("source-path-escape", outsidePath, false);
  await writeFile(outputPath, JSON.stringify({ schema: "actuation.prime-native-bindings-proof/v1",
    installation: resolve(installationPath), extension, checks, model_calls: 0 }, null, 2) + "\n");
  process.stdout.write(JSON.stringify({ checks: checks.length, model_calls: 0 }) + "\n");
} finally {
  await rm(temporary, { recursive: true, force: true });
}
