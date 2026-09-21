# Series 1 round 5 — results reading (2026-09-17)

> **Corpus status: COMPLETE for classic + ql-direct across five task families; ql-deep attempted on RESTRAINT and failed on budget. Determination: pending-human-review.**

```text
benchmark_revision   series1-round5-accept-2026-09-17
host                 rust-native-acceptance @ 9f8e3e2 (release binaries, acceptance worktree)
model                zai:glm-5.3-flash (supplied body, one-shot-json)
owner instrument      08d14e89c6427cb885e117c2e9bc9dc61a0f90b7 (matched)
budget               max_calls 64, max_contexts 32
recipe               acceptance-run.mjs (copied here); raw run trees in ~/.cache/actuation/round5-experiments/
```

Task families: S1-CODE-001 (code), S1-EPISTEMIC-001 (epistemic-understanding),
S1-RESEARCH-001 (local-research), S1-RESTRAINT-001 (bounded-restraint),
S1-SKILL-001 (skill application). Conditions: `classic` and `ql-direct` on all
five; `ql-deep` on RESTRAINT only. Held constants verified machine-side on
every task (`held_constants.valid = true`, zero mismatches).

## The numbers

| Task | condition | status | checks | model calls | tokens | wall |
|------|-----------|--------|--------|------------:|-------:|-----:|
| CODE | classic | completed | pass | 5 | 7,262 | 76 s |
| CODE | ql-direct | completed | pass | 14 | 75,315 | 755 s |
| EPISTEMIC | classic | completed | pass | 4 | 4,730 | 92 s |
| EPISTEMIC | ql-direct | completed | pass | 15 | 73,122 | 768 s |
| RESEARCH | classic | completed | pass | 4 | 4,310 | 81 s |
| RESEARCH | ql-direct | completed | pass | 15 | 83,784 | 986 s |
| RESTRAINT | classic | completed | pass | 2 | 957 | 14 s |
| RESTRAINT | ql-direct | completed | pass | 7 | 27,161 | 285 s |
| RESTRAINT | ql-deep | **failed** | pass | 63 | 474,824 | 2,655 s |
| SKILL | classic | completed | pass | 6 | 9,320 | 93 s |
| SKILL | ql-direct | completed | pass | 22 | 133,491 | 1,552 s |

Totals over the five tasks: classic 21 calls / 26,579 tokens / 355 s;
ql-direct 73 calls / 392,873 tokens / 4,346 s. The direct-face QL loop costs
**×3.5 the model calls, ×14.8 the tokens, ×12.3 the wall clock** of classic
for identical verification outcomes.

## Reading

1. **Correctness is invariant across loop conditions.** All ten completed
   trials pass every objective check: tests pass and exports are preserved
   (CODE), workspaces byte-identical (EPISTEMIC, RESEARCH, RESTRAINT),
   deliverable present with protected sources untouched (SKILL). On this
   corpus, changing the recurrence structure — plain iteration vs a QL circuit
   with native closure — does not change whether the model can do the work.

2. **Cost is the difference.** The token multiple (×14.8) far exceeds the call
   multiple (×3.5): each QL call is itself heavier, carrying frame, residue and
   closure context. RESTRAINT shows the spread at its purest — the same task
   costs 2 calls / 957 tokens classically and 7 calls / 27,161 tokens on the
   direct face, both answering correctly.

3. **Closure quality is genuinely good.** Every ql-direct trial closes its
   circuit c0 at position 5 with a determination ref, evidence refs, an
   inspection recording `objective_checks_pass = true`, and a reentry delta
   with zero unresolved refs into a renewed frame (`S1-<TASK>0+`). The
   closures' `success_state` is honest about its own reach:
   `operation: true, circuit: true, task: unknown, harmonic: unknown` — task-
   and harmonic-level success are exactly what the pending human review owes.

4. **ql-deep failed structurally, not epistemically.** On a task the direct
   face closes in 7 calls, deep burned 63 of 64 calls and 475 k tokens and
   ended `research host call budget exhausted`. The trace shows 13
   conjugate-face circuits spawned, all as depth-0 siblings of c0, none ever
   reaching closure. Two named defects:
   - **Unbounded conjugate re-entry.** Conjugate cycles spawn new cycles
     without consuming any allowance; there is no mechanism that forces the
     deep loop toward determination. (The designed fix is the typed conjugate
     allowance, refusal routed to determination.)
   - **Intent re-wrapping at the gate.** Circuits 8–10 (parent `gate:7`) carry
     `initiating_intent` as a dict *containing* `initiating_intent` — the gate
     composes the frame by wrapping instead of passing through, compounding
     structure cycle over cycle.
   Notably, restraint held even inside the failure: the workspace stayed
   byte-identical through 63 calls.

5. **The deliverable is now unblocked.** `determination` remains
   `pending-human-review` and `human_acceptance` is false on every closure.
   The corpus is complete, held-constant-verified and staged here; the first
   human-review pass over it (per Factory #110/#137/#138) is the actual
   deliverable this round exists to serve.

## What this sets up

- **Conjugate allowance** (small Rust change), then ql-deep across the corpus
  at max_calls 64 — this run is the defect evidence that motivates it.
- **Compressed arm** (`QL_COMPRESSED_CONTROL=1`) across the corpus — the ×14.8
  token multiple is the target it attacks.
- **Human review pass** over this directory — the machine side is done; the
  determination is not ours to make.

## Files

```text
round5-S1-<TASK>-001.json   full machine run per task (ql-series1-run/0.3; embeds
                            prompts, workspaces, traces, verification, usage)
acceptance-run.mjs          the one-command recipe that produced the corpus
```

Raw trial trees (full world states per trial) remain at
`~/.cache/actuation/round5-experiments/S1-<TASK>-001/trial-*/`.
