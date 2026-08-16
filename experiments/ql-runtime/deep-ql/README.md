# Deep QL runtime profile

This directory implements the Deep QL profile above the frozen Direct Core foundation. It does not modify the Foundation source or the completed Pi, Pydantic AI or Native baseline host implementations.

## Foundation pin

- merged Foundation branch point: `7690069846eb6fc89f6aa78dcf7aab886ac7c737`
- frozen executable runtime candidate: `11c7be5767735c25f49139906b59398bcdf3bf42`

Direct Core semantics remain invariant: real Circuit state; 4+2; all `Rij`; `π 0/1` / `ρ 1/0`; difference before interpretation; explicit P5 determination; `R51/R52/R53/R54/R50` as continuation of the same open process; `QLClosure` as the only positive closure; `ReentryDelta`/P0+ only after closure; Run != Closure.

## Deep profile

- `operators.js` implements fresh-context whole/current-position conjugation (§24) and P4-oriented recursive depth (§25).
- `operator-session.js` makes those operators observable through portable conjugate/child/reopening lifecycle events without changing the frozen host contract.
- `formal/pairing-grammar.js` implements the corrected A/B/C pairing grammar, canonical cross-pass relations and D1/D2/D3 2/3/4-element conjugate-modulation frames from `../QL-PAIRING-SQUARES-CLARIFICATION-08-14-2026.md`.
- `conformance/` exposes the 61 stable required QLC IDs as substantive executable assertions plus portable schemas.
- `typing-corpus/` materialises the shared 100-case semantic benchmark with stable identities, benchmark provenance and optional real human witnesses.
- `render/` renders Direct, Conjugate and child Circuit state without treating carrier prose as QL meaning.
- `comparison/` compares full portable semantic signatures rather than relation strings alone.
- `convergence/` proves deterministic structural/conformance convergence across the three baseline host adapters. It does **not** claim model-capability improvement.

## Pairing/square correction

The earlier Deep branch incorrectly collapsed the three within-pass families into `D/O/ABC` and treated the resulting fixture as research-only. That fixture has been removed.

The corrected formal basis is:

```text
A = (0,1) (2,3) (4,5)
B = (1,2) (3,4) (5,0)
C = (0,5) (1,4) (2,3)
```

The 3×3 apparatus contains nine family entries, eight unique oriented pair structures and seven unique unordered address tetrads. Family/orientation provenance remains authoritative where raw address sets coincide.

For the software-facing conjugate modulation of a selected A/B/C pair:

```text
D1 = 2 elements
D2 = 3 elements, with one left/right conjugate projection
D3 = 4 elements, the full square
```

The grammar is specified formal structure. Whether an agent should invoke a square/modulation on a task is an experimental policy question.

## Review policy

`../TYPING-CORPUS-REVIEW-CLARIFICATION-08-14-2026.md` governs corpus review. The 100 cases are an executable semantic benchmark, not a 100/100 manual annotation gate.

More importantly, deterministic conformance is **not** product acceptance. The actual human/product question is tested under `../comparison/series1/`: does Classic vs Direct QL vs Deep QL change ordinary chat/coding capability, reliability, recovery, unnecessary work, token/call cost and latency?

## Commands

From this directory:

```text
npm test
npm run conformance
npm run corpus
npm run corpus:ndjson
npm run series1:preflight
```

`npm run readiness` now means **structural readiness** only and outputs `capability_effect_evidence_ready:false`. `npm run review` records deterministic structural traces only.

The live experiment is run through `../comparison/series1/run.mjs` / the `QL Series 1 Live` workflow. Fixture providers are explicitly ineligible for capability-effect claims.

## Current readiness boundary

Structural/conformance readiness is green across the baseline Pi, Pydantic AI and Native adapters. That proves the runtime mechanics and portable semantics are reproducible.

**Live capability-effect readiness is separate and is not yet a result merely because this branch is green.** A Series 1 result requires a configured real model/API credential, a real provider path for the selected host, matched Classic/Direct/Deep conditions, and an independent judge for semantic/chat tasks.

Genuinely unresolved computational structures stay under `extensions/` / `research/`, including state64 bit semantics, literal epogdoon metrics, higher topological control and undeveloped MEF/context technological roles.