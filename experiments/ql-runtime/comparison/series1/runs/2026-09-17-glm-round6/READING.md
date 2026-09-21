# Series 1 round 6 — results reading (2026-09-17)

> **Two post-round-5 arms, both corpus-complete on the five task families:
> ql-deep with the typed conjugate allowance (5/5 closed — the round 5 wedge
> is gone), and ql-direct under compressed control (5/5 closed, roughly half
> the cost of free-English control). Determination: pending-human-review.**

```text
benchmark_revision   series1-round5-accept-2026-09-17 (same frozen corpus as round 5)
host                 rust-native-acceptance, release binaries from the acceptance
                     worktree at techne/deep-conjugate-allowance (171a80a/25801e0)
model                zai:glm-5.3-flash (supplied body, one-shot-json)
budget               max_calls 64, max_contexts 32
recipe               acceptance-run.mjs (BASE/BODY now env-selectable; model body
                     program moved to the durable ~/.cache path)
raw trees            ~/.cache/actuation/round6-deep-experiments/, round6-compressed-experiments/
```

## Arm 1 — ql-deep with the typed conjugate allowance

Round 5 finding: deep mode wedged — `Outcome::Conjugate` forced on every
determination, 13 conjugate circuits spawned as depth-0 siblings on
RESTRAINT, budget exhausted with no closure (63 calls, 474,824 tokens).

The fix (Actuation `techne/deep-conjugate-allowance`, commit 171a80a):
conjugate cycles consume a typed allowance declared with the frame schedule
(key `conjugate`, 2, with the standard one-recorded grace); while it holds
the P′ face executes per the owner's conjugacy law; when spent, the typed
refusal rides the determination (`conjugate-allowance-refused:…` in
`unresolved_refs`) and closure proceeds on the ordinary law.

| Task | status | checks | calls | tokens | wall | conjugate children | closure |
|------|--------|--------|------:|-------:|-----:|-------------------:|---------|
| CODE | completed | pass | 20 | 129,391 | 1,260 s | 1 | yes |
| EPISTEMIC | completed | pass | 21 | 152,435 | 1,450 s | 1 | yes |
| RESEARCH | completed | pass | 23 | 163,374 | 1,388 s | 1 | yes |
| RESTRAINT | completed | pass | 15 | 65,455 | 696 s | 1 | yes |
| SKILL | completed | pass | 28 | 204,637 | 1,701 s | 1 | yes |

**Totals: 107 calls / 715,292 tokens / 6,495 s — vs classic 21 / 26,579 /
355 s; vs ql-direct 73 / 392,873 / 4,346 s.**

Reading:

1. **The wedge is fixed and the fix is law-like, not lucky.** Every task ran
   exactly one conjugate cycle and then closed. The uniform structure across
   five independent tasks shows the allowance binding behaviour: the deep
   loop takes its backward reading and terminates.
2. **The previously failing task now closes in 15 calls / 65 k tokens** —
   against 63 calls / 475 k tokens and a failed status pre-fix.
3. **The P′ face has a bounded, knowable premium**: ×1.5 ql-direct calls,
   ×1.8 tokens. Whether that premium buys anything (the conjugate backward
   reading improving determinations) is precisely what the human review pass
   over deep vs direct records can now judge — the corpus pairs both faces
   on every task.

## Arm 2 — ql-direct under compressed control

Compressed control (existing `QL_COMPRESSED_CONTROL=1`): interpret-return is
decided by the loop's own law in code — no model call — with the rule
recorded in the witness; the model keeps acts, determination, closure.

| Task | status | checks | calls | tokens | wall | (round 5 ql-direct) |
|------|--------|--------|------:|-------:|-----:|--------------------|
| CODE | completed | pass | 9 | 40,431 | 347 s | 14 / 75,315 / 755 s |
| EPISTEMIC | completed | pass | 8 | 38,043 | 379 s | 15 / 73,122 / 768 s |
| RESEARCH | completed | pass | 9 | 40,800 | 375 s | 15 / 83,784 / 986 s |
| RESTRAINT | completed | pass | 8 | 29,673 | 320 s | 7 / 27,161 / 285 s |
| SKILL | completed | pass | 14 | 79,995 | 798 s | 22 / 133,491 / 1,552 s |

**Totals: 48 calls / 228,942 tokens / 2,218 s — ×0.66 calls, ×0.58 tokens,
×0.51 wall clock of free-English ql-direct; ×2.29 / ×8.61 of classic.**

Reading:

1. **Code-ruled interpretation loses nothing measurable on this corpus.** All
   five tasks pass every objective check, closures and reentry records are
   well-formed, and cost drops by roughly half across calls, tokens and wall
   clock. The one task where compressed did not beat the baseline on calls
   (RESTRAINT, 8 vs 7) still passed.
2. **This is direct evidence for the vak direction**: if hand-ruled
   interpret loses nothing, the remaining question for kernel-native control
   is only whether the kernel's office law names better acts than the model
   does — see `../VAK-CONTROL-ARM-ASSESSMENT-2026-09-17.md`.

## Environment incident (recorded honestly)

The first compressed SKILL attempt failed with `owner instrument
unavailable`: the acceptance worktree at `~/.cache/actuation/acceptance` was
deleted from outside the session mid-run (cache directory mtime 19:53 BST;
nothing in this session's transcript issued a deletion). The failed record
is preserved as `failed-S1-SKILL-001.json`. The worktree was restored per
the standing recipe (worktree add at 25801e0, release rebuild of
`actuation-research` and the owner instrument) and the trial re-run passed
at 797.9 s. **The deletion is unexplained and worth the owner's attention** —
it removed a built worktree mid-experiment once already; the recipe makes
restoration cheap, but the cause is unknown.

## Files

```text
deep-S1-<TASK>-001.json        ql-deep with conjugate allowance, full run records
compressed-S1-<TASK>-001.json  ql-direct under compressed control
failed-S1-SKILL-001.json       the owner-instrument-loss attempt (kept as evidence)
acceptance-run.mjs             the recipe (BASE/BODY env-selectable)
```
