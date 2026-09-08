// Live secret-source scanner for actuation.secret-detection/v1. Two lanes:
//
//   declared  — run each catalog descriptor's probes; presence becomes
//               verified (fingerprint evidence), proved absence becomes
//               absent, a probe that cannot run becomes unavailable.
//   discovery — enumerate secret material OUTSIDE the declared sources:
//               env names matching the secret-name pattern (minus declared
//               names) and secret-shaped files under the scan roots (minus
//               paths already claimed by declared file-pattern sources).
//               Every discovery hit is a violation with a centralise_to ref.
//
// Usage: node detection/secret-sources/scan.mjs [--roots dir1,dir2] [--json]
import { createHash } from "node:crypto";
import { homedir } from "node:os";
import { join } from "node:path";

import { SECRET_DETECTION_VERSION, secretScan } from "../../contracts/secret-detection.mjs";
import { secretSourceCatalog } from "./catalog.mjs";
import { defaultEffects } from "./probes.mjs";

const ENV_DISCOVERY_PATTERN = "(KEY|TOKEN|SECRET|PASSWORD|PASS|CREDENTIAL)";
const FILE_DISCOVERY_PATTERNS = [
  "client_secret*.json",
  "credentials*.json",
  "*.pem",
  "*.p12",
  "*.pfx",
  "id_rsa",
  "id_ed25519",
  "id_ecdsa",
];
const DEFAULT_ROOTS = ["~/Central/Work"];
const CENTRALISE_TO = "op://Central/central-security";

function expandHome(path) {
  return path.startsWith("~") ? join(homedir(), path.slice(1)) : path;
}

function slugify(prefix, name) {
  const base = name.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
  const short = createHash("sha256").update(name).digest("hex").slice(0, 8);
  return `${prefix}-${base.slice(0, 48)}-${short}`;
}

function runDeclared(descriptor, effects) {
  const probes = [];
  const evidence = [];
  let unavailableReason = null;
  let anyMatch = false;

  for (const [kind, spec] of Object.entries(descriptor.probe)) {
    const effect = effects[kind];
    if (effect == null) {
      unavailableReason = `no effect registered for probe kind ${kind}`;
      probes.push({ kind, result: "fail", spec: JSON.stringify(spec), detail: "effect unavailable" });
      continue;
    }
    const result = effect(spec);
    if (!result.ok) {
      unavailableReason = result.reason ?? `${kind} probe failed`;
      probes.push({ kind, result: "fail", spec: JSON.stringify(spec), detail: unavailableReason });
      continue;
    }
    probes.push({ kind, result: "pass", spec: JSON.stringify(spec) });

    if (kind === "env") {
      for (const [name, item] of Object.entries(result.matched ?? {})) {
        anyMatch = true;
        evidence.push({ where: name, fingerprint_sha256: item.fingerprint_sha256, byte_length: item.byte_length });
      }
    } else if (kind === "file-pattern") {
      for (const item of result.matched ?? []) {
        anyMatch = true;
        evidence.push({
          where: item.path,
          fingerprint_sha256: item.fingerprint_sha256,
          byte_length: item.byte_length,
        });
      }
    } else if (kind === "vault-item") {
      anyMatch = Boolean(result.found);
      if (result.found) {
        probes[probes.length - 1].detail = `item ${result.item_id ?? "?"} in vault ${result.vault ?? "?"}`;
      }
    } else if (kind === "cli-presence") {
      anyMatch = Boolean(result.found);
    }
  }

  const entry = {
    slug: descriptor.slug,
    source_ref: `secret-source/${descriptor.slug}`,
    probes,
  };
  if (unavailableReason != null && !anyMatch) {
    entry.state = "unavailable";
    entry.unavailable_reason = unavailableReason;
  } else if (anyMatch) {
    entry.state = "verified";
    if (evidence.length > 0) entry.evidence = evidence;
  } else {
    entry.state = "absent";
  }
  return entry;
}

function runDiscovery({ declaredEnvNames, declaredPaths, roots, effects }) {
  const entries = [];

  const envResult = effects.env({ name_pattern: ENV_DISCOVERY_PATTERN });
  if (envResult.ok) {
    for (const name of Object.keys(envResult.matched ?? {})) {
      if (declaredEnvNames.has(name)) continue;
      const item = envResult.matched[name];
      entries.push({
        slug: slugify("env", name),
        source_ref: `secret-source/${slugify("env", name)}`,
        state: "violation",
        violation_class: "uncentralised-env",
        centralise_to: CENTRALISE_TO,
        probes: [{ kind: "env", result: "pass", spec: `name_pattern ${ENV_DISCOVERY_PATTERN}` }],
        evidence: [{ where: name, fingerprint_sha256: item.fingerprint_sha256, byte_length: item.byte_length }],
      });
    }
  } else {
    entries.push({
      slug: slugify("env", "env-discovery-unavailable"),
      source_ref: `secret-source/${slugify("env", "env-discovery-unavailable")}`,
      state: "unavailable",
      unavailable_reason: envResult.reason ?? "env discovery probe failed",
      probes: [{ kind: "env", result: "fail", spec: `name_pattern ${ENV_DISCOVERY_PATTERN}` }],
    });
  }

  const fileResult = effects["file-pattern"]({ patterns: FILE_DISCOVERY_PATTERNS, roots });
  if (!fileResult.ok) {
    entries.push({
      slug: slugify("file", "file-discovery-unavailable"),
      source_ref: `secret-source/${slugify("file", "file-discovery-unavailable")}`,
      state: "unavailable",
      unavailable_reason: fileResult.reason ?? "file discovery probe failed",
      probes: [{ kind: "file-pattern", result: "fail", spec: FILE_DISCOVERY_PATTERNS.join(",") }],
    });
  } else if (fileResult.truncated) {
    // A capped walk is "could not complete", never "ran and found nothing":
    // partial discovery results must not masquerade as a clean tree.
    entries.push({
      slug: slugify("file", "file-discovery-truncated"),
      source_ref: `secret-source/${slugify("file", "file-discovery-truncated")}`,
      state: "unavailable",
      unavailable_reason: `file walk hit its ${fileResult.files_scanned}-file budget before completing — discovery results incomplete`,
      probes: [{ kind: "file-pattern", result: "fail", spec: FILE_DISCOVERY_PATTERNS.join(",") }],
    });
  } else {
    for (const item of fileResult.matched ?? []) {
      if (declaredPaths.has(item.path)) continue;
      const slug = slugify("file", item.path);
      entries.push({
        slug,
        source_ref: `secret-source/${slug}`,
        state: "violation",
        violation_class: "stray-plaintext",
        centralise_to: CENTRALISE_TO,
        probes: [{ kind: "file-pattern", result: "pass", spec: FILE_DISCOVERY_PATTERNS.join(",") }],
        evidence: [{ where: item.path, fingerprint_sha256: item.fingerprint_sha256, byte_length: item.byte_length }],
      });
    }
  }

  return entries;
}

export function scanSecretSources({ roots = DEFAULT_ROOTS, effects = defaultEffects, scanner } = {}) {
  const catalog = secretSourceCatalog();
  const expandedRoots = roots.map(expandHome);

  const declaredEnvNames = new Set();
  const declaredPaths = new Set();
  for (const descriptor of catalog.descriptors) {
    if (descriptor.probe.env?.names != null) {
      for (const name of descriptor.probe.env.names) declaredEnvNames.add(name);
    }
  }

  const sources = catalog.descriptors.map((descriptor) => runDeclared(descriptor, effects));
  for (const entry of sources) {
    for (const item of entry.evidence ?? []) {
      if (item.where.startsWith("/")) declaredPaths.add(item.where);
    }
  }

  sources.push(
    ...runDiscovery({ declaredEnvNames, declaredPaths, roots: expandedRoots, effects }),
  );

  const violations = sources.filter((entry) => entry.state === "violation").map((entry) => entry.slug);
  const unavailableCount = sources.filter((entry) => entry.state === "unavailable").length;

  return secretScan({
    schema: SECRET_DETECTION_VERSION,
    document: "scan",
    scan_ref: `secret-scan:${scanner?.implementation ?? "actuation secret-scan"}:${Date.now()}`,
    observed_at: new Date().toISOString(),
    catalog_revision: catalog.catalog_revision,
    scanner: {
      implementation: scanner?.implementation ?? "actuation secret-scan",
      version: scanner?.version ?? "0.1.0",
    },
    sources,
    violations,
    coverage: unavailableCount === 0 ? "complete" : "partial",
    disclosure: [
      "Evidence is fingerprint-only (SHA-256 + byte length + location). No secret value is representable in this record.",
    ],
  });
}

function parseArgs(argv) {
  const args = { roots: null, json: false };
  for (let i = 2; i < argv.length; i += 1) {
    if (argv[i] === "--json") args.json = true;
    if (argv[i] === "--roots") args.roots = argv[i + 1].split(",");
  }
  return args;
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const args = parseArgs(process.argv);
  const scan = scanSecretSources({ roots: args.roots ?? DEFAULT_ROOTS });
  if (args.json) {
    console.log(JSON.stringify(scan, null, 2));
  } else {
    const counts = scan.sources.reduce((acc, entry) => {
      acc[entry.state] = (acc[entry.state] ?? 0) + 1;
      return acc;
    }, {});
    console.log(`secret-scan ${scan.scan_ref}`);
    console.log(`coverage: ${scan.coverage}  states: ${JSON.stringify(counts)}`);
    for (const entry of scan.sources) {
      const where = (entry.evidence ?? []).map((item) => item.where).join(", ");
      console.log(`  ${entry.state.padEnd(12)} ${entry.slug}${where ? ` — ${where}` : ""}`);
    }
    if (scan.violations.length > 0) {
      console.log(`violations (${scan.violations.length}) — centralise to ${CENTRALISE_TO}`);
    }
  }
}
