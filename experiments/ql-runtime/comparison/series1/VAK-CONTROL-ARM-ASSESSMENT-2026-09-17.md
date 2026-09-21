# Vak-control arm — assessment (2026-09-17)

Subject: `origin/act-rust/vak-control` (commit 891457e, preserved WIP from the
parked `/private/tmp/actuation-vak` worktree) and the design question it
carries: kernel-admitted Vāk control vs whittled-English model control.

## What the lane is

A third control arm beside `Model` (free English) and `Compressed`
(hand-ruled interpret-return), selected explicitly at the run seam
(`QL_VAK_CONTROL=1` + `QL_VAK_BIN=<path to ql>`; the two flags are mutually
exclusive and refuse silently-defaulting):

- **interpret-return**: the structural facts of a returned difference
  (carrier kind, operation success, content presence) map through an explicit
  table onto a Vāk relation operator — failure→P1, successful
  read/list→P1, write→P2, unshaped capability→P3, evaluation-shaped→P4,
  delivered model content→P5. The QL-MEF kernel (`ql vak compose`) admits the
  reading against its registry and positional law; the admitted operator's
  own office is the destination.
- **next-act**: the kernel names the next act from the circuit's positional
  field — the leading vacant office's faculty — or issues a closure request
  when no exterior act remains.
- **Binding is fail-closed**: the exact `ql` binary is digest-pinned at bind
  and re-verified on every call; a refused or mismatched kernel turn is an
  error. Control never degrades to the model.
- Determination synthesis and closure evaluation stay with the model, as in
  the other arms.

The fixture set (9 tests) pins the glyph table to the kernel's own
operator-position law and the request/response shapes to captured real
kernel traffic — the parser is proven against what the instrument actually
says, not an imagined envelope.

## The design question

Vak control is not a rival to whittled English on the same axis; it is the
terminal point of what the whittling was reaching for. The English prompts
try to hold the model to the loop's own semantics with progressively refined
prose; each refinement spends tokens and still leaves the decision to
sampling. The vak arm moves those two decisions (where a difference belongs,
what the next act serves) into law — a deterministic table admitted by the
kernel — and leaves the model the parts that need a model: the deliverable
acts, the determination synthesis, the closure judgement.

The round 5 corpus gives the cost case: ql-direct spends ×3.5 calls and
×14.8 tokens against classic for identical verification outcomes. Most of
the extra calls are control turns (interpret + next-act per act cycle).
Vak control removes them entirely; compressed control (queue item 3) removes
half. If the compressed arm's results confirm that code-ruled interpret
loses nothing in correctness, the vak arm is the same move completed.

## State and honest remainers

- **Merges cleanly** with the conjugate-allowance branch
  (`techne/deep-conjugate-allowance` @ 171a80a): auto-merge, no conflicts,
  both touch policy.rs in disjoint regions.
- **Never run E2E.** Unit behaviour is fixture-pinned; no live corpus run
  has exercised the arm. The natural next increment is one direct-mode
  corpus pass under `QL_VAK_CONTROL=1` once the deep and compressed arms
  have had their runs — it attacks the ×3.5 call multiple directly and
  would join the corpus as its own condition.
- **Instrument dependency**: needs a built QL-MEF `ql` binary at
  `QL_VAK_BIN` with the pinned owner revision (`4e35f499…`) matching the
  AIKit operative-syntax acceptance the kernel validates against. A run
  recipe must name and verify that binary the way the acceptance runner
  names the owner instrument.
- **One structural note**: the kernel-named next act chooses capability by
  *office* (the leading vacant office's faculty). Tasks whose realisable
  intent needs a specific capability the office ordering does not imply
  will lean on the model deliverable call to carry the choice anyway. The
  corpus (read → write → test shapes) fits the office law well; that is
  worth confirming live rather than assuming.

## Recommendation

Keep the lane; it is well-formed, fail-closed, and correctly scoped (acts,
determination and closure stay with the model). Sequence it after the deep
and compressed arms as the third cost-reduction increment, run live on this
corpus, and let the records — not the design — settle the Vak-vs-English
question.
