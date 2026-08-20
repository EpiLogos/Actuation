export const REALISED_ACTUATION_VERSION = "actuation.realised/v1";

const LOOP_RECURRENCES = new Set(["single-shot", "turn-based", "event-driven", "continuous"]);
const OBSERVATION_STATES = new Set(["observed", "partial", "unavailable"]);

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

export function validateRealisedActuation(input) {
  const receipt = object(input, "RealisedActuation");
  if (receipt.schema !== REALISED_ACTUATION_VERSION) {
    throw new TypeError(`RealisedActuation.schema must equal ${REALISED_ACTUATION_VERSION}`);
  }

  ref(receipt.realised_ref, "RealisedActuation.realised_ref");
  ref(receipt.actuation_ref, "RealisedActuation.actuation_ref");
  ref(receipt.agent_ref, "RealisedActuation.agent_ref");
  ref(receipt.agency_ref, "RealisedActuation.agency_ref");
  ref(receipt.world_binding_ref, "RealisedActuation.world_binding_ref");

  const loop = object(receipt.loop, "RealisedActuation.loop");
  if (!LOOP_RECURRENCES.has(loop.recurrence)) {
    throw new TypeError("RealisedActuation.loop.recurrence must name a supported portable recurrence shape");
  }
  if (loop.acting !== true) {
    throw new TypeError("RealisedActuation.loop.acting must be true; inert model availability is not realised Actuation");
  }
  ref(loop.entrypoint_ref, "RealisedActuation.loop.entrypoint_ref", { optional: true });
  strings(loop.observed_faculties, "RealisedActuation.loop.observed_faculties", { optional: true });

  if (receipt.body != null) {
    const body = object(receipt.body, "RealisedActuation.body");
    ref(body.harness_ref, "RealisedActuation.body.harness_ref", { optional: true });
    ref(body.session_ref, "RealisedActuation.body.session_ref", { optional: true });
    ref(body.process_ref, "RealisedActuation.body.process_ref", { optional: true });
    ref(body.model_condition_ref, "RealisedActuation.body.model_condition_ref", { optional: true });
    ref(body.material_binding_ref, "RealisedActuation.body.material_binding_ref", { optional: true });
  }

  refs(receipt.participating_loci, "RealisedActuation.participating_loci", { optional: true });
  ref(receipt.stream_ref, "RealisedActuation.stream_ref", { optional: true });
  ref(receipt.return_ref, "RealisedActuation.return_ref", { optional: true });

  const observation = object(receipt.observation, "RealisedActuation.observation");
  if (!OBSERVATION_STATES.has(observation.state)) {
    throw new TypeError("RealisedActuation.observation.state must be observed, partial or unavailable");
  }
  const evidence = refs(observation.evidence_refs, "RealisedActuation.observation.evidence_refs", { optional: true });
  strings(observation.unsupported_faculties, "RealisedActuation.observation.unsupported_faculties", { optional: true });
  strings(observation.degraded_faculties, "RealisedActuation.observation.degraded_faculties", { optional: true });
  if (observation.state === "observed" && evidence.length === 0) {
    throw new TypeError("Observed realised Actuation must carry at least one evidence ref");
  }

  if (receipt.lifecycle != null) {
    const lifecycle = object(receipt.lifecycle, "RealisedActuation.lifecycle");
    for (const field of ["interrupt", "cancel", "terminate"]) {
      if (lifecycle[field] != null && typeof lifecycle[field] !== "boolean") {
        throw new TypeError(`RealisedActuation.lifecycle.${field} must be boolean when declared`);
      }
    }
  }

  return receipt;
}

export function realisedActuationReadModel(input) {
  const receipt = validateRealisedActuation(input);
  return {
    schema: REALISED_ACTUATION_VERSION,
    realised_ref: receipt.realised_ref,
    actuation_ref: receipt.actuation_ref,
    agent_ref: receipt.agent_ref,
    agency_ref: receipt.agency_ref,
    world_binding_ref: receipt.world_binding_ref,
    recurrence: receipt.loop.recurrence,
    harness_ref: receipt.body?.harness_ref ?? null,
    session_ref: receipt.body?.session_ref ?? null,
    model_condition_ref: receipt.body?.model_condition_ref ?? null,
    material_binding_ref: receipt.body?.material_binding_ref ?? null,
    stream_ref: receipt.stream_ref ?? null,
    return_ref: receipt.return_ref ?? null,
    participating_loci: [...(receipt.participating_loci ?? [])],
    observation: {
      state: receipt.observation.state,
      evidence_refs: [...(receipt.observation.evidence_refs ?? [])],
      unsupported_faculties: [...(receipt.observation.unsupported_faculties ?? [])],
      degraded_faculties: [...(receipt.observation.degraded_faculties ?? [])],
    },
  };
}

export function continuityDelta(previousInput, nextInput) {
  const previous = validateRealisedActuation(previousInput);
  const next = validateRealisedActuation(nextInput);
  return {
    same_agent: previous.agent_ref === next.agent_ref,
    same_agency: previous.agency_ref === next.agency_ref,
    same_world_binding: previous.world_binding_ref === next.world_binding_ref,
    same_actuation: previous.actuation_ref === next.actuation_ref,
    harness_changed: previous.body?.harness_ref !== next.body?.harness_ref,
    session_changed: previous.body?.session_ref !== next.body?.session_ref,
    process_changed: previous.body?.process_ref !== next.body?.process_ref,
    model_condition_changed: previous.body?.model_condition_ref !== next.body?.model_condition_ref,
    material_binding_changed: previous.body?.material_binding_ref !== next.body?.material_binding_ref,
    recurrence_changed: previous.loop.recurrence !== next.loop.recurrence,
  };
}
