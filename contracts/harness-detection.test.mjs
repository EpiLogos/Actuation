import assert from "node:assert/strict";
import test from "node:test";

import {
  HARNESS_DETECTION_VERSION,
  harnessDescriptor,
  harnessDetection,
  validateHarnessDescriptor,
  validateHarnessDetection,
} from "./harness-detection.mjs";
import schema from "./harness-detection-v1.schema.json" with { type: "json" };

function descriptor(extra = {}) {
  return {
    schema: HARNESS_DETECTION_VERSION,
    document: "descriptor",
    slug: "test-harness",
    native_kind: "harness",
    edition: "cli",
    native_owner: "Test Upstream",
    summary: "a fixture harness",
    aliases: ["th"],
    probe: {
      executable: { names: ["test-harness"], version_args: ["--version"] },
      "config-dir": { path: "~/.test-harness" },
    },
    facets: { skills: { path: "~/.test-harness/skills" } },
    provenance: { authored_by: "test", catalog_revision: 1 },
    ...extra,
  };
}

function detectionEntry(slug, state, extra = {}) {
  return {
    slug,
    harness_ref: `harness/${slug}`,
    state,
    ...extra,
  };
}

function detection(extra = {}) {
  return {
    schema: HARNESS_DETECTION_VERSION,
    document: "detection",
    detection_ref: "detection:test:1",
    observed_at: "2026-09-05T00:00:00Z",
    catalog_revision: 1,
    detector: { implementation: "actuation detect", version: "0.1.0" },
    harnesses: [
      detectionEntry("present", "detected", {
        version: "1.2.3",
        receipts: { executable: "/usr/bin/present", sha256: "a".repeat(64), mtime: 1, size: 2 },
        probes: [{ kind: "executable", result: "pass", detail: "resolved /usr/bin/present" }],
        facets: [{ kind: "skills", path: "~/.present/skills", exists: true, count: 3 }],
      }),
      detectionEntry("absent-one", "not-installed", {
        probes: [{ kind: "config-dir", result: "pass", detail: "~/.absent-one not found" }],
      }),
    ],
    absent: ["absent-one"],
    availability: "complete",
    ...extra,
  };
}
import { readFileSync } from "node:fs";

test("descriptor round-trips through the constructor", () => {
  const value = harnessDescriptor(descriptor());
  assert.equal(value.slug, "test-harness");
  assert.equal(value.schema, HARNESS_DETECTION_VERSION);
  assert.deepEqual(Object.keys(value), Object.keys(descriptor()));
});

test("descriptor requires a probe spec and refuses unknown probe or facet kinds", () => {
  assert.throws(() => validateHarnessDescriptor(descriptor({ probe: {} })), /must declare at least one probe/);
  assert.throws(
    () => validateHarnessDescriptor(descriptor({ probe: { oracle: {} } })),
    /not a supported probe kind/,
  );
  assert.throws(
    () => validateHarnessDescriptor(descriptor({ facets: { gripes: { path: "~/.x" } } })),
    /not a supported facet kind/,
  );
  const bad = descriptor();
  delete bad.provenance.catalog_revision;
  assert.throws(() => validateHarnessDescriptor(bad), /catalog_revision must be an integer/);
});

test("detected requires a passing probe and same-run receipts", () => {
  assert.throws(
    () => validateHarnessDetection(detection({
      harnesses: [detectionEntry("ghost", "detected", {
        receipts: { executable: "/usr/bin/ghost" },
        probes: [{ kind: "executable", result: "fail", detail: "could not run" }],
      })],
      absent: [],
    })),
    /requires at least one passing probe/,
  );
  assert.throws(
    () => validateHarnessDetection(detection({
      harnesses: [detectionEntry("noverify", "detected", {
        probes: [{ kind: "executable", result: "pass" }],
      })],
      absent: [],
    })),
    /receipts \(mandatory when detected\)/,
  );
});

test("unavailable carries a mandatory reason and degrades availability", () => {
  assert.throws(
    () => validateHarnessDetection(detection({
      harnesses: [detectionEntry("flaky", "unavailable")],
      absent: [],
    })),
    /unavailable_reason \(mandatory when unavailable\)/,
  );
  const partial = harnessDetection(detection({
    harnesses: [detectionEntry("flaky", "unavailable", { unavailable_reason: "version probe hung" })],
    absent: [],
    availability: "partial",
  }));
  assert.equal(partial.availability, "partial");
  assert.equal(partial.absent.length, 0);
});

test("not-installed requires evidence of absence and forbids observed facets", () => {
  assert.throws(
    () => validateHarnessDetection(detection({
      harnesses: [detectionEntry("missing", "not-installed", { probes: [] })],
    })),
    /not an empty probe list/,
  );
  assert.throws(
    () => validateHarnessDetection(detection({
      harnesses: [detectionEntry("lying", "not-installed", {
        probes: [{ kind: "config-dir", result: "pass" }],
        facets: [{ kind: "skills", path: "~/.lying/skills", exists: true }],
      })],
    })),
    /facet evidence implies presence/,
  );
});

test("absent list and availability must agree with the entries", () => {
  assert.throws(() => validateHarnessDetection(detection({ absent: [] })), /absent must list exactly/);
  assert.throws(
    () => validateHarnessDetection(detection({
      harnesses: detection().harnesses,
      absent: ["absent-one", "extra"],
    })),
    /absent must list exactly/,
  );
  assert.throws(
    () => validateHarnessDetection(detection({
      harnesses: [detectionEntry("flaky", "unavailable", { unavailable_reason: "probe hung" })],
      absent: [],
      availability: "complete",
    })),
    /availability must be "partial"/,
  );
});

test("harness_ref derivation, duplicate slugs and sha256 shape are checked", () => {
  assert.throws(
    () => validateHarnessDetection(detection({
      harnesses: [{ ...detectionEntry("present", "detected"), harness_ref: "tool/present" }],
    })),
    /harness_ref must be harness\/<slug>/,
  );
  assert.throws(
    () => validateHarnessDetection(detection({
      harnesses: [
        detection().harnesses[0],
        detectionEntry("present", "not-installed", { probes: [{ kind: "config-dir", result: "pass" }] }),
      ],
    })),
    /duplicate slug/,
  );
  assert.throws(
    () => validateHarnessDetection(detection({
      harnesses: [detectionEntry("present", "detected", {
        receipts: { executable: "/x", sha256: "NOT-HEX" },
        probes: [{ kind: "executable", result: "pass" }],
      })],
      absent: [],
    })),
    /sha256 must be 64 lowercase hex/,
  );
});

test("schema document agrees with the contract version", () => {
  const consts = JSON.stringify(schema).match(/actuation\.harness-detection\/v1/g) ?? [];
  assert.ok(consts.length >= 2, "schema pins the contract version");
  assert.equal(schema.$id.includes("harness-detection-v1.schema.json"), true);
});
