// The durable ActuationStream store: the first-party persistence decision for
// occurrence recording (PROGRAMME W3 — "boundary events land in
// actuation-stream-v1; durable stream store decided and built").
//
// Decision: one append-only JSONL file per stream under a store root. Line 1 is
// the stream header (identities, lifecycle); every following line is exactly
// one committed event. Event appends are single-line appends; lifecycle
// transitions rewrite the file atomically (temp + rename). The portable
// contract stays the only law — the folded file must validate as an
// ActuationStream with an exact contiguous cursor, and a torn or invalid tail
// refuses to load rather than being silently dropped.
//
// `ActuationStreamJournal` (actuation-stream.mjs) remains the behavioural
// reference; this module persists the same contract, it does not replace it.

import {
  randomUUID,
} from "node:crypto";
import {
  appendFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  renameSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { homedir } from "node:os";
import {
  ACTUATION_STREAM_VERSION,
  actuationStreamReadModel,
  appendActuationStreamEvent,
  closeActuationStream,
  validateActuationStream,
  validateActuationStreamEvent,
} from "./actuation-stream.mjs";
import { capabilityDescriptorBySlug } from "../detection/catalog.mjs";

export const ACTUATION_STREAM_STORE_DEFAULT_ROOT = join(homedir(), ".actuation", "streams");

export function streamStoreRoot({ store } = {}) {
  const root = store ?? process.env.ACTUATION_STREAM_STORE ?? ACTUATION_STREAM_STORE_DEFAULT_ROOT;
  if (typeof root !== "string" || root.trim() === "") {
    throw new TypeError("stream store root must be a non-empty directory path");
  }
  return root;
}

// Refs may carry "/", ":" and other path-hostile characters; percent-encoding
// keeps the filename reversible and cannot traverse.
export function streamFileName(streamRef) {
  if (typeof streamRef !== "string" || streamRef.trim() === "") {
    throw new TypeError("stream_ref must be a non-empty ref");
  }
  return `${encodeURIComponent(streamRef)}.jsonl`;
}

function streamFilePath(root, streamRef) {
  return join(streamStoreRoot({ store: root }), streamFileName(streamRef));
}

function emptyStreamDocument(identity, { startedAt } = {}) {
  const header = {
    schema: ACTUATION_STREAM_VERSION,
    stream_ref: identity.stream_ref,
    actuation_ref: identity.actuation_ref,
    agency_ref: identity.agency_ref,
    agent_session_ref: identity.agent_session_ref,
    lifecycle: { state: "open" },
    cursor: { last_sequence: 0, next_sequence: 1 },
    events: [],
  };
  if (identity.world_binding_ref != null) header.world_binding_ref = identity.world_binding_ref;
  if (startedAt != null) header.lifecycle.started_at = startedAt;
  if (identity.provenance != null) header.provenance = identity.provenance;
  return header;
}

function checkIdentityConsistency(existing, identity) {
  for (const key of ["actuation_ref", "agency_ref", "agent_session_ref"]) {
    if (identity[key] != null && existing[key] !== identity[key]) {
      throw new TypeError(
        `ActuationStream ${key} is already ${existing[key]}; refusing to reopen ${existing.stream_ref} as ${identity[key]}`,
      );
    }
  }
}

// ---------------------------------------------------------------------------
// Fold: header + event lines -> one validated portable stream
// ---------------------------------------------------------------------------

export function foldStreamFile(raw) {
  const lines = raw.split("\n").filter((line, index, all) => {
    // Drop only the trailing empty piece a final newline produces; an empty
    // line anywhere else is a hole in the journal, not formatting.
    return index !== all.length - 1 || line !== "";
  });
  if (lines.length === 0) {
    throw new TypeError("durable ActuationStream file is empty; refusing to fold");
  }
  let header;
  try {
    header = JSON.parse(lines[0]);
  } catch (error) {
    throw new TypeError(`durable ActuationStream header is not valid JSON (${error.message})`);
  }
  // The header's cursor is derived state; the event lines are authoritative.
  // The fold always starts from zero and recomputes it.
  const stream = { ...header, events: [], cursor: { last_sequence: 0, next_sequence: 1 } };
  for (let index = 1; index < lines.length; index += 1) {
    let event;
    try {
      event = JSON.parse(lines[index]);
    } catch (error) {
      throw new TypeError(
        `durable ActuationStream has a torn or invalid event line at position ${index + 1} (${error.message}); the tail is not silently dropped`,
      );
    }
    try {
      validateActuationStreamEvent(event, { expectedSequence: stream.cursor.next_sequence });
    } catch (error) {
      throw new TypeError(
        `durable ActuationStream event at position ${index + 1} violates the portable contract: ${error.message}`,
      );
    }
    stream.events.push(event);
    stream.cursor.last_sequence = event.sequence;
    stream.cursor.next_sequence = event.sequence + 1;
  }
  return validateActuationStream(stream);
}

function loadStreamFile(path) {
  if (!existsSync(path)) return null;
  return foldStreamFile(readFileSync(path, "utf8"));
}

function atomicRewrite(path, stream) {
  // The stored header never carries event material or a live cursor: those are
  // the event lines' truth, and the fold recomputes them from zero.
  const header = { ...structuredClone(stream), events: [], cursor: { last_sequence: 0, next_sequence: 1 } };
  const lines = [JSON.stringify(header)];
  for (const event of stream.events) lines.push(JSON.stringify(event));
  const temporary = `${path}.tmp-${randomUUID()}`;
  writeFileSync(temporary, `${lines.join("\n")}\n`, { encoding: "utf8" });
  renameSync(temporary, path);
}

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------

export function openDurableStream({ root, stream_ref, actuation_ref, agency_ref, agent_session_ref, world_binding_ref, provenance, started_at } = {}) {
  const identity = { stream_ref, actuation_ref, agency_ref, agent_session_ref, world_binding_ref, provenance };
  for (const key of ["stream_ref", "actuation_ref", "agency_ref", "agent_session_ref"]) {
    if (typeof identity[key] !== "string" || identity[key].trim() === "") {
      throw new TypeError(`opening a durable ActuationStream requires a non-empty ${key}`);
    }
  }
  const path = streamFilePath(root, stream_ref);
  const existing = loadStreamFile(path);
  if (existing) {
    checkIdentityConsistency(existing, identity);
    return actuationStreamReadModel(existing);
  }
  mkdirSync(streamStoreRoot({ store: root }), { recursive: true });
  const header = validateActuationStream(emptyStreamDocument(identity, { startedAt: started_at }));
  atomicRewrite(path, header);
  return actuationStreamReadModel(header);
}

export function loadDurableStream({ root, stream_ref } = {}) {
  const loaded = loadStreamFile(streamFilePath(root, stream_ref));
  if (!loaded) {
    throw new TypeError(`no durable ActuationStream named ${stream_ref} under ${streamStoreRoot({ store: root })}`);
  }
  return loaded;
}

export function replayDurableStream({ root, stream_ref, afterSequence = 0, limit } = {}) {
  const stream = loadDurableStream({ root, stream_ref });
  const options = { afterSequence };
  if (limit != null) options.limit = limit;
  return actuationStreamReadModel(stream, options);
}

export function closeDurableStream({ root, stream_ref, state = "closed", ended_at } = {}) {
  const path = streamFilePath(root, stream_ref);
  const stream = loadStreamFile(path);
  if (!stream) {
    throw new TypeError(`no durable ActuationStream named ${stream_ref} under ${streamStoreRoot({ store: root })}`);
  }
  // A close records when it happened; a caller-supplied timestamp wins.
  const closed = closeActuationStream(stream, { state, endedAt: ended_at ?? new Date().toISOString() });
  atomicRewrite(path, closed);
  return actuationStreamReadModel(closed);
}

// ---------------------------------------------------------------------------
// Boundary occurrence recording
// ---------------------------------------------------------------------------

export function recordBoundaryOccurrence({ root, stream_ref, harness, native_event, identity, event_ref, observed_at, content, native_trace_ref, metadata } = {}) {
  if (typeof stream_ref !== "string" || stream_ref.trim() === "") {
    throw new TypeError("recording a boundary occurrence requires a non-empty stream_ref");
  }
  if (typeof harness !== "string" || harness.trim() === "") {
    throw new TypeError("recording a boundary occurrence requires a non-empty harness slug");
  }
  if (typeof native_event !== "string" || native_event.trim() === "") {
    throw new TypeError("recording a boundary occurrence requires a non-empty native_event");
  }
  const capability = capabilityDescriptorBySlug(harness);
  if (!capability) {
    throw new TypeError(`no capability descriptor declared for harness ${harness}; occurrence recording is descriptor-driven`);
  }
  const declared = capability.native_events.find((event) => event.native_name === native_event);
  if (!declared) {
    throw new TypeError(
      `harness ${harness} does not declare native event ${native_event}; declared: ${capability.native_events.map((event) => event.native_name).join(", ")}`,
    );
  }
  const boundary = declared.event === "custom" ? null : declared.event;

  const path = streamFilePath(root, stream_ref);
  let stream = loadStreamFile(path);
  if (!stream) {
    if (!identity) {
      throw new TypeError(
        `durable ActuationStream ${stream_ref} does not exist; supply its identity (actuation_ref, agency_ref, agent_session_ref) to open it with this occurrence`,
      );
    }
    openDurableStream({ root, ...identity, stream_ref });
    stream = loadStreamFile(path);
  } else if (identity) {
    checkIdentityConsistency(stream, identity);
  }

  const occurrence = {
    event_ref: event_ref ?? `actuation:event:${randomUUID()}`,
    sequence: stream.cursor.next_sequence,
    kind: "harness-event",
    metadata: {
      harness,
      native_event,
      boundary,
      catalog_revision: capability.provenance?.catalog_revision ?? null,
      ...(metadata ?? {}),
    },
  };
  if (observed_at != null) occurrence.observed_at = observed_at;
  if (content != null) occurrence.content = content;
  if (native_trace_ref != null) occurrence.native_trace_ref = native_trace_ref;

  const appended = appendActuationStreamEvent(stream, occurrence);
  // The fold validated the whole event already; commit is a single append.
  appendFileSync(path, `${JSON.stringify(appended.events.at(-1))}\n`, { encoding: "utf8" });
  return {
    stream_ref,
    event: structuredClone(appended.events.at(-1)),
    cursor: { ...appended.cursor },
    lifecycle: { ...appended.lifecycle },
  };
}
