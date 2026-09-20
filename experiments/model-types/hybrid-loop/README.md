# E-MT-4 — The faculty-composed loop

One loop, five faculties — the composed condition this whole class exists to test.
The architecture is the loop programme's own host/runtime split, kept exactly:
loop semantics stay in `actuation-research` (`policy.rs`, `relational.rs`) behind
the unchanged `RuntimeHost` seam (`actuation-runtime::loop_runtime`); what changes
is which model type serves each model-facing function.

The pairing law (owner, 2026-09-17): **the LLM owns semantics — generation,
reflection, explanation; Jev owns typed classification across QL positions and
MEF lenses.** This is the vendored TypeSafe skill's law ("code owns the workflow;
the model supplies programmable common sense") lifted one level: Rust owns the
recurrence — budgets, refusals, execution; the LLM owns meaning; each model type
owns the cognitive function natively its.

## The composition

| Faculty | Specimen | Seam |
|---|---|---|
| Possibility generation | Mercury 2.5 (`INCEPTION_API_KEY`, OpenAI-compatible `api.inceptionlabs.ai/v1`) | a `ModelBody` specimen over the existing transport pattern (`sdk.rs`) — the host's `call_model` carrier |
| Typed judgement | Jev at the five decision points (`../jev/question-catalogue.json`, loop-refereeing family) | a policy-side referee: decision contexts go out as typed questions, answers come back as probabilities |
| Compatibility gate | EBM over the bimba field (`../ebm/`, Phase 2 head + Phase 3 wiring) | candidate acts/transitions scored before execution; refusals are typed, with the energy attached as evidence — the `policy.rs` allowance-schedule refusal pattern, seconded by the field |
| Reflective explanation | GLM (Series-1 incumbent, `ZAI_API_KEY`) | synthesis text and rationales; the only faculty whose output is free-text claims |
| World formation | JEPA over loop traces (`../jepa/`) | offline at first: the learned latent field read against live traces; a later pass could gate recurrence on predicted-latent viability |

Judgement discipline in the composed loop: where Jev and the incumbent disagree,
the disagreement is **recorded as evidence, not silently decisive**; the
determination stays `pending-human-review`, and no faculty output is promoted on
its own authority.

## The A/B

`ql-faculty` runs as a fourth loop condition beside `classic` / `ql-direct` /
`ql-deep` on the same frozen Series-1 corpus
(`experiments/native-research/tasks.json`), under the same held-constant digest
law and the same run-record shape (`comparison/series1/BENCHMARK-V0.1.md`). The
comparison that matters is within-host and within-task: does moving a function
onto a model type natively shaped for it change calibration, cost, speed, or
capability-effect outcomes — without changing the loop's semantics?

## Falsification conditions

The separation hypothesis dies if any of these hold:

- Jev's typed answers are no better calibrated than the incumbent's JSON-protocol
  answers at the same decision points (E-MT-2 answers this first, offline);
- the composed loop's capability-effect outcomes degrade against `ql-direct`
  under human review;
- generation speed (Mercury) changes outcome quality — which it should not if
  generation really is separable from judgement;
- the EBM gate separates nothing on the real graph (Phase 1 already measures
  this) or its learned head only reproduces position classes.

## Implementation path (ordered, through named seams only)

1. **Decision-context exporter** — run-record JSON → typed decision contexts.
   Shared with E-MT-2; pure addition, no policy changes.
2. **Offline refereeing** — Jev scores replayed contexts; calibration report
   against recorded incumbent decisions. No loop changes.
3. **Mercury `ModelBody`** — speed-regime A/B on the corpus with generation
   swapped, judgement unchanged. One new specimen body; no policy changes.
4. **EBM gate** — behind the Phase-2 learned head, typed refusal wired at the
   `ql-next-act` acceptance point.
5. **The composed condition** — a policy variant consumed by the same
   `ResearchHost`; run manifests pin every faculty's identity and revision in
   `basis` fields.

`policy.rs` is in-flight code; steps 4–5 land on a branch through proposal, not
silently. Suite-level registration of the new model kinds (ModelSurface material
for ai-kit's `aikit.model-modality/v1`, which Actuation consumes as data) is
future work recorded in `../specimens.json`.
