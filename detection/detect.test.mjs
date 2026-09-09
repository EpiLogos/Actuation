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

test("live catalog reports every harness honestly on this machine", () => {
  // Live-machine evidence, not a presence requirement: a CI runner has none
  // of these installed and "not-installed" is the truthful state there. The
  // subset is not transcribed — the whole catalog is probed by discovery, so
  // a new descriptor joins this check automatically. Strict presence for a
  // specific machine is opt-in via ACTUATION_EXPECT_DETECTED=<slugs>.
  const record = runDetection({ descriptors: harnessDescriptors() });
  const catalogued = harnessDescriptors().map((descriptor) => descriptor.slug).sort();
  assert.deepEqual(record.harnesses.map((entry) => entry.slug).sort(), catalogued);
  for (const entry of record.harnesses) {
    if (entry.state === "detected") {
      assert.ok(
        entry.receipts?.executable || entry.receipts?.executable_is,
        `${entry.slug} is detected and must carry a presence receipt`,
      );
    } else {
      assert.equal(entry.state, "not-installed", `${entry.slug} must not be unavailable on a readable machine`);
      assert.ok(record.absent.includes(entry.slug), `${entry.slug} absent list agrees with its state`);
    }
  }
  const pinned = (process.env.ACTUATION_EXPECT_DETECTED ?? "")
    .split(",").map((slug) => slug.trim()).filter(Boolean);
  for (const slug of pinned) {
    const entry = entryOf(record, slug);
    assert.ok(entry, `ACTUATION_EXPECT_DETECTED names unknown slug: ${slug}`);
    assert.equal(entry.state, "detected", `${slug} pinned live via ACTUATION_EXPECT_DETECTED`);
    assert.ok(entry.receipts?.executable || entry.receipts?.executable_is, `${slug} carries a receipt`);
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

test("probeVersions stamps entry.version from the descriptor-declared args", () => {
  let probedWith;
  const record = runDetection({
    descriptors: [descriptor("versioned", {
      probe: { executable: { names: ["versioned"], version_args: ["version"] } },
    })],
    effects: stubEffects({
      resolveExecutable: () => ({ found: true, path: "/opt/bin/versioned" }),
      versionProbe: (path, args) => {
        probedWith = { path, args };
        return { ok: true, version: "versioned 7.3.1" };
      },
    }),
    probeVersions: true,
  });
  const entry = entryOf(record, "versioned");
  assert.equal(entry.version, "versioned 7.3.1");
  assert.deepEqual(probedWith, { path: "/opt/bin/versioned", args: ["version"] });
});

test("version probing is opt-in: plain detection never spawns binaries", () => {
  let spawned = false;
  const record = runDetection({
    descriptors: [descriptor("versioned", { probe: { executable: { names: ["versioned"] } } })],
    effects: stubEffects({
      resolveExecutable: () => ({ found: true, path: "/opt/bin/versioned" }),
      versionProbe: () => {
        spawned = true;
        return { ok: true, version: "1.0.0" };
      },
    }),
  });
  assert.equal(spawned, false);
  assert.equal(entryOf(record, "versioned").version, undefined);
});

test("version probe failure is disclosed and never changes state", () => {
  const record = runDetection({
    descriptors: [descriptor("grumpy", { probe: { executable: { names: ["grumpy"] } } })],
    effects: stubEffects({
      resolveExecutable: () => ({ found: true, path: "/opt/bin/grumpy" }),
      versionProbe: () => ({ ok: false, reason: "exit 1: keychain prompt refused" }),
    }),
    probeVersions: true,
  });
  const entry = entryOf(record, "grumpy");
  assert.equal(entry.state, "detected");
  assert.equal(entry.version, undefined);
  assert.ok(record.disclosure.some((line) => line.startsWith("grumpy: version probe failed")));
});

test("config-dir receipts are not version-probed", () => {
  let spawned = false;
  const record = runDetection({
    descriptors: [descriptor("configonly", { probe: { "config-dir": { path: "~/.configonly" } } })],
    effects: stubEffects({
      statProbe: (path) => path.startsWith("/home/tester/.configonly")
        ? { exists: true, isDir: true, mtimeMs: 1, size: 2 }
        : { exists: false },
      versionProbe: () => {
        spawned = true;
        return { ok: true, version: "1.0.0" };
      },
    }),
    probeVersions: true,
  });
  assert.equal(spawned, false);
  assert.equal(entryOf(record, "configonly").version, undefined);
});

test("a live service probe is presence evidence with receipts via config-dir", () => {
  const record = runDetection({
    descriptors: [descriptor("daemonish", {
      probe: {
        "config-dir": { path: "~/.daemonish" },
        service: { kind: "daemon", name: "daemonish" },
      },
    })],
    effects: stubEffects({
      statProbe: (path) => path.startsWith("/home/tester/.daemonish")
        ? { exists: true, isDir: true, mtimeMs: 1, size: 2 }
        : { exists: false },
      serviceProbe: () => ({ ok: true, detail: "daemon daemonish running" }),
    }),
  });
  const entry = entryOf(record, "daemonish");
  assert.equal(entry.state, "detected");
  assert.equal(entry.receipts.executable_is, "config-dir");
  const service = entry.probes.find((probe) => probe.kind === "service");
  assert.equal(service.result, "pass");
  assert.equal(service.detail, "daemon daemonish running");
});

test("a clean service absence is not presence evidence", () => {
  const record = runDetection({
    descriptors: [descriptor("sleepy", {
      probe: { "config-dir": { path: "~/.sleepy" }, service: { kind: "http", default_url: "http://127.0.0.1:1" } },
    })],
    effects: stubEffects({
      serviceProbe: () => ({ ok: true, detail: "no listener at http://127.0.0.1:1" }),
    }),
  });
  const entry = entryOf(record, "sleepy");
  assert.equal(entry.state, "not-installed");
  const service = entry.probes.find((probe) => probe.kind === "service");
  assert.equal(service.result, "pass");
  assert.equal(service.detail, "no listener at http://127.0.0.1:1");
});

test("a failed service probe is a mechanism failure, never absence", () => {
  const record = runDetection({
    descriptors: [descriptor("brokensvc", {
      probe: { service: { kind: "http", default_url: "http://127.0.0.1:1" } },
    })],
    effects: stubEffects({
      serviceProbe: () => ({ ok: false, reason: "curl unavailable: spawn ENOENT" }),
    }),
  });
  const entry = entryOf(record, "brokensvc");
  assert.equal(entry.state, "unavailable");
  assert.match(entry.unavailable_reason, /all probes failed/);
});

// --- typed model-provider inventory ------------------------------------
//
// The break these close: the shared directory probe answers "count: 3" for
// ~/.ollama/models, so nothing downstream can name a single model. The
// declared service inventory names them; the directory count stays a
// presence signal and the shared probe is untouched.

function modelProvider(extra = {}) {
  return descriptor("modelhost", {
    native_kind: "model-provider",
    probe: {
      "config-dir": { path: "~/.modelhost" },
      service: { kind: "http", default_url: "http://127.0.0.1:11434" },
    },
    facets: {
      models: {
        path: "~/.modelhost/models",
        inventory: {
          kind: "http-json",
          from: "service",
          route: "/api/tags",
          collection: "models",
          id_field: "model",
          also_id_fields: ["name"],
          detail_fields: ["digest", "size"],
        },
      },
    },
    ...extra,
  });
}

function modelHostEffects(overrides = {}) {
  return stubEffects({
    statProbe: (path) => (path.startsWith("/home/tester/.modelhost")
      ? { exists: true, mtimeMs: 5, size: 1, isDir: !path.endsWith(".json") }
      : { exists: false }),
    dirCountProbe: () => ({ exists: true, count: 3 }),
    serviceProbe: () => ({ ok: true, detail: "http 200 from http://127.0.0.1:11434" }),
    ...overrides,
  });
}

test("a live declared service inventory names provider-native models, not a directory count", () => {
  const record = runDetection({
    descriptors: [modelProvider()],
    effects: modelHostEffects({
      httpJsonProbe: (url) => {
        assert.equal(url, "http://127.0.0.1:11434/api/tags");
        return {
          ok: true,
          body: {
            models: [
              { name: "llama3.2:latest", model: "llama3.2:latest", digest: "d1", size: 10 },
              { name: "smollm2:135m", model: "smollm2:135m", digest: "d2", size: 20 },
              { name: "qwen2.5-coder:7b", model: "qwen2.5-coder:7b", digest: "d3", size: 30 },
            ],
          },
        };
      },
    }),
  });
  const facet = entryOf(record, "modelhost").facets.find((item) => item.kind === "models");
  assert.equal(facet.count, 3, "the directory count stays a presence signal");
  assert.deepEqual(facet.inventory.map((item) => item.id), [
    "llama3.2:latest",
    "smollm2:135m",
    "qwen2.5-coder:7b",
  ]);
  assert.deepEqual(facet.inventory[0], { id: "llama3.2:latest", digest: "d1", size: 10 });
  assert.equal(facet.inventory_receipt.source, "http://127.0.0.1:11434/api/tags");
  assert.equal(facet.inventory_receipt.item_count, 3);
});

test("a differing secondary id field is carried as also_known_as, never as a second identity", () => {
  const record = runDetection({
    descriptors: [modelProvider()],
    effects: modelHostEffects({
      httpJsonProbe: () => ({ ok: true, body: { models: [{ name: "llama3.2:latest", model: "llama3.2" }] } }),
    }),
  });
  const facet = entryOf(record, "modelhost").facets.find((item) => item.kind === "models");
  assert.deepEqual(facet.inventory, [{ id: "llama3.2", also_known_as: ["llama3.2:latest"] }]);
});

test("a failed inventory read is a disclosed reason, never an empty offering", () => {
  const record = runDetection({
    descriptors: [modelProvider()],
    effects: modelHostEffects({
      httpJsonProbe: () => ({ ok: false, reason: "curl exit 7: connection refused" }),
    }),
  });
  const facet = entryOf(record, "modelhost").facets.find((item) => item.kind === "models");
  assert.equal(facet.inventory, undefined);
  assert.match(facet.inventory_unavailable_reason, /connection refused/);
  assert.ok(record.disclosure.some((line) => line.startsWith("modelhost: model inventory read failed")));
});

test("a service that is not proven live is never read for inventory", () => {
  let called = false;
  const record = runDetection({
    descriptors: [modelProvider()],
    effects: modelHostEffects({
      serviceProbe: () => ({ ok: true, detail: "no listener at http://127.0.0.1:11434" }),
      httpJsonProbe: () => { called = true; return { ok: true, body: { models: [] } }; },
    }),
  });
  const facet = entryOf(record, "modelhost").facets.find((item) => item.kind === "models");
  assert.equal(called, false);
  assert.equal(facet.count, 3);
  assert.match(facet.inventory_unavailable_reason, /did not prove a live endpoint/);
});

test("facets without a declared inventory never gain one (unrelated directories stay counted, not listed)", () => {
  const record = runDetection({
    descriptors: [descriptor("plainskills", {
      probe: { "config-dir": { path: "~/.plainskills" } },
      facets: { skills: { path: "~/.plainskills/skills" } },
    })],
    effects: stubEffects({
      statProbe: (path) => (path.startsWith("/home/tester/.plainskills")
        ? { exists: true, mtimeMs: 1, size: 1, isDir: true }
        : { exists: false }),
      dirCountProbe: () => ({ exists: true, count: 9 }),
      httpJsonProbe: () => { throw new Error("must never be reached"); },
    }),
  });
  assert.deepEqual(entryOf(record, "plainskills").facets, [
    { kind: "skills", path: "~/.plainskills/skills", exists: true, count: 9 },
  ]);
});

test("every catalogued descriptor's native_kind rides onto its detection entry", () => {
  const descriptors = harnessDescriptors();
  const record = runDetection({ descriptors, effects: stubEffects() });
  for (const declared of descriptors) {
    const entry = entryOf(record, declared.slug);
    assert.equal(entry.native_kind, declared.native_kind, `${declared.slug} must carry its declared native_kind`);
  }
  assert.equal(entryOf(record, "ollama").native_kind, "model-provider");
  assert.equal(entryOf(record, "claude-code").native_kind, "harness");
});
