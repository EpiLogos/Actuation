import { validateActuationStream } from "./actuation-stream.mjs";

export const ACTIVITY_VERSION = "actuation.activity/v1";

const PHASES = new Set([
  "queued",
  "running",
  "waiting",
  "completed",
  "failed",
  "interrupted",
  "cancelled",
]);
const OUTCOMES = new Set(["pending", "succeeded", "failed", "degraded", "refused", "cancelled"]);
const SALIENCE = new Set(["ambient", "normal", "important", "critical"]);

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

function text(value, name) {
  if (typeof value !== "string" || value.trim() === "") {
    throw new TypeError(`${name} must be a non-empty string`);
  }
}

function timestamp(value, name, { optional = false } = {}) {
  if (value == null && optional) return;
  if (typeof value !== "string" || value.trim() === "" || Number.isNaN(Date.parse(value))) {
    throw new TypeError(`${name} must be an ISO-compatible timestamp string`);
  }
}

function validateActor(value) {
  const actor = object(value, "Activity.actor");
  ref(actor.agent_ref, "Activity.actor.agent_ref", { optional: true });
  ref(actor.agency_ref, "Activity.actor.agency_ref", { optional: true });
  ref(actor.participant_ref, "Activity.actor.participant_ref", { optional: true });
  ref(actor.locus_ref, "Activity.actor.locus_ref", { optional: true });
  if (actor.agent_ref == null && actor.agency_ref == null && actor.participant_ref == null && actor.locus_ref == null) {
    throw new TypeError("Activity.actor must attribute activity to an Agent, Agency, Participant or locus");
  }
  return actor;
}

function validateTrace(value) {
  const trace = object(value, "Activity.trace");
  ref(trace.stream_ref, "Activity.trace.stream_ref");
  refs(trace.event_refs, "Activity.trace.event_refs", { optional: true });
  refs(trace.native_trace_refs, "Activity.trace.native_trace_refs", { optional: true });
  if (trace.from_sequence != null && (!Number.isSafeInteger(trace.from_sequence) || trace.from_sequence < 1)) {
    throw new TypeError("Activity.trace.from_sequence must be a positive safe integer");
  }
  if (trace.through_sequence != null && (!Number.isSafeInteger(trace.through_sequence) || trace.through_sequence < 1)) {
    throw new TypeError("Activity.trace.through_sequence must be a positive safe integer");
  }
  if (
    trace.from_sequence != null &&
    trace.through_sequence != null &&
    trace.through_sequence < trace.from_sequence
  ) {
    throw new TypeError("Activity.trace.through_sequence must not precede from_sequence");
  }
  return trace;
}

export function validateActivity(input) {
  const activity = object(input, "Activity");
  if (activity.schema !== ACTIVITY_VERSION) {
    throw new TypeError(`Activity.schema must equal ${ACTIVITY_VERSION}`);
  }
  ref(activity.activity_ref, "Activity.activity_ref");
  validateActor(activity.actor);
  ref(activity.agent_session_ref, "Activity.agent_session_ref", { optional: true });
  ref(activity.subject_ref, "Activity.subject_ref");
  ref(activity.native_owner, "Activity.native_owner");
  ref(activity.action_ref, "Activity.action_ref", { optional: true });
  ref(activity.invocation_ref, "Activity.invocation_ref", { optional: true });
  ref(activity.actuation_ref, "Activity.actuation_ref", { optional: true });
  ref(activity.result_ref, "Activity.result_ref", { optional: true });
  ref(activity.return_ref, "Activity.return_ref", { optional: true });
  refs(activity.evidence_refs, "Activity.evidence_refs", { optional: true });
  text(activity.verb, "Activity.verb");
  text(activity.object, "Activity.object");
  text(activity.summary, "Activity.summary");
  if (!PHASES.has(activity.phase)) {
    throw new TypeError("Activity.phase must name a supported semantic phase");
  }
  if (!OUTCOMES.has(activity.outcome)) {
    throw new TypeError("Activity.outcome must name a supported semantic outcome");
  }
  if (!SALIENCE.has(activity.salience)) {
    throw new TypeError("Activity.salience must be ambient, normal, important or critical");
  }
  if (typeof activity.needs_attention !== "boolean") {
    throw new TypeError("Activity.needs_attention must be boolean");
  }
  validateTrace(activity.trace);
  timestamp(activity.started_at, "Activity.started_at");
  timestamp(activity.updated_at, "Activity.updated_at");
  timestamp(activity.completed_at, "Activity.completed_at", { optional: true });
  if (["completed", "failed", "interrupted", "cancelled"].includes(activity.phase) && activity.completed_at == null) {
    throw new TypeError("terminal Activity.phase requires completed_at");
  }
  if (["queued", "running", "waiting"].includes(activity.phase) && activity.completed_at != null) {
    throw new TypeError("non-terminal Activity.phase must not declare completed_at");
  }
  if (activity.metadata != null) object(activity.metadata, "Activity.metadata");
  return activity;
}

function phaseFromStream(stream) {
  switch (stream.lifecycle.state) {
    case "open":
      return "running";
    case "closed":
      return "completed";
    case "interrupted":
      return "interrupted";
    case "cancelled":
      return "cancelled";
    default:
      throw new TypeError(`unsupported ActuationStream lifecycle ${stream.lifecycle.state}`);
  }
}

function outcomeFromPhase(phase) {
  switch (phase) {
    case "completed":
      return "succeeded";
    case "cancelled":
      return "cancelled";
    case "interrupted":
      return "degraded";
    default:
      return "pending";
  }
}

function actorFromEvent(stream, event) {
  const actor = event?.actor ?? {};
  const resolved = {
    agent_ref: actor.agent_ref,
    agency_ref: actor.agency_ref ?? stream.agency_ref,
    participant_ref: actor.participant_ref,
    locus_ref: actor.locus_ref,
  };
  return Object.fromEntries(Object.entries(resolved).filter(([, value]) => value != null));
}

/**
 * Project one semantic Activity over an existing canonical ActuationStream.
 * This does not mutate the stream and does not create Notification/Attention state.
 */
export function activityFromActuationStream(
  streamInput,
  {
    activityRef,
    subjectRef,
    nativeOwner,
    verb,
    object: activityObject,
    summary,
    salience = "normal",
    needsAttention = false,
    actionRef,
    invocationRef,
    resultRef,
    evidenceRefs = [],
    returnRef,
    outcome,
    metadata,
  },
) {
  const stream = validateActuationStream(streamInput);
  const lastEvent = stream.events.at(-1);
  const phase = phaseFromStream(stream);
  const startedAt = stream.lifecycle.started_at ?? stream.events.at(0)?.observed_at;
  const updatedAt = stream.lifecycle.ended_at ?? lastEvent?.observed_at ?? startedAt;
  if (startedAt == null || updatedAt == null) {
    throw new TypeError("Activity projection requires stream lifecycle/event timestamps");
  }

  const activity = {
    schema: ACTIVITY_VERSION,
    activity_ref: activityRef,
    actor: actorFromEvent(stream, lastEvent),
    agent_session_ref: stream.agent_session_ref,
    subject_ref: subjectRef,
    native_owner: nativeOwner,
    action_ref: actionRef,
    invocation_ref: invocationRef,
    actuation_ref: stream.actuation_ref,
    result_ref: resultRef,
    evidence_refs: [...evidenceRefs],
    return_ref: returnRef ?? lastEvent?.return_ref,
    verb,
    object: activityObject,
    summary,
    phase,
    outcome: outcome ?? outcomeFromPhase(phase),
    salience,
    needs_attention: needsAttention,
    trace: {
      stream_ref: stream.stream_ref,
      event_refs: stream.events.map((event) => event.event_ref),
      native_trace_refs: stream.events.flatMap((event) => event.native_trace_ref ? [event.native_trace_ref] : []),
      ...(stream.events.length > 0 ? { from_sequence: 1, through_sequence: stream.events.length } : {}),
    },
    started_at: startedAt,
    updated_at: updatedAt,
    ...(phase === "completed" || phase === "interrupted" || phase === "cancelled"
      ? { completed_at: updatedAt }
      : {}),
    ...(metadata == null ? {} : { metadata: structuredClone(metadata) }),
  };
  return validateActivity(activity);
}

/**
 * Project a single meaningful portable stream event as semantic Activity. It is
 * valid for the event to have no Action/Invocation identity: observation is not
 * counterfeited into an Action merely to make it visible.
 */
export function activityFromStreamEvent(
  streamInput,
  eventRef,
  {
    activityRef,
    subjectRef,
    nativeOwner,
    verb,
    object: activityObject,
    summary,
    phase = "completed",
    outcome = "succeeded",
    salience = "normal",
    needsAttention = false,
    actionRef,
    invocationRef,
    resultRef,
    evidenceRefs = [],
    returnRef,
  },
) {
  const stream = validateActuationStream(streamInput);
  const event = stream.events.find((candidate) => candidate.event_ref === eventRef);
  if (!event) throw new TypeError(`ActuationStream contains no event ${eventRef}`);
  const observedAt = event.observed_at ?? stream.lifecycle.started_at;
  if (observedAt == null) throw new TypeError("Activity event projection requires a timestamp");

  return validateActivity({
    schema: ACTIVITY_VERSION,
    activity_ref: activityRef,
    actor: actorFromEvent(stream, event),
    agent_session_ref: stream.agent_session_ref,
    subject_ref: subjectRef,
    native_owner: nativeOwner,
    action_ref: actionRef,
    invocation_ref: invocationRef,
    actuation_ref: stream.actuation_ref,
    result_ref: resultRef,
    evidence_refs: [...evidenceRefs, ...(event.evidence_refs ?? [])],
    return_ref: returnRef ?? event.return_ref,
    verb,
    object: activityObject,
    summary,
    phase,
    outcome,
    salience,
    needs_attention: needsAttention,
    trace: {
      stream_ref: stream.stream_ref,
      event_refs: [event.event_ref],
      native_trace_refs: event.native_trace_ref ? [event.native_trace_ref] : [],
      from_sequence: event.sequence,
      through_sequence: event.sequence,
    },
    started_at: observedAt,
    updated_at: observedAt,
    ...(["completed", "failed", "interrupted", "cancelled"].includes(phase)
      ? { completed_at: observedAt }
      : {}),
  });
}

export function activityNeedsAttention(activityInput) {
  return validateActivity(activityInput).needs_attention;
}
