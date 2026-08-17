import assert from "node:assert/strict";
import test from "node:test";

import {
  MODEL_BEARING_CONTRACT_VERSION,
  modelBearingReceipt,
  validateActuationReceipt,
  validateModelAccessProfile,
  validateModelRelation,
} from "./model-bearing.mjs";

const schema = MODEL_BEARING_CONTRACT_VERSION;

function accessProfile() {
  return {
    schema,
    inference: { allowed: ["invoke", "stream", "structured-output"], denied: [] },
    control: { allowed: [], denied: ["acquire", "stop", "replace", "adapt"] },
    interior: { depth: "behavioral", allowed: ["observe-output"], denied: ["state-read"] },
  };
}

function relation({ engine, placement, binding }) {
  return {
    schema,
    model_ref: "model:qwen2.5-coder-32b",
    variant_ref: "variant:q4-k-m",
    engine,
    material: {
      binding_ref: binding,
      placement,
      facts: { accelerator: placement === "local" ? "metal" : "cuda" },
    },
    inference_surface: {
      contract_ref: "contract:openai-compatible-chat/v1",
      binding_ref: `endpoint:${binding}`,
    },
  };
}

function receipt(modelRelation) {
  return {
    schema,
    actuation_ref: "actuation:identity-matrix-run-42",
    agency_ref: "agency:mahamaya-development",
    world_binding_ref: "world-binding:project-a",
    harness_ref: "harness:pi",
    harness_composition_ref: "harness-composition:body-7",
    agent_session_ref: "agent-session:stable-184",
    model_relation: modelRelation,
    access_profile: accessProfile(),
    bounds_refs: ["bound:no-egress"],
    evidence_refs: ["evidence:smoke-42"],
    return_ref: "return:run-42",
    observed_at: "2026-08-16T20:30:00Z",
  };
}

test("locality belongs to material placement, not higher model or agency identity", () => {
  const ollama = receipt(relation({ engine: { implementation_ref: "engine:ollama", provider_ref: "provider:aikit/ollama" }, placement: "local", binding: "workcell:mac/ollama-42" }));
  const vllm = receipt(relation({ engine: { implementation_ref: "engine:vllm", provider_ref: "provider:aikit/vllm" }, placement: "remote", binding: "workcell:gpu-server/vllm-9" }));
  validateActuationReceipt(ollama);
  validateActuationReceipt(vllm);
  assert.equal(ollama.agency_ref, vllm.agency_ref);
  assert.equal(ollama.agent_session_ref, vllm.agent_session_ref);
  assert.equal(ollama.model_relation.model_ref, vllm.model_relation.model_ref);
  assert.equal(ollama.model_relation.inference_surface.contract_ref, vllm.model_relation.inference_surface.contract_ref);
  assert.notEqual(ollama.model_relation.material.binding_ref, vllm.model_relation.material.binding_ref);
  assert.notEqual(ollama.model_relation.material.placement, vllm.model_relation.material.placement);
});

test("inference access never implies model control or model-interior access", () => {
  const access = accessProfile();
  validateModelAccessProfile(access);
  assert.deepEqual(access.inference.allowed, ["invoke", "stream", "structured-output"]);
  assert.deepEqual(access.control.allowed, []);
  assert.ok(access.control.denied.includes("replace"));
  assert.equal(access.interior.depth, "behavioral");
});

test("Colibri remains an experimental engine/material intervention rather than a provider ontology", () => {
  const colibri = receipt(relation({ engine: { implementation_ref: "engine:colibri@4ef9a992", facts: { technique: "storage-ram-vram-staging", experimental: true } }, placement: "local", binding: "workcell:experiment/colibri-1" }));
  colibri.experiment = {
    held_constant_refs: ["task:model-interior-probe", "harness:pi"],
    variables: [{ name: "inference-materialisation-technique", value_ref: "engine:colibri@4ef9a992", facts: { comparison: "ollama-llamacpp-vllm" } }],
  };
  const validated = modelBearingReceipt(colibri);
  assert.equal(validated.model_relation.engine.implementation_ref, "engine:colibri@4ef9a992");
  assert.equal(validated.experiment.variables[0].name, "inference-materialisation-technique");
  assert.equal("local_model_provider" in validated, false);
});

test("provider-native scalar facts are preserved without becoming canonical fields", () => {
  const input = relation({ engine: { implementation_ref: "engine:llama.cpp", provider_ref: "provider:aikit/llama.cpp", facts: { context_size: 32768, flash_attention: true, native_mode: "llama-server" } }, placement: "local", binding: "workcell:mac/llama-server-3" });
  validateModelRelation(input);
  assert.equal(input.engine.facts.context_size, 32768);
  assert.equal(input.engine.facts.native_mode, "llama-server");
});

test("invalid placement and malformed access fail before a receipt can be accepted", () => {
  const badRelation = relation({ engine: { implementation_ref: "engine:ollama" }, placement: "provider-ontology", binding: "workcell:bad" });
  assert.throws(() => validateModelRelation(badRelation), /placement/);
  const badAccess = accessProfile();
  badAccess.interior.depth = "god-mode";
  assert.throws(() => validateModelAccessProfile(badAccess), /research depth/);
});
