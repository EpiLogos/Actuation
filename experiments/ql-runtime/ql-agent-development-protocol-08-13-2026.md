# `ql-agent-development-protocol`

## Development Protocol for the QL Agent Runtime Experiments

**Date:** 2026-08-13  
**Status:** Active development-order companion  
**Governing semantic specification:** `ql-agent-spec-08-13-2026.md`  
**Primary experiments:** `ql-pi`, `ql-pydantic`, `ql-native`

---

## 1. Purpose

This protocol defines **how to reach the experiment-ready QL agent system efficiently without changing what the QL agent specification requires**.

The governing specification defines the semantic target. This protocol defines:

- the common runtime architecture needed before the three experiments diverge;
- the smallest foundation that must be proven before parallel implementation begins;
- the shared interfaces that let an agent host select a classical or QL loop runtime per run;
- the repository and branch shape for the three experiments;
- the separate deep-QL development path;
- the convergence gates that return every experiment to full `QL Experiment Readiness` before formal Series 1 comparison.

This document is therefore a **development-order document**, not a reduced QL specification.

Nothing described as subsequent, parallel or extension work is thereby optional in the final experiment when the governing specification requires it. The distinction is only between:

```text
must exist before parallel work can begin
```

and:

```text
must exist before the full experiment may be frozen and treated as evidence
```

If this protocol and the governing specification disagree about QL semantics, the specification governs.

---

## 2. Central architectural decision

The experiment uses **Loop Runtime** as an independently selectable concern beneath an agent host or harness.

```text
                         AGENT HOST / HARNESS
                                │
          ┌─────────────────────┼─────────────────────┐
          │                     │                     │
      model I/O            capabilities          context/session
          │                     │                     │
          └─────────────────────┼─────────────────────┘
                                │
                          LOOP RUNTIME
                         selected per run
                                │
                    ┌───────────┴───────────┐
                    │                       │
              ClassicRuntime           QLRuntime
```

The host owns the mechanisms needed to encounter the exterior world.

The loop runtime owns the semantics of recurrence.

This division is the experiment's most important control boundary. It allows the same host, model, tools, task, environment and success conditions to be used while varying the agent loop itself.

It also makes the architecture portable. A new harness should be adaptable by implementing the host boundary rather than by rebuilding QL semantics inside that harness from scratch.

---

## 3. Responsibility boundary

### 3.1 Host responsibility

A host or harness adapter owns ordinary operational machinery such as:

```text
model invocation and streaming
tool/capability invocation
message/session transport
context delivery
cancellation and interruption
credentials and provider configuration
model selection
tool definitions
raw carrier events
human or external input channels
```

A host MAY have its own graph, event system, message representation or session model.

Those carrier structures do not define QL position or QL transition semantics.

### 3.2 Loop Runtime responsibility

A Loop Runtime owns the recurrent control question:

```text
what happens next, why, and under what condition is this run complete?
```

It therefore owns semantic equivalents of:

```text
form or request the next act
project an act into the exterior
absorb the exterior return
determine returned difference
interpret what the return means for the current whole
choose the next recurrent responsibility
maintain runtime-specific state
determine whether work remains open
propose and evaluate completion
close, fail, cancel or continue
retain the state required for recurrence and re-entry
```

For `ClassicRuntime`, this remains intentionally simple.

For `QLRuntime`, these responsibilities must realise the semantics defined by the QL specification rather than mimic an ordinary tool loop with QL metadata.

### 3.3 Shared experiment boundary

The common architecture standardises only what is required to swap loop runtimes against the same host.

It MUST NOT attempt to turn Pi, Pydantic AI and the native experiment into one universal agent framework.

Framework-native differences remain part of the experiment.

---

## 4. Minimum common runtime contract

The first implementation should keep the abstraction narrow.

Conceptually:

```text
LoopRuntime {
    id
    run(request, host, observer, signal) -> RunResult
}

RuntimeHost {
    call_model(...)
    execute_capability(...)
    receive_external_input(...)
    read_context(...)
}

RuntimeObserver {
    emit(event)
}
```

Concrete language APIs may differ.

The common contract must preserve these invariants:

1. `classic` and `ql` are selectable without changing the task-facing host surface.
2. A runtime can make more than one model or capability call.
3. A runtime can continue even when a model requests no tool.
4. A runtime can stop for a positive semantic reason rather than only because no carrier work remains.
5. Runtime-specific state can exist without leaking into the common host contract.
6. Raw host/carrier events and semantic runtime events can both be observed and correlated.
7. Cancellation, failure and exhaustion are distinguishable from successful completion.

The common contract should not contain QL-only concepts such as `P3`, `R43`, `ConjugatePacket` or `ChildCircuitSummary`. Those belong inside the QL runtime and its optional semantic event payloads.

---

## 5. The classical control runtime

`ClassicRuntime` is the smallest faithful control for an ordinary agent loop over the same host ports.

Its purpose is experimental comparison, not product architecture.

A sufficient form is:

```text
receive request
    ↓
call model
    ↓
if tool/capability calls exist:
    execute them
    append results
    call model again
    ↓
if follow-up/steering input exists:
    incorporate it
    call model again
    ↓
otherwise complete with the model outcome
```

The exact control must respect the host being adapted. For Pi, for example, the classical condition should preserve Pi's actual upstream recurrence as closely as practical. For the minimal native host, the classical condition should remain deliberately boring.

The classical runtime MUST NOT acquire QL determination, position, relation, residue or closure semantics merely so both branches look structurally symmetrical.

---

## 6. The first QL runtime: Direct Core recurrence kernel

The first QL runtime proves that QL can govern recurrence through the same host boundary.

This foundation is drawn from the following normative areas of `ql-agent-spec-08-13-2026.md`:

```text
§2    QL changes recurrence itself
§3    P0–P5 and the 4+2 distinction
§4    L1 + L4′ initial refraction
§5    constitutive circulation without chronological coercion
§6    all 36 Rij relations are structurally representable
§8    π0/1 projection and ρ1/0 return
§9    carrier ontology remains distinct from QL function
§10   Circuit as QL runtime state
§11   Frame / P0
§12   six residue responsibilities
§13   QLAct
§14   Projection
§15   Return and required difference
§16   interpretation before semantic transition
§18   distinct success dimensions, at least as needed for direct Core
§19   explicit P5 determination
§20   closure evaluation
§21   distinct P5 reopening relations
§22   positive QLClosure
§23   retained ReentryDelta and P0+
§35   required semantic implementation interface
§36   reference loop
§37   forbidden simplifications
§61   Core conformance definition
```

The Direct Core runtime therefore requires a real QL circuit state and a real recurrence loop.

It MUST prove at minimum:

```text
Frame establishes P0 ground.

An act has a current QL responsibility.

Exterior encounters cross a π0/1 → carrier → ρ1/0 boundary.

The returned result becomes an interpreted difference.

The semantic destination is chosen from what the difference means, not from carrier identity.

Rij records the movement of the whole.

Residue persists independently enough from transcript to survive ordinary context handling.

P5 produces explicit determination.

P5 may reopen P0–P4 for distinct reasons.

Closure is positive and inspectable.

Closure produces retained difference for P0+.
```

The runtime MUST NOT enforce `P0 → P1 → P2 → P3 → P4 → P5` as six required chronological calls.

### 6.1 Lean first representation

The implementation should use the least elaborate data structures that preserve the normative semantics.

For example, the six residue responsibilities may initially be implemented through one tagged residue structure:

```text
Residue {
    id
    kind: frame | material | effect | form | evaluation | determination
    value
    provenance
}
```

provided the six semantic kinds remain distinguishable and every required operation remains representable.

Likewise, `Rij` should initially be a validated `(from, to)` relation rather than 36 separate classes or handlers.

The 4+2 distinction must remain semantically recoverable and enforceable, but the shared experiment does not require every implementation language to adopt the same in-memory algebraic type.

These choices are implementation economy, not semantic reduction.

---

## 7. Foundation observability

A/B comparison requires common run optics from the beginning.

The common observer layer should record a small framework-neutral envelope for every run, including:

```text
run id
host id
runtime id
runtime version/spec revision
task/fixture id
model identity and parameters where available
capability/tool surface identity
start/end status
model call count
tool/capability call count
elapsed time where measured
raw host event references
runtime-semantic event references
final artifact/outcome reference
```

For the QL runtime, semantic events MUST additionally be able to express the portable QL event model required by §§29–30 of the specification.

The foundation does not require every final event-specific schema, witness field, renderer or metric before the first runtime switch works. It does require enough stable event identity and ordering that later conformance and comparison work can build on the same trace instead of replacing it.

A run that cannot be replayed or compared must not be used as experimental evidence.

---

## 8. Foundation Freeze

The **Foundation Freeze** is an earlier engineering gate than the specification's final `QL Experiment Readiness` gate.

It exists only to unlock parallel work.

Foundation Freeze passes when:

```text
1. A host can select ClassicRuntime or QLDirectCoreRuntime per run.

2. The same reference task can run through both runtimes without changing the model/tool host surface.

3. Direct QL recurrence is not reducible to classic recurrence plus metadata.

4. QL destination follows interpreted return rather than carrier identity.

5. P5 determination and positive closure are distinct from tool absence/process exit.

6. P5 reopening and retained re-entry work in reference fixtures.

7. A minimal shared trace records host condition, runtime condition and QL semantic events.

8. A small deterministic fixture set proves the above without relying on subjective benchmark performance.

9. The runtime interface exposes stable extension seams for conjugation and child circuits without pretending those operators are already implemented.

10. The foundation revision is tagged or otherwise pinned so all parallel branches begin from the same substrate.
```

Foundation Freeze does **not** claim full QL Experiment Readiness.

It is the point at which the three harness experiments and the deep-QL path may proceed in parallel.

---

## 9. Parallel repository structure

The repository should materialise the experiment as one programme with shared foundation and separate experiment bodies.

Recommended structure:

```text
experiments/ql-runtime/
├── ql-agent-spec-08-13-2026.md
├── ql-agent-development-protocol-08-13-2026.md
│
├── foundation/
│   ├── runtime-contract/
│   ├── classic-runtime/
│   ├── ql-core-runtime/
│   ├── optics/
│   ├── fixtures/
│   └── README.md
│
├── experiments/
│   ├── pi/
│   │   ├── shared/
│   │   ├── classic/
│   │   ├── ql/
│   │   ├── runs/
│   │   └── README.md
│   │
│   ├── pydantic/
│   │   ├── shared/
│   │   ├── classic/
│   │   ├── ql/
│   │   ├── runs/
│   │   └── README.md
│   │
│   └── native/
│       ├── shared/
│       ├── classic/
│       ├── ql/
│       ├── runs/
│       └── README.md
│
├── deep-ql/
│   ├── conjugation/
│   ├── depth/
│   ├── conformance/
│   ├── typing-corpus/
│   ├── render/
│   ├── extensions/
│   └── research/
│
└── comparison/
    ├── task-corpus/
    ├── run-manifests/
    ├── comparator/
    └── reports/
```

This layout is a development surface, not a new semantic ontology.

A concrete implementation may use packages/workspaces appropriate to its language. The important invariant is that shared foundation, three experiment-specific hosts, deep-QL work and comparison artifacts remain independently attributable.

---

## 10. Branch protocol

The work should proceed as one development programme with parallel branches.

A practical branch shape is:

```text
main
  │
  └── ql/runtime-foundation
          │
          ├── ql/experiment-pi
          ├── ql/experiment-pydantic
          ├── ql/experiment-native
          └── ql/deep-runtime
```

The exact branch names may vary.

The invariant is:

1. the four parallel tracks fork from the same Foundation Freeze revision;
2. experiment branches may advance host integration without waiting for deep QL;
3. the deep-QL branch may evolve conjugation, recursive depth and richer conformance without destabilising the baseline A/B seam;
4. cross-cutting fixes to the runtime contract are first proven against the foundation and then propagated deliberately;
5. no experiment silently changes QL semantics only inside its own adapter;
6. no deep-QL change becomes part of the baseline comparison merely because it exists on the research branch.

The baseline runtime should remain reproducible while deep QL is under active development.

---

## 11. The three experiment tracks

### 11.1 Pi

The Pi experiment asks:

```text
What happens when the actual recurrent core of a mature loop-based agent is made runtime-selectable and a QL runtime inhabits that seam?
```

The Pi adapter should preserve upstream host facilities where useful while moving recurrence behind the shared Loop Runtime boundary.

Its `classic` condition should approximate the upstream Pi loop faithfully.

Its `ql` condition must satisfy the QL adapter requirements of specification §45.1 as the deep-QL features become available.

The Pi experiment MAY alter Pi's core loop substantially. Compatibility with upstream architecture is not itself the experiment.

### 11.2 Pydantic AI

The Pydantic experiment asks:

```text
Can the same QL recurrent semantics inhabit or modulate a graph/state execution environment without becoming six QL graph nodes?
```

The adapter should identify the cleanest boundary between framework execution mechanics and loop-runtime recurrence.

Its `classic` condition uses the framework's ordinary graph/state recurrence.

Its `ql` condition must document where QL semantics live across state, edges, events, effects or a composed runtime layer, as required by specification §45.2.

### 11.3 Native

The native experiment asks:

```text
What agent architecture emerges when QL is allowed to generate the recurrent architecture with the smallest necessary host beneath it?
```

It begins only from:

```text
QL
LLM interface
capability interface
environment
success conditions
transport
```

as required by specification §45.3.

Its `classic` condition is a deliberately minimal ordinary tool loop over the same host ports.

Its `ql` condition must not copy Pi or Pydantic architecture merely to achieve parity.

---

## 12. Deep QL parallel path

The deep-QL branch begins from Foundation Freeze and develops everything required for the **full** experimental QL profile that does not need to block initial host adaptation.

### 12.1 Conjugation

Implement specification §24 and the corresponding `QLC-J*` tests:

```text
Direct / Conjugate face identity
J: Pi → Pi′
K: P5 → P0′
Whole conjugation
CurrentPosition conjugation
fresh-context ConjugatePacket
ConjugateDelta
reopening of the direct circuit
```

The fresh-context boundary is part of the operator, not a same-context critic label.

### 12.2 Recursive depth

Implement specification §25 and `QLC-N*`:

```text
child CircuitId
parent relation
depth
independent child P0 Frame
ordinary child QL traces
typed ChildCircuitSummary
parent reintegration without requiring the child transcript
```

### 12.3 Conformance and semantic typing

Develop the remaining shared apparatus from §§29–44:

```text
portable QL event schemas
reference fixtures and negative fixtures
stable QLC identifiers
typing witnesses
human-reviewed typing corpus
agreement metrics
reference renderer
cross-runtime trace comparator
```

This work is essential before formal experimental evidence is claimed, but it should harden the already-working runtime rather than determine its architecture retroactively.

### 12.4 Research extensions

Research structures from §§26–28 and §§55–60 remain namespaced extensions until promoted by the specification's own criteria.

This includes:

```text
harmonic operators
64-state field
full MEF operations
context-frame deepening
epogdoon / retained-difference quantification
higher topological control
```

The deep-QL branch may investigate and implement these freely behind extension boundaries.

They do not enter the controlled baseline without explicit versioned promotion.

---

## 13. Convergence into full Experiment Readiness

The programme converges only after both sides have matured:

```text
three host/harness experiment adapters
              +
full required QL operator/conformance body
```

Each experiment must then satisfy the governing specification's §62 definition of `QL Experiment Readiness`:

```text
Core conformance passes.
Whole and current-position conjugation pass.
Fresh-context ConjugatePacket passes.
ql_depth / equivalent child circuit passes.
Child typed reintegration passes.
Reference renderer works.
Typing corpus can be replayed.
Cross-runtime trace comparator can ingest output.
All required QLC-* tests pass.
Manual semantic dry runs are intelligible.
No unresolved bug can silently restore ordinary tool-loop closure semantics.
```

This is the real pre-Series-1 readiness gate.

Foundation Freeze does not replace it.

---

## 14. The controlled experiment matrix

Once the three hosts expose the common runtime boundary, the basic experimental matrix becomes:

```text
                         LOOP RUNTIME
                     Classic        QL

Pi                     Pi-C          Pi-Q

Pydantic               Py-C          Py-Q

Native                 N-C           N-Q
```

This supports two important comparisons.

### Within-host comparison

```text
Pi-C  ↔ Pi-Q
Py-C  ↔ Py-Q
N-C   ↔ N-Q
```

This asks what changes when recurrence semantics change while the host is held as constant as possible.

### Across-host QL comparison

```text
Pi-Q ↔ Py-Q ↔ N-Q
```

This asks which QL properties survive differences in harness architecture.

The combination is stronger than comparing three QL implementations without classical controls because it separates QL effects from host effects.

---

## 15. Operator matrix

After the deep-QL operators are integrated, each QL condition can expose the specification's operator ablations:

```text
QL Direct Core
QL + depth
QL + conjugation
QL + depth + conjugation
```

Depth and conjugation should normally be agent/runtime-selectable capabilities rather than mandatory actions on every run.

Forced-use variants may exist for diagnosis but must not substitute for voluntary-use conditions.

---

## 16. Comparison discipline

Where practical, a matched run manifest should pin:

```text
task id
starting artifact state
model and model version
model parameters
tool/capability surface
external environment
success conditions
host revision
runtime revision
spec revision
operator profile
```

The run record must make deviations visible.

A/B testing should first ask whether the runtime conditions are semantically faithful and comparable. Performance interpretation comes after that.

The optics defined in specification §52 can then be derived from the shared traces, including:

```text
task/artifact/circuit success
model and tool calls
elapsed time
Rij transitions
position dwell
P5 reopening
child/depth use
conjugate use
typing disagreement
human intervention
trajectory motifs and recovery paths
```

---

## 17. Development stages

### Stage F0 — Specify and scaffold the runtime seam

Deliver:

```text
common RuntimeHost / LoopRuntime contract
runtime selector
experiment workspace layout
shared run manifest
extension seams
```

### Stage F1 — Implement both foundation runtimes

Develop in parallel:

```text
ClassicRuntime
QL Direct Core Runtime
```

### Stage F2 — Prove A/B switching and freeze foundation

Deliver:

```text
same-host classic/QL execution
minimal deterministic Core fixtures
matched trace envelope
reopening / closure / re-entry proof
foundation revision pin
```

### Stage E — Parallel host experiments

Begin concurrently:

```text
Pi adapter
Pydantic AI adapter
Native host
```

Each creates matched `classic` and `ql` surfaces and records runs under the shared manifest.

### Stage D — Parallel deep QL

Begin concurrently with Stage E:

```text
conjugation
recursive depth
full conformance fixtures/schemas
semantic typing corpus
renderer/comparator
research extensions
```

### Stage X — Integrate and converge

Bring the required deep-QL operator/conformance body into all three experiment branches and pass the same required profile.

### Stage S1 — Freeze Series 1

Pin:

```text
spec
schemas
runtime revisions
host revisions
models
tools
task corpus
typing protocol
evaluation/comparison harness
```

Only runs produced from the frozen profile count as formal Series 1 evidence.

---

## 18. Anti-overengineering rules

These rules protect the experiment from infrastructure becoming the project.

### Standardise the seam, not the world

Only standardise the interfaces needed to exchange Loop Runtime beneath different hosts.

Do not prematurely unify all framework context, message, graph, tool and session models.

### One semantic type before a hierarchy

Prefer one validated relation type over 36 relation classes.

Prefer one tagged residue container over six large implementation hierarchies where the normative distinction is preserved.

Prefer one portable event envelope with staged event-specific hardening over a large schema organisation before a run exists.

### Build semantics before dashboards

Trace identity and replayability are foundation requirements.

A polished renderer, large metrics surface and extensive annotation tooling are convergence requirements.

### Do not hide QL in prompts

QL recurrence belongs in runtime state and control.

Prompt-language support may help the model understand the active responsibility, but removing the prompt decorations must not erase the runtime's QL semantics.

### Do not make QL ceremonial

No mandatory six-step walk.

No mandatory conjugate pass.

No mandatory child circuit.

No automatic harmonic traversal.

Optional operators become meaningful when intelligence can invoke them because the situation warrants them.

### Do not defer required final semantics

Conjugation, depth, typing, renderer and the complete required conformance profile must still arrive before formal Series 1 freeze where the specification requires them.

Parallelisation changes schedule, not meaning.

---

## 19. Evidence gates

Every development ticket should complete with inspectable evidence appropriate to its level.

### Runtime seam evidence

```text
classic and QL runtimes execute through the same reference host
runtime selection is explicit in run manifest
host code does not branch on QL positions
```

### Direct Core evidence

```text
interpreted-return routing
same-carrier different QL destination
explicit determination
P5 reopening
positive closure
retained re-entry
no six-stage coercion
```

### Adapter evidence

```text
host-native classic condition remains faithful
QL condition uses the common runtime semantics
same task/tool/model surface can be run both ways
framework-specific events remain distinguishable from QL events
```

### Deep QL evidence

```text
operator fixtures pass
child/conjugate summaries do not depend on full transcript inheritance
witness disagreement is retained
portable traces validate and compare across hosts
```

### Final readiness evidence

```text
all three QL implementations pass the same required QLC profile
all three replay the typing corpus
each has inspectable run rendering
cross-runtime comparator ingests all conditions
manual dry runs remain semantically intelligible
```

---

## 20. Change protocol

The foundation runtime contract is intentionally small and may reveal missing requirements when applied to real harnesses.

When that happens:

1. record the concrete host pressure that exposed the missing distinction;
2. decide whether it belongs to the common host/runtime seam, one adapter, or QL semantics;
3. change the shared contract only when at least two hosts or a genuine architecture invariant require it;
4. change QL semantics only through the governing specification/versioning process;
5. add a fixture that proves why the change is necessary;
6. propagate the change deliberately to active branches.

Do not generalise a framework-specific convenience into the common runtime solely because it appears first.

---

## 21. Completion condition

This development protocol is fulfilled when:

```text
A harness-neutral Loop Runtime seam exists and is proven.

Classic and QL runtime conditions are selectable per run.

The Direct Core QL recurrence kernel works independently of any one harness.

Pi, Pydantic AI and Native experiment directories each contain matched classic/QL conditions.

The three experiment tracks can run and record comparable outputs in parallel.

Deep QL development can proceed independently from the same frozen foundation.

Conjugation, recursive depth and the required conformance/typing/rendering body converge back into every QL experiment.

All three QL conditions meet the same governing QL Experiment Readiness profile.

The six-cell experiment matrix is reproducible from pinned manifests.

Series 1 can be frozen without conflating host architecture, loop runtime and optional QL operator effects.
```

At that point the programme is no longer preparing to test a QL-native agent loop.

It has built the controlled system in which that experiment can genuinely be run.
