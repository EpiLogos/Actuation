import assert from "node:assert/strict";
import test from "node:test";

import {
  SECRET_DETECTION_VERSION,
  secretScan,
  secretSourceCatalog,
  secretSourceDescriptor,
  validateSecretScan,
  validateSecretSourceCatalog,
  validateSecretSourceDescriptor,
} from "./secret-detection.mjs";
import schema from "./secret-detection-v1.schema.json" with { type: "json" };

const FP = "a".repeat(64);

function descriptor(extra = {}) {
  return {
    schema: SECRET_DETECTION_VERSION,
    document: "descriptor",
    slug: "test-source",
    source_kind: "env-var",
    ref_schemes: ["env://"],
    probe: { env: { names: ["TEST_SOURCE_TOKEN"] } },
    provenance: { authored_by: "test", catalog_revision: 1 },
    ...extra,
  };
}

function scanEntry(slug, state, extra = {}) {
  return {
    slug,
    source_ref: `secret-source/${slug}`,
    state,
    ...extra,
  };
}

function scan(extra = {}) {
  return {
    schema: SECRET_DETECTION_VERSION,
    document: "scan",
    scan_ref: "secret-scan:test:1",
    observed_at: "2026-09-08T00:00:00Z",
    catalog_revision: 1,
    scanner: { implementation: "test scan", version: "0.0.1" },
    sources: [
      scanEntry("declared-env", "verified", {
        probes: [{ kind: "env", result: "pass", spec: '{"names":["X"]}' }],
        evidence: [{ where: "X_TOKEN", fingerprint_sha256: FP, byte_length: 24 }],
      }),
      scanEntry("stray-file", "violation", {
        violation_class: "stray-plaintext",
        centralise_to: "op://Central/central-security",
        probes: [{ kind: "file-pattern", result: "pass" }],
        evidence: [{ where: "/work/client_secret.json", fingerprint_sha256: FP, byte_length: 120 }],
      }),
      scanEntry("gone", "absent", {
        probes: [{ kind: "vault-item", result: "pass", spec: '{"item_ref":"op://Central/x"}' }],
      }),
    ],
    violations: ["stray-file"],
    coverage: "complete",
    ...extra,
  };
}

test("descriptor round-trips through the constructor", () => {
  const value = secretSourceDescriptor(descriptor());
  assert.equal(value.slug, "test-source");
  assert.equal(value.schema, SECRET_DETECTION_VERSION);
  assert.deepEqual(Object.keys(value), Object.keys(descriptor()));
});

test("descriptor requires a probe spec and refuses unknown kinds", () => {
  assert.throws(() => validateSecretSourceDescriptor(descriptor({ probe: {} })), /at least one probe/);
  assert.throws(
    () => validateSecretSourceDescriptor(descriptor({ probe: { oracle: {} } })),
    /not a supported probe kind/,
  );
  assert.throws(
    () => validateSecretSourceDescriptor(descriptor({ source_kind: "vault" })),
    /not a supported kind/,
  );
  assert.throws(
    () => validateSecretSourceDescriptor(descriptor({ probe: { env: {} } })),
    /requires names or name_pattern/,
  );
  assert.throws(
    () => validateSecretSourceDescriptor(descriptor({ probe: { "file-pattern": {} } })),
    /patterns/,
  );
  const bad = descriptor();
  delete bad.provenance.catalog_revision;
  assert.throws(() => validateSecretSourceDescriptor(bad), /catalog_revision must be an integer/);
});

test("verified requires a passing probe; violation requires class and centralise_to", () => {
  assert.throws(
    () => validateSecretScan(scan({ sources: [scanEntry("ghost", "verified", { probes: [] })] })),
    /at least one passing probe/,
  );
  assert.throws(
    () =>
      validateSecretScan(
        scan({
          sources: [
            scanEntry("noviolation", "violation", {
              probes: [{ kind: "env", result: "pass" }],
              evidence: [{ where: "X", fingerprint_sha256: FP }],
            }),
          ],
        }),
      ),
    /violation_class must be one of/,
  );
  assert.throws(
    () =>
      validateSecretScan(
        scan({
          sources: [
            scanEntry("nocentral", "violation", {
              violation_class: "uncentralised-env",
              probes: [{ kind: "env", result: "pass" }],
              evidence: [{ where: "X", fingerprint_sha256: FP }],
            }),
          ],
        }),
      ),
    /centralise_to \(mandatory when violation/,
  );
});

test("unavailable carries a mandatory reason and degrades coverage", () => {
  assert.throws(
    () =>
      validateSecretScan(
        scan({
          sources: [scanEntry("flaky", "unavailable")],
          violations: [],
        }),
      ),
    /unavailable_reason \(mandatory when unavailable\)/,
  );
  const partial = secretScan(
    scan({
      sources: [
        scan().sources[0],
        scanEntry("flaky", "unavailable", { unavailable_reason: "keychain probe not built yet" }),
      ],
      violations: [],
      coverage: "partial",
    }),
  );
  assert.equal(partial.coverage, "partial");
});

test("violations list must agree with entries; coverage with unavailable count", () => {
  assert.throws(() => validateSecretScan(scan({ violations: [] })), /must list exactly/);
  assert.throws(
    () => validateSecretScan(scan({ violations: ["stray-file", "extra"] })),
    /must list exactly/,
  );
  assert.throws(
    () =>
      validateSecretScan(
        scan({
          sources: [scanEntry("flaky", "unavailable", { unavailable_reason: "down" })],
          violations: [],
          coverage: "complete",
        }),
      ),
    /coverage must be "partial"/,
  );
});

test("evidence is fingerprint-only: bad hex rejected, value-shaped keys refused at any depth", () => {
  assert.throws(
    () =>
      validateSecretScan(
        scan({
          sources: [
            scanEntry("badfp", "verified", {
              probes: [{ kind: "env", result: "pass" }],
              evidence: [{ where: "X", fingerprint_sha256: "NOT-HEX" }],
            }),
          ],
          violations: [],
        }),
      ),
    /64 lowercase hex/,
  );
  assert.throws(
    () =>
      validateSecretScan(
        scan({
          sources: [
            scanEntry("leak", "verified", {
              probes: [{ kind: "env", result: "pass" }],
              evidence: [{ where: "X", fingerprint_sha256: FP, value: "hunter2" }],
            }),
          ],
          violations: [],
        }),
      ),
    /forbidden value-shaped key "value"/,
  );
  assert.throws(
    () =>
      validateSecretScan(
        scan({
          sources: [
            scanEntry("nested-leak", "verified", {
              probes: [{ kind: "env", result: "pass" }],
              evidence: [{ where: "X", fingerprint_sha256: FP, meta: { password: "x" } }],
            }),
          ],
          violations: [],
        }),
      ),
    /forbidden value-shaped key "password"/,
  );
});

test("source_ref derivation and duplicate slugs are checked", () => {
  assert.throws(
    () =>
      validateSecretScan(
        scan({
          sources: [{ ...scan().sources[0], source_ref: "secret/wrong" }],
        }),
      ),
    /source_ref must be secret-source\/<slug>/,
  );
  const first = scan().sources[0];
  assert.throws(
    () =>
      validateSecretScan(
        scan({
          sources: [first, scanEntry(first.slug, "absent", { probes: [{ kind: "env", result: "pass" }] })],
          violations: [],
        }),
      ),
    /duplicate slug/,
  );
});

test("only violations carry a violation_class", () => {
  assert.throws(
    () =>
      validateSecretScan(
        scan({
          sources: [
            scanEntry("confused", "verified", {
              violation_class: "unknown",
              probes: [{ kind: "env", result: "pass" }],
            }),
          ],
          violations: [],
        }),
      ),
    /only violations carry a violation_class/,
  );
});

test("catalog validates and pins unique slugs", () => {
  const catalog = secretSourceCatalog({
    schema: SECRET_DETECTION_VERSION,
    document: "catalog",
    catalog_revision: 1,
    descriptors: [descriptor()],
  });
  assert.equal(catalog.descriptors.length, 1);
  assert.throws(
    () =>
      validateSecretSourceCatalog({
        schema: SECRET_DETECTION_VERSION,
        document: "catalog",
        catalog_revision: 1,
        descriptors: [descriptor(), descriptor()],
      }),
    /duplicate slug/,
  );
});

test("schema document agrees with the contract version", () => {
  const consts = JSON.stringify(schema).match(/actuation\.secret-detection\/v1/g) ?? [];
  assert.ok(consts.length >= 2, "schema pins the contract version");
  assert.equal(schema.$id.includes("secret-detection-v1.schema.json"), true);
});
