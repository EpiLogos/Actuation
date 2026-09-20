# E-MT-2 — Jev at the QL loop's decision points (first live result)

**Date:** 2026-09-18 · **Model:** `jev-latest` via TypeSafe System One API
(`POST /v1/systemone`) · **Corpus:** 150 replay states exported from the
retained Series-1 run records (`2026-09-13-glm` + smoke-restraint, ql-*
conditions only).

**Standing:** every Jev answer is an instrument reading
(`semantic-stochastic`, promotion none); the run determination is
`pending-human-review`. The truth labels here are the incumbent LLM's own
recorded decisions or mechanical derivations from the records — agreement with
them measures **substitutability**, not correctness.

## Results

| Question | Type | n | Accuracy vs loop | Brier | Mean conf | Latency p50/p90 |
|---|---|---|---|---|---|---|
| `next-carrier` | choice model/capability/internal_control | 21 | **90.5%** (19/21) | 0.166 | 0.937 | 271 / 463 ms |
| `next-position` | choice P0–P5 | 21 | **61.9%** (13/21) | 0.456 | 0.542 | 266 / 621 ms |
| `requested-outcome` | choice close/reopen | 20 | **90.0%** (18/20) | 0.112 | 0.823 | 284 / 423 ms |
| `intent-already-achieved` | noul | 20 | **95.0%** (19/20) | 0.025 | — (mean noul 0.095) | 264 / 356 ms |
| `subject-verifiable` | noul | 20 | **100%** (20/20) | 0.055 | — (mean noul 0.213) | 248 / 398 ms |
| `stipulation-violated` | noul | 20 | no truth (reading) | — | mean noul 0.079, 0/20 ≥ 0.5 | 275 / 450 ms |
| `genuinely-local-whole` | noul | 8 | no truth (reading) | — | mean noul 0.362, 0/8 ≥ 0.5 | 243 / 342 ms |
| `act-adequacy` | score 0–3 | 20 | no truth (reading) | — | mean score 2.28 (conf 0.407) | 262 / 397 ms |

Overall latency p50 **266 ms**, p90 **423 ms**, max 857 ms over 150 questions.
Tokens: **145,517 input / 5,135 output** total. Zero API errors, zero retries
needed; every state in `emt2/states.jsonl` was asked exactly once.

## What the numbers say

- **Carrier choice and determination outcome look substitutable.** 90%+
  agreement with the loop's recorded decisions on both, at high stated
  confidence (0.94 / 0.82).
- **Both `next-carrier` misses are the same shape:** at two genuine closure
  points the loop answered `internal_control` and Jev answered `capability`
  (0.74 and 0.97 mass) — Jev wanted one more exterior act. The single
  `intent-already-achieved` miss is one of those same two points (deeplane
  RESTRAINT, noul 0.40 vs true). Consistent failure mode: Jev under-detects
  "the intent is already achieved; request closure."
- **Both `requested-outcome` misses** are native-S1-RESEARCH runs whose
  circuits closed; Jev read `reopen` (0.61 / 0.68). Whether the loop *should*
  have closed those is exactly what human review has not yet decided — this
  is the most interesting disagreement class to put in front of the owner.
- **`next-position` is the weak point (61.9%, chance 16.7%).** Confusion
  concentrates on (truth P0 → predicted P1) ×3 — at a fresh frame Jev jumps to
  evidence-gathering rather than P0 framing — and P1↔P5 swaps. If Jev ever
  drives recurrence resolution, this decision point needs the incumbent or a
  kernel check behind it.
- **`subject-verifiable` 100% is degenerate:** all 20 truths are `false`
  (recorded `claimed_subject` values are free text, never the bare task id,
  and the derivation also requires `objective_checks_pass`). Jev never said
  yes (mean noul 0.213). Agreement, but no positive class — it measures the
  question's discrimination, not much about Jev.
- **Readings without truth lean the same way as the incumbent's own
  evaluations:** no stipulation violations (mean noul 0.079; the recorded
  verdicts were all "met") and no genuinely local whole (mean noul 0.362;
  every recorded `ql-conjugate-scope` answer was `scope: whole`, no conjugate
  spawned). These are agreement-of-readings between two instruments, not
  accuracy. `act-adequacy` sits at 2.28/3 — between "partial" and "adequate".

## Honest limits

1. **The loop's recorded decisions are not ground truth.** They are the
   incumbent GLM's own choices. High agreement means Jev could sit behind the
   same decision points and vote the same way most of the time; it does not
   mean either instrument is right. The misses (RESEARCH closures, the two
   premature-act points) are candidates for the human-review queue, not
   errors to fix.
2. **Truth derivations are record-relative:** `requested-outcome` truth comes
   from the circuit's own closure state; `intent-already-achieved` truth is
   the loop's revealed preference (exterior act → false; `internal_control` on
   a completed, closed, objectively-passing run → true; other
   `internal_control` points omitted); `subject-verifiable` truth is the
   mechanical string comparison plus the objective-check flag. Each is stated
   in `emt2/manifest.json` under `truth_standing`.
3. **No truth was invented for the reading-only questions.** The records do
   carry the incumbent's own stipulation verdicts and conjugate-scope answers,
   but those are the same incumbent's evaluations — comparing Jev to them is
   agreement-of-readings, so they are reported as mass statistics only.
4. **Corpus is small and skewed:** 150 states (cap, round-robin over 8
   question ids from ~1,000 generated contexts); the underlying runs are 27
   closed / 3 open circuits from one benchmark series, so the `close`/exterior-
   act base rates dominate several questions.
5. **State fidelity:** contexts are rebuilt from the loop's own controller
   prompts (mode, task, stipulations, success conditions, capabilities,
   allowance schedule/consumed, residue summaries, recent event window) minus
   the answer; bounded summaries are not the full transcript the incumbent
   saw. The two smoke-run records use the older control shape (`carrier` as a
   plain string) and are normalised by the exporter.

## R4 re-measure — allowance state included, mechanical/semantic split (2026-09-19)

The fix named above was executed: the exporter now fills the catalogue's
`allowance_state` field on every `next-position` state (per-position schedule,
consumed counts, active position — sourced from the last `ql-next-act`
controller prompt at or before the producing act, which is where the loop
restates it), and classifies each transition's destination basis from its
recorded witness (`emt2v2/destination-basis.json`): `mechanical-closure-request`
(closure-request law routes to P5), `mechanical-reopen-routing` (Rust reopen
routing), `mechanical-allowance-exhaustion`, `model-interpretation` (the
incumbent's `ql-interpret-return` decided — the semantic remainder).

Live re-run (`emt2v2/runs/jev-run-2026-09-19T18-20-34-583Z.json`, 150 states,
147.6K input / 5.1K output tokens, zero errors):

| Question | n | agreement | vs 09-18 run |
|---|---|---|---|
| next-carrier | 21 | 90.5% | identical |
| next-position | 21 | 61.9% | identical aggregate |
| requested-outcome | 20 | 90.0% | identical |
| intent-already-achieved | 20 | 95.0% | identical |
| subject-verifiable | 20 | 100% | identical |

The aggregate didn't move — but the decomposition is the result:

- **Mechanical transitions: 3/4.** All three closure-request transitions
  correct (the machinery is now visible and the answer is the rule's); the one
  miss is the P5→P1 reopen routing — pure Rust-owned routing that no model
  should be asked. This confirms the design split: mechanical transitions stay
  in Rust, and where the machinery is visible a referee answers them for free.
- **Semantic remainder: 10/17 = 58.8%.** The fair re-measure set. Every miss
  but two is a **frame re-entry**: truth P0, Jev predicts P1 at 0.56–0.63
  confidence, on acts whose intent literally says "ground the operative frame".
  With the allowance schedule showing active_position P0 and consumed P0:1,
  Jev still reads "survey the workspace" as P1 material-gathering. This is a
  law-knowledge gap, not an information gap: the frame law (initial grounding
  belongs to P0) has to be stated in the question's instructions or routed by
  the loop, not learned from state.
- **Stability finding:** the four unchanged questions reproduced the 09-18
  numbers exactly (same correct/incorrect splits, Brier within noise) — Jev's
  typed answers are effectively deterministic on these states.

Next concrete steps for this question: (1) state the frame-entry law in the
`next-position` question instructions and re-measure the remainder; (2) in the
composed loop, ask the model only `model-interpretation` transitions —
mechanical classes are already Rust's; (3) n=17 remainder — one miss swings
~6 points; treat the number as directional until the corpus grows.

Reproduce: `python3 export_decision_contexts.py --limit 150 --out emt2v2`
then `node harness.mjs --states emt2v2/states.jsonl --truth emt2v2/truth.jsonl
--out-dir emt2v2/runs`.

## Files

- Exporter: `experiments/model-types/jev/export_decision_contexts.py`
  (R4: fills `allowance_state`, writes `destination-basis.json`)
- Original 09-18 run (untouched): `emt2/states.jsonl` (150),
  `emt2/truth.jsonl` (102 with truth), `emt2/manifest.json`,
  `emt2/runs/jev-run-2026-09-18T02-01-52-947Z.json`
- R4 re-measure (2026-09-19): `emt2v2/states.jsonl`, `emt2v2/truth.jsonl`,
  `emt2v2/manifest.json`, `emt2v2/destination-basis.json`,
  `emt2v2/runs/jev-run-2026-09-19T18-20-34-583Z.json` (digest-pinned,
  determination `pending-human-review`, promotion none)
- E-MT-1 files separate (repo root of `jev/`, rerun 2026-09-19:
  `runs/jev-run-2026-09-19T18-23-56-501Z.json`).
