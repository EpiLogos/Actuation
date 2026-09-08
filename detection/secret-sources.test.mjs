import assert from "node:assert/strict";
import test from "node:test";

import { SECRET_DETECTION_VERSION } from "../contracts/secret-detection.mjs";
import {
  cliPresenceProbe,
  envFingerprintProbe,
  filePatternProbe,
  vaultItemProbe,
} from "./secret-sources/probes.mjs";
import { scanSecretSources } from "./secret-sources/scan.mjs";

test("env probe returns fingerprints, never values", () => {
  const env = { MY_API_KEY: "super-secret-material", UNRELATED: "x", EMPTY: "" };
  const result = envFingerprintProbe({ name_pattern: "(KEY|TOKEN)" }, env);
  assert.equal(result.ok, true);
  assert.deepEqual(Object.keys(result.matched), ["MY_API_KEY"]);
  const item = result.matched.MY_API_KEY;
  assert.equal(item.fingerprint_sha256.length, 64);
  assert.match(item.fingerprint_sha256, /^[0-9a-f]{64}$/);
  assert.equal(item.byte_length, 21);
  assert.equal(JSON.stringify(result).includes("super-secret-material"), false);
});

test("env probe honours explicit names and reports absence cleanly", () => {
  const result = envFingerprintProbe({ names: ["PRESENT", "MISSING"] }, { PRESENT: "x" });
  assert.equal(result.ok, true);
  assert.deepEqual(Object.keys(result.matched), ["PRESENT"]);
});

test("file probe matches basenames and fingerprints contents without emitting them", () => {
  const files = {
    "/work/client_secret_abc.json": Buffer.from('{"installed":{"client_secret":"live-material"}}'),
    "/work/README.md": Buffer.from("hello"),
    "/work/nested/id_rsa": Buffer.from("PRIVATE KEY MATERIAL"),
  };
  const fakeFs = {
    readdirSync: (dir) => {
      if (dir === "/work") {
        return [
          { name: "client_secret_abc.json", isFile: () => true, isDirectory: () => false },
          { name: "README.md", isFile: () => true, isDirectory: () => false },
          { name: "nested", isFile: () => false, isDirectory: () => true },
        ];
      }
      if (dir === "/work/nested") {
        return [{ name: "id_rsa", isFile: () => true, isDirectory: () => false }];
      }
      const error = new Error("ENOENT");
      error.code = "ENOENT";
      throw error;
    },
    statSync: (path) => ({ size: files[path].length }),
    readFileSync: (path) => files[path],
  };
  const result = filePatternProbe(
    { patterns: ["client_secret*.json", "id_rsa"], roots: ["/work"] },
    fakeFs,
  );
  assert.equal(result.ok, true);
  assert.equal(result.matched.length, 2);
  for (const item of result.matched) {
    assert.match(item.fingerprint_sha256, /^[0-9a-f]{64}$/);
  }
  const serialized = JSON.stringify(result);
  assert.equal(serialized.includes("live-material"), false);
  assert.equal(serialized.includes("PRIVATE KEY MATERIAL"), false);
});

test("file probe skips dependency dirs but walks hidden dirs (material hides there)", () => {
  const fakeFs = {
    readdirSync: (dir) => {
      if (dir === "/r") {
        return [
          { name: "node_modules", isFile: () => false, isDirectory: () => true },
          { name: ".git", isFile: () => false, isDirectory: () => true },
          { name: ".secret", isFile: () => false, isDirectory: () => true },
        ];
      }
      if (dir === "/r/node_modules" || dir === "/r/.git") {
        return [{ name: "id_rsa", isFile: () => true, isDirectory: () => false }];
      }
      if (dir === "/r/.secret") {
        return [{ name: "client_secret_live.json", isFile: () => true, isDirectory: () => false }];
      }
      const error = new Error("ENOENT");
      error.code = "ENOENT";
      throw error;
    },
    statSync: () => ({ size: 3 }),
    readFileSync: () => Buffer.from("abc"),
  };
  const result = filePatternProbe({ patterns: ["id_rsa", "client_secret*.json"], roots: ["/r"] }, fakeFs);
  assert.equal(result.ok, true);
  assert.deepEqual(
    result.matched.map((item) => item.path),
    ["/r/.secret/client_secret_live.json"],
  );
});

test("file probe surfaces truncation instead of reading a capped walk as complete", () => {
  const fakeFs = {
    readdirSync: (dir) => {
      if (dir === "/r") {
        return [
          { name: "a.pem", isFile: () => true, isDirectory: () => false },
          { name: "b.pem", isFile: () => true, isDirectory: () => false },
          { name: "c.pem", isFile: () => true, isDirectory: () => false },
        ];
      }
      const error = new Error("ENOENT");
      error.code = "ENOENT";
      throw error;
    },
    statSync: () => ({ size: 3 }),
    readFileSync: () => Buffer.from("abc"),
  };
  const result = filePatternProbe({ patterns: ["*.pem"], roots: ["/r"], max_files: 2 }, fakeFs);
  assert.equal(result.ok, true);
  assert.equal(result.truncated, true);
  assert.equal(result.files_scanned, 2);
  assert.equal(result.matched.length, 2);
  const full = filePatternProbe({ patterns: ["*.pem"], roots: ["/r"] }, fakeFs);
  assert.equal(full.truncated, false);
  assert.equal(full.matched.length, 3);
});

test("cli-presence probe resolves executables and captures a version receipt", () => {
  const result = cliPresenceProbe(
    { names: ["definitely-not-a-real-tool-xyz"] },
    { resolveExecutable: () => ({ found: false, path: null }) },
  );
  assert.equal(result.ok, true);
  assert.equal(result.found, false);

  const found = cliPresenceProbe(
    { names: ["fake"] },
    {
      resolveExecutable: () => ({ found: true, path: "/usr/bin/fake" }),
      versionProbe: () => ({ ok: true, version: "fake 1.0.0" }),
    },
  );
  assert.equal(found.found, true);
  assert.equal(found.version, "fake 1.0.0");
});

test("vault-item probe emits metadata only, even when op returns field values", () => {
  const opOutput = JSON.stringify({
    id: "abc123",
    title: "Central Security",
    updated_at: "2026-09-08T00:00:00Z",
    vault: { name: "Central" },
    fields: [
      { label: "credential", value: "LIVE-SECRET-VALUE" },
      { label: "notes", value: "also secret" },
    ],
  });
  const result = vaultItemProbe(
    { item_ref: "op://Central/central-security" },
    { spawnSync: () => ({ status: 0, stdout: opOutput, stderr: "" }) },
  );
  assert.equal(result.ok, true);
  assert.equal(result.found, true);
  assert.equal(result.field_count, 2);
  assert.equal(result.vault, "Central");
  const serialized = JSON.stringify(result);
  assert.equal(serialized.includes("LIVE-SECRET-VALUE"), false);
  assert.equal(serialized.includes("fields"), false);
});

test("vault-item probe reports not-found without failing the scan lane", () => {
  const result = vaultItemProbe(
    { item_ref: "op://Central/missing" },
    { spawnSync: () => ({ status: 1, stdout: "", stderr: "[ERROR] 2026/09/08 item not found" }) },
  );
  assert.equal(result.ok, true);
  assert.equal(result.found, false);
});

const FP = "b".repeat(64);

function fakeEffects(overrides = {}) {
  return {
    env: () => ({ ok: true, matched: {} }),
    "file-pattern": () => ({ ok: true, matched: [] }),
    "cli-presence": () => ({ ok: true, found: false }),
    "vault-item": () => ({ ok: true, found: false }),
    ...overrides,
  };
}

test("scanner: declared sources verify, discovery hits become violations, declared names are excluded", () => {
  const effects = fakeEffects({
    env: (spec) => {
      if (spec.names != null) {
        const matched = {};
        if (spec.names.includes("OP_SERVICE_ACCOUNT_TOKEN")) {
          matched.OP_SERVICE_ACCOUNT_TOKEN = { fingerprint_sha256: FP, byte_length: 40 };
        }
        return { ok: true, matched };
      }
      return {
        ok: true,
        matched: {
          OP_SERVICE_ACCOUNT_TOKEN: { fingerprint_sha256: FP, byte_length: 40 },
          STRAY_API_KEY: { fingerprint_sha256: FP, byte_length: 20 },
        },
      };
    },
    "file-pattern": (spec) => {
      if ((spec.patterns ?? []).includes("client_secret*.json")) {
        return {
          ok: true,
          matched: [{ path: "/work/glade/client_secret_x.json", fingerprint_sha256: FP, byte_length: 100 }],
        };
      }
      return { ok: true, matched: [] };
    },
  });

  const scan = scanSecretSources({
    roots: ["/work"],
    effects,
    scanner: { implementation: "test", version: "0.0.1" },
  });

  assert.equal(scan.schema, SECRET_DETECTION_VERSION);
  const bySlug = Object.fromEntries(scan.sources.map((entry) => [entry.slug, entry]));

  assert.equal(bySlug["op-service-account-token"].state, "verified");
  assert.equal(bySlug["op-service-account-token"].evidence[0].where, "OP_SERVICE_ACCOUNT_TOKEN");
  assert.equal(bySlug["op-vault-security-protocol"].state, "absent");

  const envViolation = scan.sources.find((entry) => entry.slug.startsWith("env-stray-api-key"));
  assert.ok(envViolation, "stray env var becomes a violation");
  assert.equal(envViolation.state, "violation");
  assert.equal(envViolation.violation_class, "uncentralised-env");
  assert.equal(envViolation.centralise_to, "op://Central/central-security");

  const fileViolation = scan.sources.find((entry) => entry.slug.startsWith("file-work-glade-client-secret"));
  assert.ok(fileViolation, "stray client_secret file becomes a violation");
  assert.equal(fileViolation.violation_class, "stray-plaintext");

  // the declared OP_SERVICE_ACCOUNT_TOKEN must NOT appear as a violation
  assert.equal(
    scan.sources.some(
      (entry) => entry.state === "violation" && JSON.stringify(entry).includes("OP_SERVICE_ACCOUNT_TOKEN"),
    ),
    false,
  );
  assert.deepEqual(scan.violations, [envViolation.slug, fileViolation.slug].sort());
  assert.equal(scan.coverage, "complete");
});

test("scanner: unavailable probes degrade coverage with a mandatory reason", () => {
  const effects = fakeEffects({
    "vault-item": () => ({ ok: false, reason: "op not signed in" }),
  });
  const scan = scanSecretSources({
    roots: ["/work"],
    effects,
    scanner: { implementation: "test", version: "0.0.1" },
  });
  const opEntry = scan.sources.find((entry) => entry.slug === "op-vault-security-protocol");
  assert.equal(opEntry.state, "unavailable");
  assert.equal(opEntry.unavailable_reason, "op not signed in");
  assert.equal(scan.coverage, "partial");
});

test("scanner: a truncated file walk degrades the discovery lane to unavailable, never to clean", () => {
  const effects = fakeEffects({
    "file-pattern": () => ({ ok: true, matched: [], truncated: true, files_scanned: 500000 }),
  });
  const scan = scanSecretSources({
    roots: ["/work"],
    effects,
    scanner: { implementation: "test", version: "0.0.1" },
  });
  const truncated = scan.sources.find((entry) => entry.slug.startsWith("file-file-discovery-truncated"));
  assert.ok(truncated, "truncated walk becomes an unavailable entry");
  assert.equal(truncated.state, "unavailable");
  assert.match(truncated.unavailable_reason, /budget before completing/);
  assert.equal(scan.coverage, "partial");
  assert.equal(
    scan.sources.some((entry) => entry.state === "violation" && entry.violation_class === "stray-plaintext"),
    false,
  );
});

test("scanner output contains no value-shaped keys even under adversarial slugs", () => {
  const effects = fakeEffects({
    env: (spec) =>
      spec.name_pattern != null
        ? { ok: true, matched: { WEIRD_TOKEN: { fingerprint_sha256: FP, byte_length: 5 } } }
        : { ok: true, matched: {} },
  });
  const scan = scanSecretSources({
    roots: ["/work"],
    effects,
    scanner: { implementation: "test", version: "0.0.1" },
  });
  const serialized = JSON.stringify(scan);
  for (const forbidden of ['"value"', '"password"', '"secret_value"', '"api_key"', '"material"']) {
    assert.equal(serialized.includes(forbidden), false, `output must not contain ${forbidden}`);
  }
});
