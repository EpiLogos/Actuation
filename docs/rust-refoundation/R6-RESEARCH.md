# R6 — first-class research refoundation

Basis: branch `act-rust/r6-research-refoundation` at `0dfa401965e182198b4024fb79dc9492285cf14a`
(on top of the R5-accepted main `94591372975f45f7f42f35cbabbccfaf6936355e` line).
Machine record: `R6-DISPOSITION.json` alongside this file. The frozen per-source
classifications live in `ledger.json` (R6 phase: A=1, B=73, C=6, D=37) and are
not rewritten here; this record states their executed standing.

## What became native

One research crate now owns experiment orchestration and provenance:

- `crates/actuation-research` — epistemic records/store/worlds, bounded
  process/RPC, source bindings, evidence and sanitisation, the shared
  Classic/Direct/Deep execution path over `actuation-runtime`, the model-driven
  Direct/Deep policy, the Prime programme with a thin inherited language
  boundary, persistent SDK transport, comparison/review mechanics, the
  read-only DSH inspection projection, and executable readiness over the eight
  retained QLDR scenarios.

Integration fault fixed on the way: `prime.rs` bound a conditions catalogue
that existed only as JavaScript. The catalogue is now also published as
`experiments/ql-runtime/prime/conditions.json` — a provenance-stamped JSON twin
of `conditions.mjs` (origin blob `7a995256…` at `324231d`) — and the crate
unwraps the envelope, so the native and JavaScript conditions cannot drift
silently.

## Executed dispositions by class

- **A (public contract, 1 source).** `contracts/epistemic-cultivation.test.mjs`
  keeps schema/ref/null/extension and error-status semantics; the native
  counterpart executes the same law in `records.rs`/`store.rs` with the same
  schema (`actuation.epistemic-record/v0`), the same eight record kinds and the
  same six access kinds.
- **B (generic semantics, 73 sources).** The native counterparts are published
  and tested. The JavaScript sources remain served as migration oracles until
  the R7 cutover law replaces them; no served Node source was deleted in R6.
- **C (specimen-native, 6 sources).** DSH UI/plugin, Pi and Pydantic hosts stay
  in their native languages behind explicit bounded bridges; the crate speaks
  to them through one shared SDK projection (`sdk.rs`) rather than per-host
  wrappers.
- **D (authored research source, 37 sources).** Clarification documents,
  source locks, condition/task authorship and fixtures are retained
  byte-exactly with provenance; several are now compile-time bound
  (`include_str!`) so drift breaks the build instead of silently diverging.

## QL ownership

QL-MEF remains the sole formal owner. The owner instrument pins the accepted
main `e753efc91f62b5b2af09e0a852c5063e366eccbe` through `ql-core`, `ql-mef`,
`ql-cli` and `ql-wiki`; the crate binds the instrument by binary digest and
correlated replies, with no local formal fallback and no duplicate algebra.
The historical harmonic research head `42d36ed7…` stays explicitly
`closed-unmerged` in the source lock, and the Prime body is pinned to
`v0.9.4` (`f771dfce…`).

## Executed verification

- `cargo test --workspace` — 150 passed, 0 failed (82 in
  `actuation-research`, including end-to-end Classic and Direct runs over a
  scripted formal owner and all eight native readiness scenarios with real
  Node specimen verification).
- `cargo fmt --all --check`; `cargo clippy --locked --workspace
  --all-targets -- -D warnings`.
- `native-rust` gate green on ubuntu-latest and macos-15 at the published
  commit (run 34654916739), alongside migration-oracle, native-sdk (dsh/pi/
  pydantic), owner-instrument (both OSes), product-ground, structural,
  native-cli and activity-contract checks.
- The Node-native research suites (foundation, deep-ql, prime,
  epistemic-cultivation) remain green as migration oracles.

## Not claimed

Provider evidence, owner-machine evidence and human acceptance are explicitly
not assessed. The live Prime campaign still requires a configured
provider/model on the owner machine. No served Node source was removed; that
is the R7 cutover, not R6. The draft must not merge as an R6 acceptance.
