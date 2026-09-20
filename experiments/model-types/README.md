# Model-type experiments — separated faculties over one actuation ground

Status: `first-live-results-landed` (2026-09-17). This directory sets up a new
class of experiments around model types that are not autoregressive chat LLMs, and
composes them with the existing agent-loop programme in `experiments/ql-runtime/`.

**The pairing this class exists to establish:** the LLM owns semantics
(generation, reflection, explanation); Jev owns typed classification across QL
positions and MEF lenses and referees the loop's decision points; the EBM reads
the bimba coordinate field for compatibility; Mercury owns fast candidate
generation. Rust keeps rules, budgets and execution — the vendored TypeSafe
skill's law ("code owns the workflow; the model supplies programmable common
sense") lifted one level: code owns the recurrence, the model types each own the
cognitive function natively theirs.

## The separation hypothesis

The loop programme's standing result is that a QL-native recurrence can be run on an
ordinary autoregressive LLM (GLM) through a JSON protocol: the model mimes every
cognitive function — next-act choice, determination, closure verdicts, deep-mode
depth judgement — by emitting JSON shapes on request. The hypothesis this class
tests is that those functions do not have to be mimed by one model, and that
separating them onto model types whose native output *is* the required function
gives better-calibrated, cheaper, faster, and more honestly-attributable loop
decisions.

The faculties, and the specimen that natively serves each:

| Faculty | What it answers | Native specimen |
|---|---|---|
| World formation | "What must stay invariant for two moments to belong to the same world?" | JEPA / EB-JEPA (latent prediction, action-conditioned) |
| Compatibility | "Do these candidate states cohere with the learned/declared field?" | EBM over the bimba map as grounding field |
| Typed judgement | "Answer this Noul / Choice / Score question about this state, with probabilities." | Jev (TypeSafe AI, System One API) |
| Possibility generation | "Produce candidate texts/acts, fast, in parallel." | Mercury 2.5 (Inception Labs, diffusion LLM) |
| Reflective explanation | "Say what happened and why, in language a person can judge." | Autoregressive LLM (GLM — the Series-1 incumbent) |

An EBM's low energy is compatibility with learned or declared structure, not value
or correctness; every reading these specimens produce is instrument output
(`semantic-stochastic` / `research` standing), never auto-promoted, and never a
substitute for the human-review determination law the loop programme already runs
under.

## The experiments

- **E-MT-1 — Typed classification across QL positions and MEF lenses** (`jev/`):
  Jev classifies each act of the retained QL-circuit runs into its position
  responsibility (scored against the loop's own recorded positions) and reads
  subjects through the 12 MEF lenses as instrument readings, reported as mass on
  the runtime's standing L1+L4′ refraction pair — never as assigned identity.
  **Live 2026-09-17: 92% ql-position accuracy (chance 16.7%), 0.867 standing-pair
  lens mass, p50 latency 238ms.**
- **E-MT-2 — Loop decision refereeing** (`jev/`): the QL loop's five model-facing
  decision points (`ql-next-act`, `ql-propose-determination`, stipulation verdicts,
  deep-mode depth, position resolution) are natively typed questions. This
  experiment replays recorded decision contexts from retained Series-1 runs and
  compares Jev's typed answers with what the incumbent LLM decided and with human
  review. Jev is a referee instrument here, never the determination.
- **E-MT-3 — World formation over the loop's own trace** (`jepa/`): the actuation
  loop's event traces are the environment. A JEPA trained on loop-event latents
  asks what invariants survive transitions — the Two Rooms pattern pointed at our
  own recurrence.
- **E-MT-4 — The faculty-composed loop** (`hybrid-loop/`): one loop, five faculties:
  Mercury generates candidates, the EBM gates them against the bimba field, Jev
  referees the typed decisions, GLM reflects and explains. Runs on the same
  Series-1 corpus under the same held-constant law as `classic` / `ql-direct` /
  `ql-deep`, as a fourth loop condition.
- **EBM field probe** (`ebm/`): the grounding experiment for the whole class. Does
  the bimba map, read as an energy field, separate viable from incoherent states?
  Phase 1 (implemented, runs live today with no API keys) tests graph-grammar and
  semantic potentials against corrupted relations on the real graph.

## Relation to the parallel agent-loop programme

This class is designed to *ride* the loop programme, not fork it:

- **Same task corpus.** E-MT-4's composed loop runs the frozen Series-1 corpus
  (`experiments/native-research/tasks.json`), so any faculty effect is measurable
  against the retained `classic`/`ql-direct`/`ql-deep` runs rather than against new
  tasks.
- **Same evidence law.** Held-constant digests, `execution_status` ≠
  `semantic_status`, fixture providers ineligible for capability-effect claims,
  determination `pending-human-review` — inherited unchanged from
  `comparison/series1/BENCHMARK-V0.1.md` and the loop-closure clarification.
- **Same specimen seam.** A new model type enters as a `ModelBody`
  (`crates/actuation-research/src/execution.rs`) or a policy-side referee; loop
  semantics stay in `policy.rs`/`relational.rs` behind the unchanged `RuntimeHost`
  seam. No QL semantics are rebuilt inside any specimen.
- **Shared decision points.** The question catalogue (`jev/question-catalogue.json`)
  names the exact Rust decision points it referees, so a later wiring pass touches
  policy code at named seams only.
- **Suite registration path.** New model kinds are ModelSurface material for
  ai-kit's `aikit.model-modality/v1` (Actuation consumes that contract as data);
  suite-level registration of Jev/Mercury/EBM surfaces is future work, noted in
  `specimens.json`, not done here.

## What runs today vs what is key- or GPU-gated

| Piece | State |
|---|---|
| `ebm/field_probe.py` | **Live-verified with real embeddings** (1,777 M/L nodes embedded in-graph 2026-09-18 after fixing three bimba-portable embedder defects). Separation: full graph 0.902 coarse / 0.523 sibling; **M0 focus 0.743 / 0.673** — the densest metaphysical region separates strongest, confirming the owner's hypothesis. |
| `jev/harness.mjs` + `build_states.py` + catalogue | **Live-verified end to end** against the TypeSafe System One API (key in login keychain); 200 questions run, results above. |
| Mercury 2.5 | **Credential + OpenAI-compatible seam verified** (smoke call; `reasoning_effort:"low"` needed for visible content); tok/s measurement belongs to the real A/B. |
| `jepa/` | Design + data path only; needs trace volume (more runs) and a torch/`eb_jepa` bootstrap before first training. |
| `hybrid-loop/` | Integration contract only; code is a later pass through named seams. |

Vendor performance claims (Jev ~193× speed / ~444× cost, Mercury 1107 tok/s) are
vendor claims recorded as such in `specimens.json`; this programme measures its
own numbers (Jev latency already measured; Mercury tok/s pending the A/B).
