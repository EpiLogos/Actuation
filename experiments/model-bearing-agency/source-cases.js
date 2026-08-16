export const sourceLockedCases = Object.freeze({
  ollamaLocalService: {
    source_lock: {
      repository: 'ollama/ollama',
      revision: 'd67ad83426633195089509347ffd4fe795120198',
      evidence_ref: 'README.md',
      evidence_class: 'upstream-source'
    },
    upstream_facts: [
      'Ollama runs models and exposes a REST API for running and managing them.',
      'Ollama documents integrations with existing agent/application harnesses including Claude Code and Codex.'
    ],
    condition: {
      receipt_ref: 'model-condition:source-case:ollama-local-service',
      actuation_ref: 'actuation:source-conformance',
      world_ref: 'world:source-conformance',
      agent_ref: 'agent:source-conformance',
      agency_ref: 'agency:source-conformance',
      model: {
        model_ref: 'model:source-case',
        variant_ref: null,
        artifact_refs: []
      },
      engine: {
        engine_ref: 'engine:ollama',
        source_provenance_refs: ['github:ollama/ollama@d67ad83426633195089509347ffd4fe795120198']
      },
      materialisation: {
        mode: 'service',
        world_relation: 'inside-world',
        material_binding_refs: []
      },
      surface: {
        kind: 'http',
        contract_ref: 'model-surface:ollama-rest-api',
        binding_ref: null
      },
      harness: {
        harness_ref: 'harness:source-case',
        harness_composition_ref: null,
        agent_session_ref: null,
        model_relation: 'external-surface'
      },
      access: {
        inference: ['invoke', 'stream'],
        control: ['run-model', 'manage-models'],
        interior: 'behavioural'
      },
      policy_boundary_refs: [],
      evidence_refs: ['source:ollama-readme'],
      provenance_refs: ['github:ollama/ollama@d67ad83426633195089509347ffd4fe795120198'],
      provider_metadata: {
        evidence_class: 'source-conformance-template',
        deployment_assumption: 'world-local service; not a claim that every Ollama deployment is inside an O:I world'
      }
    }
  },

  llamaCppDirect: {
    source_lock: {
      repository: 'ggml-org/llama.cpp',
      revision: '4df29be4f4c3673f428170fda944a5b19f743bb8',
      evidence_ref: 'README.md',
      evidence_class: 'upstream-source'
    },
    upstream_facts: [
      'llama.cpp documents direct model execution through llama cli.',
      'llama.cpp also documents an OpenAI-compatible API server and supports inference locally and in the cloud.'
    ],
    condition: {
      receipt_ref: 'model-condition:source-case:llama-cpp-direct',
      actuation_ref: 'actuation:source-conformance',
      world_ref: 'world:source-conformance',
      agent_ref: 'agent:source-conformance',
      agency_ref: 'agency:source-conformance',
      model: {
        model_ref: 'model:source-case',
        variant_ref: 'model-variant:gguf',
        artifact_refs: []
      },
      engine: {
        engine_ref: 'engine:llama.cpp',
        source_provenance_refs: ['github:ggml-org/llama.cpp@4df29be4f4c3673f428170fda944a5b19f743bb8']
      },
      materialisation: {
        mode: 'process',
        world_relation: 'inside-world',
        material_binding_refs: []
      },
      surface: {
        kind: 'cli',
        contract_ref: 'model-surface:llama-cli',
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
        interior: 'behavioural'
      },
      policy_boundary_refs: [],
      evidence_refs: ['source:llama-cpp-readme'],
      provenance_refs: ['github:ggml-org/llama.cpp@4df29be4f4c3673f428170fda944a5b19f743bb8'],
      provider_metadata: {
        evidence_class: 'source-conformance-template',
        deployment_assumption: 'direct process inside the conformance world'
      }
    }
  },

  llamaCppServer: {
    source_lock: {
      repository: 'ggml-org/llama.cpp',
      revision: '4df29be4f4c3673f428170fda944a5b19f743bb8',
      evidence_ref: 'README.md',
      evidence_class: 'upstream-source'
    },
    upstream_facts: [
      'llama.cpp documents an OpenAI-compatible API server in addition to direct CLI inference.'
    ],
    condition: {
      receipt_ref: 'model-condition:source-case:llama-cpp-server',
      actuation_ref: 'actuation:source-conformance',
      world_ref: 'world:source-conformance',
      agent_ref: 'agent:source-conformance',
      agency_ref: 'agency:source-conformance',
      model: {
        model_ref: 'model:source-case',
        variant_ref: 'model-variant:gguf',
        artifact_refs: []
      },
      engine: {
        engine_ref: 'engine:llama.cpp',
        source_provenance_refs: ['github:ggml-org/llama.cpp@4df29be4f4c3673f428170fda944a5b19f743bb8']
      },
      materialisation: {
        mode: 'service',
        world_relation: 'inside-world',
        material_binding_refs: []
      },
      surface: {
        kind: 'http',
        contract_ref: 'model-surface:openai-compatible-v1',
        binding_ref: null
      },
      harness: {
        harness_ref: 'harness:source-case',
        harness_composition_ref: null,
        agent_session_ref: null,
        model_relation: 'external-surface'
      },
      access: {
        inference: ['invoke', 'stream'],
        control: ['start', 'stop'],
        interior: 'behavioural'
      },
      policy_boundary_refs: [],
      evidence_refs: ['source:llama-cpp-readme'],
      provenance_refs: ['github:ggml-org/llama.cpp@4df29be4f4c3673f428170fda944a5b19f743bb8'],
      provider_metadata: {
        evidence_class: 'source-conformance-template',
        deployment_assumption: 'service inside the conformance world'
      }
    }
  },

  vllmDistributedService: {
    source_lock: {
      repository: 'vllm-project/vllm',
      revision: '6b0b850a8b1764a66d7ffbb023c0b0e0bbdb900b',
      evidence_ref: 'README.md',
      evidence_class: 'upstream-source'
    },
    upstream_facts: [
      'vLLM is an inference and serving library with OpenAI-compatible, Anthropic Messages and gRPC server surfaces.',
      'vLLM documents tensor, pipeline, data, expert and context parallelism for distributed inference.'
    ],
    condition: {
      receipt_ref: 'model-condition:source-case:vllm-distributed',
      actuation_ref: 'actuation:source-conformance',
      world_ref: 'world:source-conformance',
      agent_ref: 'agent:source-conformance',
      agency_ref: 'agency:source-conformance',
      model: {
        model_ref: 'model:source-case',
        variant_ref: null,
        artifact_refs: []
      },
      engine: {
        engine_ref: 'engine:vllm',
        source_provenance_refs: ['github:vllm-project/vllm@6b0b850a8b1764a66d7ffbb023c0b0e0bbdb900b']
      },
      materialisation: {
        mode: 'distributed-service',
        world_relation: 'spans-worlds',
        material_binding_refs: []
      },
      surface: {
        kind: 'http',
        contract_ref: 'model-surface:openai-compatible-v1',
        binding_ref: null
      },
      harness: {
        harness_ref: 'harness:source-case',
        harness_composition_ref: null,
        agent_session_ref: null,
        model_relation: 'external-surface'
      },
      access: {
        inference: ['invoke', 'stream', 'structured-output', 'tool-use'],
        control: [],
        interior: 'behavioural'
      },
      policy_boundary_refs: [],
      evidence_refs: ['source:vllm-readme'],
      provenance_refs: ['github:vllm-project/vllm@6b0b850a8b1764a66d7ffbb023c0b0e0bbdb900b'],
      provider_metadata: {
        evidence_class: 'source-conformance-template',
        deployment_assumption: 'distributed materialisation chosen to exercise Actuation/Workcell placement semantics'
      }
    }
  }
});
