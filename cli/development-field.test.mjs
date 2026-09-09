import test from "node:test";
import assert from "node:assert/strict";

import { executeCommand } from "./actuation.mjs";
import {
  ACTUATION_STREAM_VERSION,
  appendActuationStreamEvent,
  closeActuationStream,
} from "../contracts/actuation-stream.mjs";
import { activityFromActuationStream } from "../contracts/activity.mjs";
import { AGENCY_CONTRACT_VERSION, validateReturn } from "../contracts/agency.mjs";
import { REQUEST_CORRELATION_VERSION, requestCorrelationReadModel } from "../contracts/request-correlation.mjs";
import { REALISED_ACTUATION_VERSION, continuityDelta } from "../contracts/realised-actuation.mjs";

function openStream() {
  return {
    schema: ACTUATION_STREAM_VERSION,
    stream_ref: "stream:development-field:1",
    actuation_ref: "actuation:development-field:1",
    agency_ref: "agency:worker",
    agent_session_ref: "agent-session:worker:1",
    world_binding_ref: "world-binding:worker",
    lifecycle: { state: "open", started_at: "2026-09-10T00:00:00Z" },
    cursor: { last_sequence: 0, next_sequence: 1 },
    events: [],
  };
}

function completedStream() {
  let stream = appendActuationStreamEvent(openStream(), {
    event_ref: "event:development-field:tool",
    sequence: 1,
    kind: "tool-result",
    observed_at: "2026-09-10T00:00:01Z",
    actor: { agent_ref: "agent:worker", agency_ref: "agency:worker" },
    native_trace_ref: "trace:native:1",
    resource_refs: ["result:development-field:1"],
  });
  return closeActuationStream(stream, { state: "closed", endedAt: "2026-09-10T00:00:02Z" });
}

function developmentActivity() {
  return activityFromActuationStream(completedStream(), {
    activityRef: "activity:development-field:1",
    subjectRef: "subject:change:1",
    nativeOwner: "actuation",
    verb: "implemented",
    object: "Development Field actuality seam",
    summary: "An actual Agency completed a bounded development act.",
    actionRef: "action:implementation:1",
    invocationRef: "invocation:implementation:1",
    planRef: "plan:development-field:s2",
    journeyRef: "journey:development-field:1",
    runRef: "run:factory:1",
    resultRef: "result:development-field:1",
    evidenceRefs: ["evidence:deterministic:1"],
    returnRef: "return:development-field:1",
  });
}

function directActivity() {
  return activityFromActuationStream(completedStream(), {
    activityRef: "activity:direct:1",
    subjectRef: "subject:direct:1",
    nativeOwner: "actuation",
    verb: "acted",
    object: "direct task",
    summary: "A direct Agency act with no Factory ancestry.",
    resultRef: "result:direct:1",
  });
}

function returnFixture() {
  return {
    schema: AGENCY_CONTRACT_VERSION,
    return_ref: "return:development-field:1",
    determination_ref: "determination:worker:1",
    from_agency_ref: "agency:worker",
    to_agency_ref: "agency:developer",
    difference_refs: ["difference:implemented:1"],
    artifact_refs: ["artifact:patch:1"],
    evidence_refs: ["evidence:deterministic:1"],
    provenance: {
      agency_lineage_refs: ["agency:developer", "agency:worker"],
      agent_refs: ["agent:developer", "agent:worker"],
      world_binding_refs: ["world-binding:worker"],
      authority_decision_refs: ["authority-decision:request:1"],
      authority_refs: ["authority:code:1"],
      bounds_refs: ["bound:repo:actuation"],
      activity_refs: ["activity:development-field:1"],
      actuation_refs: ["actuation:development-field:1"],
      result_refs: ["result:development-field:1"],
      request_refs: ["request:implementation:1"],
      agent_session_refs: ["agent-session:worker:1"],
      action_refs: ["action:implementation:1"],
      invocation_refs: ["invocation:implementation:1"],
      plan_refs: ["plan:development-field:s2"],
      journey_refs: ["journey:development-field:1"],
      run_refs: ["run:factory:1"],
      provider_refs: ["provider:github"],
      harness_refs: ["harness:external"],
      material_refs: ["material:github-hosted"],
      external_source_refs: ["github:EpiLogos/Actuation#53"],
    },
    received: true,
    recognition_state: "pending",
    world_mutation_state: "not-applied",
  };
}

test("Activity retains supplied developmental refs as correlations without collapsing identities", () => {
  const activity = developmentActivity();
  assert.equal(activity.plan_ref, "plan:development-field:s2");
  assert.equal(activity.journey_ref, "journey:development-field:1");
  assert.equal(activity.run_ref, "run:factory:1");
  assert.equal(activity.agent_session_ref, "agent-session:worker:1");
  assert.equal(activity.actuation_ref, "actuation:development-field:1");
  assert.notEqual(activity.run_ref, activity.actuation_ref);
  assert.notEqual(activity.run_ref, activity.agent_session_ref);
  assert.deepEqual(activity.trace.native_trace_refs, ["trace:native:1"]);
});

test("Direct non-Factory Activity remains valid and Factory ancestry is not fabricated", () => {
  const activity = directActivity();
  assert.equal(activity.actor.agent_ref, "agent:worker");
  assert.equal(activity.actor.agency_ref, "agency:worker");
  assert.equal(activity.plan_ref, undefined);
  assert.equal(activity.journey_ref, undefined);
  assert.equal(activity.run_ref, undefined);
  const cli = JSON.parse(executeCommand(["activity", "-", "--json"], { stdin: JSON.stringify(activity) }).stdout);
  assert.equal(cli.activity_ref, "activity:direct:1");
  assert.equal("run_ref" in cli, false);
});

test("Return provenance retains actuality and supplied development lineage through the native Agency read", () => {
  const returned = validateReturn(returnFixture());
  assert.deepEqual(returned.provenance.activity_refs, ["activity:development-field:1"]);
  assert.deepEqual(returned.provenance.world_binding_refs, ["world-binding:worker"]);
  assert.deepEqual(returned.provenance.run_refs, ["run:factory:1"]);

  const input = {
    binding: {
      schema: AGENCY_CONTRACT_VERSION,
      binding_ref: "world-binding:developer",
      agent_ref: "agent:developer",
      agency_ref: "agency:developer",
      world_ref: "world:project:actuation",
      scope_ref: "scope:project:actuation",
    },
    returns: [returned],
  };
  const read = JSON.parse(executeCommand(["agency", "-", "--json"], { stdin: JSON.stringify(input) }).stdout);
  assert.equal(read.returns.records.length, 1);
  assert.equal(read.returns.records[0].return_ref, "return:development-field:1");
  assert.deepEqual(read.returns.records[0].provenance.authority_decision_refs, ["authority-decision:request:1"]);
  assert.deepEqual(read.returns.records[0].provenance.result_refs, ["result:development-field:1"]);
});

test("request correlation recovers actual authority and Activity and keeps unknown caller state explicit", () => {
  const activity = developmentActivity();
  const decision = {
    schema: REQUEST_CORRELATION_VERSION,
    document: "authority-decision",
    decision_ref: "authority-decision:request:1",
    request_ref: "request:implementation:1",
    decision: "allowed",
    basis: "bounded implementation authority",
    determination_ref: "determination:worker:1",
    bounds_refs: ["bound:repo:actuation"],
    decided_by: "agency:developer",
    observed_at: "2026-09-10T00:00:00Z",
    evidence_refs: ["evidence:authority:1"],
    activity_refs: [activity.activity_ref],
  };
  const correlated = requestCorrelationReadModel("request:implementation:1", {
    authority_decisions: [decision],
    activities: [activity],
    streams: [],
  });
  assert.equal(correlated.state, "correlated");
  assert.equal(correlated.authority.resolution, "allowed");
  assert.deepEqual(correlated.authority.decisions[0].bounds_refs, ["bound:repo:actuation"]);
  assert.equal(correlated.activities.correlated[0].activity_ref, activity.activity_ref);

  const unknown = requestCorrelationReadModel("request:not-observed", {
    authority_decisions: [],
    activities: [],
    streams: [],
  });
  assert.equal(unknown.state, "unknown-identity");
  assert.equal(unknown.authority.resolution, "none");
  assert.deepEqual(unknown.activities.activity_refs, []);
});

test("provider and material relocation do not mint new Agent or Agency identity", () => {
  const base = {
    schema: REALISED_ACTUATION_VERSION,
    realised_ref: "realised:worker:1",
    actuation_ref: "actuation:development-field:1",
    agent_ref: "agent:worker",
    agency_ref: "agency:worker",
    world_binding_ref: "world-binding:worker",
    loop: { recurrence: "turn-based", acting: true },
    body: {
      harness_ref: "harness:one",
      session_ref: "session:one",
      process_ref: "process:one",
      material_binding_ref: "material:one",
    },
    observation: { state: "partial", evidence_refs: [] },
  };
  const relocated = structuredClone(base);
  relocated.realised_ref = "realised:worker:2";
  relocated.body = {
    harness_ref: "harness:two",
    session_ref: "session:two",
    process_ref: "process:two",
    material_binding_ref: "material:two",
  };

  const delta = continuityDelta(base, relocated);
  assert.equal(delta.same_agent, true);
  assert.equal(delta.same_agency, true);
  assert.equal(delta.same_world_binding, true);
  assert.equal(delta.same_actuation, true);
  assert.equal(delta.harness_changed, true);
  assert.equal(delta.session_changed, true);
  assert.equal(delta.process_changed, true);
  assert.equal(delta.material_binding_changed, true);
});
