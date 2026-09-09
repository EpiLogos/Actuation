// The harness detection engine: proves the catalog live and derives the
// three-state detection record the contract validates.
import { harnessDetection } from "../contracts/harness-detection.mjs";
import { realEffects } from "./probes.mjs";
import { CATALOG_REVISION } from "./catalog.mjs";

export const DETECTOR_IMPLEMENTATION = "actuation surface-probes";
export const DETECTOR_VERSION = "0.1.0";

function probeRecord(kind, result, spec, detail) {
  const record = { kind, result };
  if (spec != null) record.spec = spec;
  if (detail != null) record.detail = String(detail).slice(0, 200);
  return record;
}

function probeEntry(descriptor, effects) {
  const probes = [];
  const disclosure = [];
  const spec = descriptor.probe;
  const wanted = spec["config-dir"] ? effects.expandHome(spec["config-dir"].path) : null;

  // Executable probe: resolve on PATH; absence is a passing probe with an
  // absence detail, probe errors are failures.
  if (spec.executable) {
    const resolution = effects.resolveExecutable(spec.executable.names);
    if (resolution.error) {
      probes.push(probeRecord("executable", "fail", spec.executable.names.join(" "), resolution.error));
    } else if (resolution.found) {
      probes.push(probeRecord("executable", "pass", spec.executable.names.join(" "), resolution.path));
    } else {
      probes.push(probeRecord("executable", "pass", spec.executable.names.join(" "), "not found on PATH"));
    }
  }

  // Config-dir probe: existence is presence evidence for harnesses whose
  // binary is not on PATH.
  if (spec["config-dir"]) {
    const dirStat = effects.statProbe(wanted);
    if (dirStat.error) {
      probes.push(probeRecord("config-dir", "fail", spec["config-dir"].path, dirStat.error));
    } else {
      probes.push(probeRecord(
        "config-dir", "pass", spec["config-dir"].path,
        dirStat.exists ? `exists at ${wanted}` : "not found",
      ));
    }
  }

  // Service probe: a daemon/live endpoint counts as presence evidence.
  if (spec.service) {
    const live = effects.serviceProbe
      ? effects.serviceProbe(spec.service)
      : { ok: true, detail: "service probe not implemented; declared, not verified" };
    probes.push(probeRecord(
      "service", live.ok === false ? "fail" : "pass", spec.service.kind ?? "service",
      live.detail ?? (live.ok === false ? live.reason : null),
    ));
  }

  // Env probe: runtime identity evidence. A marker set proves "this process
  // runs inside harness X"; it is deliberately NOT presence evidence for the
  // detected state — presence keeps its receipts law (executable/config-dir/
  // service). `harness self` reads these markers for self-identification.
  if (spec.env) {
    const identity = effects.envProbe
      ? effects.envProbe(spec.env.any_of)
      : { ok: true, matched: {} };
    const markerList = identity.ok
      ? (Object.keys(identity.matched).length > 0
        ? Object.keys(identity.matched).map((name) => `${name} set`).join(", ")
        : "no marker set")
      : identity.error;
    probes.push(probeRecord("env", identity.ok ? "pass" : "fail", spec.env.any_of.join(" "), markerList));
  }
  return { probes, disclosure, wanted };
}

// Presence = executable resolved, config dir exists, or a live service probe.
// Service presence is matched on the documented detail conventions of
// probes.mjs ("http <code> from ...", "daemon <name> running"); any other
// pass — including the declared-not-verified fallback — is not presence.
const SERVICE_PRESENCE = /^(http \d{3} from |daemon .+ running$)/;

function hasPresenceEvidence(probes) {
  const executable = probes.find((probe) => probe.kind === "executable");
  const configDir = probes.find((probe) => probe.kind === "config-dir");
  const service = probes.find((probe) => probe.kind === "service");
  return Boolean(
    (executable && executable.result === "pass" && executable.detail && !/not found/.test(executable.detail))
    || (configDir && configDir.result === "pass" && /exists at /.test(configDir.detail ?? ""))
    || (service && service.result === "pass" && SERVICE_PRESENCE.test(service.detail ?? "")),
  );
}

function deriveState(probes) {
  const anyPass = probes.some((probe) => probe.result === "pass");
  const anyFail = probes.some((probe) => probe.result === "fail");
  if (!anyPass && anyFail) {
    return { state: "unavailable", reason: "all probes failed; could not run" };
  }
  if (hasPresenceEvidence(probes)) {
    return { state: "detected" };
  }
  return { state: "not-installed" };
}

function buildReceipts(descriptor, resolution, wanted, effects, disclosure) {
  const receipts = {};
  if (resolution && resolution.found) {
    receipts.executable = resolution.path;
    const hash = effects.hashProbe(resolution.path);
    if (hash.ok) receipts.sha256 = hash.sha256;
    const stat = effects.statProbe(resolution.path);
    if (stat.exists && !stat.isDir) {
      receipts.mtime = Math.round(stat.mtimeMs);
      receipts.size = stat.size;
    }
  }
  if (!receipts.executable && wanted && effects.statProbe(wanted).exists) {
    receipts.executable = wanted;
    receipts.executable_is = "config-dir";
  }
  return receipts;
}

function observeFacets(descriptor, probes, effects, disclosure, now) {
  const observed = [];
  for (const [kind, facet] of Object.entries(descriptor.facets ?? {})) {
    const path = effects.expandHome(facet.path);
    const stat = effects.statProbe(path);
    if (!stat.exists) {
      continue;
    }
    let count;
    if (stat.isDir) {
      // The shared directory probe stays a presence signal: it counts, it does
      // not name. Identities come from the declared inventory below, never
      // from listing a directory that also holds unrelated material.
      const dir = effects.dirCountProbe(path);
      if (dir.exists) count = dir.count;
    }
    const entry = { kind, path: facet.path, exists: true, ...(count != null ? { count } : {}) };
    if (facet.inventory) {
      Object.assign(entry, observeInventory(descriptor, facet.inventory, probes, effects, disclosure, now));
    }
    observed.push(entry);
  }
  return observed;
}

// The declared service endpoint is the authoritative inventory: the provider
// answers with its own names. It is read only when the same run's service
// probe proved the endpoint live — an inventory is never attempted against a
// service we have no evidence is there, and a failed read is disclosed as an
// unavailable reason, never as an empty offering.
function observeInventory(descriptor, declared, probes, effects, disclosure, now) {
  const service = probes.find((probe) => probe.kind === "service");
  const live = service && service.result === "pass" && SERVICE_PRESENCE.test(service.detail ?? "");
  if (!live) {
    const reason = service
      ? `declared service inventory not read: service probe did not prove a live endpoint (${service.detail ?? service.result})`
      : "declared service inventory not read: no service probe ran";
    return { inventory_unavailable_reason: reason };
  }
  const base = descriptor.probe.service.default_url ?? descriptor.probe.service.url;
  const url = `${base.replace(/\/$/, "")}${declared.route}`;
  if (!effects.httpJsonProbe) {
    return { inventory_unavailable_reason: `declared service inventory not read: no http-json probe effect available for ${url}` };
  }
  const answer = effects.httpJsonProbe(url);
  if (!answer.ok) {
    disclosure.push(`${descriptor.slug}: model inventory read failed (${answer.reason ?? "no reason captured"})`);
    return { inventory_unavailable_reason: `inventory read from ${url} failed: ${answer.reason ?? "no reason captured"}` };
  }
  const collection = answer.body?.[declared.collection];
  if (!Array.isArray(collection)) {
    return { inventory_unavailable_reason: `inventory read from ${url} returned no "${declared.collection}" array` };
  }
  const inventory = [];
  const seen = new Set();
  for (const item of collection) {
    if (!item || typeof item !== "object") continue;
    const id = item[declared.id_field];
    if (typeof id !== "string" || id.trim() === "" || seen.has(id)) continue;
    seen.add(id);
    const alsoKnownAs = (declared.also_id_fields ?? [])
      .map((field) => item[field])
      .filter((value) => typeof value === "string" && value.trim() !== "" && value !== id);
    const details = {};
    for (const field of declared.detail_fields ?? []) {
      const value = item[field];
      if (typeof value === "string" || typeof value === "number") details[field] = value;
    }
    inventory.push({
      id,
      ...(alsoKnownAs.length ? { also_known_as: [...new Set(alsoKnownAs)] } : {}),
      ...details,
    });
  }
  return {
    inventory,
    inventory_receipt: {
      kind: declared.kind,
      source: url,
      observed_at: now.toISOString(),
      item_count: inventory.length,
    },
  };
}

/**
 * Run one detection pass over the given descriptors. `effects` is injectable
 * for hermetic tests; the default probes this machine for real. Version
 * probing is opt-in (`probeVersions`): it executes each detected binary with
 * its descriptor-declared version_args, which can be slow and — for some
 * harnesses — can prompt the keychain, so plain detection never spawns
 * harness binaries. Failures are disclosed, never folded into state.
 */
export function runDetection({ descriptors, effects = realEffects(), now = new Date(), probeVersions = false }) {
  const entries = [];
  const disclosure = [];
  for (const descriptor of descriptors) {
    const { probes, wanted } = probeEntry(descriptor, effects);
    const derived = deriveState(probes);
    const entry = {
      slug: descriptor.slug,
      harness_ref: `harness/${descriptor.slug}`,
      native_kind: descriptor.native_kind,
      state: derived.state,
      probes,
    };
    if (derived.reason) entry.unavailable_reason = derived.reason;
    if (derived.state === "detected") {
      const resolution = probes.find((probe) => probe.kind === "executable" && probe.result === "pass");
      entry.receipts = buildReceipts(descriptor, resolution ? { found: true, path: resolution.detail } : null, wanted, effects, disclosure);
      entry.facets = observeFacets(descriptor, probes, effects, disclosure, now);
      if (entry.facets.length === 0) delete entry.facets;
      if (!Object.keys(entry.receipts).length) {
        entry.state = "not-installed";
        delete entry.receipts;
        entry.probes.push(probeRecord("receipts", "fail", null, "presence evidence collapsed; no receipts captured"));
      } else if (probeVersions) {
        enrichWithVersion(entry, descriptor, effects, disclosure);
      }
    }
    entries.push(entry);
  }
  return finishRecord(entries, descriptors, now, disclosure);
}

// Version is receipt enrichment over an already-proven executable: success
// stamps entry.version, failure is disclosed and changes nothing.
function enrichWithVersion(entry, descriptor, effects, disclosure) {
  const executable = entry.receipts.executable;
  if (!executable || entry.receipts.executable_is) return;
  const args = descriptor.probe.executable?.version_args ?? ["--version"];
  const probed = effects.versionProbe(executable, args);
  if (probed.ok) {
    entry.version = probed.version;
  } else {
    disclosure.push(`${descriptor.slug}: version probe failed (${probed.reason ?? "no reason captured"})`);
  }
}

function finishRecord(entries, descriptors, now, disclosure = []) {
  const record = harnessDetection({
    schema: "actuation.harness-detection/v1",
    document: "detection",
    detection_ref: `detection:${now.toISOString()}`,
    observed_at: now.toISOString(),
    catalog_revision: CATALOG_REVISION,
    detector: { implementation: DETECTOR_IMPLEMENTATION, version: DETECTOR_VERSION },
    harnesses: entries,
    absent: entries.filter((entry) => entry.state === "not-installed").map((entry) => entry.slug),
    availability: entries.some((entry) => entry.state === "unavailable") ? "partial" : "complete",
    ...(disclosure.length ? { disclosure } : {}),
  });
  return record;
}
