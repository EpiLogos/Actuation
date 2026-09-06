import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync, readdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { CATALOG_REVISION, harnessDescriptors } from "./catalog.mjs";
import { realEffects } from "./probes.mjs";
import { harnessCatalog } from "../contracts/harness-detection.mjs";

const DETECTION_DIR = dirname(fileURLToPath(import.meta.url));

test("every catalog descriptor validates and slugs are unique", () => {
  const descriptors = harnessDescriptors();
  assert.ok(descriptors.length >= 11, "catalog carries the surveyed field");
  const slugs = descriptors.map((descriptor) => descriptor.slug);
  assert.equal(new Set(slugs).size, slugs.length, "no duplicate slugs");
  for (const descriptor of descriptors) {
    assert.equal(descriptor.schema, "actuation.harness-detection/v1");
    assert.equal(descriptor.document, "descriptor");
    assert.ok(descriptor.provenance.catalog_revision <= CATALOG_REVISION);
  }
});

test("the declared catalog validates as a catalog document", () => {
  const catalog = harnessCatalog({
    schema: "actuation.harness-detection/v1",
    document: "catalog",
    catalog_revision: CATALOG_REVISION,
    descriptors: harnessDescriptors(),
  });
  assert.equal(catalog.descriptors.length, harnessDescriptors().length);
  assert.throws(
    () => harnessCatalog({ schema: "actuation.harness-detection/v1", document: "catalog", catalog_revision: 0, descriptors: [] }),
    /non-empty array/,
  );
});

// A descriptor that declares a probe kind the engine cannot implement is a
// silent lie ("declared, not verified" forever). This test makes the
// declared-kinds ⊆ implemented-effects invariant a build failure.
const EFFECT_FOR_KIND = {
  executable: "resolveExecutable",
  "config-dir": "statProbe",
  service: "serviceProbe",
  env: "envProbe",
};

test("every probe kind declared by the catalog is implemented by realEffects", () => {
  const effects = realEffects();
  const declared = new Set();
  for (const descriptor of harnessDescriptors()) {
    for (const kind of Object.keys(descriptor.probe)) declared.add(kind);
    if (descriptor.probe.executable?.version_args) declared.add("versionProbe");
  }
  assert.ok(declared.size > 0);
  for (const declaredKind of declared) {
    const effect = declaredKind === "versionProbe" ? "versionProbe" : EFFECT_FOR_KIND[declaredKind];
    assert.ok(effect, `probe kind ${declaredKind} has no effect mapping in this test`);
    assert.equal(typeof effects[effect], "function", `realEffects is missing ${effect} for probe kind ${declaredKind}`);
  }
});

// The catalog import list is a hand-maintained parallel structure to the
// harnesses/ directory; this keeps them equal so a descriptor file can never
// be added without being loaded (or vice versa).
test("every descriptor module in harnesses/ is imported by the catalog", () => {
  const catalogSource = readFileSync(join(DETECTION_DIR, "catalog.mjs"), "utf8");
  const imported = new Set([...catalogSource.matchAll(/\.\/harnesses\/([a-z0-9-]+)\.mjs/g)].map((match) => match[1]));
  const present = new Set(
    readdirSync(join(DETECTION_DIR, "harnesses"))
      .filter((name) => name.endsWith(".mjs"))
      .map((name) => name.replace(/\.mjs$/, "")),
  );
  assert.deepEqual(imported, present, "harnesses/ directory and catalog imports have diverged");
});
