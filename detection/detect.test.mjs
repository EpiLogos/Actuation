import assert from "node:assert/strict";
import test from "node:test";

import { runDetection } from "./detect.mjs";
import { harnessDescriptors } from "./catalog.mjs";

function stubEffects(overrides = {}) {
  return {
    expandHome: (path) => path.replace(/^~/, "/home/tester"),
    resolveExecutable: () => ({ found: false, path: null }),
    versionProbe: () => ({ ok: true, version: "1.0.0" }),
    statProbe: () => ({ exists: false }),
    hashProbe: () => ({ ok: false }),
    dirCountProbe: () => ({ exists: false }),
    ...overrides,
  };
}

function descriptor(slug, extra = {}) {
  return {
    schema: "actuation.harness-detection/v1",
    document: "descriptor",
    slug,
    native_kind: "harness",
    probe: { executable: { names: [slug] } },
    provenance: { authored_by: "test", catalog_revision: 1 },
    ...extra,
  };
}

function entryOf(record, slug) {
  return record.harnesses.find((entry) => entry.slug === slug);
}

test("executable resolution yields detected with receipts", () => {
  const record = runDetection({
    descriptors: [descriptor("happy")],
    effects: stubEffects({
      resolveExecutable: () => ({ found: true, path: "/opt/bin/happy" }),
      hashProbe: () => ({ ok: true, sha256: "a".repeat(64) }),
      statProbe: (path) => path === "/opt/bin/happy"
        ? { exists: true, mtimeMs: 1000.7, size: 42, isDir: false }
        : { exists: false },
    }),
  });
  const entry = entryOf(record, "happy");
  assert.equal(entry.state, "detected");
  assert.equal(entry.receipts.executable, "/opt/bin/happy");
  assert.equal(entry.receipts.sha256, "a".repeat(64));
  assert.equal(entry.receipts.size, 42);
  assert.equal(record.absent.length, 0);
  assert.equal(record.availability, "complete");
});

test("config-dir presence without binary still detects via config receipt", () => {
  const record = runDetection({
    descriptors: [descriptor("configonly", {
      probe: { "config-dir": { path: "~/.configonly" } },
      facets: { skills: { path: "~/.configonly/skills" } },
    })],
    effects: stubEffects({
      statProbe: (path) => path.startsWith("/home/tester/.configonly")
        ? { exists: true, isDir: true, mtimeMs: 1, size: 2 }
        : { exists: false },
      dirCountProbe: () => ({ exists: true, count: 7 }),
    }),
  });
  const entry = entryOf(record, "configonly");
  assert.equal(entry.state, "detected");
  assert.equal(entry.receipts.executable_is, "config-dir");
  assert.deepEqual(entry.facets, [{ kind: "skills", path: "~/.configonly/skills", exists: true, count: 7 }]);
});

test("absence on all probes yields not-installed with absence evidence", () => {
  const record = runDetection({
    descriptors: [descriptor("ghosty", { probe: { "config-dir": { path: "~/.ghosty" } } })],
    effects: stubEffects(),
  });
  const entry = entryOf(record, "ghosty");
  assert.equal(entry.state, "not-installed");
  assert.equal(entry.receipts, undefined);
  assert.deepEqual(record.absent, ["ghosty"]);
});


test("all-probes-failed yields unavailable with a mandatory reason, not absence", () => {
  const record = runDetection({
    descriptors: [descriptor("flaky", { probe: { executable: { names: ["flaky"] }, "config-dir": { path: "~/.flaky" } } })],
    effects: stubEffects({
      resolveExecutable: () => ({ found: false, path: null, error: "PATH unreadable" }),
      statProbe: () => ({ exists: false, error: "stat denied" }),
    }),
  });
  const entry = entryOf(record, "flaky");
  assert.equal(entry.state, "unavailable");
  assert.match(entry.unavailable_reason, /all probes failed/);
  assert.equal(entry.receipts, undefined);
  assert.equal(record.availability, "partial");
  assert.equal(record.absent.length, 0, "unavailable is never listed as absent");
});

test("safe live subset detects against the real machine", () => {
  const safe = ["claude-code", "codex", "openclaw", "kimi"];
  const record = runDetection({ descriptors: harnessDescriptors().filter((d) => safe.includes(d.slug)) });
  for (const entry of record.harnesses) {
    assert.equal(entry.state, "detected", `${entry.slug} should be live on this machine`);
    assert.ok(entry.receipts.executable, `${entry.slug} carries a receipt`);
  }
});

test("env markers are identity evidence, never presence evidence", () => {
  const record = runDetection({
    descriptors: [descriptor("ghost", {
      probe: { env: { any_of: ["GHOST_MARKER"] } },
    })],
    effects: stubEffects({
      envProbe: () => ({ ok: true, matched: { GHOST_MARKER: "1" } }),
    }),
  });
  const entry = entryOf(record, "ghost");
  assert.equal(entry.state, "not-installed");
  const envProbe = entry.probes.find((probe) => probe.kind === "env");
  assert.equal(envProbe.result, "pass");
  assert.match(envProbe.detail, /GHOST_MARKER set/);
});

test("env probe records absence honestly when no marker is set", () => {
  const record = runDetection({
    descriptors: [descriptor("quiet", {
      probe: { "config-dir": { path: "~/.quiet" }, env: { any_of: ["QUIET_MARKER"] } },
    })],
    effects: stubEffects({
      statProbe: () => ({ exists: true, isDir: true }),
      envProbe: () => ({ ok: true, matched: {} }),
    }),
  });
  const entry = entryOf(record, "quiet");
  assert.equal(entry.state, "detected");
  const envProbe = entry.probes.find((probe) => probe.kind === "env");
  assert.equal(envProbe.result, "pass");
  assert.equal(envProbe.detail, "no marker set");
});
