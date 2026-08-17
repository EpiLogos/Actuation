import test from "node:test";
import assert from "node:assert/strict";
import {
  AGENCY_CONTRACT_VERSION, agencyReadModel, isRootAgency, validateDetermination,
  validateDeterminationLineage, validateMetagencyGrant, validateReturn, validateWorldBinding,
} from "./agency.mjs";

const rootBinding = {
  schema: AGENCY_CONTRACT_VERSION, binding_ref: "actuation:world-binding:root",
  agent_ref: "agent:ordinary-1", agency_ref: "agency:root-position",
  world_ref: "oi:world:personal", scope_ref: "oi:scope:personal",
  purpose_ref: "intent:operate-personal-world", bounds_refs: ["policy:root-technical-bounds"],
  authority_refs: ["authority:suite-operation"], return_relation_ref: "actuation:return-relation:root",
  continuity_ref: "continuity:ordinary-agent-1",
  constraints: { human_authored_refs: ["central:Control/user"], security_policy_refs: ["policy:encounter-security"], evidence_refs: ["evidence:external-state"], external_reality_refs: ["world:external-others"] },
};
const rootScope = { schema: AGENCY_CONTRACT_VERSION, scope_ref: "oi:scope:personal", enclosing_world_ref: "oi:world:personal" };
const delegation = {
  schema: AGENCY_CONTRACT_VERSION, determination_ref: "determination:a", kind: "delegation",
  determining_agency_ref: "agency:root-position", differentiated_agency_ref: "agency:a",
  world_binding_ref: "actuation:world-binding:a", bounds_refs: ["bound:a"], authority_refs: ["authority:a"],
  delegated_autonomy: { allowed_action_refs: ["action:a"], denied_action_refs: ["action:dangerous"], may_determine_within_bounds: true },
  return_policy: { mode: "required", return_relation_ref: "return-relation:a" },
};
const nested = {
  schema: AGENCY_CONTRACT_VERSION, determination_ref: "determination:b", kind: "self-differentiation",
  determining_agency_ref: "agency:a", differentiated_agency_ref: "agency:b", parent_determination_ref: "determination:a",
  world_binding_ref: "actuation:world-binding:b", bounds_refs: ["bound:b"], authority_refs: ["authority:b"],
  delegated_autonomy: { allowed_action_refs: ["action:b"], may_determine_within_bounds: false },
  return_policy: { mode: "required", return_relation_ref: "return-relation:b" },
};

test("Root Agency is positional over an ordinary Agent/Agency binding", () => {
  assert.equal(validateWorldBinding(rootBinding).agent_ref, "agent:ordinary-1");
  assert.equal(isRootAgency(rootBinding, rootScope), true);
  assert.equal(isRootAgency({ ...rootBinding, world_ref: "project:world:x" }, rootScope), false);
});
test("Metagency is an explicit authority/capacity grant, not an Agent kind", () => {
  const grant = validateMetagencyGrant({ schema: AGENCY_CONTRACT_VERSION, grant_ref: "grant:meta", agency_ref: rootBinding.agency_ref, world_binding_ref: rootBinding.binding_ref, authority_ref: "authority:metagency", bounds_refs: ["bound:meta"], operations: ["determine-agency", "configure-agency", "actualise-agency", "reintegrate-return"] });
  assert.equal(grant.operations.length, 4); assert.equal("agent_kind" in grant, false);
});
test("recursive determination preserves exact parent lineage and bounded downward authority", () => {
  const lineage = validateDeterminationLineage([delegation, nested]);
  assert.equal(lineage[1].parent_determination_ref, lineage[0].determination_ref);
  assert.equal(lineage[0].differentiated_agency_ref, lineage[1].determining_agency_ref);
});
test("self-differentiation, delegation, derivation and federation remain distinct", () => {
  for (const kind of ["self-differentiation", "delegation", "derivation"]) assert.equal(validateDetermination({ ...delegation, kind }).kind, kind);
  assert.throws(() => validateDetermination({ ...delegation, kind: "federation" }), /Federation cannot silently carry determining authority/);
  assert.equal(validateDetermination({ ...delegation, kind: "federation", authority_refs: [] }).kind, "federation");
});
test("A2A or communication refs alone do not establish determination authority", () => {
  assert.throws(() => validateDetermination({ ...delegation, bounds_refs: [] }), /bounds_refs must not be empty/);
});
test("returned difference and provenance survive before recognition or synthesis", () => {
  const received = validateReturn({ schema: AGENCY_CONTRACT_VERSION, return_ref: "return:b", determination_ref: nested.determination_ref, from_agency_ref: "agency:b", to_agency_ref: "agency:a", difference_refs: ["difference:unexpected-resistance"], artifact_refs: ["artifact:raw"], claim_refs: ["claim:finding"], evidence_refs: ["evidence:runtime"], provenance: { agency_lineage_refs: ["agency:root-position", "agency:a", "agency:b"], material_refs: ["workcell:receipt:42"], external_source_refs: ["source:other"] }, received: true, recognition_state: "pending", world_mutation_state: "not-applied" });
  assert.deepEqual(received.difference_refs, ["difference:unexpected-resistance"]); assert.equal(received.recognition_state, "pending");
});
test("Return received, recognised and world-mutated are ordered but non-identical states", () => {
  const base = { schema: AGENCY_CONTRACT_VERSION, return_ref: "return:a", determination_ref: delegation.determination_ref, from_agency_ref: "agency:a", to_agency_ref: rootBinding.agency_ref, difference_refs: ["difference:a"], provenance: { agency_lineage_refs: [rootBinding.agency_ref, "agency:a"] } };
  validateReturn({ ...base, received: false, recognition_state: "pending", world_mutation_state: "not-applied" });
  validateReturn({ ...base, received: true, recognition_state: "pending", world_mutation_state: "not-applied" });
  validateReturn({ ...base, received: true, recognition_state: "recognised", world_mutation_state: "not-applied" });
  validateReturn({ ...base, received: true, recognition_state: "recognised", world_mutation_state: "applied" });
  assert.throws(() => validateReturn({ ...base, received: true, recognition_state: "pending", world_mutation_state: "applied" }), /World mutation requires an explicitly recognised Return/);
});
test("the same Agency can be governed upward while governing downward", () => {
  const model = agencyReadModel({ binding: { ...rootBinding, binding_ref: "binding:a", agent_ref: "agent:a", agency_ref: "agency:a", world_ref: "world:a", scope_ref: "scope:a", determining_agency_ref: "agency:root-position" }, determinations: [delegation, nested] });
  assert.deepEqual(model.governed_by_determination_refs, ["determination:a"]); assert.deepEqual(model.governing_determination_refs, ["determination:b"]);
});
test("human/authored/security/evidence/external-reality constraints remain above root technical authority", () => {
  const model = agencyReadModel({ binding: rootBinding, root_scope: rootScope });
  assert.equal(model.root_for_scope, true); assert.deepEqual(model.constraints.human_authored_refs, ["central:Control/user"]); assert.deepEqual(model.constraints.security_policy_refs, ["policy:encounter-security"]);
});
