import test from "node:test";
import assert from "node:assert/strict";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import { executeCommand } from "./actuation.mjs";
import { AGENCY_CONTRACT_VERSION } from "../contracts/agency.mjs";
import { AGENCY_ACTUALISATION_VERSION } from "../contracts/agency-actualisation.mjs";

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const SCHEMA = AGENCY_CONTRACT_VERSION;

function requestFor(kind = "delegation", overrides = {}) {
  const derivation = kind === "derivation";
  const federation = kind === "federation";
  const self = kind === "self-differentiation";
  const targetAgent = self ? "agent:governor" : derivation ? "agent:derived-1" : "agent:existing-1";
  const targetAgency = `agency:project:${kind}`;
  const bindingRef = `binding:project:${kind}`;
  const bound = `bound:project:${kind}`;
  const returnRef = `return-relation:project:${kind}`;
  const authorityRefs = federation ? [] : [`authority:project:${kind}`];

  const base = {
    schema: AGENCY_ACTUALISATION_VERSION,
    request_ref: `actualisation-request:${kind}`,
    requester_ref: "human:owner",
    governing_binding: {
      schema: SCHEMA,
      binding_ref: "binding:governing",
      agent_ref: "agent:governor",
      agency_ref: "agency:governing",
      world_ref: "world:personal",
      scope_ref: "scope:personal",
      bounds_refs: ["bound:personal", bound],
      authority_refs: ["authority:metagency", ...authorityRefs],
      return_relation_ref: "return-relation:governing",
    },
    metagency_grant: {
      schema: SCHEMA,
      grant_ref: "grant:project-agency",
      agency_ref: "agency:governing",
      world_binding_ref: "binding:governing",
      authority_ref: "authority:metagency",
      bounds_refs: [bound],
      operations: ["determine-agency", "actualise-agency"],
    },
    determination: {
      schema: SCHEMA,
      determination_ref: `determination:${kind}`,
      kind,
      determining_agency_ref: "agency:governing",
      differentiated_agency_ref: targetAgency,
      world_binding_ref: bindingRef,
      bounds_refs: [bound],
      authority_refs: authorityRefs,
      delegated_autonomy: {
        allowed_action_refs: ["action:bounded-work"],
        denied_action_refs: ["action:source-mutation"],
        may_determine_within_bounds: kind === "delegation",
      },
      return_policy: { mode: "required", return_relation_ref: returnRef },
    },
    differentiated_binding: {
      schema: SCHEMA,
      binding_ref: bindingRef,
      agent_ref: targetAgent,
      agency_ref: targetAgency,
      world_ref: "central:project:Example",
      scope_ref: "central:project:Example:scope",
      determining_agency_ref: "agency:governing",
      bounds_refs: [bound],
      authority_refs: authorityRefs,
      return_relation_ref: returnRef,
      continuity_ref: `continuity:${targetAgent}`,
    },
    agent_identity: {
      standing: derivation ? "actualised" : "existing",
      evidence_refs: [derivation ? "evidence:identity-allocation" : "evidence:identity-registry"],
    },
    provenance: {
      source_refs: ["actuation:#44"],
      context_refs: [],
    },
  };
  return { ...base, ...overrides };
}

function run(request) {
  return JSON.parse(executeCommand(["agency", "actualise", "-", "--json"], { stdin: JSON.stringify(request) }).stdout);
}

test("1. an authorised human binds an existing Agent into a Project-world Agency", () => {
  const process = spawnSync(resolve(REPO_ROOT, "bin/actuation"), ["agency", "actualise", "-", "--json"], {
    input: JSON.stringify(requestFor()),
    encoding: "utf8",
  });
  assert.equal(process.status, 0, process.stderr);
  const receipt = JSON.parse(process.stdout);
  assert.equal(receipt.requester_ref, "human:owner");
  assert.equal(receipt.agent_identity.standing, "existing");
  assert.equal(receipt.differentiated_binding.world_ref, "central:project:Example");
  assert.deepEqual(receipt.metagency.operations_used, ["determine-agency"]);
});

test("2. explicit determine and actualise grants derive a bounded new Agent and Agency", () => {
  const receipt = run(requestFor("derivation"));
  assert.equal(receipt.agent_identity.standing, "actualised");
  assert.deepEqual(receipt.metagency.operations_used, ["determine-agency", "actualise-agency"]);
  assert.deepEqual(receipt.bounds_refs, ["bound:project:derivation"]);

  const withoutActualise = requestFor("derivation");
  withoutActualise.metagency_grant.operations = ["determine-agency"];
  assert.throws(() => run(withoutActualise), /actualise-agency authority/);

  const sameAgent = requestFor("derivation");
  sameAgent.differentiated_binding.agent_ref = sameAgent.governing_binding.agent_ref;
  assert.throws(() => run(sameAgent), /distinct Agent identity/);
});

test("3. Project scope remains an exact WorldBinding ref, never filesystem-path identity", () => {
  const input = requestFor();
  input.provenance.context_refs = ["central-source:/Users/person/Central/Work/Example"];
  const receipt = run(input);
  assert.equal(receipt.differentiated_binding.world_ref, "central:project:Example");
  assert.equal(receipt.differentiated_binding.scope_ref, "central:project:Example:scope");
  assert.equal(receipt.agent_identity.agent_ref, "agent:existing-1");
  assert.notEqual(receipt.agent_identity.agent_ref, input.provenance.context_refs[0]);
});

test("4. source visibility and capability availability cannot replace a MetagencyGrant", () => {
  const input = requestFor();
  delete input.metagency_grant;
  input.provenance.context_refs = ["visibility:central-source", "capability:aikit-create-agent"];
  assert.throws(() => run(input), /MetagencyGrant must be an object/);

  const wrongAuthority = requestFor();
  wrongAuthority.metagency_grant.authority_ref = "authority:not-on-binding";
  assert.throws(() => run(wrongAuthority), /authority must be present/);
});

test("5. one enduring Agent can inhabit narrower and broader WorldBindings", () => {
  const narrow = requestFor("self-differentiation");
  const broad = requestFor("self-differentiation");
  broad.request_ref = "actualisation-request:self-broader";
  broad.determination.determination_ref = "determination:self-broader";
  broad.determination.world_binding_ref = "binding:portfolio";
  broad.differentiated_binding = {
    ...broad.differentiated_binding,
    binding_ref: "binding:portfolio",
    agency_ref: "agency:portfolio",
    world_ref: "central:portfolio",
    scope_ref: "central:portfolio:scope",
  };
  broad.determination.differentiated_agency_ref = "agency:portfolio";
  const [narrowReceipt, broadReceipt] = [run(narrow), run(broad)];
  assert.equal(narrowReceipt.agent_identity.agent_ref, broadReceipt.agent_identity.agent_ref);
  assert.notEqual(narrowReceipt.differentiated_binding.binding_ref, broadReceipt.differentiated_binding.binding_ref);
});

test("6. AgentSet or Factory Journey participation never grants metagency", () => {
  const input = requestFor();
  delete input.metagency_grant;
  input.provenance.context_refs = ["agent-set:builders", "factory:journey:176"];
  assert.throws(() => run(input), /MetagencyGrant must be an object/);
});

test("7. a Factory handoff preserves exact recursive determination lineage", () => {
  const input = requestFor("delegation");
  const parent = {
    schema: SCHEMA,
    determination_ref: "determination:factory-parent",
    kind: "delegation",
    determining_agency_ref: "agency:root",
    differentiated_agency_ref: "agency:governing",
    world_binding_ref: "binding:governing",
    bounds_refs: ["bound:personal", "bound:project:delegation"],
    authority_refs: ["authority:metagency"],
    delegated_autonomy: { allowed_action_refs: ["action:determine-agency"], may_determine_within_bounds: true },
    return_policy: { mode: "required", return_relation_ref: "return-relation:governing" },
  };
  input.determination.parent_determination_ref = parent.determination_ref;
  input.prior_determinations = [parent];
  input.provenance.context_refs = ["factory:journey:176"];
  const receipt = run(input);
  assert.deepEqual(receipt.lineage.determination_refs, [parent.determination_ref, input.determination.determination_ref]);
  assert.deepEqual(receipt.lineage.agency_refs, ["agency:root", "agency:governing", "agency:project:delegation"]);
  assert.deepEqual(receipt.provenance.context_refs, ["factory:journey:176"]);

  input.prior_determinations = [];
  assert.throws(() => run(input), /exact declared parent/);
});

test("8. actualisation preserves Return while performing no recognition or source mutation", () => {
  const receipt = run(requestFor());
  assert.deepEqual(receipt.return_relation, {
    mode: "required",
    return_relation_ref: "return-relation:project:delegation",
  });
  assert.deepEqual(receipt.effects, {
    semantic_relation: "actualised",
    materialisation: "not-performed",
    factory_recognition: "not-performed",
    source_mutation: "not-performed",
  });
});

test("9. the public operation cannot manufacture determination authority", () => {
  const input = requestFor();
  input.metagency_grant.operations = ["configure-agency"];
  assert.throws(() => run(input), /does not authorise determine-agency/);
});

test("10. source-level actualisation requires no VM or material body", () => {
  const input = requestFor();
  assert.equal("workcell" in input, false);
  assert.equal("vm" in input, false);
  const receipt = run(input);
  assert.equal(receipt.status, "actualised");
  assert.equal(receipt.effects.materialisation, "not-performed");
});

test("all four determination kinds remain public and semantically distinct", () => {
  for (const kind of ["self-differentiation", "delegation", "derivation", "federation"]) {
    const receipt = run(requestFor(kind));
    assert.equal(receipt.determination.kind, kind);
  }
  assert.deepEqual(run(requestFor("federation")).differentiated_binding.authority_refs, []);
});

test("exact binding, bounds, authority and Return mismatches fail closed", () => {
  const mutations = [
    ["binding", (value) => { value.determination.world_binding_ref = "binding:wrong"; }, /WorldBinding exactly/],
    ["bounds", (value) => { value.differentiated_binding.bounds_refs = ["bound:other"]; }, /bounds must exactly/],
    ["authority", (value) => { value.differentiated_binding.authority_refs = ["authority:other"]; }, /authority must exactly/],
    ["Return", (value) => { value.differentiated_binding.return_relation_ref = "return-relation:other"; }, /Return relation/],
  ];
  for (const [name, mutate, pattern] of mutations) {
    const input = requestFor();
    mutate(input);
    assert.throws(() => run(input), pattern, name);
  }

  const inventedAuthority = requestFor();
  inventedAuthority.determination.authority_refs = ["authority:not-held-by-governor"];
  inventedAuthority.differentiated_binding.authority_refs = ["authority:not-held-by-governor"];
  assert.throws(() => run(inventedAuthority), /absent from the governing WorldBinding/);

  const reorderedBounds = requestFor();
  reorderedBounds.governing_binding.bounds_refs.push("bound:secondary");
  reorderedBounds.metagency_grant.bounds_refs.push("bound:secondary");
  reorderedBounds.determination.bounds_refs.push("bound:secondary");
  reorderedBounds.differentiated_binding.bounds_refs = ["bound:secondary", ...reorderedBounds.differentiated_binding.bounds_refs];
  assert.equal(run(reorderedBounds).status, "actualised");
});
