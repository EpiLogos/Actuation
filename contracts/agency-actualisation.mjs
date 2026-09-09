import {
  AGENCY_CONTRACT_VERSION,
  validateDetermination,
  validateDeterminationLineage,
  validateMetagencyGrant,
  validateWorldBinding,
} from "./agency.mjs";

export const AGENCY_ACTUALISATION_VERSION = "actuation.agency-actualisation/v1";

function object(value, name) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new TypeError(`${name} must be an object`);
  }
  return value;
}

function ref(value, name) {
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

function sameRefSet(left, right) {
  const leftSet = new Set(left);
  const rightSet = new Set(right);
  return leftSet.size === left.length
    && rightSet.size === right.length
    && leftSet.size === rightSet.size
    && [...leftSet].every((value) => rightSet.has(value));
}

function onlyKeys(value, allowed, name) {
  const unexpected = Object.keys(value).filter((key) => !allowed.has(key));
  if (unexpected.length) throw new TypeError(`${name} contains unsupported field(s): ${unexpected.join(", ")}`);
}

function exactLineage(prior, determination) {
  const lineage = [...prior, determination];
  const unique = new Set(lineage.map((item) => item.determination_ref));
  if (unique.size !== lineage.length) throw new TypeError("Determination lineage must not contain duplicate refs");

  if (determination.parent_determination_ref == null) {
    if (prior.length !== 0) throw new TypeError("A root determination cannot carry unrelated prior lineage");
  } else if (prior.at(-1)?.determination_ref !== determination.parent_determination_ref) {
    throw new TypeError("Determination lineage must end at the exact declared parent");
  }

  for (let index = 0; index < lineage.length; index += 1) {
    const expectedParent = index === 0 ? undefined : lineage[index - 1].determination_ref;
    if (lineage[index].parent_determination_ref !== expectedParent) {
      throw new TypeError("Determination lineage must be one complete contiguous ancestry");
    }
  }

  return validateDeterminationLineage(lineage);
}

/**
 * Actualise one #4 determination as a process-local semantic relation.
 *
 * The grant is the only authority gate. Context, visibility, capability and
 * participation refs remain provenance and can never substitute for it.
 * Materialisation, recognition and source mutation belong to their native
 * owners and are deliberately not performed by this operation.
 */
export function actualiseAgency(input) {
  const request = object(input, "AgencyActualisationRequest");
  onlyKeys(request, new Set([
    "schema", "request_ref", "requester_ref", "governing_binding", "metagency_grant",
    "determination", "differentiated_binding", "prior_determinations", "agent_identity", "provenance",
  ]), "AgencyActualisationRequest");
  if (request.schema !== AGENCY_ACTUALISATION_VERSION) {
    throw new TypeError(`AgencyActualisationRequest.schema must equal ${AGENCY_ACTUALISATION_VERSION}`);
  }
  ref(request.request_ref, "AgencyActualisationRequest.request_ref");
  ref(request.requester_ref, "AgencyActualisationRequest.requester_ref");

  const governing = validateWorldBinding(request.governing_binding);
  const differentiated = validateWorldBinding(request.differentiated_binding);
  const grant = validateMetagencyGrant(request.metagency_grant);
  const determination = validateDetermination(request.determination);
  const prior = (request.prior_determinations ?? []).map(validateDetermination);
  const lineage = exactLineage(prior, determination);

  const identity = object(request.agent_identity, "AgencyActualisationRequest.agent_identity");
  onlyKeys(identity, new Set(["standing", "evidence_refs"]), "AgencyActualisationRequest.agent_identity");
  if (!new Set(["existing", "actualised"]).has(identity.standing)) {
    throw new TypeError("AgencyActualisationRequest.agent_identity.standing must be existing or actualised");
  }
  const identityEvidence = refs(identity.evidence_refs, "AgencyActualisationRequest.agent_identity.evidence_refs", { nonEmpty: true });
  if (determination.kind === "derivation" && identity.standing !== "actualised") {
    throw new TypeError("Derivation requires an explicitly actualised Agent identity");
  }
  if (determination.kind === "derivation" && differentiated.agent_ref === governing.agent_ref) {
    throw new TypeError("Derivation must actualise a distinct Agent identity");
  }
  if (determination.kind !== "derivation" && identity.standing !== "existing") {
    throw new TypeError("Only derivation may actualise a new Agent identity");
  }
  if (determination.kind === "self-differentiation" && differentiated.agent_ref !== governing.agent_ref) {
    throw new TypeError("Self-differentiation must preserve the governing Agent identity");
  }

  if (grant.agency_ref !== governing.agency_ref || grant.world_binding_ref !== governing.binding_ref) {
    throw new TypeError("MetagencyGrant must target the exact governing Agency and WorldBinding");
  }
  if (!(governing.authority_refs ?? []).includes(grant.authority_ref)) {
    throw new TypeError("MetagencyGrant authority must be present on the governing WorldBinding");
  }
  if (!grant.operations.includes("determine-agency")) {
    throw new TypeError("MetagencyGrant does not authorise determine-agency");
  }
  const operationsUsed = ["determine-agency"];
  if (determination.kind === "derivation") {
    if (!grant.operations.includes("actualise-agency")) {
      throw new TypeError("Derivation requires explicit actualise-agency authority");
    }
    operationsUsed.push("actualise-agency");
  }

  if (determination.determining_agency_ref !== governing.agency_ref) {
    throw new TypeError("Determination must originate from the governing Agency");
  }
  if (determination.differentiated_agency_ref !== differentiated.agency_ref) {
    throw new TypeError("Determination must identify the differentiated Agency exactly");
  }
  if (determination.world_binding_ref !== differentiated.binding_ref) {
    throw new TypeError("Determination must identify the differentiated WorldBinding exactly");
  }
  if (differentiated.determining_agency_ref !== governing.agency_ref) {
    throw new TypeError("Differentiated WorldBinding must retain its determining Agency");
  }

  const determinationBounds = determination.bounds_refs;
  const bindingBounds = differentiated.bounds_refs ?? [];
  if (!sameRefSet(bindingBounds, determinationBounds)) {
    throw new TypeError("WorldBinding bounds must exactly preserve Determination bounds");
  }
  const grantBounds = refs(grant.bounds_refs, "MetagencyGrant.bounds_refs", { nonEmpty: true });
  const governingBounds = refs(governing.bounds_refs, "Governing WorldBinding.bounds_refs", { nonEmpty: true });
  if (grantBounds.some((bound) => !governingBounds.includes(bound))) {
    throw new TypeError("MetagencyGrant exceeds governing WorldBinding bounds");
  }
  if (determinationBounds.some((bound) => !grantBounds.includes(bound))) {
    throw new TypeError("Determination exceeds MetagencyGrant bounds");
  }
  const governingAuthority = refs(governing.authority_refs, "Governing WorldBinding.authority_refs", { nonEmpty: true });
  const determinationAuthority = refs(determination.authority_refs, "Determination.authority_refs", { optional: true });
  if (determinationAuthority.some((authority) => !governingAuthority.includes(authority))) {
    throw new TypeError("Determination cannot delegate authority absent from the governing WorldBinding");
  }
  if (!sameRefSet(differentiated.authority_refs ?? [], determinationAuthority)) {
    throw new TypeError("WorldBinding authority must exactly preserve Determination authority");
  }

  const returnRef = determination.return_policy.return_relation_ref;
  if (returnRef == null) {
    if (differentiated.return_relation_ref != null) {
      throw new TypeError("Autonomous termination cannot manufacture a Return relation");
    }
  } else if (differentiated.return_relation_ref !== returnRef) {
    throw new TypeError("WorldBinding must exactly preserve the Determination Return relation");
  }

  const provenance = object(request.provenance, "AgencyActualisationRequest.provenance");
  onlyKeys(provenance, new Set(["source_refs", "context_refs"]), "AgencyActualisationRequest.provenance");
  const sourceRefs = refs(provenance.source_refs, "AgencyActualisationRequest.provenance.source_refs", { nonEmpty: true });
  const contextRefs = refs(provenance.context_refs, "AgencyActualisationRequest.provenance.context_refs", { optional: true });

  return {
    schema: AGENCY_ACTUALISATION_VERSION,
    receipt_ref: `${request.request_ref}:receipt`,
    request_ref: request.request_ref,
    requester_ref: request.requester_ref,
    status: "actualised",
    governing_binding: structuredClone(governing),
    differentiated_binding: structuredClone(differentiated),
    metagency: {
      grant_ref: grant.grant_ref,
      authority_ref: grant.authority_ref,
      operations_used: operationsUsed,
    },
    determination: structuredClone(determination),
    lineage: {
      determination_refs: lineage.map((item) => item.determination_ref),
      agency_refs: [lineage[0].determining_agency_ref, ...lineage.map((item) => item.differentiated_agency_ref)],
    },
    bounds_refs: [...determinationBounds],
    return_relation: {
      mode: determination.return_policy.mode,
      ...(returnRef == null ? {} : { return_relation_ref: returnRef }),
    },
    agent_identity: {
      standing: identity.standing,
      agent_ref: differentiated.agent_ref,
      evidence_refs: [...identityEvidence],
    },
    effects: {
      semantic_relation: "actualised",
      materialisation: "not-performed",
      factory_recognition: "not-performed",
      source_mutation: "not-performed",
    },
    provenance: {
      source_refs: [...sourceRefs],
      context_refs: [...contextRefs],
    },
  };
}

export { AGENCY_CONTRACT_VERSION };
