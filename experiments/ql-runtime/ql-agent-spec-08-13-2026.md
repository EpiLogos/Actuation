# **`ql-agent-spec`**

## **Normative Specification for a QL-Native Agent Runtime**

**Specification version:** `0.1.0-experimental`  
**Status:** Pre-experiment conformance specification**Intended implementations:** `ql-pi`, `ql-pydantic`, `ql-native`  
**Trace format:** JSON Lines (`application/x-ndjson`)  
**Canonical initial semantic refraction:** `L1 + L4′`  
**Canonical structural basis:** `0/1 → 4+2 → 6 → 6+6′ → contextual recursion → re-entry`

---

## **0. Normative language**

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL** are normative.

This specification distinguishes three statuses:

- **Core** — required for QL-agent conformance.
- **Operator** — part of the QL runtime vocabulary but not required to occur in every run.
- **Research** — preserved as part of the wider QL architecture but not yet allowed to constrain base runtime behaviour without a later specification revision.

An implementation MUST NOT promote a Research structure into a mandatory runtime rule while claiming plain `0.1.0` conformance. Such an implementation MUST declare an extension profile.

---

# **1. Scope**

This specification defines the semantic organisation of a **QL-native agent run** independently of:

- programming language;
- model provider;
- LLM API;
- tool protocol;
- message format;
- host application;
- CLI or GUI;
- operating system;
- graph or loop implementation;
- local or remote execution;
- single-model or multi-model operation.

It specifies:

1. the invariant `4+2` positional body;
2. QL position and lens distinction;
3. direct and conjugate faces;
4. `0/1 : 1/0` projection and return;
5. the 36 ordered intra-face relations;
6. typed acts and interpreted returns;
7. QL residue;
8. explicit determination;
9. positive closure;
10. reopening;
11. re-entry as retained difference;
12. recursive child circuits;
13. conjugate invocation;
14. success dimensions;
15. portable trace semantics;
16. typing witnesses;
17. conformance fixtures;
18. experiment-readiness tests.

The specification intentionally does **not** define a six-stage workflow.

The QL source distinguishes the canonical sixfold circulation from the actual route an intelligence may take through the responsibilities of the whole.  The runtime MUST preserve this distinction: actual traversal MAY revisit or move among positions as required without ceasing to belong to the canonical sixfold form.

---

# **2. Conformance target**

A runtime conforms to `QL-Agent Core 0.1` only when QL changes the **semantics of recurrence itself**.

The following implementation is explicitly non-conforming:

```text
ordinary agent loop
    +
metadata field "ql_position"
    +
prompt telling model what #0–#5 mean
```

The following general structure is conforming:

```text
current QL whole
      │
      │ typed intention
      ▼
   QL act
      │
      │ π 0/1
      ▼
external encounter
      │
      │ ρ 1/0
      ▼
returned difference Δ
      │
      ▼
interpretation
      │
      ▼
Rij : Pi → Pj
      │
      ▼
transformed QL whole
```

The runtime MUST permit QL semantics to affect:

- what current activity means;
- how an exterior return changes the run;
- whether a run remains open;
- what must be revisited;
- whether determination constitutes closure;
- what survives into subsequent ground.

A runtime which would behave identically after removing every QL-semantic operation from the core MUST NOT claim QL-Agent Core conformance.

---

# **3. QL position ontology**

## **3.1 Raw addresses**

The canonical position set is:

[  
P={P_0,P_1,P_2,P_3,P_4,P_5}.  
]

The canonical conjugate set is:

[  
P’={P_0’,P_1’,P_2’,P_3’,P_4’,P_5’}.  
]

A conforming runtime MUST treat these as **QL addresses prior to any single software or epistemic vocabulary**.

The source explicitly preserves the P-series as primary and treats semantic names as lens-mediated determinations.

Therefore:

```text
P3 ≠ "architecture"
P2 ≠ "tool call"
P1 ≠ "read"
P4 ≠ "test"
P5 ≠ "final response"
```

Those carriers or semantic descriptions MAY function at those positions in particular acts, but they do not define the positions.

---

## 

## 

## **3.2**

**`4+2`**

**structural distinction**

The runtime MUST represent the six positions as:

[  
\underbrace{P_1+P_2+P_3+P_4}_{\text{explicate}}_ _+_ _\underbrace{P_0+P_5}_{\text{implicate}}.  
]

The source defines the four middle positions as the explicated causal body and explicitly states that they are responsibilities of the whole rather than chronological stages.

Canonical abstract type:

```text
Position =
    Implicate(P0 | P5)
  | Explicate(P1 | P2 | P3 | P4)
```

An implementation MAY encode the position numerically for storage, but its type/API layer MUST retain the `Implicate | Explicate` distinction.

A plain `u8` or integer enum without any structural distinction is insufficient as the normative semantic representation.

---

# **4. Initial agentic refraction**

The base agent profile uses the paired lenses:

```text
L1  Causal
L4′ Scientific / Knowledge Work
```

The canonical `0.1` disclosure is:

|**Position**|**L1**|**L4′ / agent-run reading**|
|---|---|---|
|`P0`|causal ground / freedom|Prompt / operative ground|
|`P1`|Material Cause|material / Trace / evidence / givens|
|`P2`|Efficient Cause|Challenge / operation / transformation|
|`P3`|Formal Cause|Pattern / model / form|
|`P4`|Final Cause|Discovery / contextual adequacy / telos|
|`P5`|Will / determination|Insight / synthesis / realised intent|

The source establishes this paired reading directly.

A conforming runtime MUST identify this as a **refraction**, not as a renaming of raw QL positions.

Canonical type:

```text
LensRefraction {
    lenses: [LensId]
    position: Position
}
```

For Core `0.1`:

```text
default_lenses = ["L1", "L4′"]
```

An implementation MAY expose additional MEF lenses.

An implementation MUST NOT require other MEF lenses for basic Core conformance.

---

# **5. Canonical circulation**

The canonical sixfold articulation is:

[  
A:P_i\mapsto P_{i+1}  
]

with cyclic form:

[  
P_0  
\rightarrow P_1  
\rightarrow P_2  
\rightarrow P_3  
\rightarrow P_4  
\rightarrow P_5  
\rightarrow P_0^+.  
]

The runtime MUST preserve this as the **constitutive orientation of the whole**.

The runtime MUST NOT enforce it as the required chronological sequence of model calls, tool calls, or internal acts.

The following trajectory is conforming:

```text
P0 → P1 → P3 → P2 → P1 → P4 → P3 → P4 → P5
```

The following trajectory is also potentially conforming:

```text
P0 → P3 → P4 → P5
```

if no further material or effect work is required.

The following MUST NOT be treated as sufficient evidence of QL conformance:

```text
always:
P0 → P1 → P2 → P3 → P4 → P5
```

A runtime that forcibly schedules every run through this exact sequence is a six-stage workflow implementation, not yet a QL-agent implementation.

---

# **6. The ordered relation field**

## **6.1 Definition**

The canonical intra-face transition field is:

[  
R_{ij}:P_i\rightarrow P_j  
]

for all:

[  
i,j\in{0,1,2,3,4,5}.  
]

Therefore one face exposes:

[  
6^2=36  
]

ordered position relations.

The source explicitly defines this 36 relation field and includes deepening, revision, reopening, contextualisation and self-relations.

All 36 relations MUST be structurally representable.

An implementation MUST NOT globally prohibit:

```text
P4 → P1
P5 → P2
P3 → P1
P0 → P0
```

merely because they appear to move “backwards”.

---

## **6.2 Relation semantics**

A relation MUST be interpreted through the functional semantics of its source and destination.

Examples:

```text
R41 : P4 → P1
contextual encounter discloses missing or newly relevant material
```

```text
R43 : P4 → P3
contextual discovery requires revision of form
```

```text
R52 : P5 → P2
candidate determination fails because required transformation is absent
```

```text
R53 : P5 → P3
candidate determination fails because the realised form is inadequate
```

```text
R50 : P5 → P0
the work has altered the initiating ground itself
```

```text
Rii : Pi → Pi
remain with and deepen the current responsibility
```

The runtime MAY use semantic relation subtypes beyond `Rij`, but the ordered source/destination pair MUST remain recoverable in the portable trace.

---

# **7. Face**

Canonical face type:

```text
Face =
    Direct
  | Conjugate
```

The direct face corresponds to the primary `P` body.

The conjugate face corresponds to `P′`.

The direct face is initially interpreted agentically as predominantly:

```text
prospective
synthetic
outcome-forming
```

The conjugate face is initially interpreted as predominantly:

```text
retrospective
analytic
inverse
critical
```

These are operational readings, not exhaustive metaphysical definitions.

The runtime MUST preserve face identity in trace data.

The runtime MUST be capable of representing the conjugate face even when a particular run never invokes it.

---

# 

# 

# **8.**

**`0/1 : 1/0`**

**exchange**

## **8.1 Local boundary exchange**

Every semantically relevant exterior interaction MUST be representable as:

[  
P_i  
\overset{\pi^{0/1}}{\longrightarrow}  
E  
\overset{\rho^{1/0}}{\longrightarrow}  
\Delta  
]

where `E` is an exterior carrier or environment.

Possible `E` values include:

```text
LLM
tool
filesystem
process
test runner
web service
human
remote API
child circuit
specialist model
external evaluator
```

The source defines outward projection as `π^{0/1}` and return/reception as `ρ^{1/0}`.

Canonical type:

```text
ExchangePhase =
    Projection0_1
  | Return1_0
```

---

## **8.2 Position after return**

The destination position MUST NOT be statically determined by the carrier.

The runtime MUST permit:

```text
P3 → read() → P1
P3 → read() → P3
P3 → read() → P4
```

depending upon what the returned difference means for the whole.

The source states explicitly that the semantic destination is not predetermined by the carrier and places branching at the point of interpreted return.

Therefore this implementation is non-conforming:

```text
read tool     => P1
write tool    => P2
reasoning     => P3
test tool     => P4
final message => P5
```

Capabilities MAY advertise possible QL affordances, but actual invocation type MUST remain contextual.

---

# **9. Runtime carrier ontology**

The QL semantic system MUST remain distinct from runtime carriers.

Canonical carrier kinds:

```text
CarrierKind =
    Model
  | Tool
  | Human
  | Environment
  | ChildCircuit
  | InternalControl
  | Artifact
  | ExternalEvaluator
```

Implementations MAY extend this set.

A carrier MUST NOT determine position by itself.

Example:

```json
{
  "carrier": {
    "kind": "tool",
    "name": "read"
  },
  "ql": {
    "from": "P4",
    "to": "P1",
    "relation": "R41"
  }
}
```

The carrier describes **how the act crossed the boundary**.

QL describes **what function the act and its return played within the whole**.

---

# **10. Circuit**

## **10.1 Definition**

A **Circuit** is the minimal whole within which QL positions, residues, relations, closure and re-entry are meaningful.

Canonical interface:

```text
Circuit {
    id
    parent_id?
    depth

    face
    frame

    active_position

    residues
    trajectory

    closure_state
    success_state

    children
    conjugates
}
```

A circuit MUST have a unique stable `CircuitId`.

A child circuit MUST record its parent.

A direct and conjugate circuit MAY share a logical run identity while retaining distinct circuit identifiers.

---

## **10.2 Circuit state is not transcript state**

A runtime MAY maintain an ordinary transcript.

A transcript MUST NOT be the sole canonical QL state.

QL state MUST preserve typed residue and positional/relational history independently enough that context compaction can occur without destroying the QL whole.

---

# 

# **11. Frame /**

**`P0`**

## **11.1 Frame definition**

A circuit MUST begin from or inherit a `Frame`.

Canonical abstract type:

```text
Frame {
    id
    initiating_intent
    operative_scope
    constraints
    available_capabilities
    success_conditions
    inherited_delta?
    provenance
}
```

`Frame` is the primary Core residue of `P0`.

The `initiating_intent` MUST be explicitly representable.

An implementation MAY derive it from a user message, system request, parent circuit, automation, API invocation, prior re-entry, or another source.

---

## **11.2 Frame evolution**

A frame MAY be revised during a run.

A transition to `P0` MUST NOT automatically create a completely new circuit.

The runtime SHOULD distinguish:

```text
reconsider current ground
```

from:

```text
close current circuit and re-enter as P0+
```

A `P5 → P0` transition within an open circuit means that candidate determination has exposed a problem in or transformation of the initiating ground.

---

# **12. Residues**

## **12.1 Required residue classes**

Core `0.1` defines:

```text
P0 → FrameResidue
P1 → MaterialResidue
P2 → EffectResidue
P3 → FormResidue
P4 → EvaluationResidue
P5 → DeterminationResidue
```

Canonical union:

```text
Residue =
    FrameResidue
  | MaterialResidue
  | EffectResidue
  | FormResidue
  | EvaluationResidue
  | DeterminationResidue
```

Residues MUST be referencable by stable identifier.

A transition MAY create, revise, invalidate, qualify or link residues.

Residue provenance SHOULD be preserved.

---

## **12.2 Material residue**

Examples:

```text
source file
user statement
search result
test output
observed runtime state
API response
retrieved document
constraint
existing artifact
```

Material does not mean passive data only.

It means what is materially given or established for the run.

---

## **12.3 Effect residue**

Examples:

```text
file edit performed
process executed
API mutation performed
test run
artifact generated
external state changed
request issued
deployment performed
```

Effect residue SHOULD distinguish:

```text
intended effect
observed effect
verified effect
```

where the host runtime can do so.

---

## **12.4 Form residue**

Examples:

```text
candidate model
architecture
hypothesis
plan
structured representation
code form
mapping
interpretation
formal relation
```

Multiple competing forms MAY coexist.

---

## **12.5 Evaluation residue**

Examples:

```text
requirement comparison
test verdict
contextual relevance judgement
success-condition assessment
whole-relative adequacy assessment
risk judgement
discovery that changes telos
```

A mechanical verifier MAY contribute to `P4`, but `P4` is not reducible to mechanical testing.

---

## **12.6 Determination residue**

Examples:

```text
accepted synthesis
refusal
final answer
chosen implementation
decision to reopen
resolved intent
explicitly retained uncertainty
```

A Determination residue is not equivalent to closure.

---

# **13. QL Act**

Canonical abstract type:

```text
QLAct {
    id
    run_id
    circuit_id
    face

    source_position

    intent

    carrier
    input_residue_refs

    claimed_position?
    claimed_relation?

    timestamp?
}
```

A `QLAct` MUST identify its source position.

A `QLAct` SHOULD carry an explicit intent sufficiently precise to interpret its return.

A model MAY supply the claimed QL function.

The runtime MUST NOT blindly trust that claim as authoritative typing.

---

# **14. Projection**

A QL act crossing into an exterior operation creates a Projection event.

Canonical type:

```text
QLProjection {
    act_id
    phase: "0/1"
    carrier
    projected_context_refs
}
```

The runtime SHOULD expose only the context needed for the act rather than automatically forwarding the entire circuit transcript.

This permits later QL-native context construction.

---

# **15. Return**

Canonical type:

```text
QLReturn {
    id
    act_id
    phase: "1/0"

    raw_result_ref?
    observed_effect?
    difference

    operation_success
}
```

The `difference` field is REQUIRED.

It MAY be:

```text
structured object
text
artifact reference
semantic delta
error condition
environmental change
no-new-information marker
```

The runtime MUST distinguish:

```text
raw carrier result
```

from:

```text
interpreted QL difference
```

even when both are initially represented by the same text.

---

# **16. Interpretation and transition**

After return, the circuit MUST determine what the returned difference means for its whole.

Canonical type:

```text
QLInterpretation {
    return_id

    from_position
    destination_position

    relation

    rationale_ref?
    witness_refs

    residue_delta
}
```

Canonical relation identifier:

```text
R00 ... R55
```

The resulting transition is:

```text
QLTransition {
    id
    interpretation_id

    from
    to
    relation

    created_residue_refs
    revised_residue_refs
    invalidated_residue_refs

    witness_state
}
```

The relation MUST be selected after or as part of interpreting the return.

---

# **17. Typing witnesses**

## **17.1 Purpose**

During the experimental stage, QL typing MUST remain inspectable and fallible.

Three witness classes are defined.

### **Claimed witness**

The acting model declares:

```text
what responsibility it believes it is addressing;
what relation it expects.
```

### **Structural witness**

The harness records mechanically knowable facts:

```text
carrier invoked
files read
files modified
process status
model used
child created
parent relation
test result
side effects
```

### **Retrospective witness**

An independent classifier evaluates what semantic function the act actually served.

This MAY be:

```text
independent LLM call
deterministic fixture rule
human annotation
later learned classifier
formal verifier
```

---

## **17.2 Canonical witness type**

```text
TypingWitness {
    claimed_position?
    claimed_relation?

    structural_facts

    observed_position?
    observed_relation?

    confidence?
    ambiguity?
}
```

The runtime MUST NOT silently erase disagreement between witnesses.

A disagreement SHOULD be recorded as an experimental datum.

---

## **17.3 Ambiguous or multi-functional acts**

The first schema SHOULD support:

```text
primary_position
secondary_positions[]
```

rather than forcing every act to have exactly one semantic function when genuine multi-functionality is observed.

However, every transition MUST still choose one destination position as the circuit’s actual next active responsibility.

---

# **18. Success model**

Canonical success dimensions:

```text
SuccessState {
    operational?
    artifact?
    task?
    circuit?
    conjugate_stability?
}
```

Values MUST support:

```text
true
false
unknown
not_applicable
```

The runtime MUST NOT collapse these dimensions into one boolean.

---

## **18.1 Operational success**

Did a specific exterior operation execute successfully?

Example:

```text
shell command exited 0
```

---

## **18.2 Artifact success**

Does the produced artifact satisfy a mechanical or otherwise explicitly specified artifact verifier?

Example:

```text
test suite passes
schema validates
file exists
build succeeds
```

---

## **18.3 Task success**

Does the outcome satisfy the initiating intent and operative success conditions?

This is whole-relative and MAY require semantic judgement.

---

## **18.4 Circuit success**

Has the QL circuit positively closed?

This MUST be independently represented.

---

## **18.5 Conjugate stability**

If a conjugate pass is performed, does its returned delta:

```text
confirm
qualify
reopen
invalidate
```

the direct determination?

---

# 

# 

# **19.**

**`P5`**

**determination**

## **19.1 Explicit determination event**

A circuit MUST NOT terminate merely because no further tool call exists.

The agent MUST be able to express a candidate `P5` determination explicitly.

Canonical type:

```text
QLDetermination {
    id
    circuit_id

    synthesis
    intent_ref

    claimed_adequacy
    evidence_refs
    evaluation_refs

    unresolved_refs

    requested_outcome:
        close
      | reopen
      | conjugate
      | depth
}
```

The runtime MAY construct a determination on behalf of a model that cannot emit structured control, but the determination MUST still exist as an explicit runtime event.

---

# **20. Closure**

## **20.1 Closure relation**

Closure is primarily evaluated through:

[  
P_0\leftrightarrow P_4\leftrightarrow P_5.  
]

The source identifies this relation directly: `P0` contains initiating intent, `P4` determines whole-relative adequacy, and `P5` establishes what is actually accepted or realised.

A closure evaluator MUST therefore have access to:

```text
initiating Frame / P0
relevant Evaluation residue / P4
candidate Determination / P5
```

It MAY additionally inspect `P1–P3` residue.

---

## **20.2 Closure verdict**

Canonical type:

```text
ClosureVerdict {
    status:
        close
      | reopen

    reopening_relation?

    task_success

    rationale_ref?
    retained_delta_preview
}
```

A runtime MAY permit a third result:

```text
conjugate_before_close
```

as an extension, but the portable result MUST still eventually resolve to `close` or `reopen`.

---

# 

# **21. Reopening from**

**`P5`**

The following relations MUST be supported:

```text
R51 : P5 → P1
material insufficient or unsupported
```

```text
R52 : P5 → P2
required transformation absent, failed or unverified
```

```text
R53 : P5 → P3
form/model/implementation inadequate
```

```text
R54 : P5 → P4
criterion, contextual assessment or final relation unresolved
```

```text
R50 : P5 → P0
initiating intent/frame itself requires revision
```

These reopening relations are directly given in the source.

A runtime MUST NOT collapse all five into a generic:

```text
retry
```

The reopening reason MUST remain traceable through the destination position.

---

# **22. Positive closure event**

Canonical type:

```text
QLClosure {
    id
    circuit_id

    determination_ref
    frame_ref
    evaluation_refs

    success_state

    closed_at_position: "P5"

    reentry_delta_ref
}
```

A circuit SHALL be considered closed only after a `QLClosure` event.

Absence of pending work is not a closure event.

Process termination without `QLClosure` MUST be recorded separately, for example:

```text
aborted
interrupted
crashed
cancelled
exhausted
```

---

# **23. Re-entry**

## **23.1 Definition**

A closed circuit produces:

[  
P_5^{(n)}  
\overset{\Delta}{\longrightarrow}  
P_0^{(n+1)}  
]

such that:

[  
P_0^+=P_0+\Delta.  
]

The source defines this as retained difference rather than identical return.

---

## **23.2 Re-entry delta**

Canonical type:

```text
ReentryDelta {
    id
    source_circuit

    achieved_artifact_refs
    established_material_refs
    retained_form_refs
    changed_assumptions

    unresolved_refs
    revised_success_conditions

    new_capabilities?
    discovered_risks?
    opened_questions?

    provenance
}
```

The delta MAY be empty for a trivial closed act, but the runtime MUST still be capable of representing it.

---

## **23.3 New ground**

Canonical type:

```text
QLReentry {
    prior_circuit
    delta_ref
    renewed_frame
}
```

A fresh circuit created from re-entry MUST reference the delta that conditions its new frame.

A generic transcript restart MUST NOT be reported as QL re-entry.

---

# **24. Conjugation**

## **24.1 Required structural operations**

The runtime MUST be able to represent:

[  
J:P_i\rightarrow P_i’  
]

and:

[  
K:P_5\rightarrow P_0’.  
]

The source allows whole-pass, same-position and selective conjugation.

---

## **24.2 Conjugation scopes**

Core operator API:

```text
ConjugateScope =
    Whole
  | CurrentPosition
  | Position(Pi)
  | Relation(Rij)
```

Concrete implementations MAY expose fewer scopes initially, but Core Experiment Readiness requires at least:

```text
Whole
CurrentPosition
```

---

## **24.3 Fresh context protocol**

A conjugate circuit SHOULD begin from a deliberately reconstructed frame rather than inheriting the direct circuit’s complete conversational path.

Canonical conjugate input:

```text
ConjugatePacket {
    direct_circuit_ref

    intent_packet
    outcome_packet

    selected_material_refs
    selected_effect_refs
    selected_form_refs
    selected_evaluation_refs

    success_conditions

    requested_scope
}
```

The conjugate process SHOULD NOT automatically receive:

```text
every direct reasoning turn
every model rationale
the full persuasive narrative
```

unless an explicit experimental variant requires it.

---

## **24.4 Conjugate delta**

Canonical output:

```text
ConjugateDelta {
    status:
        confirm
      | qualify
      | reopen
      | invalidate

    discrepancy_type?
    target_position?
    target_relation?

    evidence_refs
    analysis_ref

    recommended_reopening_relation?
}
```

A conjugate delta that reopens the direct whole MUST be mapped back to a QL relation such as:

```text
P5 → P1
P5 → P2
P5 → P3
P5 → P4
P5 → P0
```

or another semantically justified `Rij`.

---

# **25. Recursive depth**

## **25.1 Definition**

A nested circuit is a local relation treated as sufficiently whole to warrant its own QL articulation.

The source locates the canonical recursive aperture around `P4` and develops the fuller `CF5 → nested sixfold → CF6 → CF7` relation.

Core `0.1` MUST preserve at least:

```text
parent circuit
parent P4/context relation
child circuit
typed child return
parent reintegration
```

---

## **25.2 Depth request**

Canonical operator input:

```text
DepthRequest {
    parent_circuit
    parent_position
    local_whole_intent

    selected_residue_refs
    success_conditions

    executor_preference?
}
```

Core `0.1` SHOULD require `parent_position = P4` unless an implementation declares an experimental extension permitting depth from other positions.

---

## **25.3 Child circuit**

A child MUST:

```text
have its own CircuitId;
record ParentCircuitId;
record depth = parent.depth + 1;
establish its own P0 Frame;
emit ordinary QL traces;
positively close or fail independently.
```

---

## **25.4 Child return**

Canonical type:

```text
ChildCircuitSummary {
    child_circuit
    parent_circuit

    child_intent
    determination_ref

    relevant_residue_refs
    success_state

    returned_delta
}
```

A parent MUST NOT require the entire child transcript to reintegrate the child result.

The full transcript MAY remain available as evidence.

The semantic return MUST be expressible through the typed summary.

---

# **26. Harmonic relations**

The following three dyadic systems are part of the canonical QL topology:

[  
H_D={(0,1),(2,3),(4,5)}  
]

[  
H_O={(1,2),(3,4),(5,0)}  
]

[  
H_{ABC}={(0,5),(1,4),(2,3)}.  
]

The source distinguishes all three systems and gives their agentic significance.

For Core `0.1`:

```text
harmonic relations are normative topology
automatic harmonic scheduling is NOT normative behaviour
```

Implementations SHOULD make harmonic relationships queryable.

Suggested operator:

```text
inspect_harmonic(
    family: D | O | ABC,
    pair: 1 | 2 | 3
)
```

An implementation MUST NOT force every run to traverse all harmonic pairs.

---

# **27. MEF refraction**

The full MEF is part of the wider architecture but is not mandatory per-event state in Core `0.1`.

The runtime SHOULD permit a future operation:

```text
refract(
    lens,
    position
)
```

without changing the raw position.

The distinction:

```text
QL position
≠
MEF lens
≠
context frame
```

MUST be preserved.

The source explicitly distinguishes MEF lens modulation from context-frame modulation.

---

# **28. Research-status structures**

The following structures are preserved in `ql-agent-spec` but MUST remain disabled as mandatory control logic under Core `0.1`:

```text
2^6 = 64 runtime-state reduction
64 = 4^3 harmonic state factorisation
binary occupancy of all six positions
automatic Catuṣkoṭi state assignment
12 × 6 = 72 complete MEF runtime state
12 × 7 lens/context state product
literal 9/8 epogdoon runtime metric
higher-genus agent control
homotopy-based control decisions
```

The source establishes the 64-state harmonic factorisations, but the agent implementation has not yet defined the computational meaning of a binary state at each position.

Promotion into Core requires a later specification change.

---

# **29. Event model**

Every conforming implementation MUST emit semantic events equivalent to:

```text
run_started
circuit_started

frame_established

act_created
projection
return_received
return_interpreted
transition

residue_created
residue_revised
residue_invalidated

determination_proposed
closure_evaluated
circuit_reopened
circuit_closed

conjugate_started
conjugate_completed

child_started
child_completed
child_reintegrated

reentry_created

run_completed
run_failed
run_cancelled
```

Framework-specific events MAY additionally be emitted.

Portable QL events MUST NOT be replaced by framework events such as:

```text
assistant_message
tool_call
graph_node_entered
```

although those MAY be linked as carrier metadata.

---

# **30. Canonical JSONL event envelope**

Every portable event MUST use this envelope:

```json
{
  "spec": "ql-agent/0.1",
  "schema_version": "0.1.0",

  "event_id": "evt_...",
  "event_type": "transition",

  "run_id": "run_...",
  "circuit_id": "circuit_...",
  "parent_circuit_id": null,

  "sequence": 17,

  "face": "direct",

  "ql": {},

  "payload": {},

  "witness": {},

  "timestamp": "2026-08-13T08:54:00+01:00"
}
```

`timestamp` MAY be omitted in deterministic test fixtures.

`sequence` MUST monotonically increase within one circuit.

---

# **31. Core JSON Schema**

The canonical schema language is JSON Schema Draft 2020-12.

The initial schema SHOULD be maintained as separate source files, but the normative structure is:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://epi-logos.dev/schemas/ql-agent/0.1/event.schema.json",
  "title": "QL Agent Event",
  "type": "object",
  "required": [
    "spec",
    "schema_version",
    "event_id",
    "event_type",
    "run_id",
    "circuit_id",
    "sequence",
    "face",
    "ql",
    "payload",
    "witness"
  ],
  "properties": {
    "spec": {
      "const": "ql-agent/0.1"
    },
    "schema_version": {
      "type": "string"
    },
    "event_id": {
      "type": "string",
      "minLength": 1
    },
    "event_type": {
      "enum": [
        "run_started",
        "circuit_started",
        "frame_established",
        "act_created",
        "projection",
        "return_received",
        "return_interpreted",
        "transition",
        "residue_created",
        "residue_revised",
        "residue_invalidated",
        "determination_proposed",
        "closure_evaluated",
        "circuit_reopened",
        "circuit_closed",
        "conjugate_started",
        "conjugate_completed",
        "child_started",
        "child_completed",
        "child_reintegrated",
        "reentry_created",
        "run_completed",
        "run_failed",
        "run_cancelled"
      ]
    },
    "run_id": {
      "type": "string",
      "minLength": 1
    },
    "circuit_id": {
      "type": "string",
      "minLength": 1
    },
    "parent_circuit_id": {
      "type": ["string", "null"]
    },
    "sequence": {
      "type": "integer",
      "minimum": 0
    },
    "face": {
      "enum": ["direct", "conjugate"]
    },
    "ql": {
      "$ref": "#/$defs/ql"
    },
    "payload": {
      "type": "object"
    },
    "witness": {
      "$ref": "#/$defs/witness"
    },
    "timestamp": {
      "type": "string",
      "format": "date-time"
    }
  },
  "$defs": {
    "position": {
      "enum": ["P0", "P1", "P2", "P3", "P4", "P5"]
    },
    "relation": {
      "type": "string",
      "pattern": "^R[0-5][0-5]$"
    },
    "ql": {
      "type": "object",
      "properties": {
        "from": {
          "$ref": "#/$defs/position"
        },
        "to": {
          "$ref": "#/$defs/position"
        },
        "relation": {
          "$ref": "#/$defs/relation"
        },
        "projection": {
          "enum": ["0/1"]
        },
        "return": {
          "enum": ["1/0"]
        },
        "lens": {
          "type": "array",
          "items": {
            "type": "string"
          }
        }
      }
    },
    "witness": {
      "type": "object",
      "properties": {
        "claimed_position": {
          "$ref": "#/$defs/position"
        },
        "claimed_relation": {
          "$ref": "#/$defs/relation"
        },
        "observed_position": {
          "$ref": "#/$defs/position"
        },
        "observed_relation": {
          "$ref": "#/$defs/relation"
        },
        "confidence": {
          "type": "number",
          "minimum": 0,
          "maximum": 1
        },
        "ambiguity": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/position"
          }
        },
        "structural_facts": {
          "type": "object"
        }
      }
    }
  }
}
```

This envelope schema is intentionally permissive about event-specific payloads.

The repository MUST additionally define event-specific schemas.

---

# **32. Determination schema**

```json
{
  "$id": "https://epi-logos.dev/schemas/ql-agent/0.1/determination.schema.json",
  "title": "QL Determination",
  "type": "object",
  "required": [
    "determination_id",
    "circuit_id",
    "synthesis",
    "intent_ref",
    "evidence_refs",
    "evaluation_refs",
    "unresolved_refs"
  ],
  "properties": {
    "determination_id": {
      "type": "string"
    },
    "circuit_id": {
      "type": "string"
    },
    "synthesis": {},
    "intent_ref": {
      "type": "string"
    },
    "evidence_refs": {
      "type": "array",
      "items": {
        "type": "string"
      }
    },
    "evaluation_refs": {
      "type": "array",
      "items": {
        "type": "string"
      }
    },
    "unresolved_refs": {
      "type": "array",
      "items": {
        "type": "string"
      }
    },
    "requested_outcome": {
      "enum": [
        "close",
        "reopen",
        "conjugate",
        "depth"
      ]
    }
  }
}
```

---

# **33. Re-entry schema**

```json
{
  "$id": "https://epi-logos.dev/schemas/ql-agent/0.1/reentry.schema.json",
  "title": "QL Re-entry Delta",
  "type": "object",
  "required": [
    "delta_id",
    "source_circuit",
    "achieved_artifact_refs",
    "established_material_refs",
    "changed_assumptions",
    "unresolved_refs",
    "provenance"
  ],
  "properties": {
    "delta_id": {
      "type": "string"
    },
    "source_circuit": {
      "type": "string"
    },
    "achieved_artifact_refs": {
      "type": "array",
      "items": {
        "type": "string"
      }
    },
    "established_material_refs": {
      "type": "array",
      "items": {
        "type": "string"
      }
    },
    "retained_form_refs": {
      "type": "array",
      "items": {
        "type": "string"
      }
    },
    "changed_assumptions": {
      "type": "array"
    },
    "unresolved_refs": {
      "type": "array",
      "items": {
        "type": "string"
      }
    },
    "revised_success_conditions": {
      "type": "array"
    },
    "new_capabilities": {
      "type": "array"
    },
    "discovered_risks": {
      "type": "array"
    },
    "opened_questions": {
      "type": "array"
    },
    "provenance": {
      "type": "object"
    }
  }
}
```

---

# **34. Conjugate schema**

```json
{
  "$id": "https://epi-logos.dev/schemas/ql-agent/0.1/conjugate.schema.json",
  "title": "QL Conjugate Packet",
  "type": "object",
  "required": [
    "direct_circuit_ref",
    "scope",
    "intent_packet",
    "outcome_packet",
    "success_conditions"
  ],
  "properties": {
    "direct_circuit_ref": {
      "type": "string"
    },
    "scope": {
      "oneOf": [
        {
          "const": "whole"
        },
        {
          "const": "current_position"
        },
        {
          "type": "object",
          "required": ["position"],
          "properties": {
            "position": {
              "enum": ["P0", "P1", "P2", "P3", "P4", "P5"]
            }
          }
        },
        {
          "type": "object",
          "required": ["relation"],
          "properties": {
            "relation": {
              "type": "string",
              "pattern": "^R[0-5][0-5]$"
            }
          }
        }
      ]
    },
    "intent_packet": {},
    "outcome_packet": {},
    "material_refs": {
      "type": "array"
    },
    "effect_refs": {
      "type": "array"
    },
    "form_refs": {
      "type": "array"
    },
    "evaluation_refs": {
      "type": "array"
    },
    "success_conditions": {}
  }
}
```

---

# **35. Required implementation interface**

Every implementation MUST expose semantic equivalents of:

```text
start_circuit(frame) -> CircuitId

current_state(circuit) -> CircuitState

project(act) -> Projection

absorb(return) -> Difference

interpret(
    circuit,
    difference
) -> QLInterpretation

apply_transition(
    circuit,
    transition
)

propose_determination(
    circuit,
    determination
)

evaluate_closure(
    circuit,
    determination
) -> ClosureVerdict

close(
    circuit,
    verdict
) -> QLClosure

reenter(
    closure
) -> QLReentry
```

Experiment-readiness profile additionally requires:

```text
open_conjugate(...)
open_child(...)
```

Concrete method names MAY differ.

The semantics MUST not.

---

# **36. Reference loop**

A conforming implementation MUST be semantically equivalent to:

```text
circuit = start_circuit(frame)

while circuit is open:

    position = circuit.active_position

    act = form_next_act(
        circuit,
        position
    )

    emit(act)

    projection = project_0_1(act)

    raw_result = execute_exterior(projection)

    returned = absorb_1_0(raw_result)

    difference = establish_difference(returned)

    interpretation = interpret(
        circuit,
        difference
    )

    transition = resolve_Rij(
        position,
        interpretation.destination
    )

    apply_transition(
        circuit,
        transition
    )

    if circuit has candidate P5 determination:

        verdict = evaluate_closure(
            circuit,
            determination
        )

        if verdict is reopen:
            apply_transition(
                circuit,
                verdict.reopening_relation
            )

        else:
            closure = close(
                circuit,
                verdict
            )

            break

delta = produce_reentry_delta(closure)
```

This pseudocode does not prescribe how `form_next_act()` is implemented.

An LLM MAY choose the next act.

A deterministic policy MAY choose it.

A user MAY choose it.

A child circuit MAY return it.

Agency concerns the traversal of the typed whole rather than the identity of the chooser alone.

---

# **37. Forbidden simplifications**

A Core-conforming implementation MUST NOT define QL as any of the following:

```text
six prompt headings
six mandatory reasoning turns
one QL position per tool
one QL position per message role
one QL position per graph node
one Boolean "ql_complete"
one critic call labelled "Pratibimba"
one subagent call labelled "#4"
tool absence as #5
process exit as closure
transcript summary as re-entry
```

Any of these MAY appear as a carrier-level implementation detail only if the deeper Core semantics remain intact.

---

# **38. Reference fixture format**

Fixtures SHALL use:

```yaml
id: QLF-###
title: Human-readable fixture title

initial:
  face: direct
  position: P#
  frame: ...

act:
  intent: ...
  carrier: ...
  claimed_position: ...

return:
  raw: ...
  difference: ...

expected:
  relation: R##
  destination: P#
  residue:
    create: [...]
    revise: [...]
  closure:
    state: open|closed
```

---

# **39. Canonical transition fixtures**

## **QLF-001 — Material acquisition deepens material**

```yaml
id: QLF-001
title: Material acquisition remains at P1

initial:
  face: direct
  position: P1
  frame:
    intent: Repair failing parser

act:
  intent: Inspect the parser source
  carrier:
    kind: tool
    name: read
  claimed_position: P1

return:
  raw: Source file contents
  difference: Relevant parser implementation acquired

expected:
  relation: R11
  destination: P1
  residue:
    create:
      - MaterialResidue
  closure:
    state: open
```

---

## **QLF-002 — Material discloses form**

```yaml
id: QLF-002
title: Source inspection produces a formal hypothesis

initial:
  face: direct
  position: P1

act:
  intent: Determine how parser states are represented
  carrier:
    kind: tool
    name: read

return:
  raw: Source definition
  difference: Parser state is encoded as a recursive tagged union

expected:
  relation: R13
  destination: P3
  residue:
    create:
      - MaterialResidue
      - FormResidue
  closure:
    state: open
```

---

## **QLF-003 — Effect exposes form defect**

```yaml
id: QLF-003
title: Failed execution reopens formal work

initial:
  face: direct
  position: P2

act:
  intent: Execute the implementation
  carrier:
    kind: tool
    name: test

return:
  raw: Type mismatch failure
  difference: Existing representation cannot express required recursive state

expected:
  relation: R23
  destination: P3
  residue:
    create:
      - EffectResidue
      - FormResidue
  closure:
    state: open
```

---

## **QLF-004 — Form requires new material**

```yaml
id: QLF-004
title: Formal work reveals missing evidence

initial:
  face: direct
  position: P3

act:
  intent: Construct a compatible implementation form
  carrier:
    kind: model

return:
  raw: Candidate architecture plus explicit uncertainty
  difference: Public interface contract has not been inspected

expected:
  relation: R31
  destination: P1
  closure:
    state: open
```

---

## **QLF-005 — Context invalidates form**

```yaml
id: QLF-005
title: Requirement comparison reopens form

initial:
  face: direct
  position: P4

act:
  intent: Compare implementation against required compatibility
  carrier:
    kind: tool
    name: read

return:
  raw: Requirement text
  difference: Proposed architecture breaks required public API

expected:
  relation: R43
  destination: P3
  residue:
    create:
      - EvaluationResidue
  closure:
    state: open
```

---

## **QLF-006 — Determination reopens material**

```yaml
id: QLF-006
title: Unsupported synthesis reopens P1

initial:
  face: direct
  position: P5

act:
  intent: Determine whether the task is complete
  carrier:
    kind: internal_control

return:
  raw: Candidate completion
  difference: No evidence exists that Windows behaviour was inspected

expected:
  relation: R51
  destination: P1
  closure:
    state: open
```

---

## **QLF-007 — Determination reopens effect**

```yaml
id: QLF-007
title: Unperformed transformation reopens P2

initial:
  face: direct
  position: P5

return:
  difference: Correct patch exists in memory but has not been written

expected:
  relation: R52
  destination: P2
  closure:
    state: open
```

---

## **QLF-008 — Determination reopens form**

```yaml
id: QLF-008
title: Inadequate implementation shape reopens P3

initial:
  face: direct
  position: P5

return:
  difference: Tests pass but implementation violates required abstraction boundary

expected:
  relation: R53
  destination: P3
  closure:
    state: open
```

---

## **QLF-009 — Determination reopens finality**

```yaml
id: QLF-009
title: Unclear success criterion reopens P4

initial:
  face: direct
  position: P5

return:
  difference: Two interpretations of the requirement remain unresolved

expected:
  relation: R54
  destination: P4
  closure:
    state: open
```

---

## **QLF-010 — Determination revises ground**

```yaml
id: QLF-010
title: Discovery changes the initiating problem

initial:
  face: direct
  position: P5

return:
  difference: Reported parser bug is actually malformed upstream input

expected:
  relation: R50
  destination: P0
  closure:
    state: open
```

---

## **QLF-011 — Positive closure**

```yaml
id: QLF-011
title: P0-P4-P5 coherence closes the circuit

initial:
  face: direct
  position: P5

return:
  difference: >
    Required parser behaviour is implemented,
    tests verify the expected behaviour,
    public compatibility is preserved,
    no specified requirement remains unresolved.

expected:
  destination: P5
  closure:
    state: closed
  reentry:
    required: true
```

---

## **QLF-012 — Same tool, different type**

This fixture SHALL be a grouped test containing at least three `read` invocations:

```text
read as P1 material acquisition
read as P3 formal interrogation
read as P4 whole-relative verification
```

The implementation MUST demonstrate that the same tool name does not statically determine position.

---

## **QLF-013 — Tool-free closure**

A purely conceptual task with no tool calls SHALL be capable of:

```text
P0 → P3 → P4 → P5 → closure
```

A runtime failing this fixture because “no tool was called” is non-conforming.

---

## **QLF-014 — Tool-rich non-closure**

A run SHALL execute several successful tools while failing to satisfy the initiating intent.

It MUST remain open or fail explicitly.

Successful tool execution MUST NOT imply circuit closure.

---

## **QLF-015 — Child circuit**

```yaml
id: QLF-015
title: P4 opens a nested local whole

initial:
  face: direct
  position: P4

act:
  intent: Determine whether migration compatibility is itself resolvable
  carrier:
    kind: child_circuit

expected:
  child:
    required: true
    parent_position: P4
    independent_frame: true
    typed_return: true
  closure:
    state: open
```

---

## **QLF-016 — Fresh-context conjugation**

A direct circuit SHALL produce:

```text
IntentPacket
OutcomePacket
selected QL residue
SuccessConditions
```

A conjugate circuit SHALL be created without automatically inheriting the complete direct transcript.

The fixture contains a known formal defect.

Expected result:

```text
conjugate detects defect
ConjugateDelta.status = reopen
target = P3
```

---

## **QLF-017 — Re-entry**

A completed circuit SHALL produce:

```text
artifact A
new evidence E
unresolved question Q
```

The renewed `P0+` frame MUST preserve A/E/Q as typed retained difference.

The new circuit MUST NOT require the complete prior transcript to know them.

---

# **40. Negative fixtures**

## **QLN-001 — No-tool automatic stop**

Given:

```text
model emits a candidate answer
no tools are requested
closure has not been evaluated
```

Expected:

```text
MUST NOT emit circuit_closed
```

---

## **QLN-002 — Carrier-fixed typing**

Given:

```text
read() invoked for whole-relative verification
```

Expected:

```text
runtime MUST NOT force P1 solely because tool == read
```

---

## **QLN-003 — P5 equals stop**

Given:

```text
P5 determination lacks required evidence
```

Expected:

```text
MUST reopen
MUST NOT treat P5 itself as successful termination
```

---

## **QLN-004 — Restart reported as re-entry**

Given:

```text
new run starts with generic summary
no retained delta or provenance
```

Expected:

```text
MUST NOT label event reentry_created
```

---

## **QLN-005 — Child transcript dependency**

Given:

```text
child returns valid typed summary
child transcript withheld
```

Expected:

```text
parent MUST remain capable of reintegration
```

---

## **QLN-006 — Six-step coercion**

Given a task whose adequate trajectory is:

```text
P0 → P3 → P4 → P5
```

Expected:

```text
runtime MUST NOT manufacture fake P1/P2 acts solely to satisfy sequence
```

---

# **41. Conformance test IDs**

The automated suite SHALL expose stable test identifiers.

### **Structural**

```text
QLC-S001  4+2 type distinction exists
QLC-S002  P0-P5 classified implicate
QLC-S003  P1-P4 classified explicate
QLC-S004  direct and conjugate faces representable
QLC-S005  all R00-R55 relations representable
```

### **Exchange**

```text
QLC-E001  projection emitted as 0/1
QLC-E002  return emitted as 1/0
QLC-E003  carrier does not statically determine destination
QLC-E004  interpreted return precedes semantic transition
QLC-E005  same carrier supports different QL functions
```

### **Determination and closure**

```text
QLC-C001  P5 determination is explicit
QLC-C002  no-tool condition does not auto-close
QLC-C003  P5→P1 reopening works
QLC-C004  P5→P2 reopening works
QLC-C005  P5→P3 reopening works
QLC-C006  P5→P4 reopening works
QLC-C007  P5→P0 reopening works
QLC-C008  positive closure emits QLClosure
QLC-C009  process interruption is distinct from closure
QLC-C010  P0/P4/P5 evidence is inspectable at closure
```

### **Residue**

```text
QLC-R001  all six Core residue classes exist
QLC-R002  residue has stable identifiers
QLC-R003  residue provenance preserved
QLC-R004  transition can create residue
QLC-R005  transition can revise residue
QLC-R006  transition can invalidate residue
```

### **Success**

```text
QLC-U001  operational success distinct from circuit closure
QLC-U002  artifact success distinct from task success
QLC-U003  success supports unknown/not-applicable
```

### **Re-entry**

```text
QLC-G001  closure creates ReentryDelta
QLC-G002  renewed P0 references retained delta
QLC-G003  restart without delta cannot masquerade as re-entry
QLC-G004  unresolved residue survives re-entry
QLC-G005  irrelevant transcript history need not survive re-entry
```

### **Conjugation**

```text
QLC-J001  conjugate face representable
QLC-J002  J:Pi→Pi′ representable
QLC-J003  K:P5→P0′ representable
QLC-J004  whole conjugation supported
QLC-J005  current-position conjugation supported
QLC-J006  conjugate fresh-context packet supported
QLC-J007  conjugate delta can reopen direct circuit
```

### **Recursive depth**

```text
QLC-N001  child has independent CircuitId
QLC-N002  child records parent
QLC-N003  child records depth
QLC-N004  child establishes independent P0
QLC-N005  child emits ordinary QL traces
QLC-N006  child returns typed summary
QLC-N007  parent reintegrates without child transcript
```

### **Trace**

```text
QLC-T001  JSONL envelope validates
QLC-T002  sequence monotonic per circuit
QLC-T003  framework event and QL event remain distinguishable
QLC-T004  relation source/destination recoverable
QLC-T005  witness disagreement preserved
QLC-T006  carrier metadata preserved
QLC-T007  every semantic transition has stable event ID
```

### **Anti-collapse**

```text
QLC-A001  no mandatory six-step traversal
QLC-A002  tool types do not equal QL positions
QLC-A003  messages do not equal QL positions
QLC-A004  graph nodes do not inherently equal QL positions
QLC-A005  critic call alone does not constitute conjugation
QLC-A006  subagent call alone does not constitute QL depth
```

All tests prefixed `QLC-` are REQUIRED for **Core Experiment Readiness** unless explicitly marked by the implementation profile as unsupported before that profile enters experimental comparison.

The three primary experiments MUST pass the same required set.

---

# **42. Typing validation suite**

Before behavioural benchmarking, the shared repository SHALL contain a human-reviewed semantic typing corpus.

Minimum initial corpus:

```text
100 typed acts
```

with at least:

```text
12 examples centred on each explicate position;
8 examples centred on each implicate position;
12 intentionally ambiguous/multi-functional examples;
10 P5 reopening examples;
10 cross-carrier same-function examples;
10 same-carrier cross-function examples.
```

Overlap between categories is permitted.

Each fixture SHALL store:

```text
human reference type
model-claimed type
retrospective type
structural facts
ambiguity notes
```

Experiment Series 1 SHALL NOT begin until all three implementations can replay the same corpus.

---

# **43. Agreement metrics**

The test harness SHALL report:

```text
position exact agreement
relation exact agreement
face agreement
primary-vs-secondary position agreement
closure/reopen agreement
reopening-destination agreement
```

It SHOULD separately report:

```text
claimed ↔ human
retrospective ↔ human
claimed ↔ retrospective
implementation ↔ implementation
```

A single scalar “QL accuracy” SHOULD NOT replace the component metrics.

---

# **44. Runtime visualisation contract**

Every experimental implementation MUST provide a human-inspectable live or replay representation containing at least:

```text
RunId
CircuitId
parent/depth
face
active position
most recent Rij
current π/ρ exchange if active
closure state
child count
conjugate state
```

Recommended expanded view:

```text
QL  run_12 / C0.1  depth=1
FACE  DIRECT

ACTIVE
P3  Formal Cause / Pattern

RELATION
R31 → P1

EXCHANGE
π 0/1  read(src/runtime.ts)
ρ 1/0  interface mismatch

RESIDUE
+ Material M18
~ Form F07

CLOSURE
P0 ✓
P4 ○
P5 ○

CHILDREN      0
CONJUGATE     available
```

`ql-pi` SHOULD implement this as a live Pi widget.

`ql-pydantic` and `ql-native` MAY use a TUI, event viewer, terminal renderer, or equivalent.

---

# **45. Experiment adapters**

## 

## **45.1**

**`ql-pi`**

The Pi implementation MUST:

```text
make QLCircuitState part of actual run state;
replace tool-call absence as the semantic definition of completion;
wrap LLM and tool exchange in π/ρ events;
interpret return before choosing semantic destination;
support explicit determination and positive closure;
support reopening and re-entry;
emit portable traces;
provide ql_depth;
provide ql_conjugate;
provide active-state widget.
```

It MUST be permissible for the Pi fork to alter the core loop substantially.

Upstream architectural compatibility is not a conformance requirement.

---

## 

## **45.2**

**`ql-pydantic`**

The Pydantic implementation MUST NOT simply map:

```text
P0 → graph node 0
P1 → graph node 1
...
```

as its sole QL mechanism.

It SHOULD treat QL as a semantic relation field crossing or modulating the framework execution graph.

The experiment MUST document explicitly whether QL semantics are represented primarily through:

```text
state
edges
events
effects
or a composed layer.
```

---

## 

## **45.3**

**`ql-native`**

The native implementation MUST begin from only:

```text
QL
LLM interface
capability interface
environment
success conditions
transport
```

Concepts such as:

```text
turn
assistant message
tool message
subagent
critic
planner
```

MUST be introduced only where the implementation can state why they are needed.

The native implementation MUST NOT copy the Pi or Pydantic loop architecture merely to accelerate parity.

Its purpose is to test what architecture QL itself generates.

---

# **46. Repository layout**

Recommended top-level structure:

```text
ql-agent-spec/
├── README.md
├── SPEC.md
├── CHANGELOG.md
├── schemas/
│   └── 0.1/
│       ├── event.schema.json
│       ├── act.schema.json
│       ├── return.schema.json
│       ├── transition.schema.json
│       ├── residue.schema.json
│       ├── determination.schema.json
│       ├── closure.schema.json
│       ├── reentry.schema.json
│       ├── conjugate.schema.json
│       └── child-summary.schema.json
├── fixtures/
│   ├── transitions/
│   ├── closure/
│   ├── conjugation/
│   ├── depth/
│   ├── reentry/
│   └── negative/
├── typing-corpus/
│   ├── corpus.jsonl
│   ├── adjudication/
│   └── README.md
├── conformance/
│   ├── manifest.yaml
│   ├── expected-results/
│   └── runner-contract.md
├── render/
│   └── rendering-contract.md
├── extensions/
│   └── registry.md
└── research/
    ├── harmonics.md
    ├── 64-state-field.md
    ├── mef.md
    ├── context-frames.md
    └── epogdoon.md
```

The experiment repositories SHOULD vendor or version-pin the specification.

They MUST record the exact spec revision used for each experimental run.

---

# **47. Versioning**

Semantic versioning SHALL be used.

```text
0.1.x
```

means fixes or clarifications which do not intentionally alter the Core semantic model.

```text
0.2.0
```

may alter or add experimental semantic requirements.

```text
1.0.0
```

requires a stable Core suitable for claims beyond exploratory experimentation.

Trace records MUST state both:

```text
spec
schema_version
```

so data remains interpretable after later revisions.

---

# **48. Extension registry**

Optional structures SHALL use namespaced extensions rather than silently extending Core.

Examples:

```text
ql.harmonic.D2
ql.harmonic.ABC.B
ql.mef.L0
ql.mef.L5
ql.context.CF5
ql.state64
ql.catukoti
ql.epogdoon
ql.topology.genus
```

An event MAY contain:

```json
{
  "extensions": {
    "ql.harmonic.D2": {
      "active": true
    }
  }
}
```

Core consumers MUST be able to ignore unknown extensions without losing the basic QL trajectory.

---

# **49. Promotion of an extension into Core**

A Research or Operator structure MAY become a Core requirement only when all three conditions are met:

### **Semantic necessity**

The existing Core fails to represent a recurring and meaningful distinction.

### **Computational definition**

The proposed structure has a precise state, transition or invariant that can be implemented and tested.

### **Experimental or architectural value**

At least one of the following is shown:

```text
clearer conformance;
better prediction of failure/success;
reduced ambiguity;
stronger closure;
useful new capability;
significant compression;
cross-runtime invariance;
more faithful realisation of established QL relations.
```

Promotion requires a new minor specification version.

---

# **50. Pre-experiment development protocol**

The following sequence is REQUIRED.

## **Gate A — Semantic freeze**

Complete:

```text
SPEC.md
normative glossary
Core/Operator/Research classification
36 relation vocabulary
closure semantics
re-entry semantics
conjugate semantics
depth semantics
```

**Pass criterion:** two independent human or agent readers can construct equivalent fixture traces without access to implementation code.

---

## **Gate B — Schema freeze**

Complete:

```text
all Core JSON Schemas
schema tests
canonical examples
schema versioning
```

**Pass criterion:** every reference fixture validates and no Core semantic distinction exists only in prose.

---

## **Gate C — Reference corpus**

Complete:

```text
positive transition fixtures
P5 reopening fixtures
closure fixtures
re-entry fixtures
depth fixtures
conjugation fixtures
negative fixtures
```

**Pass criterion:** disagreements concern identifiable semantics rather than missing definitions.

---

## **Gate D — Headless semantic simulator**

Implement a framework-neutral scripted circuit simulator.

It MUST execute fixtures without an LLM.

It MUST support:

```text
Rij
residue mutation
closure
reopening
re-entry
child metadata
conjugate metadata
trace emission
```

**Pass criterion:** the conformance suite can test QL semantics independently of any real agent.

---

## **Gate E — Typing corpus**

Construct and adjudicate the initial semantic typing corpus.

**Pass criterion:** the typing protocol can show why a type was assigned and preserve disagreement rather than hiding it.

---

## **Gate F — Visual renderer**

Build a reference trace renderer.

**Pass criterion:** a human can watch/replay QL state changes and understand:

```text
where the circuit is;
what relation just occurred;
what exterior boundary was crossed;
why the destination changed;
whether closure is possible.
```

---

## **Gate G — Independent runtime implementations**

Build:

```text
ql-pi
ql-pydantic
ql-native
```

in parallel.

No implementation may import the executable core loop of another.

They MAY share:

```text
schemas
fixture corpus
trace parser
test vectors
non-semantic utilities
```

**Pass criterion:** all three independently pass the required `QLC-*` suite.

---

## **Gate H — Semantic dry runs**

Each implementation runs the same manual task set:

```text
tool-free conceptual task
single-file repair
multi-file coding task
failing tool task
ambiguous requirement
evidence-heavy task
nested task
direct + conjugate task
```

Humans inspect the traces.

**Pass criterion:** obvious semantic misclassifications are uncommon enough that benchmark interpretation is meaningful, and every remaining recurring ambiguity is documented.

---

## **Gate I — Experiment Series 1 freeze**

Freeze and tag:

```text
spec version
schema version
runtime commits
model versions
tool surfaces
typing protocol
task corpus
evaluation harness
```

Only after this gate do the three implementations become the formal experimental conditions.

---

# **51. Series 1 experimental invariants**

During Series 1, comparisons SHOULD hold constant wherever possible:

```text
task
model
model parameters
tool availability
external environment
success condition
starting artifact state
```

and vary:

```text
QL/Pi
QL/Pydantic
QL/Native
```

Upstream vanilla agents MAY be retained as external reference baselines.

They are not one of the three main QL experiments.

---

# **52. Series 1 required measurements**

Every run SHOULD record:

```text
task success
artifact success
circuit closure
token usage
model-call count
tool-call count
failed tool-call count
elapsed execution time
number of Rij transitions
position dwell
P5 reopening count
reopening destinations
child-circuit count
maximum depth
conjugate invocation count
conjugate reopen count
residue count
typing disagreement rate
human intervention count
```

The trace analysis layer SHOULD additionally derive:

```text
transition matrix
common trajectory motifs
position oscillation
closure attempts
recovery paths
direct/conjugate asymmetry
child return compression ratio
```

---

# **53. Operator ablations**

Within each runtime, Series 1 SHOULD compare:

```text
Core direct only

Core + ql_depth

Core + ql_conjugate

Core + depth + conjugate
```

These are capability ablations, not “intelligence levels”.

The model or runtime policy SHOULD be allowed to invoke optional depth and conjugation when available.

A separate forced-use condition MAY be run for diagnostic purposes, but MUST NOT replace voluntary-use experiments.

---

# **54. Semantic ablations**

After basic cross-runtime comparison, run:

```text
4+2 structural type
vs
six homogeneous position labels
```

```text
interpreted-return Rij routing
vs
carrier-driven routing
```

```text
positive closure
vs
ordinary no-more-tool termination
```

```text
fresh-context conjugation
vs
same-context self-critique
```

```text
typed P0+ re-entry
vs
generic transcript summary
```

```text
correct L1/L4′ semantics
vs
shuffled positional semantic labels
```

The purpose is to test whether QL semantics matter, not merely the presence of additional deliberative machinery.

---

# **55. Research questions enabled by the trace corpus**

Once enough runs exist, the project SHOULD investigate whether QL traces reveal stable signatures such as:

```text
P2↔P3 oscillation associated with implementation thrashing

P1 accumulation without P3 stabilisation

repeated P5→P2 indicating "done-before-effect"

P4→P0 patterns indicating requirement reframing

conjugate P4′ exposing direct P3 defects

depth invoked around genuine P4 local-whole formation

certain harmonic relations preceding successful recovery
```

These are hypotheses.

They MUST be discovered or rejected from trace evidence.

The runtime MUST NOT be redesigned in advance simply to manufacture them.

---

# 

# **56. Research path:**

**`64 = 2^6 = 4^3`**

The wider QL system admits the six-bit and three-quaternary factorisations described in the source.

The correct agentic research question is:

Can rich QL runtime state be quotiented into a semantically meaningful binary state at each of the six positions?

Possible candidate distinctions include:

```text
unresolved / resolved
implicit / explicit
absent / established
open / closed-at-position
```

No candidate is normative yet.

If a stable six-bit quotient emerges, the project SHOULD test:

```text
whether the 64 state loses important trajectory information;
whether the three harmonic 4³ factorisations predict behaviour;
whether L2/Catuṣkoṭi gives useful semantics to dyad states.
```

---

# **57. Research path: full MEF**

Once the raw QL address is stably typed, the full MEF may be exposed progressively.

Potential state:

```text
(
  face,
  lens,
  position,
  context_frame,
  depth
)
```

Promising early optional lens operations include:

```text
L0  — interrogative disclosure:
      what kind of question is this responsibility?

L5  — articulation disclosure:
      at what density of articulation does this presently exist?
```

The source specifically develops L0/L0′ and L5/L5′ as foundational agent-relevant refractions.

The runtime SHOULD discover their utility through explicit calls before making them implicit state.

---

# **58. Research path: harmonic operators**

The harmonic families may later become explicit self-relation operators.

Potential API:

```text
ql.inspect(
    harmonic = "D2",
    current_residue = [...]
)
```

which asks:

```text
Effect ↔ Form:
does the operation instantiate the intended form?
does the form actually answer the operative challenge?
```

Similarly:

```text
B = P1↔P4:
is this material actually relevant to the telos?
has discovery changed what counts as relevant evidence?
```

and:

```text
O3 = P5↔P0:
what has this determination made possible as new ground?
```

Such operators MUST initially be optional.

---

# **59. Research path: context frames**

The fuller seven-frame grammar SHOULD be retained in the specification research corpus.

`CF5` is especially relevant because the source identifies it as the parent frame of the nested sixfold.

The first promotion candidate is therefore:

```text
depth-request
    ↓
CF5 parent frame
    ↓
nested six
    ↓
CF6 bridge
    ↓
CF7 re-entry
```

This MAY eventually replace the simpler Core child-circuit relation once it has precise computational semantics and cross-runtime conformance tests.

---

# **60. Research path: retained difference and epogdoon**

Core implements:

[  
P_0^+=P_0+\Delta.  
]

The wider QL musical derivation relates the advancing return to the `9/8` remainder in:

[  
\frac{16}{9}\times\frac98=\frac21.  
]

The source uses this relation to motivate forward re-entry rather than identical return.

Research SHOULD investigate whether `Δ` acquires a natural quantitative structure such as:

```text
residual error
information gain
semantic displacement
changed-context measure
unresolved-relation vector
```

No literal ratio-based agent control is permitted under Core `0.1`.

---

# **61. Definition of QL-Agent Core conformance**

An implementation is **QL-Agent Core conforming** when all of the following are true:

```text
P0–P5 exist as semantic addresses.

4+2 is structurally preserved.

L1 + L4′ is treated as refraction, not ontology.

Canonical circulation exists without chronological coercion.

All 36 Rij relations are representable.

Exterior interaction is represented through π0/1 and ρ1/0.

Returned difference is interpreted before semantic destination is finalised.

Tool/message/model carrier identity does not determine QL position.

Typed residues exist independently of transcript.

P5 determination is explicit.

P5 does not automatically imply closure.

P5 can reopen P0–P4 distinctly.

Closure is a positive event.

Operational/artifact/task/circuit success remain distinguishable.

Closure produces a retained ReentryDelta.

P0+ can be established from retained difference.

Direct and conjugate faces are structurally representable.

Child circuits possess typed parentage.

Portable JSONL QL traces are emitted.

Witness disagreement is inspectable.
```

---

# **62. Definition of QL Experiment Readiness**

An implementation is **not** experiment-ready merely because it passes one scripted run.

Experiment readiness requires:

```text
Core conformance passes.

Whole and current-position conjugation pass.

Fresh-context ConjugatePacket passes.

ql_depth / equivalent child circuit passes.

Child typed reintegration passes.

Reference renderer works.

Typing corpus can be replayed.

Cross-runtime trace comparator can ingest its output.

All required QLC-* tests pass.

Manual semantic dry runs are intelligible.

No known unresolved bug can silently revert to ordinary tool-loop closure semantics.
```

The three experiment repositories MUST reach this state independently.

Only then is the shared foundation frozen for Experiment Series 1.

---

# **63. Definition of a genuine QL-agent experiment**

A run counts as evidence about QL-native agency only when:

```text
the agent began from a conforming QL circuit;

its exterior acts were typed as π/ρ exchanges;

its actual trajectory was recorded through Rij;

its semantic position was not reducible to its carrier;

its determination and closure were distinct;

its retained difference was available at closure;

and the implementation version passed the same conformance profile
used by the other experimental runtimes.
```

Otherwise the run MAY be useful engineering work, but it is not evidence from the controlled QL-agent experiment.

---

# **64. Central invariant**

The entire Core can be stated compactly as:

[  
\boxed{  
(4+2)  
+  
R_{ij}  
+  
\pi^{0/1}/\rho^{1/0}  
+  
J  
+  
N  
+  
C/R_e  
}  
]

where:

```text
4+2
    defines the sixfold causal whole;

Rij
    defines semantic mobility within that whole;

π/ρ
    defines agent/exterior exchange;

J
    defines direct/conjugate inversion;

N
    defines recursive whole-making;

C
    defines positive closure;

Re
    defines retained re-entry.
```

The agent is therefore not given a six-step procedure.

It is given a **structured whole within which intelligence can move**.

The QL source arrives at precisely this broader distinction: the agent is neither a six-stage workflow nor an unconstrained ordinary loop decorated with six labels, but an intelligence inhabiting a causal/telic field with harmonic, conjugate, recursive and refractive depth.

The implementation criterion is correspondingly severe:

**QL must be built deeply enough into the runtime that the topology itself can succeed, fail, deform, reveal ambiguity, or demand revision.**

That is the condition under which the three implementations become experiments _with_ Quaternal Logic rather than demonstrations _about_ it.