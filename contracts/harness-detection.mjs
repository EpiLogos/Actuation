export const HARNESS_DETECTION_VERSION = "actuation.harness-detection/v1";

const STATES = new Set(["detected", "unavailable", "not-installed"]);
const PROBE_KINDS = new Set(["executable", "config-dir", "service"]);
const PROBE_RESULTS = new Set(["pass", "fail"]);
const FACET_KINDS = new Set([
  "skills",
  "harness-compositions",
  "plugins",
  "hooks",
  "commands",
  "rules",
  "extensions",
  "agents",
  "settings",
  "config",
  "models",
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

/**
 * One declared harness. Provenance separates what the catalog DECLARES
 * (slug, kind, owner, probe spec, facet paths) from what only a live
 * detection run can VERIFY (presence, version, receipts). A descriptor
 * never asserts presence; that is detection's job.
 */
export function validateHarnessDescriptor(input) {
  const descriptor = record(input, "HarnessDescriptor");
  if (descriptor.schema !== HARNESS_DETECTION_VERSION) {
    throw new TypeError(`HarnessDescriptor.schema must equal ${HARNESS_DETECTION_VERSION}`);
  }
  ref(descriptor.slug, "HarnessDescriptor.slug");
  ref(descriptor.native_kind, "HarnessDescriptor.native_kind");
  ref(descriptor.edition, "HarnessDescriptor.edition", { optional: true });
  ref(descriptor.native_owner, "HarnessDescriptor.native_owner", { optional: true });
  ref(descriptor.summary, "HarnessDescriptor.summary", { optional: true });
  stringArray(descriptor.aliases, "HarnessDescriptor.aliases", { optional: true });

  const probe = record(descriptor.probe, "HarnessDescriptor.probe");
  for (const kind of Object.keys(probe)) {
    if (!PROBE_KINDS.has(kind)) {
      throw new TypeError(`HarnessDescriptor.probe.${kind} is not a supported probe kind (executable, config-dir, service)`);
    }
    record(probe[kind], `HarnessDescriptor.probe.${kind}`);
  }
  if (Object.keys(probe).length === 0) {
    throw new TypeError("HarnessDescriptor.probe must declare at least one probe");
  }

  if (descriptor.facets != null) {
    const facets = record(descriptor.facets, "HarnessDescriptor.facets");
    for (const [kind, facet] of Object.entries(facets)) {
      if (!FACET_KINDS.has(kind)) {
        throw new TypeError(`HarnessDescriptor.facets.${kind} is not a supported facet kind`);
      }
      const shape = record(facet, `HarnessDescriptor.facets.${kind}`);
      ref(shape.path, `HarnessDescriptor.facets.${kind}.path`);
    }
  }

  const provenance = record(descriptor.provenance, "HarnessDescriptor.provenance");
  ref(provenance.authored_by, "HarnessDescriptor.provenance.authored_by");
  stringArray(provenance.source_refs, "HarnessDescriptor.provenance.source_refs", { optional: true });
  ref(provenance.accepted_by, "HarnessDescriptor.provenance.accepted_by", { optional: true });
  if (provenance.catalog_revision == null || !Number.isInteger(provenance.catalog_revision)) {
    throw new TypeError("HarnessDescriptor.provenance.catalog_revision must be an integer");
  }
  return descriptor;
}

export function harnessDescriptor(input) {
  validateHarnessDescriptor(input);
  return structuredClone(input);
}

/**
 * One live detection run over the whole catalog. The three states carry
 * the detection-first law:
 *   detected     — a probe proved presence, receipts captured same-run;
 *   unavailable  — a probe could not run (reason is mandatory, never
 *                  silently read as absence);
 *   not-installed — probes ran and proved absence. "Could not run" is
 *                  never allowed to collapse into "ran and found nothing".
 */
export function validateHarnessDetection(input) {
  const detection = record(input, "HarnessDetection");
  if (detection.schema !== HARNESS_DETECTION_VERSION) {
    throw new TypeError(`HarnessDetection.schema must equal ${HARNESS_DETECTION_VERSION}`);
  }
  ref(detection.detection_ref, "HarnessDetection.detection_ref");
  ref(detection.observed_at, "HarnessDetection.observed_at");
  if (Number.isNaN(Date.parse(detection.observed_at)) ) {
    throw new TypeError("HarnessDetection.observed_at must be an ISO-compatible timestamp");
  }
  if (detection.catalog_revision == null || !Number.isInteger(detection.catalog_revision)) {
    throw new TypeError("HarnessDetection.catalog_revision must be an integer");
  }
  const detector = record(detection.detector, "HarnessDetection.detector");
  ref(detector.implementation, "HarnessDetection.detector.implementation");
  ref(detector.version, "HarnessDetection.detector.version");

  if (!Array.isArray(detection.harnesses)) {
    throw new TypeError("HarnessDetection.harnesses must be an array");
  }
  const seen = new Set();
  const absent = [];
  for (const [index, entry] of detection.harnesses.entries()) {
    const name = `HarnessDetection.harnesses[${index}]`;
    record(entry, name);
    ref(entry.slug, `${name}.slug`);
    ref(entry.harness_ref, `${name}.harness_ref`);
    if (entry.harness_ref !== `harness/${entry.slug}`) {
      throw new TypeError(`${name}.harness_ref must be harness/<slug>`);
    }
    if (!STATES.has(entry.state)) {
      throw new TypeError(`${name}.state must be detected, unavailable or not-installed`);
    }
    if (seen.has(entry.slug)) {
      throw new TypeError(`${name}: duplicate slug ${entry.slug}`);
    }
    seen.add(entry.slug);
    if (entry.state === "unavailable") {
      ref(entry.unavailable_reason, `${name}.unavailable_reason (mandatory when unavailable)`);
    }
    if (entry.state === "detected") {
      const probes = entry.probes ?? [];
      if (!Array.isArray(probes) || !probes.some((probe) => probe.result === "pass")) {
        throw new TypeError(`${name}: state detected requires at least one passing probe (presence must be proved, never assumed)`);
      }
      const receipts = record(entry.receipts, `${name}.receipts (mandatory when detected)`);
      ref(receipts.executable, `${name}.receipts.executable`);
      if (receipts.sha256 != null && !/^[0-9a-f]{64}$/.test(receipts.sha256)) {
        throw new TypeError(`${name}.receipts.sha256 must be 64 lowercase hex characters`);
      }
      if (!Array.isArray(entry.probes)) {
        throw new TypeError(`${name}.probes must be an array when detected`);
      }
    }
    if (entry.state === "not-installed" && Array.isArray(entry.probes) && entry.probes.length === 0) {
      throw new TypeError(`${name}: not-installed requires probe evidence of absence, not an empty probe list`);
    }
    if (entry.state === "not-installed" && (entry.facets ?? []).some((facet) => facet.exists)) {
      throw new TypeError(`${name}: an absent harness cannot have observed facets (facet evidence implies presence)`);
    }
    const facets = entry.facets ?? [];
    if (!Array.isArray(facets)) {
      throw new TypeError(`${name}.facets must be an array when present`);
    }
    for (const [facetIndex, facet] of facets.entries()) {
      record(facet, `${name}.facets[${facetIndex}]`);
      if (!FACET_KINDS.has(facet.kind)) {
        throw new TypeError(`${name}.facets[${facetIndex}].kind is not a supported facet kind`);
      }
      ref(facet.path, `${name}.facets[${facetIndex}].path`);
      if (typeof facet.exists !== "boolean") {
        throw new TypeError(`${name}.facets[${facetIndex}].exists must be a boolean`);
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
  }
  const notInstalled = detection.harnesses
    .filter((entry) => entry.state === "not-installed")
    .map((entry) => entry.slug);
  if (JSON.stringify(notInstalled) !== JSON.stringify(detection.absent ?? [])) {
    throw new TypeError("HarnessDetection.absent must list exactly the not-installed slugs, in order");
  }
  const unavailableCount = detection.harnesses.filter((entry) => entry.state === "unavailable").length;
  const expectedAvailability = unavailableCount === 0 ? "complete" : "partial";
  if (detection.availability !== expectedAvailability) {
    throw new TypeError(`HarnessDetection.availability must be "${expectedAvailability}" (${unavailableCount} unavailable)`);
  }
  stringArray(detection.disclosure, "HarnessDetection.disclosure", { optional: true });
  return detection;
}

export function harnessDetection(input) {
  validateHarnessDetection(input);
  return structuredClone(input);
}

