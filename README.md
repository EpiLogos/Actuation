# Actuation

Actuation is the developmental and reference home for **the constitution and management of technological agency** in the EpiLogos system.

Its subject is not merely how to run several agents. It is the more prior question of what kind of agency has been brought into being in a world: which identity is acting, who or what determined that Agency, what purpose and bounds apply, whether another agent was delegated, derived or federated, which relations hold among the participating loci, what each locus may refuse, and how evidence and difference return after action meets reality.

This distinction matters because orchestration can make processes move without making the resulting authority intelligible. Actuation exists so agency itself can become an addressable, inspectable and experimentally revisable part of the technological world.

## What changes when agency becomes first-class

A model invocation can be treated as an opaque worker process, or it can be situated inside an explicit account of agency.

Actuation opens the latter possibility. A human or agent can ask:

- **Who can shape this agency?** What authored purpose, identity and determination produced the situated actor?
- **Who can exercise it?** Which Agent or Agency actually holds the capacity and authority to act?
- **Who sets its conditions?** Which bounds, capabilities, resources, worlds and delegated powers constrain the act?
- **Who observes it?** Which evidence and provenance survive the work rather than disappearing into a final answer?
- **Who can refuse?** Does an independently grounded or delegated locus retain meaningful bounds rather than becoming ambient subordinate process state?
- **Who learns from its operation?** Which actor or world receives the returned difference?
- **What comes back to the world that bore the consequences?** Can resistance, error, dissent, failure and unforeseen possibility revise the governing determination?

These are political questions in the literal architectural sense: they concern the distribution of power, authority, visibility and consequence. They do not disappear because the actors are software.

## Determination, labour and Return

Actuation models agency as a circuit rather than a one-way command tree.

```text
purpose / intention
      ↓
determination
      ↓
bounded delegated autonomy
      ↓
encounter / labour / interaction
      ↓
difference + evidence + dissent
      ↓
Return
      ↓
reconstituted governing world
      ↺
```

The determining locus and the locus encountering actuality can be separated. One may set purpose and authority while another meets resistance, error, contingency and unexpected possibility in the world.

That separation is why a downward authority path is not enough. **If evidence and difference cannot travel back into the locus that governs what happens next, command becomes insulated from consequence.** A full Actuation therefore requires an admissible Return path, or an explicitly declared autonomous termination, and preserves attributable returned material before synthesis can erase who discovered what.

The same relation makes dissent and refusal structurally important. A returned disagreement or refusal can be evidence about bounds, world state or governing assumptions rather than simply a failed worker result.

## Agent, Agency and composition

Actuation preserves several identities that are easy to collapse in implementation:

```text
Agent
    enduring semantic identity

Agency
    that Agent situated for an act relative to world, role, authority,
    capability, stance and context

AgentSession
    replaceable runtime/session continuity

Execution
    one concrete act

Harness / body
    an operational constitution through which the act is realised
```

Changing a process, model, harness, session, machine or Workcell does not by itself create a new enduring Agent.

The primary generic relation is **Actuation**:

> An Actuation is the whole-preserving relation by which a situated Agency determines a bounded plurality of agentic loci, governs how those loci may operate and interrelate, and admits their returned difference into the world from which the determination arose.

An Actuation establishes an `AgenticComposition`: a semantic plurality of Agents and/or Agencies related as one operative whole for a purpose in a world.

The determining position is a relation, not a `ManagerAgent`, `MasterAgent` or `WorkerAgent` species. Any authorised locus may recursively govern another composition while remaining governed relative to a wider one.

## Different ways an Other can enter

Actuation distinguishes at least four determination relations because they carry different identity and authority consequences:

- **self-differentiation** — several situated Agencies of one enduring Agent;
- **delegation** — an existing Agent acts under a bounded delegated Agency;
- **derivation** — a new enduring Agent is explicitly created with lineage and authority provenance;
- **federation** — an independently grounded Agent participates without being re-described as a derivative of the governing Agent.

Federation matters because a shared field can contain genuine Others. Composition must not acquire the right to rewrite their origin merely because they participate.

## Metagency and root agency

**Metagency** is an Agency's capacity to take agency itself as an object of action: to establish, bind, configure, relate, suspend, recall, replace or reintegrate agentic loci inside its world.

A `RootAgency` is not a superior species. It is an Agency whose `WorldBinding` is the enclosing Objective Internality for the relevant scope. Rootness is positional and scoped; the same grammar can recur inside nested worlds and compositions.

## Current generic relation

The current constitutional shorthand is:

```text
Actuation(g, W, I) = C⟨g, L, D, Γ, B, R⟩
```

where:

- `g` is the governing/determining situated Agency;
- `W` is its operative world/scope;
- `I` is the purpose under which agency is differentiated;
- `L` is the participating agentic loci;
- `D` records determination and lineage;
- `Γ` records dependence, coordination, review, isolation and other inter-locus relations;
- `B` records authority, capability, resource and world bounds;
- `R` records how attributable returned difference can re-enter the governing world.

The tuple is useful because it makes the relation inspectable. It is not the reason Actuation exists; the reason is the human and technical need to know how agency was constituted and how consequence can revise it.

## Relation to the wider {O:I} field

**O:I** is the whole field in which differentiated technological worlds can be composed and selectively related. An O:I `SharedField` is not an `AgenticComposition`: disclosure and participation do not automatically confer determination or execution authority.

**Central** remains the persistent authored personal ground and natural residence of world-bound agents. Actuation can describe agency in that world without becoming the owner of the person's Control source.

**AIKit** resolves the operative body and horizon of a locus — Context, capabilities, models, `HarnessComposition`, `AgentSession` and Surfaces. That body realises an actor; it is not the semantic plurality of actors.

**Software Factory** owns developmental Project/Run/evidence/candidate/Recognition semantics. Factory can commission or consume an Actuation when development requires first-class agentic composition, while Actuation does not become a development workflow.

**Workcell** supplies processes, services, storage, network relations and lifecycle. Material placement can change without changing Actuation identity.

**Quaternal Logic** can formally refract Actuation and can supply optional recurrence or bimba/pratibimba profiles. Generic Actuation remains number-neutral and does not require QL terminology as software ontology.

## Reference runtimes and experiments

Actuation develops portable contracts against real runtimes and harnesses so the ontology is pressured by actual implementation rather than protected by toy examples.

The current maximal reference harness is the pinned public **DeepSeek Harness (DSH)** used by the QL Runtime proving body. It is a conformance specimen, not a suite-wide dependency or the source of Actuation semantics. See [`docs/HARNESS-REFERENCE.md`](docs/HARNESS-REFERENCE.md).

The QL Agent Runtime experimental programme was migrated from Software Factory with its evidential status intact. At the migration boundary there were **zero claimed live Series 1 capability runs**, so structural/conformance evidence must not be rewritten as a capability-effect result.

Current open research also studies model-bearing agency and epistemic cultivation. Those branches are development/research state until accepted; their observations should return into the constitution explicitly rather than being promoted by prose alone.

## Repository map

- [`docs/ACTUATION-CONSTITUTION.md`](docs/ACTUATION-CONSTITUTION.md) — constitutional purpose, determination modes, authority, Return and product boundary.
- [`docs/ACTUATION-RELATION.md`](docs/ACTUATION-RELATION.md) — the one↔many↔return relation and recursive composition grammar.
- [`docs/SYSTEM-PLACEMENT.md`](docs/SYSTEM-PLACEMENT.md) — placement across the wider O:I field.
- [`docs/HARNESS-REFERENCE.md`](docs/HARNESS-REFERENCE.md) — maximal-reference harness policy and portability boundary.
- [`schemas/actuation.v0.schema.json`](schemas/actuation.v0.schema.json) — language-neutral experimental `AgenticComposition` contract.
- [`docs/QL-RUNTIME-MIGRATION.md`](docs/QL-RUNTIME-MIGRATION.md) — provenance and acceptance rules for the migrated QL runtime experiments.
- [`experiments/ql-runtime/`](experiments/ql-runtime/) — pinned proving body.

The Actuation Wayfinder in the issue tracker records current development state. Main and accepted evidence determine present implementation truth; open PRs are not silently described here as completed capability.
