# Series 1 round 7 — the vak arm runs live (2026-09-18)

> **The kernel-native control arm ran the corpus for the first time: 5/5
> completed, all objective checks passing, all circuits closed, at roughly
> half the model cost of free-English control — with the office sequence of
> every trial named by the QL-MEF kernel, not sampled from a model.**

```text
benchmark_revision   series1-round5-accept-2026-09-17 (same frozen corpus)
host                 rust-native-acceptance, release binaries at 3625cb3
                     (techne/deep-conjugate-allowance + merged act-rust/vak-control
                      + authored-office completion)
control instrument   /Users/admin/Central/Work/Quaternal-Logic/target/release/ql
                     (bind probe admitted live; digest-pinned in the run record)
model                zai:glm-5.3-flash (deliverable acts, authoring turns,
                     determination, closure evaluation)
arms on this corpus  classic / ql-direct / compressed / ql-deep / VAK — complete
```

## Results

| Task | status | checks | calls | tokens | wall | (classic / direct / compressed) |
|------|--------|--------|------:|-------:|-----:|--------------------------------|
| CODE | completed | pass | 4 | 19,411 | 255 s | 5/7,262 · 14/75,315 · 9/40,431 |
| EPISTEMIC | completed | pass | 8 | 42,167 | 563 s | 4/4,730 · 15/73,122 · 8/38,043 |
| RESEARCH | completed | pass | 9 | 47,830 | 598 s | 4/4,310 · 15/83,784 · 9/40,800 |
| RESTRAINT | completed | pass | 5 | 19,618 | 264 s | 2/957 · 7/27,161 · 8/29,673 |
| SKILL | completed | pass | 12 | 63,096 | 706 s | 6/9,320 · 22/133,491 · 14/79,995 |

**Vak totals: 38 calls / 192,122 tokens / 2,387 s — ×0.52 calls and ×0.49
tokens of the free-English ql-direct lane, ×0.79 / ×0.84 of compressed,
×1.81 / ×7.23 of classic.** On CODE the vak arm used fewer model calls than
classic (4 vs 5) while producing the full circuit record classic has none of.

The residues tell the office story; CODE's sequence is frame → material →
material → effect → evaluation → determination, each transition admitted by
the kernel against its positional law, with authored content (the fix, the
writes) supplied by the model only where the office demands it.

## What had to change to make this run honest

1. **The preserved lane never compiled.** Two unbalanced `json!` blocks, a
   fixture path one level outside the repo, and three request-builder
   mismatches against the captured kernel traffic (evidence placement, the
   whole step's grounding return, the bind probe's vacancy shape). All are
   repaired with the 9 fixture tests passing again — the fixtures are real
   captured kernel requests/responses, so the builders are pinned to what the
   instrument actually says.
2. **The authored-content seam.** Attempt 1 failed every task in ~1 s,
   fail-closed: the next-act walk reached P2 Affirm, whose faculty needs
   authored content, and the preserved lane refused to invent a carrier.
   The completion (3625cb3) keeps the refusal's discipline but makes it
   productive: the kernel still names the office (recorded in the act's vak
   witness), and one bounded model turn authors the act inside that office
   under the same exclusion law as the model lane. Law stays with the
   kernel; content stays with the model.
3. **One single-call variance failure.** The first SKILL trial died on a
   closure-evaluation reply that the strict parser refused ("controller
   closure verdict is not close or reopen") after the work was done and
   verified. The re-run completed clean; the failed record is kept as
   `vak-failed-S1-SKILL-001-closure-parse.json`. A malformed-turn retry is a
   candidate loop hardening, deliberately not changed mid-corpus.

## Reading

1. **Control by law is cheaper than control by prose, and lost nothing.** The
   arm dropped both control turns per act (interpret-return, next-act) for
   every office the kernel can serve directly, and paid one authoring turn at
   the offices that need content. Correctness held everywhere.
2. **The corpus now carries a complete controlled comparison**: classic (no
   QL structure), ql-direct (model control in whittled English), compressed
   (interpret ruled in code), vak (control named by kernel law), ql-deep
   (bounded conjugate return). Whether the structure buys anything beyond
   economy — closure records, office provenance, determination quality — is
   precisely what the pending human-review pass can now judge across five
   matched arms.
3. **Classic remains the economy floor** (21 calls, no QL structure). The
   vak arm at ×1.8 classic calls buys the entire circuit apparatus — every
   act office-attributed, every closure kernel-admitted, the record lawful
   end to end.

## Files

```text
vak-S1-<TASK>-001.json                        live vak runs, full records
vak-failed-S1-SKILL-001-closure-parse.json    the single-call variance failure (kept)
vak-attempt1-failed-S1-RESTRAINT-001-...json  the authored-refusal attempt (kept)
acceptance-run.mjs                            the recipe
```
