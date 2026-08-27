export const ACTUATION_STREAM_VERSION = "actuation.stream/v1";

const STREAM_STATES = new Set(["open", "closed", "interrupted", "cancelled"]);
const EVENT_KINDS = new Set([
  "human-message",
  "model-message",
  "model-delta",
  "model-result",
  "capability-request",
  "capability-result",
  "tool-request",
  "tool-result",
  "harness-event",
  "world-observation",
  "delegation",
  "locus-event",
  "permission",
  "refusal",
  "interruption",
  "cancellation",
  "execution-event",
  "artifact",
  "evidence",
  "return",
  "custom",
]);
const DISCLOSURE_STATES = new Set(["portable", "surface", "reference-only"]);

function object(value, name) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new TypeError(`${name} must be an object`);
  }
  return value;
}

function ref(value, name, { optional = false } = {}) {
  if (value == null && optional) return;
  if (typeof value !== "string" || value.trim() === "") {
    throw new TypeError(`${name} must be a non-empty ref`);
  }
}

function refs(value, name, { optional = false } = {}) {
  if (value == null && optional) return [];
  if (!Array.isArray(value) || value.some((item) => typeof item !== "string" || item.trim() === "")) {
    throw new TypeError(`${name} must be an array of non-empty refs`);
  }
  return value;
}

function strings(value, name, { optional = false } = {}) {
  if (value == null && optional) return [];
  if (!Array.isArray(value) || value.some((item) => typeof item !== "string" || item.trim() === "")) {
    throw new TypeError(`${name} must be an array of non-empty strings`);
  }
  return value;
}

function integer(value, name, { minimum = 0 } = {}) {
  if (!Number.isSafeInteger(value) || value < minimum) {
    throw new TypeError(`${name} must be a safe integer >= ${minimum}`);
  }
}

function validateTimestamp(value, name, { optional = false } = {}) {
  if (value == null && optional) return;
  if (typeof value !== "string" || value.trim() === "" || Number.isNaN(Date.parse(value))) {
    throw new TypeError(`${name} must be an ISO-compatible timestamp string`);
  }
}

function validateActor(value, name, { optional = false } = {}) {
  if (value == null && optional) return null;
  const actor = object(value, name);
  ref(actor.locus_ref, `${name}.locus_ref`, { optional: true });
  ref(actor.agency_ref, `${name}.agency_ref`, { optional: true });
  ref(actor.agent_ref, `${name}.agent_ref`, { optional: true });
  ref(actor.participant_ref, `${name}.participant_ref`, { optional: true });
  if (
    actor.locus_ref == null &&
    actor.agency_ref == null &&
    actor.agent_ref == null &&
    actor.participant_ref == null
  ) {
    throw new TypeError(`${name} must attribute the event to at least one locus, Agency, Agent or Participant ref`);
  }
  if (actor.role != null && (typeof actor.role !== "string" || actor.role.trim() === "")) {
    throw new TypeError(`${name}.role must be a non-empty string when supplied`);
  }
  return actor;
}

export function validateActuationStreamEvent(input, { expectedSequence } = {}) {
  const event = object(input, "ActuationStreamEvent");
  ref(event.event_ref, "ActuationStreamEvent.event_ref");
  integer(event.sequence, "ActuationStreamEvent.sequence", { minimum: 1 });
  if (expectedSequence != null && event.sequence !== expectedSequence) {
    throw new TypeError(`ActuationStreamEvent.sequence must equal ${expectedSequence}`);
  }
  if (!EVENT_KINDS.has(event.kind)) {
    throw new TypeError("ActuationStreamEvent.kind must name a supported portable event kind");
  }
  if (event.kind === "custom") {
    if (typeof event.custom_kind !== "string" || event.custom_kind.trim() === "") {
      throw new TypeError("custom ActuationStreamEvent requires custom_kind");
    }
  } else if (event.custom_kind != null) {
    throw new TypeError("ActuationStreamEvent.custom_kind is only valid when kind is custom");
  }
  validateTimestamp(event.observed_at, "ActuationStreamEvent.observed_at", { optional: true });
  validateActor(event.actor, "ActuationStreamEvent.actor", { optional: true });
  ref(event.surface_ref, "ActuationStreamEvent.surface_ref", { optional: true });
  ref(event.execution_ref, "ActuationStreamEvent.execution_ref", { optional: true });
  ref(event.return_ref, "ActuationStreamEvent.return_ref", { optional: true });
  ref(event.native_trace_ref, "ActuationStreamEvent.native_trace_ref", { optional: true });
  refs(event.resource_refs, "ActuationStreamEvent.resource_refs", { optional: true });
  refs(event.evidence_refs, "ActuationStreamEvent.evidence_refs", { optional: true });

  const disclosure = event.disclosure ?? "portable";
  if (!DISCLOSURE_STATES.has(disclosure)) {
    throw new TypeError("ActuationStreamEvent.disclosure must be portable, surface or reference-only");
  }
  if (disclosure === "reference-only" && event.content != null) {
    throw new TypeError("reference-only ActuationStreamEvent must not inline content");
  }
  if (event.content != null && typeof event.content !== "string") {
    throw new TypeError("ActuationStreamEvent.content must be a string when supplied");
  }
  if (event.metadata != null) object(event.metadata, "ActuationStreamEvent.metadata");

  if (event.kind === "return" && event.return_ref == null) {
    throw new TypeError("return ActuationStreamEvent requires return_ref");
  }
  if ((event.kind === "artifact" || event.kind === "evidence") && (event.resource_refs?.length ?? 0) === 0 && (event.evidence_refs?.length ?? 0) === 0) {
    throw new TypeError(`${event.kind} ActuationStreamEvent requires at least one resource/evidence ref`);
  }

  return event;
}

function validateDistinctIdentity(stream) {
  const identities = [
    ["stream_ref", stream.stream_ref],
    ["actuation_ref", stream.actuation_ref],
    ["agency_ref", stream.agency_ref],
    ["agent_session_ref", stream.agent_session_ref],
  ];
  for (let i = 0; i < identities.length; i += 1) {
    for (let j = i + 1; j < identities.length; j += 1) {
      if (identities[i][1] === identities[j][1]) {
        throw new TypeError(`ActuationStream ${identities[i][0]} and ${identities[j][0]} must remain distinct identities`);
      }
    }
  }
}

export function validateActuationStream(input) {
  const stream = object(input, "ActuationStream");
  if (stream.schema !== ACTUATION_STREAM_VERSION) {
    throw new TypeError(`ActuationStream.schema must equal ${ACTUATION_STREAM_VERSION}`);
  }
  ref(stream.stream_ref, "ActuationStream.stream_ref");
  ref(stream.actuation_ref, "ActuationStream.actuation_ref");
  ref(stream.agency_ref, "ActuationStream.agency_ref");
  ref(stream.agent_session_ref, "ActuationStream.agent_session_ref");
  validateDistinctIdentity(stream);
  ref(stream.world_binding_ref, "ActuationStream.world_binding_ref", { optional: true });
  refs(stream.participating_loci, "ActuationStream.participating_loci", { optional: true });
  refs(stream.surface_refs, "ActuationStream.surface_refs", { optional: true });
  strings(stream.provenance, "ActuationStream.provenance", { optional: true });

  const lifecycle = object(stream.lifecycle, "ActuationStream.lifecycle");
  if (!STREAM_STATES.has(lifecycle.state)) {
    throw new TypeError("ActuationStream.lifecycle.state must be open, closed, interrupted or cancelled");
  }
  validateTimestamp(lifecycle.started_at, "ActuationStream.lifecycle.started_at", { optional: true });
  validateTimestamp(lifecycle.ended_at, "ActuationStream.lifecycle.ended_at", { optional: true });
  if (lifecycle.state === "open" && lifecycle.ended_at != null) {
    throw new TypeError("open ActuationStream must not declare lifecycle.ended_at");
  }
  if (lifecycle.state !== "open" && lifecycle.ended_at == null) {
    throw new TypeError("terminal ActuationStream lifecycle requires ended_at");
  }

  if (!Array.isArray(stream.events)) {
    throw new TypeError("ActuationStream.events must be an array");
  }
  const eventRefs = new Set();
  let expectedSequence = 1;
  for (const event of stream.events) {
    validateActuationStreamEvent(event, { expectedSequence });
    if (eventRefs.has(event.event_ref)) {
      throw new TypeError(`ActuationStream event_ref ${event.event_ref} appears more than once`);
    }
    eventRefs.add(event.event_ref);
    expectedSequence += 1;
  }

  const cursor = object(stream.cursor, "ActuationStream.cursor");
  integer(cursor.next_sequence, "ActuationStream.cursor.next_sequence", { minimum: 1 });
  integer(cursor.last_sequence, "ActuationStream.cursor.last_sequence", { minimum: 0 });
  const expectedLast = stream.events.length;
  if (cursor.last_sequence !== expectedLast || cursor.next_sequence !== expectedLast + 1) {
    throw new TypeError("ActuationStream.cursor must exactly describe the contiguous portable event sequence");
  }

  return stream;
}

export function actuationStreamReadModel(input, { afterSequence = 0, limit = Number.POSITIVE_INFINITY } = {}) {
  const stream = validateActuationStream(input);
  integer(afterSequence, "afterSequence", { minimum: 0 });
  if (limit !== Number.POSITIVE_INFINITY && (!Number.isSafeInteger(limit) || limit < 0)) {
    throw new TypeError("limit must be a non-negative safe integer or Infinity");
  }
  const events = stream.events.filter((event) => event.sequence > afterSequence).slice(0, limit);
  const lastReturned = events.at(-1)?.sequence ?? afterSequence;
  return {
    schema: ACTUATION_STREAM_VERSION,
    stream_ref: stream.stream_ref,
    actuation_ref: stream.actuation_ref,
    agency_ref: stream.agency_ref,
    agent_session_ref: stream.agent_session_ref,
    world_binding_ref: stream.world_binding_ref ?? null,
    participating_loci: [...(stream.participating_loci ?? [])],
    surface_refs: [...(stream.surface_refs ?? [])],
    lifecycle: { ...stream.lifecycle },
    cursor: {
      after_sequence: afterSequence,
      returned_through: lastReturned,
      stream_last_sequence: stream.cursor.last_sequence,
      has_more: lastReturned < stream.cursor.last_sequence,
    },
    events: events.map((event) => structuredClone(event)),
    provenance: [...(stream.provenance ?? [])],
  };
}

export function appendActuationStreamEvent(streamInput, eventInput) {
  const stream = validateActuationStream(streamInput);
  if (stream.lifecycle.state !== "open") {
    throw new TypeError("cannot append to a terminal ActuationStream");
  }
  const event = validateActuationStreamEvent(eventInput, { expectedSequence: stream.cursor.next_sequence });
  if (stream.events.some((existing) => existing.event_ref === event.event_ref)) {
    throw new TypeError(`ActuationStream event_ref ${event.event_ref} already exists`);
  }
  const next = structuredClone(stream);
  next.events.push(structuredClone(event));
  next.cursor.last_sequence = event.sequence;
  next.cursor.next_sequence = event.sequence + 1;
  return validateActuationStream(next);
}

export function closeActuationStream(streamInput, { state = "closed", endedAt } = {}) {
  const stream = validateActuationStream(streamInput);
  if (stream.lifecycle.state !== "open") {
    throw new TypeError("ActuationStream is already terminal");
  }
  if (!new Set(["closed", "interrupted", "cancelled"]).has(state)) {
    throw new TypeError("terminal ActuationStream state must be closed, interrupted or cancelled");
  }
  validateTimestamp(endedAt, "endedAt");
  const next = structuredClone(stream);
  next.lifecycle = { ...next.lifecycle, state, ended_at: endedAt };
  return validateActuationStream(next);
}

export class ActuationStreamJournal {
  #stream;
  #subscribers = new Set();

  constructor(streamInput) {
    this.#stream = structuredClone(validateActuationStream(streamInput));
  }

  snapshot(options) {
    return actuationStreamReadModel(this.#stream, options);
  }

  append(eventInput) {
    this.#stream = appendActuationStreamEvent(this.#stream, eventInput);
    const event = structuredClone(this.#stream.events.at(-1));
    for (const subscriber of this.#subscribers) subscriber(event, this.snapshot());
    return event;
  }

  close(options) {
    this.#stream = closeActuationStream(this.#stream, options);
    return this.snapshot();
  }

  subscribe(listener, { afterSequence = this.#stream.cursor.last_sequence, replay = false } = {}) {
    if (typeof listener !== "function") throw new TypeError("ActuationStream subscriber must be a function");
    integer(afterSequence, "afterSequence", { minimum: 0 });
    if (replay) {
      for (const event of this.#stream.events) {
        if (event.sequence > afterSequence) listener(structuredClone(event), this.snapshot());
      }
    }
    this.#subscribers.add(listener);
    return () => this.#subscribers.delete(listener);
  }
}
