# Model-bearing agency contract

This directory contains Actuation's first executable seam for **model-bearing agency**. It is intentionally smaller than the comparative research vocabulary.

The stable v1 names are:

- `ModelRelation` — the higher model/variant relation plus nested engine, material-binding and inference-surface refs/facts;
- `ModelAccessProfile` — inference, model-control and model-interior access kept independently explicit;
- `ActuationReceipt` — the attributable record binding that condition to an Actuation, Agency, WorldBinding and optional AIKit body/session refs.

The following are **not** promoted into Actuation root primitives by this contract:

- `ModelArtifact` / `ModelVariant` objects;
- `InferenceEngine`;
- `ModelMaterialisation`;
- `ModelSurface`;
- `LocalModelProvider`;
- `ModelServer`.

Those distinctions remain representable as refs, contracts, provider-native facts and Workcell material bindings until cross-provider implementation proves that a stronger identity is required. `local`, `remote`, `distributed` and `opaque` are material placement facts and do not alter model, Agency, AgentSession or Harness identity.

## Ownership

Actuation owns the meaning of the receipt and experimental relation. AIKit is expected to resolve model/provider/Contract/Capability/HarnessComposition facts. Workcell owns the material binding referenced by `material.binding_ref`. O:I may project selected receipt/provenance fields but does not acquire their semantics.

## Verification

```sh
node --test contracts/model-bearing.test.mjs
```

The tests prove:

- the same higher model/Agency/session relation can survive local Ollama and remote vLLM materialisation;
- inference permission does not imply control or model-interior permission;
- Colibri can appear as an experimental inference/materialisation intervention without defining a provider ontology;
- provider-native scalar facts remain preservable without becoming canonical fields.
