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

// --- typed model-provider inventory ------------------------------------

function inventoryDescriptor(inventory) {
  return descriptor({
    native_kind: "model-provider",
    probe: {
      "config-dir": { path: "~/.modelhost" },
      service: { kind: "http", default_url: "http://127.0.0.1:11434" },
    },
    facets: { models: { path: "~/.modelhost/models", inventory } },
  });
}

const declaredInventory = {
  kind: "http-json",
  from: "service",
  route: "/api/tags",
  collection: "models",
  id_field: "model",
};

test("a facet may declare a service-backed inventory", () => {
  const validated = harnessDescriptor(inventoryDescriptor(declaredInventory));
  assert.equal(validated.facets.models.inventory.route, "/api/tags");
});

test("an inventory declaration must bind to a declared service probe", () => {
  assert.throws(
    () => validateHarnessDescriptor(descriptor({
      facets: { models: { path: "~/.modelhost/models", inventory: declaredInventory } },
    })),
    /declares no probe.service/,
  );
});

test("unsupported inventory kinds and sources are refused", () => {
  assert.throws(
    () => validateHarnessDescriptor(inventoryDescriptor({ ...declaredInventory, kind: "dir-listing" })),
    /inventory.kind must be one of/,
  );
  assert.throws(
    () => validateHarnessDescriptor(inventoryDescriptor({ ...declaredInventory, from: "directory" })),
    /inventory.from must be one of/,
  );
});

function detectedWithFacet(facet) {
  return {
    schema: HARNESS_DETECTION_VERSION,
    document: "detection",
    detection_ref: "detection:2026-09-09T00:00:00Z",
    observed_at: "2026-09-09T00:00:00Z",
    catalog_revision: 5,
    detector: { implementation: "test", version: "0.0.0" },
    harnesses: [detectionEntry("modelhost", "detected", {
      native_kind: "model-provider",
      receipts: { executable: "/home/tester/.modelhost", executable_is: "config-dir" },
      probes: [{ kind: "config-dir", result: "pass", detail: "exists at /home/tester/.modelhost" }],
      facets: [facet],
    })],
    absent: [],
    availability: "complete",
  };
}

const observedFacet = {
  kind: "models",
  path: "~/.modelhost/models",
  exists: true,
  count: 3,
  inventory: [{ id: "llama3.2:latest" }, { id: "smollm2:135m" }],
  inventory_receipt: {
    kind: "http-json",
    source: "http://127.0.0.1:11434/api/tags",
    observed_at: "2026-09-09T00:00:00Z",
    item_count: 2,
  },
};

test("an observed facet inventory validates with its receipt and native_kind", () => {
  const validated = harnessDetection(detectedWithFacet(observedFacet));
  assert.equal(validated.harnesses[0].native_kind, "model-provider");
  assert.equal(validated.harnesses[0].facets[0].inventory.length, 2);
});

test("an observed inventory without a receipt is refused", () => {
  const facet = { ...observedFacet };
  delete facet.inventory_receipt;
  assert.throws(() => validateHarnessDetection(detectedWithFacet(facet)), /inventory_receipt \(mandatory/);
});

test("a receipt count that disagrees with the inventory is refused", () => {
  assert.throws(
    () => validateHarnessDetection(detectedWithFacet({
      ...observedFacet,
      inventory_receipt: { ...observedFacet.inventory_receipt, item_count: 7 },
    })),
    /item_count must equal/,
  );
});

test("an inventory cannot be both observed and unavailable", () => {
  assert.throws(
    () => validateHarnessDetection(detectedWithFacet({
      ...observedFacet,
      inventory_unavailable_reason: "connection refused",
    })),
    /cannot be both observed and unavailable/,
  );
});

test("an unread inventory carries a reason and no receipt", () => {
  const validated = harnessDetection(detectedWithFacet({
    kind: "models",
    path: "~/.modelhost/models",
    exists: true,
    count: 3,
    inventory_unavailable_reason: "inventory read from http://127.0.0.1:11434/api/tags failed: curl exit 7",
  }));
  assert.equal(validated.harnesses[0].facets[0].inventory, undefined);
  assert.match(validated.harnesses[0].facets[0].inventory_unavailable_reason, /curl exit 7/);
});

test("duplicate inventory ids are refused", () => {
  assert.throws(
    () => validateHarnessDetection(detectedWithFacet({
      ...observedFacet,
      inventory: [{ id: "llama3.2:latest" }, { id: "llama3.2:latest" }],
      inventory_receipt: { ...observedFacet.inventory_receipt, item_count: 2 },
    })),
    /duplicate inventory id/,
  );
});

test("native_kind stays optional for records written before the field existed", () => {
  const record = detectedWithFacet(observedFacet);
  delete record.harnesses[0].native_kind;
  assert.equal(harnessDetection(record).harnesses[0].native_kind, undefined);
  record.harnesses[0].native_kind = "";
  assert.throws(() => validateHarnessDetection(record), /native_kind must be a non-empty string/);
});
