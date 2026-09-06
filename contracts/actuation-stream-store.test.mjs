import assert from "node:assert/strict";
import test from "node:test";
import { mkdtempSync, appendFileSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import {
  closeDurableStream,
  foldStreamFile,
  loadDurableStream,
  openDurableStream,
  recordBoundaryOccurrence,
  replayDurableStream,
  streamFileName,
} from "./actuation-stream-store.mjs";
import { capabilityDescriptorBySlug } from "../detection/catalog.mjs";

function store() {
  return mkdtempSync(join(tmpdir(), "actuation-stream-store-"));
}

const identity = {
  stream_ref: "actuation:stream:store-test",
  actuation_ref: "actuation:store-test",
  agency_ref: "agency:store-test",
  agent_session_ref: "agent-session:store-test",
};

// A boundary occurrence for the fixture stream; `identity` is supplied in the
// occurrence document's own shape so a first record can open the stream.
function occurrence(root, overrides = {}) {
  return {
    root,
    stream_ref: identity.stream_ref,
    identity: {
      actuation_ref: identity.actuation_ref,
      agency_ref: identity.agency_ref,
      agent_session_ref: identity.agent_session_ref,
    },
    harness: "zcode",
    ...overrides,
  };
}

// The zcode descriptor (catalog r4) carries both mappable boundaries and
// disclosed customs; deriving the fixtures from the descriptor itself keeps
// the tests honest across catalog revisions.
function firstMappedNativeEvent(slug = "zcode") {
  const capability = capabilityDescriptorBySlug(slug);
  const mapped = capability.native_events.find((event) => event.event !== "custom");
  assert.ok(mapped, `${slug} declares at least one mappable native event`);
  return mapped;
}

function firstCustomNativeEvent(slug = "zcode") {
  const capability = capabilityDescriptorBySlug(slug);
  const custom = capability.native_events.find((event) => event.event === "custom");
  assert.ok(custom, `${slug} declares at least one disclosed custom event`);
  return custom.native_name;
}

// ---------------------------------------------------------------------------
// Opening and appending
// ---------------------------------------------------------------------------

test("a first occurrence opens the stream and lands at sequence 1 with an exact cursor", () => {
  const root = store();
  const result = recordBoundaryOccurrence(
    occurrence(root, { native_event: firstMappedNativeEvent().native_name, observed_at: "2026-09-06T12:00:00Z" }),
  );

  assert.equal(result.event.sequence, 1);
  assert.equal(result.event.kind, "harness-event");
  assert.deepEqual(result.cursor, { last_sequence: 1, next_sequence: 2 });
  assert.equal(result.lifecycle.state, "open");
});

test("recording continues the contiguous sequence across independent calls (restart-safe)", () => {
  const root = store();
  recordBoundaryOccurrence(occurrence(root, { native_event: firstMappedNativeEvent().native_name }));
  const second = recordBoundaryOccurrence(occurrence(root, { native_event: firstMappedNativeEvent().native_name }));
  assert.equal(second.event.sequence, 2);

  const loaded = loadDurableStream({ root, stream_ref: identity.stream_ref });
  assert.equal(loaded.cursor.last_sequence, 2);
  assert.equal(loaded.cursor.next_sequence, 3);
  assert.equal(loaded.events.length, 2);
});

test("an undeclared native event is refused, naming what the descriptor declares", () => {
  const root = store();
  const capability = capabilityDescriptorBySlug("zcode");
  const declared = capability.native_events.map((event) => event.native_name);
  assert.ok(!declared.includes("Notification"), "Notification must stay undeclared for zcode");

  assert.throws(
    () => recordBoundaryOccurrence(occurrence(root, { native_event: "Notification" })),
    (error) => error.message.includes("Notification") && declared.every((name) => error.message.includes(name)),
  );
});

test("a mappable event records its boundary; a disclosed custom records an honest null", () => {
  const root = store();
  const descriptorEvent = firstMappedNativeEvent();
  const mapped = recordBoundaryOccurrence(occurrence(root, { native_event: descriptorEvent.native_name }));
  assert.equal(mapped.event.metadata.boundary, descriptorEvent.event);
  assert.equal(mapped.event.metadata.harness, "zcode");

  const custom = recordBoundaryOccurrence(
    occurrence(root, {
      stream_ref: "actuation:stream:store-test-custom",
      native_event: firstCustomNativeEvent(),
    }),
  );
  assert.equal(custom.event.metadata.boundary, null, "a custom is disclosed, never mapped");
});

test("a duplicate caller event_ref is refused, so crash-retry stays visible", () => {
  const root = store();
  recordBoundaryOccurrence(
    occurrence(root, { native_event: firstMappedNativeEvent().native_name, event_ref: "actuation:event:fixed" }),
  );
  assert.throws(
    () => recordBoundaryOccurrence(
      occurrence(root, { native_event: firstMappedNativeEvent().native_name, event_ref: "actuation:event:fixed" }),
    ),
    /already exists/,
  );
});

test("reopening a stream_ref with different identity is refused", () => {
  const root = store();
  recordBoundaryOccurrence(occurrence(root, { native_event: firstMappedNativeEvent().native_name }));
  assert.throws(
    () => openDurableStream({ root, ...identity, actuation_ref: "actuation:someone-else" }),
    /refusing to reopen/,
  );
});

test("a record into a stream that does not exist requires identity to open it", () => {
  const root = store();
  assert.throws(
    () => recordBoundaryOccurrence({
      root,
      stream_ref: "actuation:stream:ghost",
      harness: "zcode",
      native_event: firstMappedNativeEvent().native_name,
    }),
    /supply its identity/,
  );
});

// ---------------------------------------------------------------------------
// Replay and close
// ---------------------------------------------------------------------------

test("replay pages the durable sequence exactly as the portable read model does", () => {
  const root = store();
  for (let index = 0; index < 3; index += 1) {
    recordBoundaryOccurrence(occurrence(root, { native_event: firstMappedNativeEvent().native_name }));
  }

  const page = replayDurableStream({ root, stream_ref: identity.stream_ref, afterSequence: 1, limit: 1 });
  assert.deepEqual(page.events.map((event) => event.sequence), [2]);
  assert.equal(page.cursor.has_more, true);
  assert.equal(page.cursor.stream_last_sequence, 3);

  const rest = replayDurableStream({ root, stream_ref: identity.stream_ref, afterSequence: 2 });
  assert.deepEqual(rest.events.map((event) => event.sequence), [3]);
  assert.equal(rest.cursor.has_more, false);
});

test("close is terminal, records when it happened, and refuses further appends", () => {
  const root = store();
  recordBoundaryOccurrence(occurrence(root, { native_event: firstMappedNativeEvent().native_name }));

  const closed = closeDurableStream({ root, stream_ref: identity.stream_ref });
  assert.equal(closed.lifecycle.state, "closed");
  assert.ok(closed.lifecycle.ended_at, "a close records when it happened");

  assert.throws(
    () => recordBoundaryOccurrence(occurrence(root, { native_event: firstMappedNativeEvent().native_name })),
    /terminal/,
  );
  assert.throws(
    () => closeDurableStream({ root, stream_ref: identity.stream_ref }),
    /already terminal/,
  );
});

// ---------------------------------------------------------------------------
// The file on disk
// ---------------------------------------------------------------------------

test("the store file is the percent-encoded stream_ref and folds back exactly", () => {
  const root = store();
  recordBoundaryOccurrence(occurrence(root, { native_event: firstMappedNativeEvent().native_name }));

  const path = join(root, streamFileName(identity.stream_ref));
  assert.equal(streamFileName(identity.stream_ref), `${encodeURIComponent(identity.stream_ref)}.jsonl`);
  const folded = foldStreamFile(readFileSync(path, "utf8"));
  assert.equal(folded.events.length, 1);
  assert.equal(folded.cursor.last_sequence, 1);
  // Header line + one event line.
  assert.equal(readFileSync(path, "utf8").trim().split("\n").length, 2);
});

test("a torn tail refuses to load and names its position instead of being dropped", () => {
  const root = store();
  recordBoundaryOccurrence(occurrence(root, { native_event: firstMappedNativeEvent().native_name }));

  const path = join(root, streamFileName(identity.stream_ref));
  appendFileSync(path, '{"event_ref":"actuation:event:torn","seq');
  assert.throws(
    () => loadDurableStream({ root, stream_ref: identity.stream_ref }),
    (error) => error.message.includes("torn") && error.message.includes("position 3"),
  );
});

test("a corrupted event line refuses to load rather than validating a lie", () => {
  const root = store();
  recordBoundaryOccurrence(occurrence(root, { native_event: firstMappedNativeEvent().native_name }));

  const path = join(root, streamFileName(identity.stream_ref));
  // A well-formed JSON line that violates the sequence law.
  appendFileSync(path, `${JSON.stringify({ event_ref: "actuation:event:gap", sequence: 7, kind: "harness-event" })}\n`);
  assert.throws(
    () => loadDurableStream({ root, stream_ref: identity.stream_ref }),
    /violates the portable contract/,
  );
});
