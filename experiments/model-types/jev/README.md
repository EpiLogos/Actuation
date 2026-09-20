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

## A named consideration — Jev for development, planning and CI classification

**Recorded 2026-09-20 at the owner's raising. Scope is not yet set: no target
surface is adopted, and this section names candidate surfaces so a later scope
decision has something concrete to land on.** It was prompted by the third-party
reranking walkthrough in `sources/` — the use-shaped case where Jev is dropped
into an existing pipeline's decision points rather than studied on its own.

The proposal: Jev's typed judgement is not only a runtime referee for the QL
loop (E-MT-1 / E-MT-2). It may natively fit the **classification work this
repository does on itself** — capability matrices, censuses, seam and ledger
classing, drift reconciliation — and the planning and CI work around that.

### Why the fit is structural rather than opportunistic

1. **Our classification work is many independent typed judgements per artefact
   family.** One question per matrix cell, per source file, per consumer seam —
   each with the same state shape and the same criteria. That is exactly Jev's
   batch form (independent questions over one state, answered in one round
   trip), and it is why the concurrency observation in the video bears on it.
2. **Criteria-as-policy is where our rules already want to live.** The
   steerability the video is really about is "the rule is a criteria document,
   not a retraining or a rewritten prompt" — which is the repo-content stance
   that rules be checkable and reviewable source. A criteria document under
   version control and review is the thing that makes a classifier's behaviour
   arguable instead of mysterious.
3. **The standing is already correct for it.** Every reading Jev produces here
   is instrument output. Our law already has that class, and the CI surface
   below already runs at exactly that standing.

### Candidate surfaces, as they exist today

| Surface | The classification | Its present standing |
|---|---|---|
| `ProjectCentral/user/capability-matrix.md` | 6×6 seed × field grid — 26 of its 36 cells currently `Unassessed`; capability → field placement currently recorded as `agent-inference` throughout | human ground, matrix itself "agent-generated candidate" |
| `.github/product-ground.pyz` (CI) | what class of drift each finding is (missing capability record / stale reference / wording) before the named reconcile path (which lives in the Central repo, not here) is applied | advisory — explicitly does not block merge |
| R8–R9 seam classification law | consumer seam → class, whenever a consumer moves | recorded in `docs/rust-refoundation/` |
| R11 JS retirement census | 81 remaining JS files → frozen-corpus tooling vs specimen-native research | recorded, frozen history |
| `docs/rust-refoundation/ledger.json` | per-source A/B/C/D decisions for any future migration phase | frozen history, never rewritten |
| PR path-relevance for planning | which gates and jobs a change actually implicates — today hand-listed as `paths:` filters in each workflow | planning aid |

### Conditions that would have to hold

- **Advisory, never a required check.** Two runs of the same commit can differ; a
  branch-required gate must not. The repo already draws this line — the
  capability-matrix drift check is advisory and says why. A Jev reading may
  annotate a PR or hand a human a ranked classing; it may not decide a gate
  outcome. Wherever a Jev reading would feed a check, the *criterion* must stay
  recomputable without a model, as a checkable rule in its own right.
- **Criteria as reviewed source.** If the criteria live inside a workflow
  prompt string, every classification is unreviewable. They belong in the repo
  as a document with the same review standing as `repo-content.md`.
- **Credential and spend are their own decision.** The harness reads
  `TYPESAFE_API_KEY` from the macOS login keychain; a GitHub runner has no
  keychain, so CI use means a billed key in CI — per-PR spend, secret hygiene,
  runner exposure. Cheaper than an LLM reranker is not free.
- **Batch by artefact family.** One round trip per family of questions over one
  state shape, not a shell loop of one question per call — our measured p50 is
  per question, and the concurrency ceiling is a vendor claim we have not
  measured.
- **Candidate status must be carried, not implied.** A Jev-classified matrix cell
  is `semantic-stochastic` instrument output and must be labelled as such in the
  artefact — the same way loop decisions stay `pending-human-review`. Nothing it
  classifies is promoted by being written down.

### What is not claimed

Nothing here is measured. The video's numbers are the presenter's, from an
unreviewed personal run. Our own live Jev results remain E-MT-1 (92% position
classification, chance 16.7%) and E-MT-2 (agreement with the incumbent, not
correctness). This would be a **third role** for the specimen — development-work
instrument rather than runtime referee — and if it is taken up it wants its own
question catalogue and criteria documents under a named experiment, not loose
CLI invocations. Whether it belongs in the E-MT-1/E-MT-2 family or as a new
experiment is the owner's call.

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
