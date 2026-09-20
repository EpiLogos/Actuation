# Jev — typed judgement over QL/MEF, and loop refereeing (E-MT-1, E-MT-2)

Jev (TypeSafe's System One model, `jev-latest`) answers noul / choice / score
questions about a state with probabilities, in one parallel pass, at ~70–500ms.
That is natively the shape of two jobs this programme currently does other ways:
classifying across QL positions and MEF lenses, and the QL loop's model-facing
decision points.

**Scope (owner, 2026-09-17):** Jev applies to **QL and MEF lensing and
classification**. The M-coordinate census work is deliberately out of scope here.

**The pairing (owner, 2026-09-17):** the LLM owns semantics — generation,
reflection, explanation; Jev owns typed classification across QL positions and
MEF lenses. This mirrors the vendored skill's own law ("code owns the workflow;
the model supplies programmable common sense") one level up: Rust keeps the
loop's rules, budgets and execution; the LLM keeps meaning; Jev keeps typed
judgement. See `../hybrid-loop/README.md`.

## Access

Direct TypeSafe System One API: `POST https://api.typesafe.ai/v1/systemone`,
`Authorization: Bearer TYPESAFE_API_KEY`. The credential lives in the macOS
login keychain (`security find-generic-password -s TYPESAFE_API_KEY -w`; rotate
at will — the harness reads it fresh each run, env var overrides). The Vercel AI
Gateway route (`typesafe-ai/jev`, AI SDK `experimental_evaluate`) remains an
alternative; the harness uses the direct API.

The official TypeSafe agent skill is vendored at
`skill/typesafe-ai/` ([github.com/typesafe-ai/skills](https://github.com/typesafe-ai/skills)
rev `65a39f3`, 2026-09-12) — read its `SKILL.md` before designing new questions:
primitives (Choice / Noul / Score), state design, parallel questioning, and
confidence handling are all specified there.

## E-MT-1 — QL/MEF lensing and classification (`ql-mef-lensing` family)

- **`ql-position`** (choice P0–P5): classify each act of the QL circuits in the
  retained Series-1 runs (ql-direct / ql-deep conditions) into its position
  responsibility. Ground truth is the loop's own recorded `source_position` —
  loop-recorded state, not human-verified, and labelled as such everywhere.
- **`mef-lens-refraction`** (choice over the 12 MEF lenses): through which lens
  is this subject most coherently read? **No ground truth by design** — MEF is a
  manifold of disclosure, not a bucket taxonomy; subject identity is never lens
  identity, and every reading is `semantic-stochastic`, never promoted. The
  retained runs carry the runtime's standing refraction pair (L1 + L4′) on every
  event, so the harness reports Jev's mass on that pair as
  agreement-of-readings, not accuracy.

**First live result (2026-09-17, 100 acts, model `jev-latest`, seed-ordered
corpus):**

- `ql-position`: **92% accuracy** (chance 16.7%), mean Brier 0.212, mean
  confidence 0.658.
- `mef-lens-refraction`: **0.867 mean mass on the standing L1+L4′ pair**,
  mean confidence 0.503.
- Latency: p50 238ms, p90 325ms, max 914ms over 200 questions; 117K input /
  19K output tokens total.
- Run record: `runs/jev-run-2026-09-17T20-18-45-246Z.json` (digest-pinned,
  determination `pending-human-review`, promotion none).

Honest reading: 92% against loop-recorded positions shows Jev can *read* the QL
position structure from act context; whether that structure is itself right is
exactly what the loop programme's human review exists to decide. The lens mass
of 0.867 is agreement between two readings (Jev's and the runtime's standing
pair), not truth.

The QL kernel's own `classify-relation` operation (via
`experiments/ql-runtime/native-owner-instrument`, owner rev `08d14e8…`) is the
formal-ground-truth path for relation classification; wiring it in is the named
next step for a kernel-referenced corpus.

## E-MT-2 — loop decision refereeing (`loop-refereeing` family)

Five decision points in the QL loop currently answered by the incumbent LLM
emitting JSON protocol (`crates/actuation-research/src/policy.rs`) map onto
typed questions:

| Rust decision point | Typed question(s) |
|---|---|
| `ql-next-act` (next exterior act) | choice over carrier `model\|capability\|internal_control`; noul "is the realisable intent already achieved"; score act-adequacy |
| `ql-propose-determination` (P5) | choice `close\|reopen`; choice adequacy; noul "claimed subject is verifiable" |
| `classify_stipulations` | one noul per stipulation: "was this stipulation violated?" |
| deep-mode depth (P4 only) | noul "genuinely local whole warranting independent treatment" |
| recurrence resolution (`resolve_Rij`) | choice over next position `P0..P5` |

Method: replay recorded decision contexts from retained run records, ask Jev,
and compare three readings — the incumbent LLM's recorded decision, Jev's typed
answer, and human review when it lands. Jev is a referee instrument, never the
determination (`pending-human-review` law unchanged). The decision-context
exporter from run-record JSON (`question-catalogue.json` → `state_shapes`) is
the first wiring task.

**First live result (2026-09-18, 150 replayed questions, zero API errors):**

| Question | n | agreement | Brier | mean conf |
|---|---|---|---|---|
| next-carrier | 21 | **90.5%** | 0.166 | 0.937 |
| requested-outcome | 20 | **90.0%** | 0.112 | 0.823 |
| intent-already-achieved | 20 | **95.0%** | 0.025 | noul 0.095 |
| next-position | 21 | 61.9% (chance 16.7%) | 0.456 | 0.542 |
| subject-verifiable | 20 | 100% (degenerate base rate) | 0.055 | noul 0.213 |

Latency p50 266ms / p90 423ms; 145.5K input / 5.1K output tokens. The failure
signature is consistent and informative: at genuine closure points Jev wants one
more capability act (both carrier misses and the intent-achieved miss are the
same two points); both requested-outcome misses are research circuits the loop
closed and Jev would reopen — exactly the class human review should look at.
Full detail: `emt2-report.md`; run record
`emt2/runs/jev-run-2026-09-18T02-01-52-947Z.json` (digest-pinned,
pending-human-review). These are agreement-with-the-incumbent numbers, not
correctness — substitutability evidence for the composed loop, and a named
review queue, not a verdict.

**The next-position result, in plain terms.** The question was: "here is what
the loop has just done and what the task needs — which QL position's
responsibility comes next?" Jev picked the right position 62% of the time;
random guessing is 17%, so it is clearly reading the situation, but far from
substitutable. Where it fails is not random: it misses *frame entries* (the
loop restarting at P0 and stepping to P1 — Jev expects work to continue rather
than re-ground) and *P1↔P5 swaps* (the loop jumping between gathering material
and candidate determination). The design reason is that the loop's next
position is partly **mechanical, not semantic**: allowance budgets per position
and the closure-routing rules decide some transitions before any judgement is
needed, and the question as posed did not include that machinery. Fix, in
order: (1) put the allowance state and the routing rule outcome into the
question state (the catalogue's `decision_context` already has an
`allowance_state` field the exporter left empty); (2) where a transition is
fully mechanical, it should not be asked of any model at all — Rust already
owns it; (3) re-measure on the semantic remainder only.

## Files

- `question-catalogue.json` — typed questions, criteria, decision-point mapping,
  state shapes (`actuation.jev-question-catalogue/v1`), written against the
  direct API primitives.
- `build_states.py` — builds `states.jsonl` / `truth.jsonl` / `manifest.json`
  from the retained run records
  (`python3 build_states.py --limit 100`).
- `harness.mjs` — fail-closed runner, no SDK dependency
  (`node harness.mjs --limit 200`); keychain-aware credential; writes
  digest-pinned run records.
- `skill/typesafe-ai/` — the vendored official TypeSafe agent skill.
- `sources/` — external third-party material kept as recorded, never asserted:
  `jev-rag-reranking-prompt-engineering-2026-09-20.md` (a use-perspective
  reranking walkthrough, with its auto-caption decode and the presenter's claims
  quoted per timestamp) and the raw transcript it was built from.
