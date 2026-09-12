#!/usr/bin/env bash
# Verify the native Actuation Skills against reality, not just structure.
# Layer 1 (structure): the Skill documents have the required shape.
# Layer 2 (truth): every file path, native contract identifier and `actuation`
# command the Skills name still exists in the repository and in the served
# executable. A Skill that names a renamed thing fails here, in the same
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
grep -q 'actuation agency actualise' "$operator"
grep -qi 'cannot manufacture authority' "$operator"

grep -q 'actuation:extension-developer' "$extension"
grep -q 'native-owner review' "$extension"
grep -q 'Factory Claim / Run' "$extension"

# Build the served executable so the truth layer reads the real surface.
cargo build --locked -q -p actuation-cli
served="target/debug/actuation"

node --input-type=module - <<'NODE'
import { readFileSync, readdirSync, existsSync } from "node:fs";
import { execFileSync } from "node:child_process";

const skills = ["skills/actuation-operation/SKILL.md", "skills/actuation-extension/SKILL.md"];
const failures = [];

// The served command surface, derived from the command table — never a copy.
const help = execFileSync("target/debug/actuation", ["help"], { encoding: "utf8" });
const servedTokens = new Set(["help", "version", "--version"]);
for (const match of help.matchAll(/^  actuation ([a-z][a-z0-9-]*)/gm)) servedTokens.add(match[1]);

// The native contract symbols the Skills may lean on, collected from the
// crates' public API surface — the Rust continuation of the contract exports.
const symbolFiles = [];
const walk = (dir) => {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    if (entry.name === "target" || entry.name.startsWith(".")) continue;
    const path = `${dir}/${entry.name}`;
    if (entry.isDirectory()) walk(path);
    else if (entry.name.endsWith(".rs")) symbolFiles.push(path);
  }
};
walk("crates");
const corpus = symbolFiles.map((path) => readFileSync(path, "utf8")).join("\n");
const nativeSymbols = new Set();
for (const match of corpus.matchAll(/pub (?:struct|type|enum|fn|trait|const) ([A-Za-z_][A-Za-z0-9_]*)/g)) {
  nativeSymbols.add(match[1]);
}
// The public wire vocabulary (JSON field names like `agency_ref`) is contract
// surface too, published through the serde struct fields.
for (const match of corpus.matchAll(/pub ([a-z_][a-z0-9_]*):/g)) {
  nativeSymbols.add(match[1]);
}

// Vocabulary a Skill must be able to name without it being a workspace
// symbol: cross-product concepts (AIKit, Factory, O:I) and constitutional
// nouns used generically in prose.
const nonSymbolVocabulary = new Set([
  "HarnessComposition", "ExecutionDisposition", "SharedField", "Journey", "Run",
  "Agent", "Agency", "RootAgency", "Metagency", "AgentSession", "Claim", "Evidence",
]);

for (const skill of skills) {
  const text = readFileSync(skill, "utf8");
  const spans = [...text.matchAll(/`([^`\n]+)`/g)].map((match) => match[1]);

  // Truth: a backticked repository path the Skill names must exist.
  for (const span of spans) {
    if (!/^(bin|cli|contracts|detection|crates|catalog|docs|schemas|scripts|skills|experiments)\/[\w./-]+$/.test(span)) continue;
    if (!existsSync(span)) failures.push(`${skill}: names missing path \`${span}\``);
  }

  // Truth: a backticked identifier must be a public symbol of the workspace
  // crates. Qualified names (`Type::operation`) check their last segment;
  // PascalCase names are types, checked against the workspace except for the
  // declared cross-product concepts.
  for (const span of spans) {
    if (/^[A-Za-z_][A-Za-z0-9_]*::[a-z_][a-zA-Z0-9_]*$/.test(span)) {
      const name = span.split("::").pop();
      if (!nativeSymbols.has(name)) failures.push(`${skill}: names \`${span}\`, which no workspace crate publishes`);
      continue;
    }
    if (/^[A-Z][A-Za-z0-9]{3,}$/.test(span)) {
      if (!nativeSymbols.has(span) && !nonSymbolVocabulary.has(span)) {
        failures.push(`${skill}: names \`${span}\`, which no workspace crate publishes`);
      }
      continue;
    }
    if (/^[a-z][a-z0-9_]{3,}$/.test(span) && span.includes("_")) {
      if (!nativeSymbols.has(span)) failures.push(`${skill}: names \`${span}\`, which no workspace crate publishes`);
    }
  }

  // Truth: a backticked `actuation <command>` invocation must be served.
  for (const span of spans) {
    for (const match of span.matchAll(/(?:^|\s)(?:\.\/)?(?:target\/release\/)?actuation ([a-z][a-z0-9-]*)/g)) {
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
