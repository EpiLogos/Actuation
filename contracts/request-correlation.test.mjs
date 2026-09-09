import assert from "node:assert/strict";
import test from "node:test";

import {
  ACTUATION_STREAM_VERSION,
  appendActuationStreamEvent,
} from "./actuation-stream.mjs";
import {
  REQUEST_CORRELATION_VERSION,
  authorityDecision,
  requestCorrelationReadModel,
  validateAuthorityDecision,
} from "./request-correlation.mjs";
import { ACTIVITY_VERSION } from "./activity.mjs";

// Fixture identity shaped like an upstream (AIKit-issued) permission request:
// mixed case, colons and slashes — anything a normaliser would destroy.
const REQUEST_REF = "aikit:permission-request/01JZkE-xQyZ";
const ACTIVITY_REF = "activity:shell.exec:01JZkE";

function baseStream() {
  return {
    schema: ACTUATION_STREAM_VERSION,
    stream_ref: "stream:dev-1",
    actuation_ref: "actuation:dev-1",
    agency_ref: "agency:development",
    agent_session_ref: "agent-session:development",
    lifecycle: { state: "open", started_at: "2026-09-08T09:00:00Z" },
    cursor: { last_sequence: 0, next_sequence: 1 },
    events: [],
  };
}

function streamEvent(sequence, kind, extra = {}) {
  return {
    event_ref: `event:${sequence}`,
    sequence,
    kind,
    observed_at: `2026-09-08T09:00:${String(sequence).padStart(2, "0")}Z`,
    actor: { agency_ref: "agency:development", agent_ref: "agent:builder" },
    disclosure: "portable",
    ...extra,
  };
}

function baseDecision(extra = {}) {
  return {
    schema: REQUEST_CORRELATION_VERSION,
    document: "authority-decision",
    decision_ref: "decision:1",
    request_ref: REQUEST_REF,
    decision: "allowed",
    basis: "Within the delegated bounds of determination:determination/dev-1.",
    determination_ref: "determination:dev-1",
    bounds_refs: ["bounds:dev-shell"],
    decided_by: "agency:development",
    observed_at: "2026-09-08T09:01:00Z",
    activity_refs: [ACTIVITY_REF],
    ...extra,
  };
}

function baseActivity(extra = {}) {
  return {
    schema: ACTIVITY_VERSION,
    activity_ref: ACTIVITY_REF,
    actor: { agency_ref: "agency:development" },
    agent_session_ref: "agent-session:development",
    subject_ref: "run:42",
    native_owner: "actuation",
    verb: "executed",
    object: "shell command",
    summary: "Development Agent ran the bounded shell command after authority was granted.",
    phase: "completed",
    outcome: "succeeded",
    salience: "normal",
    needs_attention: false,
    trace: { stream_ref: "stream:dev-1", event_refs: ["event:1"], from_sequence: 1, through_sequence: 1 },
    started_at: "2026-09-08T09:02:00Z",
    updated_at: "2026-09-08T09:03:00Z",
    completed_at: "2026-09-08T09:03:00Z",
    ...extra,
  };
}

test("authority decision validator enforces the join hub shape", () => {
  const decision = authorityDecision(baseDecision());
  assert.equal(decision.schema, REQUEST_CORRELATION_VERSION);
  assert.equal(decision.request_ref, REQUEST_REF);

  assert.throws(() => validateAuthorityDecision({ ...baseDecision(), schema: "actuation.request-correlation/v0" }),
    /schema must equal/);
  assert.throws(() => validateAuthorityDecision({ ...baseDecision(), document: "decision" }),
    /document must be authority-decision/);
  assert.throws(() => validateAuthorityDecision({ ...baseDecision(), decision: "granted" }),
    /must be allowed, refused, pending/);
  assert.throws(() => validateAuthorityDecision({ ...baseDecision(), request_ref: "  " }),
    /request_ref must be a non-empty/);
  assert.throws(() => validateAuthorityDecision({ ...baseDecision(), activity_refs: [] }),
    /must not be empty/);
  assert.throws(() => validateAuthorityDecision({ ...baseDecision(), observed_at: "not-a-date" }),
    /ISO-compatible/);
  // Basis is mandatory: a decision without a stated basis cannot be audited.
  assert.throws(() => validateAuthorityDecision({ ...baseDecision(), basis: "" }),
    /basis must be a non-empty/);
});

test("correlates all four sides for a granted request, verbatim identities", () => {
  const stream = appendActuationStreamEvent(
    appendActuationStreamEvent(baseStream(), streamEvent(1, "permission", {
      resource_refs: [REQUEST_REF],
    })),
    streamEvent(2, "permission", {
      resource_refs: [REQUEST_REF],
      metadata: { permission_outcome: "granted" },
    }),
  );
  const model = requestCorrelationReadModel(REQUEST_REF, {
    authority_decisions: [baseDecision()],
    activities: [baseActivity()],
    streams: [stream],
  });

  assert.equal(model.schema, REQUEST_CORRELATION_VERSION);
  assert.equal(model.state, "correlated");
  // The four-side join invariant: the upstream-issued identity passes through
  // byte-identical and joins by exact equality, no re-derivation.
  assert.equal(model.request_ref, REQUEST_REF);
  assert.deepEqual(model.activities.activity_refs, [ACTIVITY_REF]);
  assert.equal(model.authority.resolution, "allowed");
  assert.deepEqual(model.authority.decision_refs, ["decision:1"]);
  assert.deepEqual(model.authority.decisions[0].determination_ref, "determination:dev-1");
  assert.equal(model.permission.outcome, "granted");
  assert.deepEqual(model.permission.event_refs, ["event:1", "event:2"]);
  assert.equal(model.attention.tracked, false);
  assert.equal(model.attention.available, true);
});

test("attention side surfaces needs_attention activity and pending permission events", () => {
  const stream = appendActuationStreamEvent(baseStream(), streamEvent(1, "permission", {
    resource_refs: [REQUEST_REF],
  }));
  const model = requestCorrelationReadModel(REQUEST_REF, {
    authority_decisions: [baseDecision()],
    activities: [baseActivity({ needs_attention: true, salience: "important", phase: "waiting", outcome: "pending", completed_at: undefined })],
    streams: [stream],
  });

  assert.equal(model.state, "correlated");
  assert.equal(model.attention.tracked, true);
  assert.deepEqual(model.attention.activity_refs, [ACTIVITY_REF]);
  assert.deepEqual(model.attention.pending_permission_event_refs, ["event:1"]);
  // Pending is a recorded state: no disposition event exists yet.
  assert.equal(model.permission.outcome, "pending");
});

test("latest stream disposition wins: refusal overrides an earlier permission request", () => {
  const stream = appendActuationStreamEvent(
    appendActuationStreamEvent(baseStream(), streamEvent(1, "permission", {
      resource_refs: [REQUEST_REF],
    })),
    streamEvent(2, "refusal", { resource_refs: [REQUEST_REF] }),
  );
  const model = requestCorrelationReadModel(REQUEST_REF, {
    authority_decisions: [baseDecision({ decision: "refused", basis: "Outside delegated bounds." })],
    activities: [baseActivity({ phase: "cancelled", outcome: "refused", completed_at: "2026-09-08T09:03:00Z" })],
    streams: [stream],
  });

  assert.equal(model.state, "correlated");
  assert.equal(model.authority.resolution, "refused");
  assert.equal(model.permission.outcome, "refused");
  assert.deepEqual(model.permission.event_refs, ["event:1", "event:2"]);
  assert.equal(model.attention.tracked, false);
});

test("latest authority decision by observed_at wins the resolution", () => {
  const model = requestCorrelationReadModel(REQUEST_REF, {
    authority_decisions: [
      baseDecision({ decision_ref: "decision:1", decision: "pending", observed_at: "2026-09-08T09:01:00Z" }),
      baseDecision({ decision_ref: "decision:2", decision: "allowed", observed_at: "2026-09-08T09:05:00Z" }),
    ],
    activities: [baseActivity()],
    streams: [baseStream()],
  });

  assert.equal(model.state, "correlated");
  assert.equal(model.authority.resolution, "allowed");
  assert.deepEqual(model.authority.decision_refs, ["decision:1", "decision:2"]);
});

test("unknown-identity is explicit when sides are queried and nothing references the identity", () => {
  const model = requestCorrelationReadModel(REQUEST_REF, {
    authority_decisions: [],
    activities: [],
    streams: [],
  });

  assert.equal(model.state, "unknown-identity");
  assert.equal(model.authority.available, true);
  assert.equal(model.authority.resolution, "none");
  assert.equal(model.permission.available, true);
  assert.equal(model.permission.outcome, "none");
  // Queried-and-empty is not the same as unavailable: no reason is attached.
  assert.equal(model.authority.unavailable_reason, undefined);
});

test("no-recorded-activity is explicit when the decision names activities the corpus does not hold", () => {
  const model = requestCorrelationReadModel(REQUEST_REF, {
    authority_decisions: [baseDecision()],
    activities: [],
    streams: [],
  });

  assert.equal(model.state, "no-recorded-activity");
  assert.equal(model.activities.available, true);
  assert.deepEqual(model.activities.correlated, []);
  // The named-but-missing identity is disclosed, never silently dropped.
  assert.deepEqual(model.activities.unrecorded_activity_refs, [ACTIVITY_REF]);
});

test("correlation-unavailable when no side was queried — absence cannot be claimed", () => {
  const model = requestCorrelationReadModel(REQUEST_REF);

  assert.equal(model.state, "correlation-unavailable");
  for (const side of [model.authority, model.activities, model.permission]) {
    assert.equal(side.available, false);
    assert.match(side.unavailable_reason, /absence cannot be claimed/);
  }
});

test("correlation-unavailable when the identity is referenced but the activity join target was not queried", () => {
  const stream = appendActuationStreamEvent(baseStream(), streamEvent(1, "permission", {
    resource_refs: [REQUEST_REF],
  }));
  const model = requestCorrelationReadModel(REQUEST_REF, {
    authority_decisions: [baseDecision()],
    streams: [stream],
  });

  assert.equal(model.state, "correlation-unavailable");
  assert.equal(model.activities.available, false);
  // The stream side still reports what it honestly can.
  assert.equal(model.permission.outcome, "pending");
});

test("validator refuses malformed corpus instead of correlating over it", () => {
  assert.throws(
    () => requestCorrelationReadModel(REQUEST_REF, { authority_decisions: [{ nope: true }] }),
    /AuthorityDecision.schema must equal/,
  );
  assert.throws(
    () => requestCorrelationReadModel(REQUEST_REF, { activities: [{ schema: ACTIVITY_VERSION }] }),
    /Activity.activity_ref/,
  );
  assert.throws(
    () => requestCorrelationReadModel(REQUEST_REF, { streams: [{ schema: ACTUATION_STREAM_VERSION }] }),
    /ActuationStream.stream_ref/,
  );
  assert.throws(() => requestCorrelationReadModel("", {}), /request_ref/);
});

test("read model is a fresh projection: mutating the corpus afterwards changes nothing", () => {
  const corpus = {
    authority_decisions: [baseDecision()],
    activities: [baseActivity()],
    streams: [],
  };
  const model = requestCorrelationReadModel(REQUEST_REF, corpus);
  corpus.activities[0].summary = "tampered";
  corpus.authority_decisions[0].decision = "refused";

  assert.equal(model.activities.correlated[0].summary, baseActivity().summary);
  assert.equal(model.authority.decisions[0].decision, "allowed");
});
