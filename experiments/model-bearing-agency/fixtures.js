export const collapsedDirectProcess = Object.freeze({
  receipt_ref: 'model-condition:collapsed-direct',
  actuation_ref: 'actuation:fixture',
  world_ref: 'world:project-alpha',
  agent_ref: 'agent:fixture',
  agency_ref: 'agency:fixture-alpha',
  model: {
    model_ref: 'model:fixture-open',
    variant_ref: 'model-variant:fixture-q4',
    artifact_refs: ['artifact:model-fixture-q4']
  },
  engine: {
    engine_ref: 'engine:fixture-direct',
    source_provenance_refs: ['source-revision:engine-direct']
  },
  materialisation: {
    mode: 'process',
    world_relation: 'inside-world',
    material_binding_refs: []
  },
  surface: {
    kind: 'cli',
    contract_ref: 'model-surface:cli-v1',
    binding_ref: null
  },
  harness: {
    harness_ref: null,
    harness_composition_ref: null,
    agent_session_ref: null,
    model_relation: 'none'
  },
  access: {
    inference: ['invoke', 'stream'],
    control: ['start', 'stop', 'replace-artifact'],
    interior: 'learning'
  },
  policy_boundary_refs: ['policy:project-local'],
  evidence_refs: [],
  provenance_refs: ['fixture:collapsed-direct'],
  provider_metadata: {
    note: 'Provider-neutral direct-process fixture; no standing service is implied.'
  }
});

export const localServiceExternalToHarness = Object.freeze({
  receipt_ref: 'model-condition:local-service',
  actuation_ref: 'actuation:fixture',
  world_ref: 'world:project-alpha',
  agent_ref: 'agent:fixture',
  agency_ref: 'agency:fixture-alpha',
  model: {
    model_ref: 'model:fixture-open',
    variant_ref: 'model-variant:fixture-q4',
    artifact_refs: ['artifact:model-fixture-q4']
  },
  engine: {
    engine_ref: 'engine:fixture-local-service',
    source_provenance_refs: ['source-revision:engine-local']
  },
  materialisation: {
    mode: 'service',
    world_relation: 'inside-world',
    material_binding_refs: ['workcell-binding:fixture-local-service']
  },
  surface: {
    kind: 'http',
    contract_ref: 'model-surface:openai-compatible-v1',
    binding_ref: 'service-binding:model-local'
  },
  harness: {
    harness_ref: 'harness:fixture-code',
    harness_composition_ref: 'harness-composition:fixture-code-a',
    agent_session_ref: 'agent-session:fixture-code-a',
    model_relation: 'external-surface'
  },
  access: {
    inference: ['invoke', 'stream', 'tool-use'],
    control: ['start', 'stop', 'load', 'unload', 'replace-artifact'],
    interior: 'learning'
  },
  policy_boundary_refs: ['policy:project-local'],
  evidence_refs: [],
  provenance_refs: ['fixture:local-service'],
  provider_metadata: {
    note: 'External to harness while materially inside the enclosing world.'
  }
});

export const remoteServiceSameHarness = Object.freeze({
  receipt_ref: 'model-condition:remote-service',
  actuation_ref: 'actuation:fixture',
  world_ref: 'world:project-alpha',
  agent_ref: 'agent:fixture',
  agency_ref: 'agency:fixture-alpha',
  model: {
    model_ref: 'model:fixture-open',
    variant_ref: 'model-variant:fixture-q4',
    artifact_refs: ['artifact:model-fixture-q4']
  },
  engine: {
    engine_ref: 'engine:fixture-remote-service',
    source_provenance_refs: ['source-revision:engine-remote']
  },
  materialisation: {
    mode: 'service',
    world_relation: 'outside-world',
    material_binding_refs: ['workcell-binding:fixture-remote-service']
  },
  surface: {
    kind: 'http',
    contract_ref: 'model-surface:openai-compatible-v1',
    binding_ref: 'service-binding:model-remote'
  },
  harness: {
    harness_ref: 'harness:fixture-code',
    harness_composition_ref: 'harness-composition:fixture-code-a',
    agent_session_ref: 'agent-session:fixture-code-b',
    model_relation: 'external-surface'
  },
  access: {
    inference: ['invoke', 'stream', 'tool-use'],
    control: [],
    interior: 'behavioural'
  },
  policy_boundary_refs: ['policy:provider-egress'],
  evidence_refs: [],
  provenance_refs: ['fixture:remote-service'],
  provider_metadata: {
    note: 'Same harness-facing contract and semantic model; materialisation is outside the world.'
  }
});
