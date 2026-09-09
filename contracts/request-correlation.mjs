// Request-correlation contract, actuation.request-correlation/v1.
// The four-side correlation read model for a permission/activity REQUEST
// IDENTITY issued by an upstream owner (AIKit tool request, kernel
// correlation, UI decision — the same ref must be usable verbatim on all
// four sides, no re-derivation). Given that verbatim request_ref, this
// contract correlates the Actuation-side view:
//
//   authority  — the AuthorityDecision records Actuation holds for the
//                identity (allowed/refused/pending + basis: determination,
//                bounds, deciding agency, evidence);
//   activity   — the actuation.activity/v1 records named by those decisions
//                (the decision is the join hub: it names the request identity
//                AND the activity identities it governs);
//   attention  — where the owning whole tracks the identity: needs_attention
//                activities plus stream permission events still awaiting a
//                recorded disposition;
//   permission — the stream-recorded disposition of the permission boundary
//                (actuation.stream/v1 permission/refusal events referencing
//                the identity verbatim).
//
// Read-only law: this contract records nothing about live users. Explicit
// states are results, never empty fabrication: an omitted corpus side is
// reported `available: false` (absence cannot be claimed for a side that was
// never queried); `unknown-identity` means sides were queried and none
// reference the identity; `no-recorded-activity` means the identity is
// referenced but no activity record correlates; `correlation-unavailable`
// means the join cannot be completed honestly (no side queried, or the
// activity side — the join target — was not queryable).
import { validateActivity } from "./activity.mjs";
import { validateActuationStream } from "./actuation-stream.mjs";

export const REQUEST_CORRELATION_VERSION = "actuation.request-correlation/v1";

const AUTHORITY_DECISIONS = new Set(["allowed", "refused", "pending"]);
const STREAM_DISPOSITION_KINDS = new Set(["permission", "refusal"]);

function record(value, name) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new TypeError(`${name} must be an object`);
  }
  return value;
}

function ref(value, name, { optional = false } = {}) {
  if (value == null && optional) return;
  if (typeof value !== "string" || value.trim() === "") {
    throw new TypeError(`${name} must be a non-empty string ref`);
  }
}

function refs(value, name, { optional = false, nonEmpty = false } = {}) {
  if (value == null && optional) return [];
  if (!Array.isArray(value) || value.some((item) => typeof item !== "string" || item.trim() === "")) {
    throw new TypeError(`${name} must be an array of non-empty refs`);
  }
  if (nonEmpty && value.length === 0) throw new TypeError(`${name} must not be empty`);
  return value;
}

function timestamp(value, name) {
  ref(value, name);
  if (Number.isNaN(Date.parse(value))) {
    throw new TypeError(`${name} must be an ISO-compatible timestamp`);
  }
}

function text(value, name) {
  if (typeof value !== "string" || value.trim() === "") {
    throw new TypeError(`${name} must be a non-empty string`);
  }
}

// A metadata.permission_outcome a stream permission event may carry. Only
// these two spellings are read; anything else leaves the disposition pending
// (pending is a recorded state, never a guess).
const RECORDED_PERMISSION_OUTCOMES = new Set(["granted", "refused"]);

/**
 * One Actuation-side authority decision over an upstream-issued request
 * identity. This is the join hub of the four-side correlation: request_ref
 * arrives verbatim from the upstream owner, and activity_refs names the
 * Activity identities the decision governs — the same activity identities the
 * other sides hold, joined by exact string equality, never re-derived.
 */
export function validateAuthorityDecision(input) {
  const decision = record(input, "AuthorityDecision");
  if (decision.schema !== REQUEST_CORRELATION_VERSION) {
    throw new TypeError(`AuthorityDecision.schema must equal ${REQUEST_CORRELATION_VERSION}`);
  }
  if (decision.document !== "authority-decision") {
    throw new TypeError("AuthorityDecision.document must be authority-decision");
  }
  ref(decision.decision_ref, "AuthorityDecision.decision_ref");
  // The upstream-issued request identity, carried verbatim. Format is owned
  // by the issuing side (e.g. AIKit); Actuation never normalises it.
  ref(decision.request_ref, "AuthorityDecision.request_ref");
  if (!AUTHORITY_DECISIONS.has(decision.decision)) {
    throw new TypeError(`AuthorityDecision.decision must be ${[...AUTHORITY_DECISIONS].join(", ")}`);
  }
  text(decision.basis, "AuthorityDecision.basis");
  ref(decision.determination_ref, "AuthorityDecision.determination_ref");
  refs(decision.bounds_refs, "AuthorityDecision.bounds_refs", { optional: true });
  ref(decision.decided_by, "AuthorityDecision.decided_by");
  timestamp(decision.observed_at, "AuthorityDecision.observed_at");
  refs(decision.evidence_refs, "AuthorityDecision.evidence_refs", { optional: true });
  // Non-empty: a decision that names no activity identity governs nothing
  // and cannot join the correlation.
  refs(decision.activity_refs, "AuthorityDecision.activity_refs", { nonEmpty: true });
  return decision;
}

export function authorityDecision(input) {
  validateAuthorityDecision(input);
  return structuredClone(input);
}

function sideUnavailable(reason) {
  return { available: false, unavailable_reason: reason };
}

const SIDE_NOT_QUERIED = "corpus not supplied for this side; absence cannot be claimed";

function validateCorpus(corpus) {
  if (!corpus || typeof corpus !== "object" || Array.isArray(corpus)) {
    throw new TypeError("corpus must be an object");
  }
  const sides = {};
  if (corpus.authority_decisions !== undefined) {
    if (!Array.isArray(corpus.authority_decisions)) {
      throw new TypeError("corpus.authority_decisions must be an array when supplied");
    }
    sides.decisions = corpus.authority_decisions.map(validateAuthorityDecision);
  }
  if (corpus.activities !== undefined) {
    if (!Array.isArray(corpus.activities)) {
      throw new TypeError("corpus.activities must be an array when supplied");
    }
    sides.activities = corpus.activities.map(validateActivity);
  }
  if (corpus.streams !== undefined) {
    if (!Array.isArray(corpus.streams)) {
      throw new TypeError("corpus.streams must be an array when supplied");
    }
    sides.streams = corpus.streams.map(validateActuationStream);
  }
  return sides;
}

function eventReferencesRequest(event, requestRef) {
  // Verbatim equality against the event's declared resource refs — the
  // canonical place a request identity attaches to a stream event. No
  // normalisation, no substring matching: the four-side join is exact.
  return (event.resource_refs ?? []).includes(requestRef);
}

function dispositionOf(event) {
  if (event.kind === "refusal") return "refused";
  const recorded = event.metadata?.permission_outcome;
  if (RECORDED_PERMISSION_OUTCOMES.has(recorded)) return recorded;
  return "pending";
}

function sortKey(entry) {
  const observed = entry.event.observed_at ?? "";
  return `${observed}${entry.stream_ref}${String(entry.event.sequence).padStart(16, "0")}`;
}

/**
 * Correlate the Actuation-side view of one upstream-issued request identity.
 * Every supplied corpus side is validated against its owning contract first;
 * an omitted side is reported unavailable rather than silently empty. The
 * result is a fresh, stamped read model; inputs are never mutated.
 */
export function requestCorrelationReadModel(requestRef, corpus = {}) {
  ref(requestRef, "request_ref (the upstream-issued request identity)");
  const sides = validateCorpus(corpus);

  const authorityAvailable = sides.decisions !== undefined;
  const activityAvailable = sides.activities !== undefined;
  const streamAvailable = sides.streams !== undefined;
  const sidesQueried = [authorityAvailable, activityAvailable, streamAvailable].filter(Boolean).length;

  const decisions = authorityAvailable
    ? sides.decisions.filter((decision) => decision.request_ref === requestRef)
    : [];
  // Latest observed decision wins the resolution; all matching decisions stay
  // disclosed in order.
  const orderedDecisions = [...decisions].sort((a, b) => Date.parse(a.observed_at) - Date.parse(b.observed_at));
  const namedActivityRefs = [...new Set(decisions.flatMap((decision) => decision.activity_refs))];

  const activities = activityAvailable
    ? sides.activities.filter((activity) => namedActivityRefs.includes(activity.activity_ref))
    : [];
  const correlatedActivityRefs = new Set(activities.map((activity) => activity.activity_ref));

  const referencingEvents = streamAvailable
    ? sides.streams
        .flatMap((stream) =>
          stream.events
            .filter((event) => STREAM_DISPOSITION_KINDS.has(event.kind) && eventReferencesRequest(event, requestRef))
            .map((event) => ({ stream_ref: stream.stream_ref, event })),
        )
        .sort((a, b) => (sortKey(a) < sortKey(b) ? -1 : sortKey(a) > sortKey(b) ? 1 : 0))
    : [];
  const permissionOutcome = referencingEvents.length === 0 ? "none" : dispositionOf(referencingEvents.at(-1).event);

  const referenced = decisions.length > 0 || referencingEvents.length > 0;

  let state;
  if (sidesQueried === 0) {
    state = "correlation-unavailable";
  } else if (!referenced) {
    state = "unknown-identity";
  } else if (!activityAvailable) {
    // The identity is referenced but the join target could not be queried;
    // claiming "no recorded activity" here would be fabrication.
    state = "correlation-unavailable";
  } else if (activities.length === 0) {
    state = "no-recorded-activity";
  } else {
    state = "correlated";
  }

  const attentionActivityRefs = activities.filter((activity) => activity.needs_attention).map((activity) => activity.activity_ref);
  // A permission event is "still pending" only when no LATER referencing
  // event carries a recorded disposition; an earlier request superseded by a
  // refusal or grant is history, not open attention.
  let lastResolvedIndex = -1;
  referencingEvents.forEach((entry, index) => {
    if (dispositionOf(entry.event) !== "pending") lastResolvedIndex = index;
  });
  const pendingPermissionEvents = referencingEvents
    .filter((entry, index) => index > lastResolvedIndex && entry.event.kind === "permission" && dispositionOf(entry.event) === "pending")
    .map((entry) => entry.event.event_ref);

  return {
    schema: REQUEST_CORRELATION_VERSION,
    request_ref: requestRef,
    state,
    authority: authorityAvailable
      ? {
          available: true,
          decisions: orderedDecisions.map((decision) => structuredClone(decision)),
          decision_refs: orderedDecisions.map((decision) => decision.decision_ref),
          resolution: orderedDecisions.length === 0 ? "none" : orderedDecisions.at(-1).decision,
        }
      : { ...sideUnavailable(SIDE_NOT_QUERIED), decisions: [], decision_refs: [], resolution: "none" },
    activities: activityAvailable
      ? {
          available: true,
          correlated: activities.map((activity) => ({
            activity_ref: activity.activity_ref,
            native_owner: activity.native_owner,
            verb: activity.verb,
            object: activity.object,
            summary: activity.summary,
            phase: activity.phase,
            outcome: activity.outcome,
            salience: activity.salience,
            needs_attention: activity.needs_attention,
            started_at: activity.started_at,
            updated_at: activity.updated_at,
          })),
          activity_refs: activities.map((activity) => activity.activity_ref),
          // Decision-named identities with no correlating record: disclosed,
          // never dropped — this is what makes "no recorded activity" explicit.
          unrecorded_activity_refs: namedActivityRefs.filter((activityRef) => !correlatedActivityRefs.has(activityRef)),
        }
      : { ...sideUnavailable(SIDE_NOT_QUERIED), correlated: [], activity_refs: [], unrecorded_activity_refs: [] },
    attention: {
      available: activityAvailable && streamAvailable,
      ...(activityAvailable && streamAvailable
        ? {}
        : { unavailable_reason: "attention context derives from activities and streams; a missing side limits it" }),
      tracked: attentionActivityRefs.length > 0 || pendingPermissionEvents.length > 0,
      activity_refs: attentionActivityRefs,
      pending_permission_event_refs: pendingPermissionEvents,
    },
    permission: streamAvailable
      ? {
          available: true,
          outcome: permissionOutcome,
          event_refs: referencingEvents.map((entry) => entry.event.event_ref),
        }
      : { ...sideUnavailable(SIDE_NOT_QUERIED), outcome: "none", event_refs: [] },
  };
}
