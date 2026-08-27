import assert from "node:assert/strict";
import test from "node:test";

import {
  ACTUATION_STREAM_VERSION,
  ActuationStreamJournal,
  actuationStreamReadModel,
  appendActuationStreamEvent,
  closeActuationStream,
  validateActuationStream,
} from "./actuation-stream.mjs";

function baseStream() {
  return {
    schema: ACTUATION_STREAM_VERSION,
    stream_ref: "stream:agency-1",
    actuation_ref: "actuation:agency-1",
    agency_ref: "agency:root",
    agent_session_ref: "agent-session:primary",
    world_binding_ref: "world:personal",
    participating_loci: ["locus:root", "locus:delegate"],
    surface_refs: ["surface:cradle", "surface:telegram"],
    lifecycle: { state: "open", started_at: "2026-08-27T10:00:00Z" },
    cursor: { last_sequence: 0, next_sequence: 1 },
    events: [],
    provenance: ["Actuation #15 fixture"],
  };
}

function event(sequence, kind, extra = {}) {
  return {
    event_ref: `event:${sequence}`,
    sequence,
    kind,
    observed_at: `2026-08-27T10:00:0${Math.min(sequence, 9)}Z`,
    actor: { locus_ref: "locus:root", agency_ref: "agency:root", agent_ref: "agent:root" },
    disclosure: "portable",
    ...extra,
  };
}

test("validates an empty open stream with distinct semantic identities", () => {
  assert.equal(validateActuationStream(baseStream()).cursor.next_sequence, 1);
  const collapsed = baseStream();
  collapsed.stream_ref = collapsed.agent_session_ref;
  assert.throws(() => validateActuationStream(collapsed), /distinct identities/);
});

test("append preserves contiguous ordering and attributable heterogeneous events", () => {
  let stream = appendActuationStreamEvent(baseStream(), event(1, "human-message", {
    surface_ref: "surface:telegram",
    content: "Please inspect the build",
  }));
  stream = appendActuationStreamEvent(stream, event(2, "tool-request", {
    native_trace_ref: "trace:harness/tool-call-42",
    resource_refs: ["capability:factory.inspect"],
  }));
  stream = appendActuationStreamEvent(stream, event(3, "tool-result", {
    native_trace_ref: "trace:harness/tool-call-42/result",
    evidence_refs: ["evidence:factory-build-state"],
  }));
  assert.equal(stream.cursor.last_sequence, 3);
  assert.equal(stream.cursor.next_sequence, 4);
  assert.throws(() => appendActuationStreamEvent(stream, event(5, "model-message")), /sequence must equal 4/);
});

test("multi-locus events retain their own attribution without changing governing Agency identity", () => {
  const stream = appendActuationStreamEvent(baseStream(), event(1, "delegation", {
    actor: { locus_ref: "locus:delegate", agency_ref: "agency:delegate", agent_ref: "agent:delegate" },
    resource_refs: ["actuation:nested"],
  }));
  assert.equal(stream.agency_ref, "agency:root");
  assert.equal(stream.events[0].actor.agency_ref, "agency:delegate");
  assert.deepEqual(stream.participating_loci, ["locus:root", "locus:delegate"]);
});

test("surface attribution is event material and surface loss does not change Stream identity", () => {
  let stream = appendActuationStreamEvent(baseStream(), event(1, "human-message", {
    surface_ref: "surface:telegram",
    content: "hello",
  }));
  stream = { ...stream, surface_refs: ["surface:cradle"] };
  const reading = actuationStreamReadModel(stream);
  assert.equal(reading.stream_ref, "stream:agency-1");
  assert.equal(reading.events[0].surface_ref, "surface:telegram");
  assert.deepEqual(reading.surface_refs, ["surface:cradle"]);
});

test("return events correlate a Return without reducing it to final text", () => {
  const stream = appendActuationStreamEvent(baseStream(), event(1, "return", {
    return_ref: "return:run-7",
    resource_refs: ["artifact:patch"],
    evidence_refs: ["evidence:test-run"],
    disclosure: "reference-only",
  }));
  assert.equal(stream.events[0].return_ref, "return:run-7");
  assert.equal(stream.events[0].content, undefined);
  assert.throws(() => appendActuationStreamEvent(baseStream(), event(1, "return")), /requires return_ref/);
});

test("reference-only events cannot inline undisclosed provider-native material", () => {
  assert.throws(
    () => appendActuationStreamEvent(baseStream(), event(1, "model-result", {
      disclosure: "reference-only",
      native_trace_ref: "trace:provider/native",
      content: "hidden provider material",
    })),
    /must not inline content/,
  );
});

test("read model supports cursor-based replay without changing canonical ordering", () => {
  let stream = baseStream();
  for (let sequence = 1; sequence <= 4; sequence += 1) {
    stream = appendActuationStreamEvent(stream, event(sequence, "model-message", { content: `turn ${sequence}` }));
  }
  const page = actuationStreamReadModel(stream, { afterSequence: 1, limit: 2 });
  assert.deepEqual(page.events.map((item) => item.sequence), [2, 3]);
  assert.equal(page.cursor.returned_through, 3);
  assert.equal(page.cursor.has_more, true);
  assert.equal(page.cursor.stream_last_sequence, 4);
});

test("journal can replay and subscribe to one canonical Stream", () => {
  const stream = appendActuationStreamEvent(baseStream(), event(1, "human-message", { content: "first" }));
  const journal = new ActuationStreamJournal(stream);
  const seen = [];
  const unsubscribe = journal.subscribe((item) => seen.push(item.sequence), { afterSequence: 0, replay: true });
  journal.append(event(2, "model-message", { content: "second" }));
  unsubscribe();
  journal.append(event(3, "world-observation", { resource_refs: ["world-observation:3"] }));
  assert.deepEqual(seen, [1, 2]);
  assert.equal(journal.snapshot().cursor.stream_last_sequence, 3);
});

test("closing a Stream preserves history and forbids further append", () => {
  const stream = appendActuationStreamEvent(baseStream(), event(1, "interruption", { content: "human interrupted" }));
  const closed = closeActuationStream(stream, { state: "interrupted", endedAt: "2026-08-27T10:01:00Z" });
  assert.equal(closed.lifecycle.state, "interrupted");
  assert.equal(closed.events.length, 1);
  assert.throws(() => appendActuationStreamEvent(closed, event(2, "model-message")), /cannot append/);
});
