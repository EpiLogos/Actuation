# Actuation

Actuation is the developmental and reference home for **first-class agentic actuation** in the EpiLogos system.

It exists to make the ontology, portable contracts, reference runtimes, harness experiments, and conformance of agency explicit. It is **not** the mandatory residence of agent instances and it is not a seventh runtime tier inserted between the existing products. In normal use an agent inhabits the world/directory to which it is bound; in the personal O:I system that is ordinarily a Central world. Actuation develops the reusable semantics by which such an agent can be constituted, differentiated, related to other agents, and changed by what they return.

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

## Repository map

- [`docs/ACTUATION-CONSTITUTION.md`](docs/ACTUATION-CONSTITUTION.md) — constitutional purpose, metagency, determination modes, invariants, and product boundary.
- [`docs/ACTUATION-RELATION.md`](docs/ACTUATION-RELATION.md) — the one↔many↔return relation and recursive composition grammar.
- [`docs/SYSTEM-PLACEMENT.md`](docs/SYSTEM-PLACEMENT.md) — placement across O:I, Central, Factory, AIKit, Workcell, and QL-MEF.
- [`schemas/actuation.v0.schema.json`](schemas/actuation.v0.schema.json) — language-neutral experimental `AgenticComposition` contract using opaque suite refs.
- [`docs/QL-RUNTIME-MIGRATION.md`](docs/QL-RUNTIME-MIGRATION.md) — provenance and acceptance rules for graduating the QL Agent Runtime experiments from Factory.
- [`experiments/ql-runtime/`](experiments/ql-runtime/) — complete pinned QL Agent Runtime proving body migrated from Factory issue #94 / draft PR #130.

The programme is tracked by the **Actuation Wayfinder** in this repository's issues.

## Product boundary

Actuation does not absorb the rest of the suite.

**Central** remains the persistent authored personal ground and natural operative residence of world-bound agents. **O:I** owns whole-level composition, selective disclosure, Projection, Participant and co-internal relations. **Software Factory** owns developmental Project/Run/evidence/candidate/repair and `ExecutionDisposition`, and consumes Actuation when developmental work requires first-class agentic composition. **AIKit** resolves Context, capabilities, models, `HarnessComposition`, AgentSession and Surfaces per locus. **Workcell** supplies material process/service/storage/network/lifecycle support. **QL-MEF** may formally refract the generic relation without making QL profile terminology mandatory software ontology.

## QL runtime graduation

The QL Agent Runtime programme began in Software Factory as an independent harness-neutral recurrence experiment. Its full experimental tree is now pinned here from:

```text
EpiLogos/agent-system-design#94
EpiLogos/agent-system-design#130
source head a654c62f68b82236061986d9215b23257fe53b17
```

Migration preserves evidential status. Structural/conformance evidence exists; at the migration boundary there are **zero claimed live Series 1 capability runs** and therefore no claimed QL capability effect. Historical Factory review remains provenance; new canonical experiment development proceeds in Actuation after the migration foundation is accepted.

## Numbering and QL profiles

This repository is intentionally **number-neutral**. Labels such as `#0`, `#1`, `0/1`, named Epi-Logos positions, bimba/pratibimba, and QL recurrence structures may be important profile/formal projections, but they are not generic runtime types.

The possible future exchange of `#0` and `#1` therefore does not destabilise the generic Actuation ontology. That numbering remains a separate constitutional/profile question.
