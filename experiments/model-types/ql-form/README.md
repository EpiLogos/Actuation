# QL-form tests — the validity floor

This directory holds the tests that must come FIRST: checks on whether the QL
form itself is happening in the retained runs, and instrument questions asked
in the QL form (type-fullness, concrescence) rather than mimicry of the loop's
bookkeeping. Standing lesson of 2026-09-19: a model-type test that measures
"can the instrument predict the loop's recorded routing" produces numbers that
look like results but are fixture behaviour — test validity is established
before any number is reported.

## `census_ql_form.py` — is the form happening?

Structural census over the retained Series-1 ql-* records (2026-09-19):

- 87 ql records; 55 are later-round record shapes that carry no circuit and
  are not form-checkable from the record — named, not pooled.
- The 32 form-checkable records: frame declared 32/32; transitions lawful
  29/29 checked; allowance-lawful 28/29 checked with **one real violation**
  (deeplane S1-SKILL-001 ql-deep consumed P1:9 against schedule 6 + grace 2 —
  an overrun beyond the one recorded grace extension); closure-lawful 28/28
  checked; the 5→0 return visible in 29/29 checked records (deep child frames
  carry the parent's initiating intent; reopen payloads recorded).
- Fidelity finding: the deeplane host restates the allowance schedule in its
  control payloads; the native host never does (14/32 vs absent-in-host-shape
  18). Not a form violation — the schedule lives in Rust — but it means
  replay states built from native records cannot show the referee the loop's
  budgets.

Verdict: the corpus stands on the QL form well enough to test against; the
one allowance overrun is named and belongs in the review queue.

## `build_concrescence_states.py` + concrescence family — the determination question

The QL-native question for a typed-judgement instrument, in the owner's form:
"given/if-we-assume that #0, given there is enough balance between and
maturity of #1-#4, I can state that #5 is/appears to be the case."

Three typed questions over each closed circuit's whole state (frame intent,
middle residues, candidate determination; loop bookkeeping excluded):

- `determination-warranted` (noul) — is #5 warranted given #0 and #1-#4?
- `middle-maturity` (score 0-3) — how mature and balanced are #1-#4?
- `closure-appropriate` (choice close/reopen) — asked of the whole.

**Instrument validation by degradation ladder** (no truth needed): each
circuit is synthetically gutted (strip P1 material; strip P4 evaluation;
strip the whole middle; shallow the determination to "Done."). An instrument
that reads type-fullness must read gutted circuits as less satisfied.

### First live result (2026-09-19, 13 distinct circuits, 195 deduped readings)

Validity verdict is MIXED, and that is the headline:

- **The score channel reads type-fullness correctly.** middle-maturity tracks
  the ladder: intact 2.10 → strip-P1 1.22 → strip-P4 2.03 → strip-middle
  0.91 → shallow 1.71. Jev sees what was removed, in the expected direction
  and magnitude.
- **The warrantedness noul is weakly coupled to the middle.** intact 0.689 →
  strip-middle 0.594, and only 2 of 13 circuits flip yes→no when the ENTIRE
  middle is removed. The channel punishes an empty determination (shallow:
  0.462) but barely notices that nothing supports it. As posed, the question
  does not yet hold judgement across types — it must be restructured
  (e.g. condition the noul on the maturity reading, or ask the conditional
  in the owner's two-step form) before any agreement number counts.
- **The closure-appropriate choice produces the real review queue.** With the
  whole-circuit state, Jev would reopen 11 of 13 distinct circuits (9 at
  confidence ≥ 0.7), spread across all six Series-1 tasks — not just the two
  RESEARCH closures the thin-state E-MT-2 test flagged. Truth here is the
  loop's own recorded closure (pending-human-review); the disagreement is
  evidence for review, not an error to fix. Note the contrast: the thin-state
  test showed 90% agreement — with almost nothing to judge, the instrument
  mimicked; given the whole, it judges.

Standing law unchanged: readings semantic-stochastic, promotion none,
determination pending-human-review.

Reproduce: `python3 build_concrescence_states.py` then from `../jev`:
`node harness.mjs --states ../ql-form/concrescence/states.jsonl --truth
../ql-form/concrescence/truth.jsonl --out-dir ../ql-form/concrescence/runs`.
