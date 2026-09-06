#!/usr/bin/env bash
# Verify the native Actuation Skills against reality, not just structure.
# Layer 1 (structure): the Skill documents have the required shape.
# Layer 2 (truth): every file path, contract identifier and `actuation`
# command the Skills name still exists in the repository and in the served
# CLI surface. A Skill that names a renamed thing fails here, in the same
# change that renamed it — not at the next reader.
set -euo pipefail

operator="skills/actuation-operation/SKILL.md"
extension="skills/actuation-extension/SKILL.md"

for skill in "$operator" "$extension"; do
  test -f "$skill"
  head -n 1 "$skill" | grep -qx -- '---'
  grep -q '^name:' "$skill"
  grep -q '^description:' "$skill"
  grep -q '^## Contract metadata' "$skill"
  grep -q '^## .*Procedure' "$skill"
done

grep -q 'actuation:operator' "$operator"
grep -q 'Skill available != Capability granted' "$operator"
grep -q 'world mutation' "$operator"

grep -q 'actuation:extension-developer' "$extension"
grep -q 'native-owner review' "$extension"
grep -q 'Factory Claim / Run' "$extension"

node --input-type=module - <<'NODE'
import { readFileSync, readdirSync, existsSync } from "node:fs";
import { execFileSync } from "node:child_process";

const skills = ["skills/actuation-operation/SKILL.md", "skills/actuation-extension/SKILL.md"];
const failures = [];

// The served command surface, derived from the command table — never a copy.
const help = execFileSync(process.execPath, ["bin/actuation", "help"], { encoding: "utf8" });
const servedTokens = new Set(["help", "version", "--version"]);
for (const match of help.matchAll(/^  actuation ([a-z][a-z0-9-]*)/gm)) servedTokens.add(match[1]);

// The contract exports the Skills may lean on.
const contractExports = new Set();
const contractsUrl = new URL("contracts/", `file://${process.cwd()}/`);
for (const name of readdirSync("contracts")) {
  if (!name.endsWith(".mjs") || name.endsWith(".test.mjs")) continue;
  const module = await import(new URL(name, contractsUrl));
  for (const key of Object.keys(module)) contractExports.add(key);
}

for (const skill of skills) {
  const text = readFileSync(skill, "utf8");
  const spans = [...text.matchAll(/`([^`\n]+)`/g)].map((match) => match[1]);

  // Truth: a backticked repository path the Skill names must exist.
  for (const span of spans) {
    if (!/^(bin|cli|contracts|detection|docs|schemas|scripts|skills|experiments)\/[\w./-]+$/.test(span)) continue;
    if (!existsSync(span)) failures.push(`${skill}: names missing path \`${span}\``);
  }

  // Truth: a backticked camelCase identifier must be a contract export.
  for (const span of spans) {
    if (!/^[a-z][a-zA-Z0-9]{5,}$/.test(span) || !/[A-Z]/.test(span)) continue;
    if (!contractExports.has(span)) failures.push(`${skill}: names \`${span}\`, which no contracts/*.mjs exports`);
  }

  // Truth: a backticked `actuation <command>` invocation must be served.
  for (const span of spans) {
    for (const match of span.matchAll(/(?:^|\s)(?:\.\/)?(?:bin\/)?actuation ([a-z][a-z0-9-]*)/g)) {
      if (!servedTokens.has(match[1])) {
        failures.push(`${skill}: invokes \`actuation ${match[1]}\`, which the served surface does not declare`);
      }
    }
  }
}

if (failures.length) {
  console.error(failures.join("\n"));
  process.exit(1);
}
console.log("Actuation native Skills: structure and named surfaces verified against reality");
NODE
