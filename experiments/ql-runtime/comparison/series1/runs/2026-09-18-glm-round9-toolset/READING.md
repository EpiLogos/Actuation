# Series 1 round 9 — the toolset matrix (2026-09-18)

> **The paradigm question answered on the full corpus: the implicit toolset
> carries the QL form on its own — adding the explicit relational prompt
> changed nothing measurable — and the toolset arm is the cheapest QL arm
> ever run, at essentially classic's call economy with the full circuit
> record.**

```text
benchmark_revision   series1-round5-accept-2026-09-17 (same frozen corpus, five tasks)
host                 rust-native-acceptance, release binaries at 64331c1/3112f29
                     (single experiment tree per TESTING-PROTOCOL.md)
arms                 A toolset implicit (self check)      B toolset + relational prompt
                     C toolset, model close-check         D toolset, jev close-check
```

## The matrix

| arm | result | calls | tokens | wall |
|-----|--------|------:|-------:|-----:|
| A implicit, self check | 5/5 | 23 | 37,809 | 411 s |
| B + relational prompt | 5/5 | 21 | 33,580 | 371 s |
| C model close-check | 5/5 | 29 | 48,617 | 598 s |
| D jev close-check | 4/5 + 1 recorded refusal | 23 | 42,838 | 529 s |

Baselines on the same corpus: classic 21 / 26,579 / 356 s (no QL record);
vak 38 / 192,122 / 2,387 s; compressed 48 / 228,942 / 2,219 s; ql-direct
73 / 392,873 / 4,346 s; ql-deep 107 / 715,292 / 6,495 s.

## Answers

1. **Does the implicit approach need the explicit prompt?** No. Arm B ran
   with the full relational standing prompt appended to every turn and came
   out marginally *cheaper* than A (×0.89 tokens, ×0.91 calls) — within
   noise. The office law carried in the tool descriptions is sufficient; the
   model needs no theory of QL to act inside it.
2. **Does QL's presence among the tools bake in the paradigm?** Yes — the
   residue trails show lawful structure with zero explanation: material
   chains at P1, effects at P2, evaluation at P4, determination at P5
   (CODE: `frame → P1×5 → P2 → P4 → P5`). `close` was the only loop verb the
   model needed in nearly every trial; `situate` stayed almost unused — the
   tool offices settle the field by themselves.
3. **Closure-check options, priced.** Self is free (23 calls). A separate
   model judge costs ×1.26 calls, +10.8k tokens, +187 s. Jev costs no
   model-body calls (instrument latency ~150–800 ms) and refused one
   borderline closure honestly — RESTRAINT closed at
   `close 0.45 / reopen 0.55` and the refusal, with the full distribution,
   rides the record (`armD-jev-refusal-S1-RESTRAINT-001.json`). Jev sits
   close to the decision boundary on terse syntheses; that conservatism is
   fail-closed and inspectable, exactly what a referee should be.
4. **Variance, disclosed.** One unparseable-model-reply failure each in arms
   B and C (preserved as `*-variance-fail-*.json`, retried clean); arm D's
   first two passes failed on instrument wiring (PATH, then stdin contract —
   fixed in 64331c1, 3112f29, failed trees preserved as
   `round9-toolset-close-jev-{envfail,stdinfail}` in cache).

## Standing

The toolset arm is now the strongest QL condition on the corpus: classic's
call economy (23 vs 21 calls, ×1.42 tokens) with the complete circuit record
that classic lacks, and ~×0.17 of the vak arm's tokens for the same record.
Combined with the jev+MEF thread's reconciliation (landing-office truth
raises the code table to 74%; jev's residual failures are structured — it
reads successful reads as "effect" where the law says "material", a one-line
criteria refinement away), the implicit paradigm has both a working base and
a measurable referee.

Determination remains `pending-human-review` throughout; the review pass
over arms A–D plus rounds 5–7 is the deliverable only the owner can make.
