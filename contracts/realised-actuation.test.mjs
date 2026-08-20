import test from "node:test";
import assert from "node:assert/strict";

import {
  REALISED_ACTUATION_VERSION,
  continuityDelta,
  realisedActuationReadModel,
  validateRealisedActuation,
} from "./realised-actuation.mjs";

function collapsed(overrides = {}) {
  return {
    schema: REALISED_ACTUATION_VERSION,
    realised_ref: "realised/agent-loop/demo/1",
    actuation_ref: "actuation/demo",
    agent_ref: "agent/demo",
    agency_ref: "agency/demo/project-coding",
    world_binding_ref: "world/demo-project",
    loop: {
      recurrence: "turn-based",
      acting: true,
      entrypoint_ref: "native:cli/invoke",
      observed_faculties: ["model-call", "filesystem", "shell"],
    },
    body: {
      model_condition_ref: "actuation:model-bearing/demo",
    },
    observation: {
      state: "observed",
      evidence_refs: ["evidence/native-invocation/1"],
    },
    ...overrides,
  };
}

test("collapsed directory-bound loop is valid without AIKit, HarnessComposition, SessionSpace or Workcell identity", () => {
  const receipt = validateRealisedActuation(collapsed());
  assert.equal(receipt.agent_ref, "agent/demo");
  assert.equal(receipt.body.harness_ref, undefined);
  assert.equal(receipt.body.session_ref, undefined);
  assert.equal(receipt.body.material_binding_ref, undefined);
});

test("inert model availability is not promoted into realised Actuation", () => {
  assert.throws(
    () => validateRealisedActuation(collapsed({ loop: { recurrence: "single-shot", acting: false } })),
    /inert model availability/,
  );
});

test("rich native harness refs remain opaque external body facts", () => {
  const receipt = collapsed({
    body: {
      harness_ref: "aikit:harness/cordis",
      session_ref: "aikit:agent-session/42",
      process_ref: "native:process/901",
      model_condition_ref: "actuation:model-bearing/demo",
      material_binding_ref: "workcell:binding/901",
    },
    participating_loci: ["agency/demo/project-coding", "agency/demo/reviewer"],
    stream_ref: "actuation:stream/42",
    return_ref: "actuation:return/42",
  });
  const read = realisedActuationReadModel(receipt);
  assert.equal(read.harness_ref, "aikit:harness/cordis");
  assert.equal(read.stream_ref, "actuation:stream/42");
  assert.deepEqual(read.participating_loci, ["agency/demo/project-coding", "agency/demo/reviewer"]);
});

test("unsupported native faculties are absent/degraded rather than fabricated", () => {
  const read = realisedActuationReadModel(collapsed({
    observation: {
      state: "partial",
      evidence_refs: ["evidence/native-invocation/2"],
      unsupported_faculties: ["live-cancel", "subagent-events"],
      degraded_faculties: ["stream-detail"],
    },
  }));
  assert.deepEqual(read.observation.unsupported_faculties, ["live-cancel", "subagent-events"]);
  assert.deepEqual(read.observation.degraded_faculties, ["stream-detail"]);
});

test("body/session/model/material change does not silently change Agent identity", () => {
  const before = collapsed({
    body: {
      harness_ref: "native:pi",
      session_ref: "aikit:agent-session/1",
      model_condition_ref: "actuation:model-bearing/model-a",
    },
  });
  const after = collapsed({
    realised_ref: "realised/agent-loop/demo/2",
    body: {
      harness_ref: "native:codex",
      session_ref: "aikit:agent-session/2",
      model_condition_ref: "actuation:model-bearing/model-b",
      material_binding_ref: "workcell:binding/2",
    },
  });
  const delta = continuityDelta(before, after);
  assert.equal(delta.same_agent, true);
  assert.equal(delta.same_agency, true);
  assert.equal(delta.same_world_binding, true);
  assert.equal(delta.harness_changed, true);
  assert.equal(delta.session_changed, true);
  assert.equal(delta.model_condition_changed, true);
  assert.equal(delta.material_binding_changed, true);
});

test("observed state requires evidence rather than reachability inference", () => {
  assert.throws(
    () => validateRealisedActuation(collapsed({ observation: { state: "observed", evidence_refs: [] } })),
    /at least one evidence ref/,
  );
});
