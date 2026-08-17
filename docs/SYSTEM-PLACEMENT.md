# Actuation in the EpiLogos system

**Status:** proposed placement, v0.1

Actuation is deliberately a developmental/referential repository rather than a new mandatory runtime tier. The actual system is a vertical composition of responsibilities, and agents may inhabit worlds at different scopes inside that composition.

## 1. The personal whole-level case

```text
Human authorship / intention
          │
          ▼
O:I objective-internal whole / disclosure relations
          │
          ▼
Central persistent personal world
          │
          │ WorldBinding(root agency)
          ▼
Root Agency / metagency
          │
          │ ordinary action or Actuation
          ├──────────────────────────────┐
          │                              │
          ▼                              ▼
     direct capability              Software Factory
                                     developmental need
                                           │
                                           ▼
                                 AgenticComposition demand
                                           │
                                           ▼
                                      Actuation
                                           │
                                  one or many loci
                                           │
                                           ▼
                                        AIKit
                              context/body/session/surfaces
                                           │
                                           ▼
                                       Workcell
                                  material execution
                                           │
                                           ▼
                              evidence / returned difference
                                           │
                                           └──────────↺ Central / O:I world
```

The diagram is a responsibility flow, not a claim that every request must traverse every product.

## 2. Central is residence, not semantic ownership

The personal system's ordinary agent instance should accrete with the Central world it inhabits. It may have authored identity/context material, durable preferences, histories, bindings, and machine intent rooted in that world.

That does not make Central the canonical owner of generic Agent/Agency/Actuation semantics or a hidden orchestration database. Central remains the persistent human-authored ground and machine-operable filesystem/product surface. Actuation provides the reusable semantics and runtime experiments that let a world-bound agent be understood and reproduced.

This distinction allows many possible installations:

```text
Central/
  Control/
  Work/
    project-a/    ← project-bound agent may inhabit here
    project-b/    ← another agent may inhabit here
  ...

whole Central/O:I scope
  ↑ RootAgency may inhabit this enclosing world
```

The files that express an agent's local identity or state MAY live beside its world. The portable ontology and experimental implementations do not need to be copied into every world.

## 3. O:I relation

O:I gives the whole system a selective disclosure/composition layer and an account of Objective Internality / Objective Co-Internality.

Actuation uses that interior as a possible world of agency. O:I may disclose or project:

- an Agent;
- an Agency;
- an AgenticComposition;
- a Return;
- a participant in an Actuation where disclosure is authorised.

But `Participant` is not `AgenticLocus`, and `SharedField` is not `AgenticComposition`.

An Agent may participate in an internal Actuation without being projected into a shared field. Conversely an independently grounded Other may become an O:I Participant and later enter a federated AgenticComposition without ceasing to be Other.

## 4. Software Factory relation

Factory answers a developmental question: what semantic work, evidence, candidates, repair, sequencing, synthesis, or execution disposition is required to progress a Project/Run?

It may conclude that the required work is best realised through a plurality of agentic loci. At that boundary Factory requests or consumes Actuation rather than defining the generic ontology of the actors it uses.

```text
Factory Run / semantic act
        ↓
Execution Intelligence
        ↓
need for one or many agentic loci
        ↓
Actuation contract
        ↓
AgenticComposition
        ↓
ExecutionDisposition enacts the composition
```

This keeps `ExecutionDisposition` valuable without overloading it. It remains the answer to **how should this developmental act be enacted?** Actuation answers **what first-class agentic whole is being constituted, who/what are its loci, how are they related, under what bounds, and how does their difference return?**

The existing Epi-Logos 0/1 reader-composer and six canonical Agent identities therefore become a Factory profile/client of the generic Actuation grammar, not the definition of generic agency.

## 5. AIKit relation

AIKit resolves an operative body for each agentic locus:

```text
Agent / Agency
    ↓
Context + capabilities + model
    ↓
Harness / HarnessComposition
    ↓
AgentSession
    ↓
CLI / TUI / GUI / messaging / API / webhook Surfaces
```

Actuation does not replace this model.

The key invariant is:

> `AgenticComposition` is semantic actor plurality; `HarnessComposition` is operational body composition for an actor.

Actuation may state semantic requirements that AIKit must satisfy — persistence, capabilities, communication, isolation, authenticated interaction, recurrence support, observation — while AIKit remains responsible for resolving a concrete composition of providers/components/surfaces.

## 6. Workcell relation

Workcell provides material conditions:

- process/service lifecycle;
- storage;
- network/fabric relationships;
- service bindings;
- control plane;
- local or remote materialisation.

Actuation does not give Workcell Agent ontology. A material process is not automatically an Agent, and moving an agentic locus between Workcells does not create a new semantic Agent merely because the machine changed.

A useful conformance target is that one `AgenticComposition` can span multiple Workcells and survive provider/path rematerialisation while preserving Agent, Agency, composition, authority and return provenance.

## 7. QL-MEF relation

QL-MEF is the optional formal/refraction layer over the generic relation.

Actuation provides a stable object of study:

```text
one governing locus
        ↕
bounded differentiated plurality
        ↕
returned difference / reconstitution
```

QL-MEF may ask whether QL positions, conjugate faces, pairing families, MEF lenses, bimba/pratibimba, or other formal structures provide useful recurrent or semantic attractors for the composition. Those hypotheses are tested through Actuation-hosted runtime experiments rather than inserted into every consumer as mandatory metaphysics.

## 8. Why Actuation is a repository at all

The repo exists because the system needs one place where the generic relation can be developed independently of any one client:

```text
ontology
contracts
reference runtime seams
harness experiments
conformance
comparative evidence
QL runtime research
```

If those things remained inside Factory, first-class agency would continue to appear as a special developmental execution mechanism. If they lived only in Central, persistent residence would be confused with runtime ownership. If they lived only in AIKit, semantic agentic plurality would be confused with harness/body composition.

Actuation is therefore a **developmental home for portability**, while Central/O:I worlds remain the natural **operative home for instances**.
