# Actuation Relation

**Status:** proposed normative relation, v0.1

This document makes the one-to-many relation explicit without turning "one" and "many" into different kinds of Agent.

## 1. Minimal form

Let:

- `g` be the governing/determining situated Agency;
- `W` be the operative world/scope;
- `I` be the intention/purpose under which differentiation occurs;
- `L = {l₁ ... lₙ}` be the participating agentic loci;
- `D` be determination and lineage relations;
- `Γ` be the interrelation/dependency graph among loci;
- `B` be authority, capability, resource, time and world bounds;
- `R` be the admissible return/convergence relation.

Then:

```text
Actuation(g, W, I) = C⟨g, L, D, Γ, B, R⟩
```

where `C` is an `AgenticComposition`.

The composition is not reducible to `L`. The same set of Agents under different `D`, `Γ`, `B`, or `R` is a different composition because the relation governing how the many may act is different.

## 2. One-to-many-to-one circuit

The generic actuation circuit is:

```text
                    determination
        g ─────────────────────────────► C(l₁ ... lₙ)
        ▲                                      │
        │                                      │ labour / interaction
        │                                      │ under Γ and B
        │                                      ▼
        └────────── R(returned difference) ─ ΔW
```

A successful return need not restore the original state. Its point is precisely that labour may alter the operative world and therefore alter what the governing Agency can subsequently know, intend, or determine.

In shorthand:

```text
g ⊳[W,I] C(L; D, Γ, B) ⟲[R] ΔW
```

This notation means only "a governing Agency differentiates a bounded composition and admits its return." It does not assert QL numerology, hierarchy of worth, or phenomenology.

## 3. Locus model

An `AgenticLocus` is a participation position in one composition, not a replacement identity.

Conceptually:

```text
AgenticLocus {
  locus_ref
  agent_ref
  agency_ref
  world_binding
  entry_mode
  authority_bounds
  capability_requirements
  relation_roles
  return_obligation
}
```

`entry_mode` is one of:

```text
self-differentiation | delegation | derivation | federation
```

A locus MAY refer to the governing Agent itself through a different `AgencyRef`. A locus MAY refer to an independently grounded Other. A composition therefore does not imply a tree of created identities even when its operational execution happens to be tree-shaped.

## 4. Relation graph

`Γ` is first-class because "one agent governs many agents" is incomplete unless the many's relations to one another are also representable.

At minimum, `Γ` must be able to express:

```text
precedes(a,b)
blocks(a,b)
requires(a,b)
provides(a,b)
reviews(a,b)
contrasts(a,b)
coordinates(a,b)
communicates(a,b)
shares_world(a,b)
isolates(a,b)
returns_to(a,g)
```

These are relation families, not a closed enum. Product-specific relations MAY extend them. Factory `ExecutionDisposition`, for example, can operationalise parallelism, barriers, candidate fan-out, synthesis or repair without becoming the canonical owner of `Γ`.

The governing locus determines or accepts the composition-level rules. It does not thereby micromanage every local act. The point of differentiating agency is to allow bounded loci to encounter and return realities not already present in the governing locus's initial determination.

## 5. Governing relation

"Governor", "manager", "orchestrator", "master", and profile names are not generic Agent identities. The generic semantic fact is:

```text
Governs(g, C, scope)
```

A locus governs a composition when it is accountable for establishing or accepting its purpose, membership constraints, authority envelope, relation topology, and return conditions for that scope.

A child locus may itself govern another composition:

```text
C₀(g₀, l₁, l₂)
       │
       └── l₂ governs C₁(l₂, m₁, m₂, m₃)
```

This produces recursive first-class agency without creating a privileged `OrchestratorAgent` species.

## 6. Determination versus domination

`Governs` does not mean that every participant is derived from, owned by, or exhaustively controlled by `g`.

The authority relation is bounded by `B`. A federated participant can remain independently grounded. A delegated agent can retain identity and standing commitments outside this composition. A derived agent has explicit lineage but is still represented as an Agent, not as an anonymous tool call.

This is how Actuation remains compatible with O:I's independently grounded Other and with genuine multi-agent collaboration rather than reducing all plurality to one hidden controller.

## 7. Return

`R` is not merely a callback. It identifies how the composition's actuality can become operative for the determining world.

A return MAY contain attributable:

```text
ArtifactRef
ClaimRef
EvidenceRef
DecisionRef
ContributionRef
world delta
failure
refusal
dissent
open question
continuation demand
```

A composition may have multiple staged returns, not only one terminal synthesis.

A return is valid when it preserves enough provenance to answer:

- which Agent/Agency/locus produced this difference;
- under what world and authority bounds;
- through what Execution/AgentSession/harness/material provenance where relevant;
- what transformation or interpretation occurred before it reached the governing locus.

## 8. Root agency

For a local composition, `g` may be any authorised Agency.

For the personal whole-level case:

```text
WorldBinding(g) = enclosing Objective Internality
```

and `g` is therefore a `RootAgency` for that scope.

Nothing else changes. Root actuation uses the same `C⟨g,L,D,Γ,B,R⟩` grammar as nested actuation. This is a deliberate constraint: the personal whole-level agent must be a first-class instance of the same ontology it can apply recursively, not a hidden control-plane exception.

## 9. Relation to harnesses

An `AgenticComposition` is not a `HarnessComposition`.

```text
AgenticComposition
    semantic plurality of agentic loci

HarnessComposition
    operational body resolved for one locus/actor
```

An Actuation with four loci may therefore resolve four different bodies, or several loci may use the same host/runtime technology, without changing the semantic composition. Conversely a richly componentised harness may embody one AgenticLocus and must not be misread as a multi-agent composition merely because its body contains many components.

## 10. Relation to execution

Factory and other clients may operationalise `C` as one, sequential, parallel, fan-out, barriered, nested, repairing, or synthesising executions. Those choices belong to execution intelligence.

The semantic ordering is:

```text
intent / semantic need
        ↓
Actuation + AgenticComposition
        ↓
execution demand / disposition
        ↓
AIKit body + session + surface resolution
        ↓
Workcell materialisation
        ↓
execution evidence
        ↓
Return
```

Execution topology can change while the composition remains semantically the same, provided the declared relations, bounds, identities and return obligations are preserved.

## 11. QL projection seam

QL-MEF may project the generic relation through profile-specific forms such as standing/canonical possibility, contextual articulation, conjugation, bimba/pratibimba, 0/1 and 1/0, or a 4+2 circuit.

Those projections may generate strong hypotheses for how `D`, `Γ`, and `R` should be structured or tested. They do not redefine the generic equation so that `bimba = one Agent` or `pratibimba = many Agents`. The QL relation remains a formal/refraction reading over the generic first-class agentic structure.
