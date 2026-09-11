// Actuation's Wave 5 System contribution: the native settings disclosure
// document (oi.product-settings-disclosure/v2, wave-5/system.1).
//
// This module is a READ-ONLY projection. It never mutates state, never
// fabricates a live value, and never reaches past the owner seam. It derives
// every fact from the existing command surface (cli/surface.mjs), the existing
// contract version constants, and one live harness-detection pass — the same
// read-only probes `actuation harness detect` runs. There is no settings
// database, no duplicate action catalogue, and no duplicate provider registry
// here: the actions below are a disclosure-grade projection of the single
// command table, annotated with the seam metadata (availability / exposure /
// authority) that the command table does not carry.
import { createHash } from "node:crypto";
import { existsSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { ACTUATION_CLI_SURFACE, ACTUATION_CLI_VERSION } from "../cli/surface.mjs";
import { SYSTEM_DISCLOSURE_VERSION, SYSTEM_DISCLOSURE_CONTRACT_REVISION } from "./system-disclosure-version.mjs";
import { AGENCY_CONTRACT_VERSION } from "./agency.mjs";
import { ACTIVITY_VERSION } from "./activity.mjs";
import { ACTUATION_STREAM_VERSION } from "./actuation-stream.mjs";
import { runDetection } from "../detection/detect.mjs";
import { resolveSelf } from "../detection/self.mjs";
import { harnessDescriptors, CATALOG_REVISION } from "../detection/catalog.mjs";
import { ACTUATION_STREAM_STORE_DEFAULT_ROOT, loadStreamFile, streamStoreRoot } from "./actuation-stream-store.mjs";

export { SYSTEM_DISCLOSURE_VERSION, SYSTEM_DISCLOSURE_CONTRACT_REVISION };

const PRODUCT_ID = "actuation";
const OWNER_REF = "actuation:cli";
const READING_COMMAND = ["actuation", "system", "--json"];

// provenance.path is a location, not a command (§4.6). Each owner-namespace ref
// names the real source file the value comes from; the command that reads it
// belongs in native_path, never in provenance.path.
const PATH = Object.freeze({
  surface: "cli/surface.mjs",
  commands: "cli/commands.mjs",
  system: "contracts/system-disclosure.mjs",
  agency: "contracts/agency.mjs",
  agencyActualisation: "contracts/agency-actualisation.mjs",
  catalog: "detection/catalog.mjs",
  detect: "detection/detect.mjs",
  self: "detection/self.mjs",
  streamStore: "contracts/actuation-stream-store.mjs",
});

// Authority is never inferred from UI location or root-agent identity. The
// read/verify actions below require no authority (they are read-only and the
// owner self-authorises them); the single actualising action names its exact
// MetagencyGrant authority and is refused without it. Nothing here grants
// authority because a caller happens to be "root" or sits on a particular
// surface.
const NO_AUTHORITY = Object.freeze({ requires: [], granted_by: "actuation:cli", evidence_ref: null });
const NO_EXPLAIN = Object.freeze({ ref: null, command: null });
const NO_HISTORY = Object.freeze({ ref: null, command: null });

function provenance(ownerRef, path, observedAtMs) {
  return { owner_ref: ownerRef, path, observed_at_unix_ms: observedAtMs };
}

function axis(value, prov) {
  return { value, provenance: prov };
}

function stagedAxis(prov, { stageState = "none", stageRef = null } = {}) {
  return {
    value: null,
    provenance: prov,
    stage_ref: stageRef,
    stage_state: stageState,
  };
}

function expectedEffect({ summary = null, ref = null } = {}) {
  return {
    summary: summary ?? "No stage prepared; this setting is declared code, not applied configuration.",
    ref,
  };
}

function drift(state, between = ["declared", "effective"], remediationActionRef = null) {
  return { state, between, remediation_action_ref: remediationActionRef };
}

// §4.5: the canonical reading body is the whole descriptor with every
// *_unix_ms field zeroed and reading_digest itself held null. Hashing that
// body makes the digest a function of the reading, never of the clock.
export function canonicalReadingBody(descriptor) {
  const body = structuredClone(descriptor);
  const zeroClock = (node) => {
    if (Array.isArray(node)) {
      node.forEach(zeroClock);
      return;
    }
    if (node && typeof node === "object") {
      for (const key of Object.keys(node)) {
        if (key.endsWith("_unix_ms")) node[key] = 0;
        else zeroClock(node[key]);
      }
    }
  };
  zeroClock(body);
  body.owner.reading_digest = null;
  return body;
}

// §4.7: availability is probed, not asserted. Owner-level availability is
// derived from the live detection pass; a partial detection (a probe that
// failed, not a clean absence) degrades the reading.
export function deriveAvailability(detection) {
  if (detection.availability === "complete") {
    return { state: "available", reason: null };
  }
  return {
    state: "degraded",
    reason: "harness detection is partial: at least one probe failed (see degradations)",
  };
}

// §4.3: per-subject degradations are separate from owner-level availability,
// and are derived from the same live detection pass, never a hardcoded list.
// A harness whose probes failed is unavailable with a reason; a clean absence
// (not-installed) is not a degradation of this owner.
export function deriveDegradations(detection) {
  return detection.harnesses
    .filter((entry) => entry.state === "unavailable")
    .map((entry) => ({
      subject_ref: entry.harness_ref,
      state: "unavailable",
      reason: entry.unavailable_reason ?? "all probes failed; could not run",
      native_error: null,
    }));
}

// The durable stream store's active state is a real filesystem observation:
// whether the effective default root exists, and how many streams in it are
// genuinely open. Openness is read from each stream's own lifecycle state —
// the header line of its durable file — never inferred from the presence of a
// `.jsonl` file. The authored default root lives in the declared axis; the
// env-overridden root is the effective axis.
function defaultStreamStoreState() {
  const root = streamStoreRoot({});
  const state = { root, exists: false, in_use: false, open_streams: 0, closed_streams: 0 };
  let exists;
  try {
    exists = existsSync(root);
  } catch {
    state.exists = null;
    state.reason = "default stream store root is not observable on this machine";
    return state;
  }
  state.exists = exists;
  if (!exists) return state;

  let names;
  try {
    names = readdirSync(root).filter((name) => name.endsWith(".jsonl"));
  } catch (error) {
    state.in_use = null;
    state.reason = `default stream store root exists but is not readable: ${error.message}`;
    return state;
  }

  // Each `.jsonl` file is one durable stream. Its lifecycle.state is the only
  // truth about whether it is open: count genuinely open streams, and treat a
  // stream whose lifecycle cannot be determined as neither open nor closed,
  // naming it instead of guessing.
  const undetermined = [];
  let open = 0;
  let closed = 0;
  for (const name of names) {
    try {
      const stream = loadStreamFile(join(root, name));
      if (stream == null) {
        undetermined.push({ file: name, reason: "stream file vanished during the scan" });
        continue;
      }
      if (stream.lifecycle.state === "open") open += 1;
      else closed += 1;
    } catch (error) {
      undetermined.push({ file: name, reason: error.message });
    }
  }
  state.open_streams = open;
  state.closed_streams = closed;
  state.in_use = open > 0;
  if (undetermined.length > 0) {
    state.undetermined_streams = undetermined.length;
    state.reason = `${undetermined.length} stream(s) lifecycle undetermined; counted as neither open nor closed: ${undetermined
      .map((entry) => `${entry.file} (${entry.reason})`)
      .join("; ")}`;
  }
  return state;
}

// A declared/effective/active triple over a static contract fact: the value is
// authored in code and validated live, but nothing is materialised at runtime
// (Actuation performs no materialisation by design). All three axes are still
// present — the agreement (and the empty active) is itself the information.
function contractSetting(key, title, kind, declaredValue, declaredPath, observedAtMs) {
  const declared = axis(structuredClone(declaredValue), provenance(OWNER_REF, declaredPath, observedAtMs));
  const effective = axis(structuredClone(declaredValue), provenance("actuation:cli:capabilities", PATH.surface, observedAtMs));
  const active = axis(null, provenance("actuation:cli:system", PATH.system, observedAtMs));
  return {
    key,
    title,
    kind,
    axes: {
      declared,
      effective,
      active,
      staged: stagedAxis(provenance(OWNER_REF, declaredPath, observedAtMs)),
      expected_effect: expectedEffect(),
    },
    mutable: false,
    native_path: "actuation (read-only projection; no mutation disclosed)",
    bootstrap: false,
    drift: drift("none"),
  };
}

// A declared/effective/active triple over an enumerated vocabulary that the
// contract validators accept. declared = authored vocabulary, effective = the
// same vocabulary enforced by the validator, active = empty (no live
// determinations/grants/returns are materialised by this product).
function vocabularySetting(key, title, declaredValue, declaredPath, observedAtMs) {
  const declared = axis(structuredClone(declaredValue), provenance(OWNER_REF, declaredPath, observedAtMs));
  const effective = axis(structuredClone(declaredValue), provenance("actuation:cli:agency", PATH.agency, observedAtMs));
  const active = axis([], provenance("actuation:cli:system", PATH.system, observedAtMs));
  return {
    key,
    title,
    kind: "table",
    axes: {
      declared,
      effective,
      active,
      staged: stagedAxis(provenance(OWNER_REF, declaredPath, observedAtMs)),
      expected_effect: expectedEffect(),
    },
    mutable: false,
    native_path: "actuation (read-only projection; no mutation disclosed)",
    bootstrap: false,
    drift: drift("none"),
  };
}

function action({
  actionRef, title, args = [], availability, unavailableReason = null,
  subjectKinds = [], authority = NO_AUTHORITY, exposure = { ui: false, agent: false, headless: true },
  explain = NO_EXPLAIN, history = NO_HISTORY,
}) {
  return {
    action_ref: actionRef,
    title,
    args,
    availability,
    unavailable_reason: unavailableReason,
    subject_kinds: subjectKinds,
    authority,
    exposure,
    explain,
    history,
  };
}

const HEADLESS = Object.freeze({ ui: false, agent: true, headless: true });
const NATIVE_ONLY = Object.freeze({ ui: false, agent: false, headless: true });

// The canonical Actions. The five states are deliberately kept distinct and
// named: exists (a command is in the table), selected (a surface picks it for
// a subject — not a fact this owner asserts), exposed (exposure.ui/agent/
// headless), authorised (authority.requires/granted_by), invoked (a receipt
// returns). This list is a projection of the command table, not a second
// catalogue: the test suite asserts every disclosed action_ref matches a
// command name.
function canonicalActions() {
  const read = (actionRef, title, args, subjectKinds) => action({
    actionRef, title, args, availability: "disclosed",
    subjectKinds, authority: NO_AUTHORITY, exposure: HEADLESS,
  });
  return [
    read("capabilities", "Read the CLI surface and native contract versions", [], ["actuation.cli"]),
    read("contract.list", "List native contract versions", [], ["actuation.contract"]),
    read("harness.catalog", "Declare the harness detection catalog", [], ["actuation.harness"]),
    read("harness.detect", "Prove harness presence live on this machine", [], ["actuation.harness"]),
    read("harness.self", "Identify the harness this process runs inside", [], ["actuation.harness"]),
    read("verify", "Run the native verification gate (test suites)", [], ["actuation.verification"]),
    read("agency.read", "Project an Agency read model", [{ name: "document", kind: "json" }], ["actuation.agency"]),
    read("realised.read", "Project a realised-actuation read model", [{ name: "document", kind: "json" }], ["actuation.realised"]),
    read("stream.read", "Project an ActuationStream read model", [{ name: "document", kind: "json" }], ["actuation.stream"]),
    read("activity.read", "Validate and project an Activity read model", [{ name: "document", kind: "json" }], ["actuation.activity"]),
    read("instantiation.read", "Validate an instantiation receipt", [{ name: "document", kind: "json" }], ["actuation.instantiation"]),
    action({
      actionRef: "agency.actualise",
      title: "Actualise a determination as a semantic relation",
      args: [{ name: "document", kind: "json" }],
      availability: "unavailable",
      unavailableReason: "Native CLI operation; Actuation discloses no O:I intent/invoke seam this wave and performs no materialisation (receipt effect: materialisation not-performed).",
      subjectKinds: ["actuation.agency", "actuation.determination"],
      authority: {
        requires: ["metagency-grant:determine-agency", "metagency-grant:actualise-agency (derivation only)"],
        granted_by: "the governing Agency's exact MetagencyGrant (authority is never derived from caller identity or surface position)",
        evidence_ref: "contracts/agency-actualisation.mjs",
      },
      exposure: NATIVE_ONLY,
    }),
    action({
      actionRef: "stream.durable",
      title: "Durable stream open/record/replay/close",
      args: [{ name: "store", kind: "path" }, { name: "document", kind: "json" }],
      availability: "unavailable",
      unavailableReason: "Native CLI operation; no intent/invoke seam disclosed. Store is caller-supplied (--store <dir>); there is no product-owned live stream registry.",
      subjectKinds: ["actuation.stream"],
      authority: NO_AUTHORITY,
      exposure: NATIVE_ONLY,
    }),
    action({
      actionRef: "instantiation.record",
      title: "Append a bound instantiation receipt (JSONL)",
      args: [{ name: "document", kind: "json" }, { name: "out", kind: "path" }],
      availability: "unavailable",
      unavailableReason: "Native CLI operation; no intent/invoke seam disclosed. Writes caller-supplied JSONL only.",
      subjectKinds: ["actuation.instantiation"],
      authority: NO_AUTHORITY,
      exposure: NATIVE_ONLY,
    }),
    action({
      actionRef: "stream.usage",
      title: "Record a model-usage observation into a durable stream",
      args: [{ name: "adapter", kind: "string" }, { name: "document", kind: "json" }],
      availability: "unavailable",
      unavailableReason: "Native CLI operation; no intent/invoke seam disclosed.",
      subjectKinds: ["actuation.model-usage"],
      authority: NO_AUTHORITY,
      exposure: NATIVE_ONLY,
    }),
    action({
      actionRef: "ecology.read",
      title: "Read the live actor ecology (instantiated/realised actuations)",
      args: [],
      availability: "missing_native_obligation",
      subjectKinds: ["actuation.ecology"],
      authority: NO_AUTHORITY,
      exposure: { ui: false, agent: false, headless: false },
    }),
    action({
      actionRef: "attach",
      title: "Attach an observer to a live actor/session",
      args: [{ name: "actor_ref", kind: "string" }],
      availability: "missing_native_obligation",
      subjectKinds: ["actuation.ecology"],
      authority: NO_AUTHORITY,
      exposure: { ui: false, agent: false, headless: false },
    }),
  ];
}

/**
 * Build the Wave 5 System disclosure for Actuation. Read-only: it runs the
 * same live harness-detection and self-identification probes the CLI already
 * runs, then projects the frozen descriptor. `now` is injectable for hermetic
 * tests.
 */
export function buildSystemDisclosure({ now = new Date() } = {}) {
  const observedAtMs = now.getTime();
  const detection = runDetection({ descriptors: harnessDescriptors() });
  const self = resolveSelf({ descriptors: harnessDescriptors() });

  const descriptors = harnessDescriptors();
  const declaredCount = descriptors.length;
  const detectedCount = detection.harnesses.filter((entry) => entry.state === "detected").length;

  const sections = [
    {
      id: "agency",
      title: "Agent / Agency / WorldBinding",
      settings: [
        contractSetting("agency.contract", "Agency contract", "scalar", AGENCY_CONTRACT_VERSION, "contracts/agency.mjs", observedAtMs),
        vocabularySetting("agency.determination.kinds", "Determination kinds", ["self-differentiation", "delegation", "derivation", "federation"], "contracts/agency.mjs", observedAtMs),
        vocabularySetting(
          "agency.world_binding.constraints",
          "WorldBinding constraint categories",
          ["human_authored_refs", "security_policy_refs", "evidence_refs", "external_reality_refs"],
          "contracts/agency.mjs",
          observedAtMs,
        ),
      ],
    },
    {
      id: "authority",
      title: "Authority / Metagency",
      settings: [
        vocabularySetting("authority.metagency.operations", "Metagency operations", ["determine-agency", "configure-agency", "actualise-agency", "reintegrate-return"], "contracts/agency.mjs", observedAtMs),
        contractSetting("authority.derivation.rule", "Derivation authority requirement", "scalar", "Derivation requires explicit actualise-agency authority and an explicitly actualised Agent identity", "contracts/agency-actualisation.mjs", observedAtMs),
        contractSetting("authority.federation.rule", "Federation authority rule", "scalar", "Federation cannot silently carry determining authority; use an explicit delegation", "contracts/agency.mjs", observedAtMs),
      ],
    },
    {
      id: "availability",
      title: "Actual actuation availability",
      settings: [
        {
          key: "availability.contracts",
          title: "Native contract surface",
          kind: "table",
          axes: {
            declared: axis(structuredClone(ACTUATION_CLI_SURFACE.native_contracts), provenance(OWNER_REF, PATH.surface, observedAtMs)),
            effective: axis(structuredClone(ACTUATION_CLI_SURFACE.native_contracts), provenance("actuation:cli:capabilities", PATH.surface, observedAtMs)),
            active: axis(structuredClone(ACTUATION_CLI_SURFACE.native_contracts), provenance("actuation:cli:system", PATH.system, observedAtMs)),
            staged: stagedAxis(provenance(OWNER_REF, "cli/surface.mjs", observedAtMs)),
            expected_effect: expectedEffect(),
          },
          mutable: false,
          native_path: "actuation (read-only projection; no mutation disclosed)",
          bootstrap: false,
          drift: drift("none"),
        },
        {
          key: "availability.harness.catalog",
          title: "Harness detection catalog vs live detection",
          kind: "table",
          axes: {
            declared: axis(
              { catalog_revision: CATALOG_REVISION, descriptor_count: declaredCount, slugs: descriptors.map((d) => d.slug) },
              provenance("actuation:harness:catalog", PATH.catalog, observedAtMs),
            ),
            effective: axis(
              {
                states: Object.fromEntries(detection.harnesses.map((entry) => [entry.slug, entry.state])),
                detected: detection.harnesses.filter((e) => e.state === "detected").map((e) => e.slug),
                unavailable: detection.harnesses.filter((e) => e.state === "unavailable").map((e) => e.slug),
                not_installed: detection.harnesses.filter((e) => e.state === "not-installed").map((e) => e.slug),
              },
              provenance("actuation:harness:detect", PATH.detect, observedAtMs),
            ),
            active: axis(
              { running_in: self.resolved ? { slug: self.resolved.slug, harness_ref: self.resolved.harness_ref } : null, ambiguity: self.ambiguity },
              provenance("actuation:harness:self", PATH.self, observedAtMs),
            ),
            staged: stagedAxis(provenance("actuation:harness:catalog", PATH.catalog, observedAtMs)),
            expected_effect: expectedEffect(),
          },
          mutable: false,
          native_path: "actuation harness detect (no mutation disclosed)",
          bootstrap: false,
          drift: drift(
            detectedCount === declaredCount ? "none" : "diverged",
            ["declared", "effective"],
            null,
          ),
        },
        {
          key: "availability.verification",
          title: "Verification gate",
          kind: "presence",
          axes: {
            declared: axis({ present: true, command: ["actuation", "verify", "--json"] }, provenance(OWNER_REF, "cli/commands.mjs", observedAtMs)),
            effective: axis({ present: true, command: ["actuation", "verify", "--json"] }, provenance("actuation:cli:capabilities", PATH.commands, observedAtMs)),
            active: axis({ running: false, last_run: null }, provenance("actuation:cli:system", PATH.system, observedAtMs)),
            staged: stagedAxis(provenance(OWNER_REF, "cli/commands.mjs", observedAtMs)),
            expected_effect: expectedEffect(),
          },
          mutable: false,
          native_path: "actuation verify",
          bootstrap: false,
          drift: drift("none"),
        },
      ],
    },
    {
      id: "activity",
      title: "Activity / ActuationStream",
      settings: [
        contractSetting("activity.contract", "Activity contract", "scalar", ACTIVITY_VERSION, "contracts/activity.mjs", observedAtMs),
        contractSetting("stream.contract", "ActuationStream contract", "scalar", ACTUATION_STREAM_VERSION, "contracts/actuation-stream.mjs", observedAtMs),
        {
          key: "stream.durable_store",
          title: "Durable stream store",
          kind: "presence",
          axes: {
            declared: axis(
              { default_root: ACTUATION_STREAM_STORE_DEFAULT_ROOT },
              provenance(OWNER_REF, PATH.streamStore, observedAtMs),
            ),
            effective: axis(
              { root: streamStoreRoot({}) },
              provenance("actuation:cli:capabilities", PATH.streamStore, observedAtMs),
            ),
            active: axis(
              defaultStreamStoreState(),
              provenance("actuation:cli:system", PATH.system, observedAtMs),
            ),
            staged: stagedAxis(provenance(OWNER_REF, PATH.streamStore, observedAtMs)),
            expected_effect: expectedEffect(),
          },
          mutable: false,
          native_path: "actuation stream open --store <dir>",
          bootstrap: false,
          drift: drift("none"),
        },
      ],
    },
    {
      id: "return",
      title: "Return",
      settings: [
        contractSetting("return.contract", "Return contract", "scalar", AGENCY_CONTRACT_VERSION, "contracts/agency.mjs", observedAtMs),
        vocabularySetting("return.modes", "Return policy modes", ["required", "optional", "autonomous-termination"], "contracts/agency.mjs", observedAtMs),
      ],
    },
  ];

  const obligations = [
    "ecology.read — no product-owned live registry of instantiated/realised actuations; receipts are caller-supplied documents, not a live ledger.",
    "attach — no native operation binds an observer to a live actor/session.",
    "Actuation discloses no kernel intent/invoke engagement seam; mutating operations (agency actualise, stream lifecycle, instantiation record, stream usage) are native CLI only and are not mountable through the O:I System surface this wave.",
    "Factory<->Actuation discovery/intent/authority seam — NOT owned by either track; returned for the serialized owner decision (do not invent). Actuation's actualise/realised receipts deliberately perform no Factory recognition and no source mutation; who owns the materialisation hand-off is undecided.",
  ];

  const descriptor = {
    schema: SYSTEM_DISCLOSURE_VERSION,
    product_id: PRODUCT_ID,
    contract_revision: SYSTEM_DISCLOSURE_CONTRACT_REVISION,
    disclosed_at_unix_ms: observedAtMs,
    owner: {
      owner_id: PRODUCT_ID,
      owner_ref: OWNER_REF,
      owner_version: ACTUATION_CLI_VERSION,
      reading_command: [...READING_COMMAND],
      reading_digest: null,
      reading_digest_covers: "whole descriptor, every *_unix_ms field zeroed, owner.reading_digest null",
      observed_at_unix_ms: observedAtMs,
    },
    about: "Actuation is the constitution and management of technological agency: it validates and projects Agent/Agency/WorldBinding, determination, bounds, authority and Return contracts, and proves harness presence live on this machine. It performs no agency materialisation and discloses no O:I engagement seam; mutating operations are native CLI only.",
    sections,
    actions: canonicalActions(),
    availability: deriveAvailability(detection),
    degradations: deriveDegradations(detection),
    obligations,
  };

  // The canonical reading digest is the sha256 of the canonical body — the
  // whole descriptor with every *_unix_ms field zeroed and reading_digest held
  // null — so two readings of an unchanged world produce the same digest and a
  // changed digest means a changed reading, never a changed clock (§4.5).
  descriptor.owner.reading_digest = createHash("sha256")
    .update(JSON.stringify(canonicalReadingBody(descriptor)))
    .digest("hex");

  return descriptor;
}
