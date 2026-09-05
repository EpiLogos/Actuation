import assert from "node:assert/strict";
import test from "node:test";

import { resolveSelf } from "./self.mjs";
import { validateHarnessSelf } from "../contracts/harness-detection.mjs";

function stubEffects(overrides = {}) {
  return {
    expandHome: (path) => path.replace(/^~/, "/home/tester"),
    resolveExecutable: () => ({ found: false, path: null }),
    versionProbe: () => ({ ok: true, version: "1.0.0" }),
    statProbe: () => ({ exists: false }),
    hashProbe: () => ({ ok: false }),
    dirCountProbe: () => ({ exists: false }),
    envProbe: () => ({ ok: true, matched: {} }),
    ...overrides,
  };
}

function descriptor(slug, envVars) {
  return {
    schema: "actuation.harness-detection/v1",
    document: "descriptor",
    slug,
    native_kind: "harness",
    probe: {
      "config-dir": { path: `~/.${slug}` },
      env: { any_of: envVars },
    },
    provenance: { authored_by: "test", catalog_revision: 1 },
  };
}

test("a single marker match resolves self and cross-checks detection", () => {
  const self = resolveSelf({
    descriptors: [descriptor("alpha", ["ALPHA_MARKER"]), descriptor("beta", ["BETA_MARKER"])],
    effects: stubEffects({
      envProbe: (names) => ({ ok: true, matched: names.includes("ALPHA_MARKER") ? { ALPHA_MARKER: "1" } : {} }),
    }),
    env: { ALPHA_MARKER: "1" },
  });
  assert.equal(self.matched.length, 1);
  assert.equal(self.resolved.slug, "alpha");
  assert.equal(self.resolved.harness_ref, "harness/alpha");
  assert.deepEqual(self.resolved.markers, ["ALPHA_MARKER"]);
  assert.equal(self.ambiguity, false);
  // alpha is not-installed in the stubbed detection (no config-dir evidence);
  // the cross-check states disclose that tension instead of hiding it.
  assert.equal(self.detection.states.alpha, "not-installed");
});

test("multiple marker matches are honest ambiguity, never a silent pick", () => {
  const self = resolveSelf({
    descriptors: [descriptor("alpha", ["ALPHA_MARKER"]), descriptor("beta", ["BETA_MARKER"])],
    effects: stubEffects({
      envProbe: () => ({ ok: true, matched: { ALPHA_MARKER: "1", BETA_MARKER: "1" } }),
    }),
    env: { ALPHA_MARKER: "1", BETA_MARKER: "1" },
  });
  assert.equal(self.matched.length, 2);
  assert.equal(self.resolved, null);
  assert.equal(self.ambiguity, true);
});

test("no markers matched resolves null without ambiguity", () => {
  const self = resolveSelf({
    descriptors: [descriptor("alpha", ["ALPHA_MARKER"])],
    effects: stubEffects(),
    env: {},
  });
  assert.deepEqual(self.matched, []);
  assert.equal(self.resolved, null);
  assert.equal(self.ambiguity, false);
});

test("descriptors without env probes never match", () => {
  const self = resolveSelf({
    descriptors: [{
      schema: "actuation.harness-detection/v1",
      document: "descriptor",
      slug: "plain",
      native_kind: "harness",
      probe: { "config-dir": { path: "~/.plain" } },
      provenance: { authored_by: "test", catalog_revision: 1 },
    }],
    effects: stubEffects(),
    env: {},
  });
  assert.deepEqual(self.matched, []);
});

test("the self record passes contract validation", () => {
  const self = resolveSelf({
    descriptors: [descriptor("alpha", ["ALPHA_MARKER"])],
    effects: stubEffects({
      envProbe: () => ({ ok: true, matched: { ALPHA_MARKER: "1" } }),
    }),
    env: { ALPHA_MARKER: "1" },
  });
  assert.equal(validateHarnessSelf(self), self);
});

test("resolved may only be set for exactly one match", () => {
  assert.throws(
    () => validateHarnessSelf({
      schema: "actuation.harness-detection/v1",
      document: "self",
      self_ref: "self:t",
      observed_at: "2026-09-05T00:00:00Z",
      catalog_revision: 1,
      matched: [],
      resolved: { slug: "alpha", harness_ref: "harness/alpha", markers: ["A"] },
      ambiguity: false,
      detection_ref: "detection:t",
      detection: { states: {} },
    }),
    /may only be set when exactly one/,
  );
});
