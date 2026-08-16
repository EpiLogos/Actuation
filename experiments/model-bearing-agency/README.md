# Model-Bearing Agency — executable floor

This directory materialises the first executable slice of the Actuation model-bearing-agency programme from `docs/MODEL-BEARING-AGENCY-RESEARCH-AND-MATERIALISATION.md`.

It is deliberately small and provider-neutral. It does **not** implement an inference engine, provider registry, Workcell planner or AIKit resolver.

## Contracts

`ModelConditionReceipt` records one realised model-bearing condition without turning material/provider facts into Agent identity.

```text
Actuation / World / Agent / Agency
        ↓
Model + Variant
        ↓
InferenceEngine
        ↓
ModelMaterialisation
        ↓
ModelSurface
        ↓
Harness / HarnessComposition / AgentSession
        ↓
inference · control · interior-access declarations
```

The implementation keeps several distinctions executable:

```text
external-to-harness ≠ external-to-world
surface contract ≠ endpoint/material binding
inference access ≠ material control access ≠ model-interior access
process/service/provider change ≠ Agent identity change
local/remote placement ≠ Model identity
```

`ModelConditionExperiment` compares two receipts as a matched condition. It rejects drift in the situated `Actuation`, `World`, `Agent` or `Agency`, can require selected axes to remain constant, and can require intended experimental variables to have actually changed.

## Fixtures

`fixtures.js` contains three provider-neutral conformance fixtures:

1. **collapsed direct process** — a model CLI/process can be valid situated Actuation with no standing service, explicit Harness or AgentSession;
2. **world-local service external to harness** — the Harness reaches a model through an external surface while the materialisation remains inside the enclosing world;
3. **remote service with the same model/harness relation** — model and Harness identity survive rematerialisation while engine, material binding, session and access conditions can change.

These fixtures are the semantic floor for later source-pinned Ollama, llama.cpp, vLLM/SGLang and Colibri cases. Provider-specific claims belong in those conformance cases, not in this contract.

## Verification

```bash
cd experiments/model-bearing-agency
npm test
```

Node 22+ is required. There are no external runtime dependencies.

The published language-neutral schema is `schemas/model-condition.v0.schema.json`.

## Product boundary

- **Actuation** owns this condition/experiment meaning.
- **AIKit** may resolve the Model, engine/provider, surface, HarnessComposition and effective access declarations.
- **Workcell** may supply the material binding refs and observations for processes, services, storage, accelerators and connectivity.
- **O:I** may selectively project a receipt or experiment as provenance/research material.

Opaque refs are intentional. This experiment does not redefine the shared suite identity/provenance contract.
