# Actuation — Model-Bearing Agency, Research and Materialisation

**Status:** research and product direction  
**Scope:** Actuation with AIKit, Workcell and O:I seams  
**Method:** O:I research protocol — Discover → Source-lock → Study → Interpret → Abstract → Compare → Operationalise → Experiment → Return

## 1. Research object

Contemporary AI is already a continuous experiment in the **management of agency**.

Models do not arrive as finished capacities which are only later wrapped by secondary software. Their realised agency changes through inference engines, harnesses, loops, skills, tools, context, memory, material placement and human practice. Labs and product companies learn from those uses; users and independent developers discover capabilities, failures and techniques; new models, harnesses and practices return into the field.

Actuation should make that experimental condition explicit. Its research object is not only the model and not only the agent loop. It is **model-bearing agency as constituted in a world**: which model capacity is available, how it is materialised, through which surface and harness it acts, what access exists to its interior and learning process, and how changing those conditions changes realised agency.

The political question is the same problem at another scale. Managing agency means deciding which capacities can act, under which conditions, with which permissions, evaluators, interfaces and return paths. The relevant questions are therefore direct: **who determines those conditions, who can inspect or alter them, who can refuse or substitute them, who learns from the experiment, and how does what is learned return to the people and worlds which produced it?**

This follows the existing Antykathera framing of the hidden governing ground and the sovereign commons: technical coordination is healthy when its grounds, dependencies, permissions and returns remain legible and contestable across plural centres rather than disappearing into an unanswerable platform position.

O:I should therefore remain local-first without turning locality into a moral binary. Local model capacity increases the user's practical ability to retain, reproduce, inspect, modify and experimentally recombine the conditions of agency. Data-centre inference remains a legitimate and often necessary materialisation for models, accelerators and concurrency beyond a personal machine. The architecture must make the difference visible rather than bake either location into Agent identity.

## 2. Source-study baseline

The first local-model semantics should be derived from several established systems rather than one implementation.

| Source | Inspected revision | What it pressure-tests |
|---|---|---|
| `ollama/ollama` | `d67ad83426633195089509347ffd4fe795120198` | managed model environment: install/run/manage models, persistent API, agent integrations |
| `ggml-org/llama.cpp` | `4df29be4f4c3673f428170fda944a5b19f743bb8` | direct inference engine: CLI and OpenAI-compatible server, broad local hardware, CPU/GPU hybrid execution |
| `vllm-project/vllm` | `fdab2b10bcac00a16c406f8b17a75a1c3f729e59` | high-performance model serving: OpenAI/Anthropic/gRPC surfaces, adapters, multi-GPU and distributed inference |
| `sgl-project/sglang` | `ace7314173c8221ecf5f213575302eab98f4e84f` | second rich serving/runtime comparison for distributed and high-performance cases |
| `JustVugg/colibri` | `4ef9a9920dd4290b4ac26f74e73913fd18379bc9` | experimental inference/materialisation: very large MoE models staged across VRAM, RAM and storage |

The source roles matter. Ollama is a strong first **product/conformance implementation** because it covers the ordinary local user journey and already connects to existing harnesses. llama.cpp and vLLM prevent Ollama's product shape from becoming the ontology. Colibri is deliberately not the baseline provider: it is a research specimen showing that an unusual inference technique can sit behind ordinary API/harness relations.

## 3. Stable model-bearing relation

The comparison suggests the following separable field:

```text
ModelArtifact / ModelVariant
        ↓
InferenceEngine
        ↓
ModelMaterialisation
        ↓
ModelSurface
        ↓
Harness / HarnessComposition
        ↓
AgentSession + situated Agency
```

These are relations and responsibilities before they are canonical new types.

- **ModelArtifact / Variant** — checkpoint/weights, tokenizer, quantisation and adapters with source, licence and revision provenance.
- **InferenceEngine** — the software which executes the artifact: llama.cpp, vLLM, SGLang, Colibri, an embedded engine, or a provider-owned equivalent.
- **ModelMaterialisation** — one running realisation with placement, resources, lifecycle and service bindings. `local` or `remote` describes this layer, not the model's identity.
- **ModelSurface** — the operative inference interface: in-process call, CLI/stdin, OpenAI-compatible HTTP, Anthropic Messages, native provider API or another declared contract.
- **Harness** — the body which turns the model relation into situated agency through loop, tools, skills, context and interaction surfaces.

A model surface and a material control surface must also remain distinguishable. A client may be able to infer through a model without being allowed to acquire, stop, replace, tune or modify the materialisation which serves it.

Actuation therefore needs to express both:

```text
Inference access
  invoke / stream / tool-use / structured-output / modality capabilities

Control access
  acquire / prepare / start / stop / load / unload / configure /
  inspect / replace / adapt / intervene
```

The existing graded model-interior research field remains orthogonal and can deepen from behavioural and output access through internal read/write, causal intervention and learning access.

## 4. Actuation product range

Actuation must be a real system while preserving the ordinary way agents are already instantiated.

### Collapsed case

```text
open a directory
      ↓
start an existing harness or model CLI
      ↓
act in that world
```

This is already a valid situated Actuation. No special daemon or Actuation-owned residence is required. The system should be able to launch, attach to, describe or receipt the condition with minimal ceremony.

### Resolved case

AIKit makes the operative body explicit: Agent/Agency binding, Context, Model, Harness, capabilities, `HarnessComposition`, `AgentSession` and Surfaces. Model, harness, session, endpoint and provider changes remain observable without silently changing enduring Agent identity.

### Managed local case

Actuation becomes the user-facing place where a model-bearing body can be constituted:

```text
choose model / variant
choose or accept inference engine
choose local or remote materialisation
resolve model surface
bind harness
bind world
start / enter Agency
```

For the first implementation, Ollama is the ordinary local reference; llama.cpp must prove the direct-engine/CLI/server collapse; vLLM must prove the same relation when serving moves toward multi-GPU or multi-node infrastructure. Colibri later proves that the inference/materialisation technique can change radically while the higher relation survives.

### Research case

Every resolved Actuation can become an experimental condition without every session becoming a formal experiment. When required, record the configuration sufficiently to compare:

```text
same task / starting state
hold selected relations stable
change model, engine, materialisation, surface, harness,
skills, context, recurrence or interior-access condition
observe difference
return attributable evidence
```

This extends the existing `J_A ↔ J_N` programme: research may compare changes in the wider Agent Judgement Space with model-interior observations where access permits.

## 5. Product ownership seams

**Actuation** owns the meaning of model-bearing agency, its collapsed-to-developed product path, its experimental conditions, `AgenticComposition`, bounds and Return. It should not become a package manager, GPU scheduler or provider registry.

**AIKit** resolves the effective model/harness/body relation. Its existing Resource/provider and `HarnessComposition` work should be extended only as needed to express model artifact/variant refs, inference-surface contracts, control capabilities, provider bindings and material binding provenance. It should not implement inference engines.

**Workcell** materialises the required processes, services, storage, accelerators, endpoints, connectivity and lifecycle. A local Ollama daemon, a `llama-server` process, a vLLM service on a GPU host, or a Colibri service are ordinary material workloads/providers beneath stable semantic demand. Workcell does not decide which model means what to the Agent.

**O:I** makes the whole condition legible through Objective Internality, whole-level composition, research projection and selective return. A provider/model/harness/materialisation topology may be disclosed as provenance of an Actuation or experiment without O:I acquiring model semantics or becoming telemetry infrastructure.

**Central** remains the authored human ground for preferences, permissions and durable world policy. Human choices such as permitted egress, preferred local/remote model use, acceptable providers and experiment boundaries belong here when they are enduring authored intent rather than runtime facts.

## 6. Implementation direction

Do not introduce `LocalModelProvider` as a root abstraction. Begin with conformance over existing seams.

1. **Actuation:** add a compact experimental/model-materialisation contract or receipt sufficient to distinguish model/variant, inference engine, model surface, material binding, inference/control access and model-interior access depth. Keep it subordinate to Agent/Agency/Actuation identity.
2. **AIKit:** resolve those relations using the existing Model/provider/Contract/`HarnessComposition` architecture. First prove Ollama; then llama.cpp and vLLM. Provider-specific model names, ports and process IDs remain provider facts.
3. **Workcell:** add model-serving conformance demands over ordinary process/service/storage/accelerator/lifecycle contracts. Prove collapsed-local and remote-capable materialisation without a `ModelServer` ontology if existing provider ports suffice.
4. **O:I:** project the resulting condition/receipt as optional provenance and research material, preserving the distinction between constitutive conditions of agency and the selected material disclosed publicly.
5. **Research:** use Colibri and later engines as interventions on the inference/materialisation relation; use harness changes, skills and QL recurrence as independent experimental variables.

The architectural direction is therefore simple:

> **Local model capacity should become a normal Actuation capability; remote model capacity remains another materialisation of the same model relation.**

The broader aim is not merely to run models locally. It is to make the constitution and management of model-bearing agency increasingly explicit, reproducible, alterable and return-bearing for the humans and agents who participate in the experiment.
