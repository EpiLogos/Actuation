# EBM field gate at the `ql-next-act` acceptance point — design (R5)

**Status:** design only. No Rust has been written or changed. `policy.rs` is
in-flight code; the implementation lands on a branch through proposal, never
silently (programme law, `../README.md` → hybrid-loop implementation path
step 4).

**What is designed here:** scoring candidate acts against the bimba field at
the moment the loop accepts an act, with typed refusals that carry the energy
as evidence — mirroring the allowance-schedule refusal pattern already in
`policy.rs` (typed refusal event, no silent stop, routed onward by rule).

## 1. The seam

`ModelPolicy::next_act` (`crates/actuation-research/src/policy.rs`) is the
acceptance point: the controller model has just returned a `ql-next-act`
control (intent + carrier), and the act is about to execute. The gate sits
between "control returned" and "act executes". The same design applies
unchanged to `interpret` (candidate transitions) — Phase 3 may gate both, but
`next_act` is the first seam because a refused act is cheap (nothing has run)
while a refused interpretation already costs the executed act.

The gate is a **policy-side instrument**, not a host change: the
`RuntimeHost` seam stays untouched, loop semantics stay in `policy.rs`.

## 2. What is scored

The field reads states of the bimba map: directed relations between bimba
coordinate nodes, scored by the Phase-2.6 head (semantic channel + typed
conditioning + sibling mining; `../ebm/evidence/energy-head-v2b-*.json`). A
loop act is not itself a bimba relation, so the gate needs an honest mapping
and an honest scope:

- **Scored state:** `(anchor, candidate)` where `anchor` is the bimba node the
  act works on, and `candidate` is the bimba node the act proposes to touch or
  produce, when both exist. The act's intent text is embedded with the same
  model and dims as the field (3072, `models/gemini-embedding-2`) and its
  embedding is matched to the nearest field node — that match IS the
  candidate. The anchor comes from the act's arguments (a coordinate, a uuid,
  a named subject resolvable in the map).
- **Scope law (abstention):** when either side cannot be resolved to the field
  — file-manipulation acts, generic model turns, tasks that do not concern the
  map — the gate **abstains**: no energy is computed, no refusal is possible,
  and the abstention is recorded as a counter on the run. The field is a map
  of the bimba ground, not of everything; a gate that invented refusals
  outside its scope would be manufacturing authority.
- **Threshold:** the gate threshold is pinned from the head's calibration
  evidence (ECE ≈ 0.05 after mining; Youden thresholds recorded per operator
  in the v2b evidence). The threshold value travels in the run manifest, so
  any run's gate behaviour is reconstructable.

## 3. The refusal shape (mirroring allowance exhaustion)

`policy.rs` already has the pattern: on overrun, the allowance schedule
returns a typed refusal, the act becomes an `internal_control` close request
with `metadata.allowance_refusal` attached, a recorded event carries the
details, and the loop routes onward by rule — no silent stop. The field gate
refusal copies that shape exactly:

```rust
// after the controller returns a candidate act, before execution:
match field_gate.score(&cx, &candidate) {
    Scored { energy, threshold_ok: true, .. } => { /* act proceeds; energy recorded */ }
    Scored { energy, threshold_ok: false, evidence } => {
        // typed refusal: recorded event `field_gate_refused`, act NOT executed,
        // the controller is asked again with the refusal evidence in view,
        // per-circuit gate budget bounds re-asks (mirrors GRACE_ALLOWANCE).
        return Ok(Some(Act {
            metadata: json!({
                "field_gate_refusal": {
                    "energy": energy,
                    "threshold": evidence.threshold,
                    "scored_state": evidence.state_digest,
                    "anchor": evidence.anchor_coord,
                    "candidate": evidence.candidate_coord,
                    "head_revision": evidence.head_revision,
                }
            }),
            ...
        }));
    }
    Abstain { reason } => { /* proceed; abstention counted, never a refusal */ }
}
```

Three discipline points, all inherited:

1. **Refusal is not verdict.** Low energy is incompatibility with learned
   structure, not wrongness (`../ebm/README.md` honesty law). The refusal
   forces a second look by the controller; it never determines the run.
2. **Evidence attached, human-reviewed.** Every refusal event lands in the
   run record with its energy, threshold, and the scored pair — exactly the
   material the E-MT-2 review queue exists to judge. Determination stays
   `pending-human-review`.
3. **Budgeted re-asks.** Like the allowance grace, the gate budget bounds how
   often the same candidate pair can be refused and re-asked in one circuit;
   exhaustion of the gate budget routes to determination instead of looping.

## 4. Availability and failure

The allowance schedule is Rust-owned law, so it fails closed. The field gate
is an instrument standing on a live graph and an embedding service, so:

- **Graph or embedding service unreachable** → the gate abstains for the rest
  of the run, one typed `field_gate_unavailable` event records the cause, and
  the run proceeds. An unavailable instrument must not fabricate refusals, and
  must not brick the loop.
- **Scoring error or unparseable coordinates** → abstain, counted, recorded.

This asymmetry is deliberate: Rust law refuses (closed world), instruments
report (open world). The run manifest records gate availability as a held
constant for the A/B.

## 5. Cost and latency

Head scoring is numpy-scale (sub-millisecond) once both embeddings exist. The
one real cost is embedding the act text (~one API call, p50 in the hundreds of
milliseconds); embeddings are cached by text digest, so repeated acts on the
same subject are free. For the A/B this lands entirely in the `ql-faculty`
condition — the held-constant law requires the gate to be absent, not
abstaining, in the `ql-direct` comparison arms.

## 6. What would falsify the gate

Inherited from `../README.md` falsification conditions, made concrete:

- Refusals that human review judges sound (no incompatibility actually
  present) at a rate above the head's false-positive rate on the ladder — the
  gate is manufacturing refusals.
- Refusals concentrated on acts outside the map's scope — the abstention rule
  is too narrow.
- No measurable difference in `ql-faculty` vs `ql-direct` outcomes or friction
  — the gate is ceremony (a valid finding, recorded as such).

## 7. Implementation path (when commissioned)

1. Branch off the current `policy.rs` base (proposal first, owner adopts).
2. `FieldGate` struct behind the same module pattern as the allowance
   schedule: schedule-style typed returns, no I/O inside `next_act` itself —
   the graph read and embedding call go through a small port trait so tests
   run against a fixture field.
3. Frozen-fixture tests: refusal shape, abstention shape, budget exhaustion,
   unavailable-instrument abstention — before any live wiring.
4. Live wiring behind an env flag (`QL_FIELD_GATE=shadow|enforce`); `shadow`
   records energies without refusing, which is the first live mode and the
   calibration source for the enforce threshold.
