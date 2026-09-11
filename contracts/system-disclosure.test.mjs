import test from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  buildSystemDisclosure,
  canonicalReadingBody,
  deriveAvailability,
  deriveDegradations,
  SYSTEM_DISCLOSURE_VERSION,
  SYSTEM_DISCLOSURE_CONTRACT_REVISION,
} from "./system-disclosure.mjs";
import { closeDurableStream, openDurableStream } from "./actuation-stream-store.mjs";
import { COMMANDS } from "../cli/commands.mjs";
import { executeCommand } from "../cli/actuation.mjs";
import { ACTUATION_CLI_VERSION } from "../cli/surface.mjs";

const AVAILABILITY_STATES = new Set(["available", "degraded", "unavailable", "unknown"]);
const ACTION_AVAILABILITY = new Set(["disclosed", "missing_native_obligation", "unavailable"]);
const AXIS_NAMES = ["declared", "effective", "active", "staged", "expected_effect"];
const DRIFT_STATES = new Set(["none", "diverged", "unknown"]);

// The single group ref that covers several native stream commands; every other
// unavailable/disclosed action_ref must name a real command.
const GROUP_REFS = new Set(["stream.durable"]);

function build() {
  return buildSystemDisclosure({ now: new Date("2026-09-10T12:00:00Z") });
}

test("the descriptor carries the frozen v2 identity", () => {
  const value = build();
  assert.equal(value.schema, SYSTEM_DISCLOSURE_VERSION);
  assert.equal(value.product_id, "actuation");
  assert.equal(value.contract_revision, SYSTEM_DISCLOSURE_CONTRACT_REVISION);
  assert.equal(typeof value.disclosed_at_unix_ms, "number");
});

test("the owner block carries the stable ref, real argv and a recomputable digest", () => {
  const value = build();
  assert.deepEqual(value.owner.reading_command, ["actuation", "system", "--json"]);
  assert.equal(value.owner.owner_id, "actuation");
  assert.equal(value.owner.owner_ref, "actuation:cli");
  assert.equal(value.owner.owner_version, ACTUATION_CLI_VERSION);
  assert.equal(typeof value.owner.observed_at_unix_ms, "number");
  assert.equal(typeof value.owner.reading_digest_covers, "string");

  // reading_digest must be the sha256 of the canonical body: every *_unix_ms
  // field zeroed and reading_digest held null (§4.5).
  const expected = createHash("sha256").update(JSON.stringify(canonicalReadingBody(value))).digest("hex");
  assert.equal(value.owner.reading_digest, expected);
});

test("the reading digest is stable across readings of an unchanged world (§4.5)", () => {
  const a = buildSystemDisclosure({ now: new Date("2026-09-10T12:00:00Z") });
  const b = buildSystemDisclosure({ now: new Date("2026-09-10T12:00:01Z") });
  assert.equal(a.owner.reading_digest, b.owner.reading_digest);
});

test("every setting exposes all five axes with provenance, never a bare value", () => {
  const value = build();
  assert.ok(Array.isArray(value.sections) && value.sections.length > 0);
  let settings = 0;
  for (const section of value.sections) {
    assert.ok(section.id, "section must have an id");
    assert.ok(section.title, "section must have a title");
    assert.ok(Array.isArray(section.settings));
    for (const setting of section.settings) {
      settings += 1;
      assert.ok(setting.key, "setting must have a dotted key");
      assert.ok(setting.title, "setting must have a title");
      for (const name of AXIS_NAMES) {
        assert.ok(name in setting.axes, `${setting.key} missing axis ${name}`);
      }
      for (const name of ["declared", "effective", "active"]) {
        const axis = setting.axes[name];
        assert.ok("value" in axis, `${setting.key}.axes.${name} missing value`);
        assert.equal(typeof axis.provenance.owner_ref, "string");
        assert.equal(typeof axis.provenance.path, "string");
        assert.equal(typeof axis.provenance.observed_at_unix_ms, "number");
      }
      assert.ok("stage_state" in setting.axes.staged, `${setting.key} staged axis missing stage_state`);
      assert.ok("summary" in setting.axes.expected_effect, `${setting.key} expected_effect missing summary`);
      assert.equal(typeof setting.mutable, "boolean");
      assert.ok(setting.native_path);
      assert.equal(typeof setting.bootstrap, "boolean");
      assert.ok(DRIFT_STATES.has(setting.drift.state), `${setting.key} drift state invalid`);
      assert.ok(Array.isArray(setting.drift.between));
      assert.ok("remediation_action_ref" in setting.drift);
    }
  }
  assert.ok(settings >= 8, "the disclosure must cover the required subjects, not a stub");
});

test("the required subjects are all present as sections/settings", () => {
  const value = build();
  const keys = value.sections.flatMap((section) => section.settings.map((setting) => setting.key));
  const subjects = [
    "agency.contract", "agency.determination.kinds", // Agent / Agency / WorldBinding, bounds / determination
    "authority.metagency.operations", "authority.derivation.rule", "authority.federation.rule", // authority
    "availability.harness.catalog", "availability.verification", // actual actuation availability
    "activity.contract", "stream.contract", "stream.durable_store", // Activity / ActuationStream
    "return.contract", "return.modes", // Return
  ];
  for (const key of subjects) {
    assert.ok(keys.includes(key), `required subject setting ${key} missing`);
  }
});

test("action availability is one of the three frozen states, with a reason only when unavailable", () => {
  const value = build();
  assert.ok(Array.isArray(value.actions) && value.actions.length > 0);
  for (const action of value.actions) {
    assert.ok(ACTION_AVAILABILITY.has(action.availability), `${action.action_ref} availability invalid`);
    if (action.availability === "unavailable") {
      assert.ok(action.unavailable_reason, `${action.action_ref} unavailable without reason`);
    }
    assert.equal(typeof action.exposure.ui, "boolean");
    assert.equal(typeof action.exposure.agent, "boolean");
    assert.equal(typeof action.exposure.headless, "boolean");
    assert.ok(Array.isArray(action.authority.requires));
    assert.ok("granted_by" in action.authority);
    assert.ok("evidence_ref" in action.authority);
    assert.ok(Array.isArray(action.args));
  }
});

test("the five action states never collapse: exists != selected != exposed != authorised != invoked", () => {
  const value = build();
  const actualise = value.actions.find((action) => action.action_ref === "agency.actualise");
  assert.ok(actualise, "actualise action must be disclosed");

  // exists: the command exists natively (in the command table).
  assert.ok(COMMANDS.some((entry) => entry.name === "agency.actualise"), "actualise exists as a native command");

  // exposed vs authorised are distinct facts: it is headless-exposed but not
  // UI-exposed, and it is NOT authorised merely by being exposed — it demands
  // an explicit MetagencyGrant.
  assert.equal(actualise.exposure.headless, true);
  assert.equal(actualise.exposure.ui, false);
  assert.ok(actualise.authority.requires.length > 0, "actualise authority must not be empty (exposed must not imply authorised)");
  assert.match(actualise.authority.granted_by, /MetagencyGrant/);

  // invoked: nothing claims invocation — no receipt/selected field is fabricated.
  assert.ok(!("selected" in actualise), "the owner must not assert a selected state");
  assert.ok(!("invoked" in actualise), "the owner must not assert an invoked state");

  // selected is not an owner fact anywhere in the descriptor.
  assert.ok(!JSON.stringify(value).includes('"selected"'));
});

test("authority is never inferred from UI location or root-agent identity", () => {
  const value = build();
  for (const action of value.actions) {
    for (const required of action.authority.requires) {
      assert.ok(!/root/i.test(required), `authority.requires must not name a root identity: ${required}`);
    }
    assert.ok(!/root/i.test(action.authority.granted_by), `authority.granted_by must not name a root grantor: ${action.authority.granted_by}`);
    assert.ok(!/ui[-_ ]?location/i.test(action.authority.granted_by), "authority must not infer from UI location");
  }
});

test("disclosed and unavailable actions project the command table; obligations do not fabricate commands", () => {
  const value = build();
  const commandNames = new Set(COMMANDS.map((entry) => entry.name));
  for (const action of value.actions) {
    if (action.availability === "disclosed") {
      assert.ok(commandNames.has(action.action_ref), `disclosed ${action.action_ref} must be a real command`);
    } else if (action.availability === "unavailable") {
      assert.ok(
        commandNames.has(action.action_ref) || GROUP_REFS.has(action.action_ref),
        `unavailable ${action.action_ref} must name a real command or a documented group`,
      );
    } else {
      // missing_native_obligation: must NOT already exist as a command (an
      // obligation is a genuinely missing native operation, never a fake one).
      assert.ok(!commandNames.has(action.action_ref), `obligation ${action.action_ref} must not already exist as a command`);
    }
  }
  // Every obligation in the obligations array is named by a missing action.
  const obligationActions = value.actions
    .filter((action) => action.availability === "missing_native_obligation")
    .map((action) => action.action_ref);
  for (const ref of obligationActions) {
    assert.ok(
      value.obligations.some((obligation) => obligation.startsWith(`${ref} `) || obligation.startsWith(`${ref} —`)),
      `obligation ${ref} must be named in the obligations array`,
    );
  }
});

test("the Factory<->Actuation seam is returned as a decision, never invented", () => {
  const value = build();
  const seam = value.obligations.find((obligation) => obligation.includes("Factory<->Actuation"));
  assert.ok(seam, "the Factory<->Actuation seam must be named");
  assert.match(seam, /serialized owner decision/);
  assert.match(seam, /do not invent/);
});

test("owner-level availability and degradations are separate honest facts", () => {
  const value = build();
  assert.ok(AVAILABILITY_STATES.has(value.availability.state));
  assert.ok(Array.isArray(value.degradations), "degradations must be an array (empty is proof, L3)");
  assert.ok(Array.isArray(value.obligations) && value.obligations.length > 0);
});

test("availability is derived from the live detection pass, never a literal (§4.7)", () => {
  assert.deepEqual(deriveAvailability({ availability: "complete" }), { state: "available", reason: null });
  const degraded = deriveAvailability({ availability: "partial" });
  assert.equal(degraded.state, "degraded");
  assert.match(degraded.reason, /partial/);
});

test("degradations list only probes that failed, each with a reason (§4.3)", () => {
  const detection = {
    harnesses: [
      { slug: "zcode", harness_ref: "harness/zcode", state: "detected" },
      { slug: "gemini", harness_ref: "harness/gemini", state: "unavailable", unavailable_reason: "all probes failed; could not run" },
      { slug: "pi", harness_ref: "harness/pi", state: "not-installed" },
    ],
  };
  const degradations = deriveDegradations(detection);
  assert.equal(degradations.length, 1);
  assert.equal(degradations[0].subject_ref, "harness/gemini");
  assert.equal(degradations[0].state, "unavailable");
  assert.equal(degradations[0].reason, "all probes failed; could not run");
});

test("provenance.path names a source location, never a command string (§4.6)", () => {
  const value = build();
  for (const section of value.sections) {
    for (const setting of section.settings) {
      for (const name of ["declared", "effective", "active", "staged"]) {
        const path = setting.axes[name].provenance.path;
        assert.ok(!path.includes("--json"), `${setting.key}.${name} provenance.path is a command string: ${path}`);
        assert.ok(!/^actuation /.test(path), `${setting.key}.${name} provenance.path is a command string: ${path}`);
      }
    }
  }
});

test("stream.durable_store reports the real default store, not a fabricated absence", () => {
  const value = build();
  const setting = value.sections.flatMap((section) => section.settings).find((entry) => entry.key === "stream.durable_store");
  assert.ok(setting, "stream.durable_store setting must be present");
  // declared carries the authored default-root declaration.
  assert.equal(typeof setting.axes.declared.value.default_root, "string");
  assert.ok(setting.axes.declared.value.default_root.includes(".actuation"), "declared default_root must name the authored ~/.actuation/streams");
  // effective carries the resolved root (env override or the default).
  assert.equal(typeof setting.axes.effective.value.root, "string");
  // active probes the filesystem: exists/root/in_use, never a hardcoded absence.
  assert.equal(typeof setting.axes.active.value.root, "string");
  assert.equal(typeof setting.axes.active.value.exists, "boolean");
  assert.equal(typeof setting.axes.active.value.in_use, "boolean");
  assert.equal(typeof setting.axes.active.value.open_streams, "number");
});

test("a store holding only closed streams reports open_streams 0, never a fabricated open count", () => {
  // A real durable stream, genuinely closed on disk. Openness must be read
  // from lifecycle state, not from the presence of the .jsonl file (§4.7).
  const root = mkdtempSync(join(tmpdir(), "actuation-system-disclosure-"));
  const identity = {
    stream_ref: "actuation:stream:closed-only",
    actuation_ref: "actuation:closed-only",
    agency_ref: "agency:closed-only",
    agent_session_ref: "agent-session:closed-only",
  };
  openDurableStream({ root, ...identity });
  closeDurableStream({ root, stream_ref: identity.stream_ref });

  const previous = process.env.ACTUATION_STREAM_STORE;
  process.env.ACTUATION_STREAM_STORE = root;
  try {
    const value = build();
    const setting = value.sections.flatMap((section) => section.settings).find((entry) => entry.key === "stream.durable_store");
    assert.equal(setting.axes.active.value.open_streams, 0);
    assert.equal(setting.axes.active.value.closed_streams, 1);
    assert.equal(setting.axes.active.value.in_use, false);
  } finally {
    if (previous == null) delete process.env.ACTUATION_STREAM_STORE;
    else process.env.ACTUATION_STREAM_STORE = previous;
  }
});

test("a stream whose lifecycle cannot be determined is counted as neither open nor closed and named", () => {
  // A `.jsonl` file whose header cannot be folded (torn journal) must not be
  // guessed as open: it is reported as undetermined with a reason, not counted.
  const root = mkdtempSync(join(tmpdir(), "actuation-system-disclosure-undetermined-"));
  writeFileSync(join(root, "broken.jsonl"), '{"schema":"actuation-stream/v1","stream_ref":"actuation:stream:broken","lifecycle":{"state":"open"}}\n{"torn', { encoding: "utf8" });

  const previous = process.env.ACTUATION_STREAM_STORE;
  process.env.ACTUATION_STREAM_STORE = root;
  try {
    const value = build();
    const setting = value.sections.flatMap((section) => section.settings).find((entry) => entry.key === "stream.durable_store");
    assert.equal(setting.axes.active.value.open_streams, 0);
    assert.equal(setting.axes.active.value.closed_streams, 0);
    assert.equal(setting.axes.active.value.undetermined_streams, 1);
    assert.equal(setting.axes.active.value.in_use, false);
    assert.match(setting.axes.active.value.reason, /neither open nor closed/);
  } finally {
    if (previous == null) delete process.env.ACTUATION_STREAM_STORE;
    else process.env.ACTUATION_STREAM_STORE = previous;
  }
});

test("the CLI system command emits the disclosure end to end", () => {
  const result = executeCommand(["system", "--json"]);
  const value = JSON.parse(result.stdout);
  assert.equal(value.schema, "oi.product-settings-disclosure/v2");
  assert.equal(value.product_id, "actuation");
  assert.equal(value.availability.state, "available");
  // Human rendering carries the digest and honest availability.
  assert.match(executeCommand(["system"]).stdout, /Actuation settings disclosure/);
  assert.match(executeCommand(["system"]).stdout, /Availability: available/);
});
