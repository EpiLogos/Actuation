export const AGENCY_CONTRACT_VERSION = "actuation.agency/v1";

const DETERMINATION_KINDS = new Set([
  "self-differentiation",
  "delegation",
  "derivation",
  "federation",
]);
const RETURN_MODES = new Set(["required", "optional", "autonomous-termination"]);
const RECOGNITION_STATES = new Set(["pending", "recognised", "rejected"]);
const MUTATION_STATES = new Set(["not-applied", "applied"]);
const METAGENCY_OPERATIONS = new Set([
  "determine-agency",
  "configure-agency",
  "actualise-agency",
  "reintegrate-return",
]);

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

function refArray(value, name, { optional = false, nonEmpty = false } = {}) {
  if (value == null && optional) return;
  if (!Array.isArray(value) || value.some((item) => typeof item !== "string" || item.trim() === "")) {
    throw new TypeError(`${name} must be an array of non-empty refs`);
  }
  if (nonEmpty && value.length === 0) throw new TypeError(`${name} must not be empty`);
}

function schema(value, name) {
  if (value !== AGENCY_CONTRACT_VERSION) {
    throw new TypeError(`${name}.schema must equal ${AGENCY_CONTRACT_VERSION}`);
  }
}

export function validateWorldBinding(input) {
  const binding = record(input, "WorldBinding");
  schema(binding.schema, "WorldBinding");
  ref(binding.binding_ref, "WorldBinding.binding_ref");
  ref(binding.agent_ref, "WorldBinding.agent_ref");
  ref(binding.agency_ref, "WorldBinding.agency_ref");
  ref(binding.world_ref, "WorldBinding.world_ref");
  ref(binding.scope_ref, "WorldBinding.scope_ref");
  ref(binding.purpose_ref, "WorldBinding.purpose_ref", { optional: true });
  ref(binding.continuity_ref, "WorldBinding.continuity_ref", { optional: true });
  ref(binding.determining_agency_ref, "WorldBinding.determining_agency_ref", { optional: true });
  ref(binding.return_relation_ref, "WorldBinding.return_relation_ref", { optional: true });
  refArray(binding.bounds_refs, "WorldBinding.bounds_refs", { optional: true });
  refArray(binding.authority_refs, "WorldBinding.authority_refs", { optional: true });

  if (binding.constraints != null) {
    const constraints = record(binding.constraints, "WorldBinding.constraints");
    refArray(constraints.human_authored_refs, "WorldBinding.constraints.human_authored_refs", { optional: true });
    refArray(constraints.security_policy_refs, "WorldBinding.constraints.security_policy_refs", { optional: true });
    refArray(constraints.evidence_refs, "WorldBinding.constraints.evidence_refs", { optional: true });
    refArray(constraints.external_reality_refs, "WorldBinding.constraints.external_reality_refs", { optional: true });
  }
  return binding;
}

export function validateRootScope(input) {
  const scope = record(input, "RootScope");
  schema(scope.schema, "RootScope");
  ref(scope.scope_ref, "RootScope.scope_ref");
  ref(scope.enclosing_world_ref, "RootScope.enclosing_world_ref");
  return scope;
}

/** Root Agency is a positional relation, never an Agent subtype. */
export function isRootAgency(worldBinding, rootScope) {
  const binding = validateWorldBinding(worldBinding);
  const scope = validateRootScope(rootScope);
  return binding.scope_ref === scope.scope_ref && binding.world_ref === scope.enclosing_world_ref;
}

export function validateMetagencyGrant(input) {
  const grant = record(input, "MetagencyGrant");
  schema(grant.schema, "MetagencyGrant");
  ref(grant.grant_ref, "MetagencyGrant.grant_ref");
  ref(grant.agency_ref, "MetagencyGrant.agency_ref");
  ref(grant.world_binding_ref, "MetagencyGrant.world_binding_ref");
  ref(grant.authority_ref, "MetagencyGrant.authority_ref");
  refArray(grant.bounds_refs, "MetagencyGrant.bounds_refs", { optional: true });
  if (!Array.isArray(grant.operations) || grant.operations.length === 0) {
    throw new TypeError("MetagencyGrant.operations must be a non-empty array");
  }
  for (const operation of grant.operations) {
    if (!METAGENCY_OPERATIONS.has(operation)) {
      throw new TypeError(`MetagencyGrant.operations contains unsupported operation: ${operation}`);
    }
  }
  return grant;
}

export function validateDetermination(input) {
  const determination = record(input, "Determination");
  schema(determination.schema, "Determination");
  ref(determination.determination_ref, "Determination.determination_ref");
  if (!DETERMINATION_KINDS.has(determination.kind)) {
    throw new TypeError("Determination.kind must distinguish self-differentiation, delegation, derivation or federation");
  }
  ref(determination.determining_agency_ref, "Determination.determining_agency_ref");
  ref(determination.differentiated_agency_ref, "Determination.differentiated_agency_ref");
  ref(determination.world_binding_ref, "Determination.world_binding_ref");
  ref(determination.parent_determination_ref, "Determination.parent_determination_ref", { optional: true });
  refArray(determination.bounds_refs, "Determination.bounds_refs", { nonEmpty: true });
  refArray(determination.authority_refs, "Determination.authority_refs", { optional: true });

  const autonomy = record(determination.delegated_autonomy, "Determination.delegated_autonomy");
  refArray(autonomy.allowed_action_refs, "Determination.delegated_autonomy.allowed_action_refs", { optional: true });
  refArray(autonomy.denied_action_refs, "Determination.delegated_autonomy.denied_action_refs", { optional: true });
  if (typeof autonomy.may_determine_within_bounds !== "boolean") {
    throw new TypeError("Determination.delegated_autonomy.may_determine_within_bounds must be boolean");
  }

  const returnPolicy = record(determination.return_policy, "Determination.return_policy");
  if (!RETURN_MODES.has(returnPolicy.mode)) {
    throw new TypeError("Determination.return_policy.mode must be required, optional or autonomous-termination");
  }
  ref(returnPolicy.return_relation_ref, "Determination.return_policy.return_relation_ref", {
    optional: returnPolicy.mode === "autonomous-termination",
  });

  if (determination.kind === "federation" && determination.authority_refs?.length) {
    throw new TypeError("Federation cannot silently carry determining authority; use an explicit delegation if authority is granted");
  }
  return determination;
}

export function validateReturn(input) {
  const returned = record(input, "Return");
  schema(returned.schema, "Return");
  ref(returned.return_ref, "Return.return_ref");
  ref(returned.determination_ref, "Return.determination_ref");
  ref(returned.from_agency_ref, "Return.from_agency_ref");
  ref(returned.to_agency_ref, "Return.to_agency_ref");
  refArray(returned.difference_refs, "Return.difference_refs", { nonEmpty: true });
  refArray(returned.artifact_refs, "Return.artifact_refs", { optional: true });
  refArray(returned.claim_refs, "Return.claim_refs", { optional: true });
  refArray(returned.evidence_refs, "Return.evidence_refs", { optional: true });

  const provenance = record(returned.provenance, "Return.provenance");
  refArray(provenance.agency_lineage_refs, "Return.provenance.agency_lineage_refs", { nonEmpty: true });
  refArray(provenance.material_refs, "Return.provenance.material_refs", { optional: true });
  refArray(provenance.external_source_refs, "Return.provenance.external_source_refs", { optional: true });

  if (typeof returned.received !== "boolean") throw new TypeError("Return.received must be boolean");
  if (!RECOGNITION_STATES.has(returned.recognition_state)) {
    throw new TypeError("Return.recognition_state must be pending, recognised or rejected");
  }
  if (!MUTATION_STATES.has(returned.world_mutation_state)) {
    throw new TypeError("Return.world_mutation_state must be not-applied or applied");
  }
  if (!returned.received && returned.recognition_state !== "pending") {
    throw new TypeError("A Return cannot be recognised/rejected before it is received");
  }
  if (returned.world_mutation_state === "applied" && returned.recognition_state !== "recognised") {
    throw new TypeError("World mutation requires an explicitly recognised Return");
  }
  return returned;
}

export function validateDeterminationLineage(determinations) {
  if (!Array.isArray(determinations) || determinations.length === 0) {
    throw new TypeError("Determination lineage must be a non-empty array");
  }
  const validated = determinations.map(validateDetermination);
  const byRef = new Map(validated.map((item) => [item.determination_ref, item]));
  for (const item of validated) {
    if (!item.parent_determination_ref) continue;
    const parent = byRef.get(item.parent_determination_ref);
    if (!parent) throw new TypeError(`Missing parent determination ${item.parent_determination_ref}`);
    if (parent.differentiated_agency_ref !== item.determining_agency_ref) {
      throw new TypeError("Recursive determination lineage must continue through the agency determined by its parent");
    }
    if (!parent.delegated_autonomy.may_determine_within_bounds) {
      throw new TypeError("A differentiated Agency may govern downward only when delegated autonomy permits determination");
    }
  }
  return validated;
}

export function agencyReadModel({ binding, root_scope, metagency_grants = [], determinations = [], returns = [] }) {
  const worldBinding = validateWorldBinding(binding);
  const grants = metagency_grants.map(validateMetagencyGrant).filter((grant) => grant.agency_ref === worldBinding.agency_ref);
  const lineage = determinations.length ? validateDeterminationLineage(determinations) : [];
  const returnModels = returns.map(validateReturn);
  return {
    schema: AGENCY_CONTRACT_VERSION,
    agency_ref: worldBinding.agency_ref,
    agent_ref: worldBinding.agent_ref,
    world_binding_ref: worldBinding.binding_ref,
    world_ref: worldBinding.world_ref,
    scope_ref: worldBinding.scope_ref,
    root_for_scope: root_scope ? isRootAgency(worldBinding, root_scope) : false,
    metagency: {
      available: grants.length > 0,
      operations: [...new Set(grants.flatMap((grant) => grant.operations))].sort(),
      grant_refs: grants.map((grant) => grant.grant_ref),
    },
    governing_determination_refs: lineage
      .filter((item) => item.determining_agency_ref === worldBinding.agency_ref)
      .map((item) => item.determination_ref),
    governed_by_determination_refs: lineage
      .filter((item) => item.differentiated_agency_ref === worldBinding.agency_ref)
      .map((item) => item.determination_ref),
    returns: {
      pending: returnModels.filter((item) => item.to_agency_ref === worldBinding.agency_ref && !item.received).map((item) => item.return_ref),
      received: returnModels.filter((item) => item.to_agency_ref === worldBinding.agency_ref && item.received).map((item) => item.return_ref),
      recognised: returnModels.filter((item) => item.to_agency_ref === worldBinding.agency_ref && item.recognition_state === "recognised").map((item) => item.return_ref),
      world_mutated: returnModels.filter((item) => item.to_agency_ref === worldBinding.agency_ref && item.world_mutation_state === "applied").map((item) => item.return_ref),
    },
    constraints: worldBinding.constraints ?? {},
  };
}

export const EPI_LOGOS_ROOT_AGENCY_READING = Object.freeze({
  standing: ["Bimba", "Mono", "Possibility", "Determination", "0/1"],
  articulated: ["Pratibimba", "Poly", "Actualisation", "Labour", "1/0"],
  return: ["Return", "reconstitution", "1/1"],
});
