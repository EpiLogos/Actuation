// Secret-source detection contract, actuation.secret-detection/v1.
// Extends the harness-detection primitive (declared descriptor + probe +
// evidence + schema-validated record) to secret material. The load-bearing
// law: a detector that can emit a value is itself a leak source, so this
// contract makes values unrepresentable — evidence carries fingerprints
// only, and the validator refuses any record that carries a value-shaped
// key anywhere inside it.
export const SECRET_DETECTION_VERSION = "actuation.secret-detection/v1";

const SOURCE_KINDS = new Set([
  "op-item",
  "keychain-entry",
  "varlock-blob",
  "env-var",
  "plaintext-file",
]);

const PROBE_KINDS = new Set(["env", "file-pattern", "cli-presence", "vault-item"]);
const PROBE_RESULTS = new Set(["pass", "fail"]);

const SCAN_STATES = new Set(["verified", "violation", "absent", "unavailable"]);

const VIOLATION_CLASSES = new Set([
  "stray-plaintext",
  "uncentralised-env",
  "legacy-env-ref",
  "unknown",
]);

// Keys that must never appear anywhere inside a scan record. This is the
// mechanical backstop for the fingerprint-only law: not "don't fill them",
// but "their presence invalidates the record".
const FORBIDDEN_VALUE_KEYS = new Set([
  "value",
  "material",
  "secret_value",
  "secretvalue",
  "plaintext",
  "secret",
  "password",
  "token_value",
  "api_key",
  "apikey",
  "credential",
]);

function record(value, name) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new TypeError(`${name} must be an object`);
  }
  return value;
}

function ref(value, name, { optional = false } = {}) {
  if (value == null && optional) return;
  if (typeof value !== "string" || value.trim() === "") {
    throw new TypeError(`${name} must be a non-empty string ref`);
  }
}

function stringArray(value, name, { optional = false } = {}) {
  if (value == null && optional) return;
  if (!Array.isArray(value) || value.some((item) => typeof item !== "string" || item.trim() === "")) {
    throw new TypeError(`${name} must be an array of non-empty strings`);
  }
}

function fingerprint(value, name, { optional = false } = {}) {
  if (value == null && optional) return;
  if (typeof value !== "string" || !/^[0-9a-f]{64}$/.test(value)) {
    throw new TypeError(`${name} must be 64 lowercase hex characters (a SHA-256 fingerprint, never a value)`);
  }
}

function assertNoValueKeys(node, path, seen = new Set()) {
  if (node == null || seen.has(node)) return;
  if (typeof node === "object") {
    seen.add(node);
    for (const [key, child] of Object.entries(node)) {
      if (FORBIDDEN_VALUE_KEYS.has(key.toLowerCase())) {
        throw new TypeError(
          `scan record carries forbidden value-shaped key "${key}" at ${path} — ` +
            "detection evidence is fingerprint-only, a detector must never emit material",
        );
      }
      assertNoValueKeys(child, `${path}.${key}`, seen);
    }
  }
}

/**
 * One declared secret source. Provenance separates what the catalog DECLARES
 * (slug, source kind, ref-scheme support, probe spec) from what only a live
 * scan run can VERIFY (presence, fingerprint match, violation class). A
 * descriptor never asserts presence, and never carries material.
 */
export function validateSecretSourceDescriptor(input) {
  const descriptor = record(input, "SecretSourceDescriptor");
  if (descriptor.schema !== SECRET_DETECTION_VERSION) {
    throw new TypeError(`SecretSourceDescriptor.schema must equal ${SECRET_DETECTION_VERSION}`);
  }
  if (descriptor.document !== "descriptor") {
    throw new TypeError("SecretSourceDescriptor.document must be descriptor");
  }
  ref(descriptor.slug, "SecretSourceDescriptor.slug");
  if (!SOURCE_KINDS.has(descriptor.source_kind)) {
    throw new TypeError(
      `SecretSourceDescriptor.source_kind is not a supported kind (${[...SOURCE_KINDS].join(", ")})`,
    );
  }
  // Which secret-ref prefixes this source can satisfy (central.security/v1
  // scheme: op://, keychain://, varlock://, env:// legacy-only).
  stringArray(descriptor.ref_schemes, "SecretSourceDescriptor.ref_schemes");

  const probe = record(descriptor.probe, "SecretSourceDescriptor.probe");
  for (const kind of Object.keys(probe)) {
    if (!PROBE_KINDS.has(kind)) {
      throw new TypeError(`SecretSourceDescriptor.probe.${kind} is not a supported probe kind`);
    }
    record(probe[kind], `SecretSourceDescriptor.probe.${kind}`);
  }
  if (probe.env != null) {
    const spec = probe.env;
    if (spec.names != null) stringArray(spec.names, "probe.env.names");
    if (spec.name_pattern != null) ref(spec.name_pattern, "probe.env.name_pattern");
    if (spec.names == null && spec.name_pattern == null) {
      throw new TypeError("probe.env requires names or name_pattern");
    }
  }
  if (probe["file-pattern"] != null) {
    stringArray(
      probe["file-pattern"].patterns,
      "probe.file-pattern.patterns (basenames or glob segments)",
    );
    if (probe["file-pattern"].roots != null) {
      stringArray(probe["file-pattern"].roots, "probe.file-pattern.roots");
    }
  }
  if (probe["cli-presence"] != null) {
    stringArray(probe["cli-presence"].names, "probe.cli-presence.names");
  }
  if (probe["vault-item"] != null) {
    ref(probe["vault-item"].item_ref, "probe.vault-item.item_ref");
  }
  if (Object.keys(probe).length === 0) {
    throw new TypeError("SecretSourceDescriptor.probe must declare at least one probe");
  }

  const provenance = record(descriptor.provenance, "SecretSourceDescriptor.provenance");
  ref(provenance.authored_by, "SecretSourceDescriptor.provenance.authored_by");
  stringArray(provenance.source_refs, "SecretSourceDescriptor.provenance.source_refs", { optional: true });
  if (provenance.catalog_revision == null || !Number.isInteger(provenance.catalog_revision)) {
    throw new TypeError("SecretSourceDescriptor.provenance.catalog_revision must be an integer");
  }
  return descriptor;
}

export function secretSourceDescriptor(input) {
  validateSecretSourceDescriptor(input);
  return structuredClone(input);
}

/**
 * One live scan run over the declared sources plus the discovery lanes
 * (env enumeration, file patterns) that find material OUTSIDE declared
 * sources — those become violations. States carry the detection-first law:
 *   verified    — a declared source was found and conforms (fingerprint
 *                 captured, no value);
 *   violation   — secret material was found outside declared sources;
 *                 violation_class and centralise_to are mandatory;
 *   absent      — probes ran and proved absence;
 *   unavailable — a probe could not run (reason mandatory, never read as
 *                 absence).
 */
export function validateSecretScan(input) {
  const scan = record(input, "SecretScan");
  if (scan.schema !== SECRET_DETECTION_VERSION) {
    throw new TypeError(`SecretScan.schema must equal ${SECRET_DETECTION_VERSION}`);
  }
  if (scan.document !== "scan") {
    throw new TypeError("SecretScan.document must be scan");
  }
  ref(scan.scan_ref, "SecretScan.scan_ref");
  ref(scan.observed_at, "SecretScan.observed_at");
  if (Number.isNaN(Date.parse(scan.observed_at))) {
    throw new TypeError("SecretScan.observed_at must be an ISO-compatible timestamp");
  }
  if (scan.catalog_revision == null || !Number.isInteger(scan.catalog_revision)) {
    throw new TypeError("SecretScan.catalog_revision must be an integer");
  }
  const scanner = record(scan.scanner, "SecretScan.scanner");
  ref(scanner.implementation, "SecretScan.scanner.implementation");
  ref(scanner.version, "SecretScan.scanner.version");

  if (!Array.isArray(scan.sources)) {
    throw new TypeError("SecretScan.sources must be an array");
  }
  const seen = new Set();
  for (const [index, entry] of scan.sources.entries()) {
    const name = `SecretScan.sources[${index}]`;
    record(entry, name);
    ref(entry.slug, `${name}.slug`);
    ref(entry.source_ref, `${name}.source_ref`);
    if (entry.source_ref !== `secret-source/${entry.slug}`) {
      throw new TypeError(`${name}.source_ref must be secret-source/<slug>`);
    }
    if (!SCAN_STATES.has(entry.state)) {
      throw new TypeError(`${name}.state must be verified, violation, absent or unavailable`);
    }
    if (seen.has(entry.slug)) {
      throw new TypeError(`${name}: duplicate slug ${entry.slug}`);
    }
    seen.add(entry.slug);
    if (entry.state === "unavailable") {
      ref(entry.unavailable_reason, `${name}.unavailable_reason (mandatory when unavailable)`);
    }
    if (entry.state === "violation") {
      if (!VIOLATION_CLASSES.has(entry.violation_class)) {
        throw new TypeError(
          `${name}.violation_class must be one of ${[...VIOLATION_CLASSES].join(", ")}`,
        );
      }
      ref(
        entry.centralise_to,
        `${name}.centralise_to (mandatory when violation — where the material must move)`,
      );
    }
    if (entry.state !== "violation" && entry.violation_class != null) {
      throw new TypeError(`${name}: only violations carry a violation_class`);
    }
    if (entry.state === "verified") {
      const probes = entry.probes ?? [];
      if (!Array.isArray(probes) || !probes.some((probe) => probe.result === "pass")) {
        throw new TypeError(`${name}: verified requires at least one passing probe (presence must be proved)`);
      }
    }
    const probesAll = entry.probes ?? [];
    if (!Array.isArray(probesAll)) {
      throw new TypeError(`${name}.probes must be an array when present`);
    }
    for (const [probeIndex, probe] of probesAll.entries()) {
      record(probe, `${name}.probes[${probeIndex}]`);
      if (!PROBE_KINDS.has(probe.kind)) {
        throw new TypeError(`${name}.probes[${probeIndex}].kind is not a supported probe kind`);
      }
      if (!PROBE_RESULTS.has(probe.result)) {
        throw new TypeError(`${name}.probes[${probeIndex}].result must be pass or fail`);
      }
      ref(probe.spec, `${name}.probes[${probeIndex}].spec`, { optional: true });
      ref(probe.detail, `${name}.probes[${probeIndex}].detail`, { optional: true });
    }
    const evidence = entry.evidence ?? [];
    if (!Array.isArray(evidence)) {
      throw new TypeError(`${name}.evidence must be an array when present`);
    }
    for (const [evIndex, item] of evidence.entries()) {
      record(item, `${name}.evidence[${evIndex}]`);
      ref(item.where, `${name}.evidence[${evIndex}].where (location only: name, path, ref)`);
      fingerprint(item.fingerprint_sha256, `${name}.evidence[${evIndex}].fingerprint_sha256`);
      if (item.byte_length != null && (!Number.isInteger(item.byte_length) || item.byte_length < 0)) {
        throw new TypeError(`${name}.evidence[${evIndex}].byte_length must be a non-negative integer`);
      }
    }
  }

  const violations = scan.sources.filter((entry) => entry.state === "violation").map((entry) => entry.slug);
  if (JSON.stringify(violations) !== JSON.stringify(scan.violations ?? [])) {
    throw new TypeError("SecretScan.violations must list exactly the violation slugs, in order");
  }
  const unavailableCount = scan.sources.filter((entry) => entry.state === "unavailable").length;
  const expectedCoverage = unavailableCount === 0 ? "complete" : "partial";
  if (scan.coverage !== expectedCoverage) {
    throw new TypeError(`SecretScan.coverage must be "${expectedCoverage}" (${unavailableCount} unavailable)`);
  }
  stringArray(scan.disclosure, "SecretScan.disclosure", { optional: true });

  // The mechanical backstop, run last over the whole record: no value-shaped
  // key anywhere, at any depth.
  assertNoValueKeys(scan, "SecretScan");
  return scan;
}

export function secretScan(input) {
  validateSecretScan(input);
  return structuredClone(input);
}

/**
 * The declared secret-source catalog: what this product COULD detect,
 * independent of any live run. Same declare-vs-verify separation as the
 * harness catalog.
 */
export function validateSecretSourceCatalog(input) {
  const catalog = record(input, "SecretSourceCatalog");
  if (catalog.schema !== SECRET_DETECTION_VERSION) {
    throw new TypeError(`SecretSourceCatalog.schema must equal ${SECRET_DETECTION_VERSION}`);
  }
  if (catalog.document !== "catalog") {
    throw new TypeError("SecretSourceCatalog.document must be catalog");
  }
  if (catalog.catalog_revision == null || !Number.isInteger(catalog.catalog_revision)) {
    throw new TypeError("SecretSourceCatalog.catalog_revision must be an integer");
  }
  if (!Array.isArray(catalog.descriptors) || catalog.descriptors.length === 0) {
    throw new TypeError("SecretSourceCatalog.descriptors must be a non-empty array");
  }
  const seen = new Set();
  for (const [index, descriptor] of catalog.descriptors.entries()) {
    validateSecretSourceDescriptor(descriptor);
    if (seen.has(descriptor.slug)) {
      throw new TypeError(`SecretSourceCatalog.descriptors[${index}]: duplicate slug ${descriptor.slug}`);
    }
    seen.add(descriptor.slug);
  }
  assertNoValueKeys(catalog, "SecretSourceCatalog");
  return catalog;
}

export function secretSourceCatalog(input) {
  validateSecretSourceCatalog(input);
  return structuredClone(input);
}
