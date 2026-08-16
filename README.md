# Actuation

Actuation is the developmental and reference home for **first-class agentic actuation** and for the **model / harness / agent-instance research field** in the EpiLogos system.

It exists to make the ontology, portable contracts, reference runtimes, harness experiments, model research, epistemic cultivation and conformance of agency explicit. It is **not** the mandatory residence of agent instances and it is not a seventh runtime tier inserted between the existing products. In normal use an agent inhabits the world/directory to which it is bound; in the personal O:I system that is ordinarily a Central world. Actuation develops the reusable semantics by which such an agent can be constituted, differentiated, related to other agents, and changed by what they return, while also providing the canonical developmental home for studying the models and harnesses through which that agency is realised.

## Core relation

The current generic foundation is:

```text
Actuation(g, W, I) = C⟨g, L, D, Γ, B, R⟩
```

where:

- `g` is the governing/determining situated Agency;
- `W` is its operative world/scope;
- `I` is the purpose under which agency is differentiated;
- `L` is the set of participating agentic loci;
- `D` records determination/lineage relations;
- `Γ` records how those loci may depend on, coordinate with, review, isolate, or otherwise relate to one another;
- `B` records authority/capability/resource/world bounds;
- `R` records how attributable returned difference can re-enter the governing world.

`C` is an **AgenticComposition**: semantic actor plurality. It is intentionally distinct from AIKit `HarnessComposition`, Factory `ExecutionDisposition`, O:I `SharedField`, and Workcell process/service topology.

The determining position is a relation, not a `ManagerAgent`, `MasterAgent`, or `WorkerAgent` species. Any authorised locus may recursively govern another composition. Independently grounded Agents may enter by federation without being re-described as products of the governing Agent.

## Metagency and root agency

**Metagency** is an Agency's capacity to take agency itself as an object of action: to establish, bind, configure, relate, suspend, recall, replace, or reintegrate agentic loci inside its world.

A **RootAgency** is simply an Agency whose WorldBinding is the enclosing Objective Internality for a scope. The personal whole-level agent is therefore a first-class instance of the same ontology it can apply recursively; it is not a hidden control-plane exception.

A constitutional rule follows:

> **Downward authority requires upward reality.**
>
> Differentiated agency must have an admissible return path — or an explicitly declared autonomous termination — and returned evidence/difference remains attributable before synthesis.

## Model and epistemic research

Actuation also treats the model/harness/agent-instance layer as a first-class research domain.

This includes both:

```text
discovery
human corpus → trained model → latent organisation → model-interior study

cultivation
human/agent epistemic craft → corpus/dialogue/annotation → adaptation/context → changed organisation and agency
```

The programme treats **crafting epistemologies as a first-class form of AI work**. Humans and agents may deliberately organise source corpora, annotate structural relations, generate and edit QL/MEF-aligned dialogues, construct contrastive and counterfactual material, adapt models, and study the resulting changes in behaviour, Judgement Space and model-interior dynamics.

QL enters as the full structural/process language developed by Quaternal Logic: positional relation, transition, conjugacy, recursion, return and topology, not merely a six-label taxonomy. MEF provides controlled full-field refractions through which the same object can be disclosed differently and compared internally and externally.

See [`docs/EPISTEMIC-CULTIVATION-AND-MODEL-INTERIOR-RESEARCH.md`](docs/EPISTEMIC-CULTIVATION-AND-MODEL-INTERIOR-RESEARCH.md) and [`experiments/epistemic-cultivation/`](experiments/epistemic-cultivation/).

## Maximal reference harness

Actuation deliberately develops its portable agent/body seams against a demanding real harness rather than only toy runtimes. The current maximal reference is the pinned public **DeepSeek Harness (DSH)** source exercised by the QL Runtime proving body.

This is a reference and conformance pressure test, **not** a suite-wide dependency: DSH informs the breadth of the portable abstractions without becoming their ontology, and alternative harnesses or future forks/derivatives can target the same seams. See [`docs/HARNESS-REFERENCE.md`](docs/HARNESS-REFERENCE.md) for the repository-level rule and [`experiments/ql-runtime/comparison/series1/DEEPSEEK-HARNESS-MAXIMAL-REFERENCE.md`](experiments/ql-runtime/comparison/series1/DEEPSEEK-HARNESS-MAXIMAL-REFERENCE.md) for the pinned experimental composition.

## Repository map

- [`docs/ACTUATION-CONSTITUTION.md`](docs/ACTUATION-CONSTITUTION.md) — constitutional purpose, metagency, determination modes, invariants, model-research scope, and product boundary.
- [`docs/ACTUATION-RELATION.md`](docs/ACTUATION-RELATION.md) — the one↔many↔return relation and recursive composition grammar.
- [`docs/SYSTEM-PLACEMENT.md`](docs/SYSTEM-PLACEMENT.md) — placement across O:I, Central, Factory, AIKit, Workcell, and QL-MEF.
- [`docs/EPISTEMIC-CULTIVATION-AND-MODEL-INTERIOR-RESEARCH.md`](docs/EPISTEMIC-CULTIVATION-AND-MODEL-INTERIOR-RESEARCH.md) — the model-interior, epistemic-cultivation, QL/MEF and dataset/dialogue research specification.
- [`docs/HARNESS-REFERENCE.md`](docs/HARNESS-REFERENCE.md) — maximal-reference harness policy and portability boundary.
- [`schemas/actuation.v0.schema.json`](schemas/actuation.v0.schema.json) — language-neutral experimental `AgenticComposition` contract using opaque suite refs.
- [`docs/QL-RUNTIME-MIGRATION.md`](docs/QL-RUNTIME-MIGRATION.md) — provenance and acceptance rules for graduating the QL Agent Runtime experiments from Factory.
- [`experiments/ql-runtime/`](experiments/ql-runtime/) — complete pinned QL Agent Runtime proving body migrated from Factory issue #94 / draft PR #130.
- [`experiments/epistemic-cultivation/`](experiments/epistemic-cultivation/) — proving ground for inherited vs cultivated epistemic structure and model-interior experiments.

The programme is tracked by the **Actuation Wayfinder** in this repository's issues.

## Product boundary

Actuation does not absorb the rest of the suite.

**Central** remains the persistent authored personal ground and natural operative residence of world-bound agents. **O:I** owns whole-level composition, selective disclosure, Projection, Participant and co-internal relations, and can project/share Actuation research objects without owning their model semantics. **Software Factory** owns developmental Project/Run/evidence/candidate/repair and `ExecutionDisposition`, and can build or consume tooling required by Actuation research without becoming the canonical home of model/harness experiments. **AIKit** resolves Context, capabilities, models, `HarnessComposition`, AgentSession, Surfaces and research profiles per locus; it supplies the configured tools/skills/providers through which Actuation experiments run rather than owning the scientific programme. **Workcell** supplies material process/service/storage/network/lifecycle support. **QL-MEF / Quaternal Logic** owns the formal QL/MEF canon and semantic resources that Actuation can operationalise in model, corpus, disclosure and topology experiments.

## QL runtime graduation

The QL Agent Runtime programme began in Software Factory as an independent harness-neutral recurrence experiment. Its full experimental tree is now pinned here from:

```text
EpiLogos/agent-system-design#94
EpiLogos/agent-system-design#130
source head a654c62f68b82236061986d9215b23257fe53b17
```

Migration preserves evidential status. Structural/conformance evidence exists; at the migration boundary there are **zero claimed live Series 1 capability runs** and therefore no claimed QL capability effect. Historical Factory review remains provenance; new canonical experiment development proceeds in Actuation after the migration foundation is accepted.

## Numbering and QL profiles

The **generic agency ontology** in this repository remains intentionally number-neutral. Labels such as `#0`, `#1`, `0/1`, named Epi-Logos positions, bimba/pratibimba, and QL recurrence structures are not generic Agent runtime types.

That does not make QL incidental to Actuation research. The base sixfold may operate as a silent design primitive, and explicit QL/MEF structures may organise experiments, datasets, trajectories and model-interior hypotheses wherever the research requires them. The possible future exchange of `#0` and `#1` therefore does not destabilise the generic Actuation ontology while leaving the full QL research field open.
