# QL pairing / square clarification — 2026-08-14

**Status:** normative clarification for the Deep QL experiment profile.  
**Amends:** `ql-agent-spec-08-13-2026.md` §26 and the research wording in the development protocol / #108 where that wording treated the already-defined pairing grammar as unresolved research.  
**Does not amend:** frozen Direct Core recurrence, `Rij`, closure, re-entry, §24 conjugation, or §25 recursive depth.

## Why this clarification exists

The first Deep QL implementation under-derived the fuller pairing material and encoded the three within-pass pair families as `D`, `O`, and `ABC`, then placed the whole structure behind a research-only, non-controlling `ql.harmonic` fixture.

That representation is not faithful enough for the Deep profile. The formal pairing grammar is already determinate. What remains experimental is **how and when an agent should use these structures as operational aids**, not whether the grammar exists.

## Source-derived formal basis

Source basis: `ql-musical-derivation-v3.md` §§II-2.4–II-2.7 and §II-3.4, together with `QL-SOFTWARE-FACTORY-DEEP-QL-INTEGRATION-FOUNDATIONS`' distinction between invariant QL canon and progressively executable kernel semantics.

The six universal pairing families partition 3:3:

### Within-pass families

```text
A = {(0,1), (2,3), (4,5)}          adjacent-identity
B = {(1,2), (3,4), (5,0)}          offset-transition
C = {(0,5), (1,4), (2,3)}          converse-mirror
```

`A`, `B`, and `C` operate within either face/helix. They are distinct families and MUST NOT be collapsed into one `ABC` family.

### Cross-pass / inverse-pass families

The fuller source also preserves:

```text
D1 = {(n,n') for n=0..5}            same-position cross

D2-transform = {(n,(n+1)')}         forward cross-position
D2-require   = {(n,(n-1)')}         backward cross-position
D2-complete  = {(n,(5-n)')}         mirror cross-position

D3 = A, B, C applied on primed positions
```

These definitions remain queryable formal topology. They do not imply automatic traversal or mandatory invocation.

## Nine-square apparatus

For every A/B/C pair `(x,y)`, the corresponding full square is:

```text
{x, y, x', y'}
```

There are three families × three pairs = **nine square entries**.

The source explicitly identifies `A-square-2 = C-square-3`: both use the ordered pair `(2,3)` and its primed counterpart. Therefore there are **eight unique oriented pair/square structures** across the nine entries.

A further implementation-level distinction matters when a square is reduced to an **unordered set of four addresses**: `B-square-3` uses `(5,0)` while `C-square-1` uses `(0,5)`. Those are different family/orientation relations but contain the same four addresses `{P0,P5,P0',P5'}`. Consequently the nine structurally meaningful entries yield **seven unique unordered tetrad element-sets**.

The runtime MUST preserve family and ordered-pair provenance. It MUST NOT collapse `B3` and `C1` merely because their unordered address sets coincide.

## Software-operator clarification supplied 2026-08-14

For the agent-loop implementation, `D1`, `D2`, and `D3` also name **degrees of conjugate modulation of a selected A/B/C pair**. This is the software-facing cardinality rule supplied for the experiment programme:

```text
D1: 2-element frame
    the selected original A/B/C pair, treated as the conjugated pair-frame

D2: 3-element frame
    the original pair plus exactly one conjugate projection
    variants: left projection or right projection

D3: 4-element frame
    the original pair plus both conjugate projections
    this is the full square
```

This operator reading does not erase the source-derived cross-pass topology above. The implementation MUST preserve provenance indicating whether an observation refers to:

1. the canonical pairing relation (`A/B/C`, same-position cross, cross-position cross, primed invariance); or
2. the software conjugate-modulation degree (`D1/D2/D3` 2/3/4-element frame).

## Runtime consequence

The corrected Deep profile MUST provide deterministic operators that can:

- select an A/B/C family and pair index;
- return its direct pair and conjugate pair;
- materialise the nine-square apparatus;
- construct D1/D2/D3 conjugate-modulation frames with exact cardinality;
- retain family, ordered-pair, face and source provenance even where raw address sets coincide;
- reject invalid families, pair indices, D-levels, and ambiguous D2 side selection.

The grammar itself is **specified formal structure**, not a research claim.

Whether a task benefits from invoking D1, D2, D3, a particular square, or any harmonic relation is an **experimental agent-policy question**. No pairing operator is mandatory on every run.

## What remains genuinely open

Structures whose computational semantics are still not specified remain experimental/research extensions, including at minimum:

- binary semantics of the `2^6 = 64` runtime-state field;
- literal epogdoon/9:8 runtime metrics;
- higher-topological or homotopy control rules;
- technological roles for MEF operations that have not yet been operationally defined.

The existence of QL canon is not contingent on those implementation questions.