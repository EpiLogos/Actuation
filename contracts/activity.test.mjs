import assert from "node:assert/strict";
import test from "node:test";

import {
  ACTUATION_STREAM_VERSION,
  appendActuationStreamEvent,
  closeActuationStream,
} from "./actuation-stream.mjs";
import {
  ACTIVITY_VERSION,
  activityFromActuationStream,
  activityFromStreamEvent,
  activityNeedsAttention,
  validateActivity,
} from "./activity.mjs";

function baseStream() {
  return {
    schema: ACTUATION_STREAM_VERSION,
    stream_ref: "stream:dev-1",
    actuation_ref: "actuation:dev-1",
    agency_ref: "agency:development",
    agent_session_ref: "agent-session:development",
    world_binding_ref: "world:project:oi:development",
    participating_loci: ["locus:builder"],
    surface_refs: ["surface:desktop"],
    lifecycle: { state: "open", started_at: "2026-08-31T10:00:00Z" },
    cursor: { last_sequence: 0, next_sequence: 1 },
    events: [],
    provenance: ["O:I #155 fixture"],
  };
}

function event(sequence, kind, extra = {}) {
  return {
    event_ref: `event:${sequence}`,
    sequence,
    kind,
    observed_at: `2026-08-31T10:00:0${sequence}Z`,
    actor: {
      locus_ref: "locus:builder",
      agency_ref: "agency:development",
      agent_ref: "agent:builder",
    },
    disclosure: "portable",
    ...extra,
  };
}

test("projects Action → Invocation → Actuation → Result → Return lineage without changing session identity", () => {
  let stream = appendActuationStreamEvent(baseStream(), event(1, "tool-request", {
    native_trace_ref: "trace:tool/1",
    resource_refs: ["action:factory.apply"],
  }));
  stream = appendActuationStreamEvent(stream, event(2, "tool-result", {
    native_trace_ref: "trace:tool/1/result",
    evidence_refs: ["evidence:test-green"],
  }));
  stream = appendActuationStreamEvent(stream, event(3, "return", {
    return_ref: "return:journey-1",
    evidence_refs: ["evidence:return"],
  }));
  stream = closeActuationStream(stream, { endedAt: "2026-08-31T10:01:00Z" });

  const activity = activityFromActuationStream(stream, {
    activityRef: "activity:build-1",
    subjectRef: "journey:oi155",
    nativeOwner: "actuation",
    actionRef: "action:factory.apply",
    invocationRef: "invocation:factory.apply:1",
    resultRef: "result:factory.apply:1",
    verb: "implemented",
    object: "inhabitation field",
    summary: "Development Agent completed the bounded implementation and returned evidence.",
    evidenceRefs: ["evidence:test-green"],
  });

  assert.equal(activity.schema, ACTIVITY_VERSION);
  assert.equal(activity.agent_session_ref, "agent-session:development");
  assert.equal(activity.actuation_ref, "actuation:dev-1");
  assert.equal(activity.action_ref, "action:factory.apply");
  assert.equal(activity.invocation_ref, "invocation:factory.apply:1");
  assert.equal(activity.result_ref, "result:factory.apply:1");
  assert.equal(activity.return_ref, "return:journey-1");
  assert.equal(activity.phase, "completed");
  assert.equal(activity.outcome, "succeeded");
  assert.deepEqual(activity.trace.event_refs, ["event:1", "event:2", "event:3"]);
  assert.deepEqual(activity.trace.native_trace_refs, ["trace:tool/1", "trace:tool/1/result"]);
});

test("meaningful non-Action observation is Activity without counterfeit Action identity", () => {
  const stream = appendActuationStreamEvent(baseStream(), event(1, "world-observation", {
    resource_refs: ["workcell:vm-2"],
    native_trace_ref: "trace:workcell/reconcile-9",
  }));
  const activity = activityFromStreamEvent(stream, "event:1", {
    activityRef: "activity:placement-observed",
    subjectRef: "agent-set:development",
    nativeOwner: "workcell",
    verb: "observed",
    object: "placement reconciliation",
    summary: "Development AgentSet is available on the selected remote Workcell.",
  });

  assert.equal(activity.action_ref, undefined);
  assert.equal(activity.invocation_ref, undefined);
  assert.equal(activity.trace.event_refs[0], "event:1");
  assert.equal(activity.trace.native_trace_refs[0], "trace:workcell/reconcile-9");
});

test("Activity remains globally visible without becoming Attention by default", () => {
  const stream = appendActuationStreamEvent(baseStream(), event(1, "model-message", { content: "working" }));
  const activity = activityFromStreamEvent(stream, "event:1", {
    activityRef: "activity:working",
    subjectRef: "run:42",
    nativeOwner: "actuation",
    verb: "working",
    object: "run 42",
    summary: "Agent is continuing the run.",
    phase: "running",
    outcome: "pending",
  });
  assert.equal(activityNeedsAttention(activity), false);
  assert.equal(activity.completed_at, undefined);
});

test("attention signal is explicit data and carries no notification or invocation authority", () => {
  const stream = appendActuationStreamEvent(baseStream(), event(1, "permission", {
    resource_refs: ["human-request:approve-1"],
  }));
  const activity = activityFromStreamEvent(stream, "event:1", {
    activityRef: "activity:request-approval",
    subjectRef: "human-request:approve-1",
    nativeOwner: "factory",
    verb: "requested",
    object: "human recognition",
    summary: "Run requires human Recognition before applying the authored-source proposal.",
    phase: "waiting",
    outcome: "pending",
    salience: "important",
    needsAttention: true,
  });
  assert.equal(activityNeedsAttention(activity), true);
  assert.equal("notification_ref" in activity, false);
  assert.equal("authority" in activity, false);
});

test("validator rejects collapsed semantic identity and invalid terminal timing", () => {
  const input = {
    schema: ACTIVITY_VERSION,
    activity_ref: "activity:1",
    actor: { agency_ref: "agency:1" },
    agent_session_ref: "agent-session:1",
    subject_ref: "run:1",
    native_owner: "actuation",
    verb: "completed",
    object: "run",
    summary: "done",
    phase: "completed",
    outcome: "succeeded",
    salience: "normal",
    needs_attention: false,
    trace: { stream_ref: "stream:1" },
    started_at: "2026-08-31T10:00:00Z",
    updated_at: "2026-08-31T10:01:00Z",
  };
  assert.throws(() => validateActivity(input), /requires completed_at/);
});
