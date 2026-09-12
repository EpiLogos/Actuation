# R8 + R9 — cross-product harmonisation and pre-local convergence

Basis: branch `act-rust/r8-r9-harmonisation` fresh from the R7-accepted main
`4af495f6509bb0dd779112a225d71a9df8145b6e` (R7 served by PR #69; #70 is the
register day-close on the same main). Machine record:
`R8-R9-DISPOSITION.json` alongside this file. The frozen per-source
classifications live in `ledger.json` (phase R7–R9: 19
`harmonise-build-tooling`, 16 `preserve-and-harmonise`; phase R9: 20
`preserve-source-reconcile-projections`) and are not rewritten here; this
record states their executed standing.

## R8 — what the consumer survey found, and what changed

The current accepted mains were reinspected before any change (the Live
boundary law): ai-kit `586b85e`, Workcell `e4e40a9`, Factory `f12b36c`,
O:I `11e9790`. Per the seam classification law:

```text
consumer seam                                   classification
O-I surfaces.json actuation install descriptor  stale Node/package assumption → owner change required
O-I suite/manifest.json actuation test contract stale npm assumption → owner change required
O-I development-field-ci.py (actuation lines)   genuine owner-native change required
O-I development-field cut.json (actuation pin)  stale cut pin → advanced
O-I continuous-work sources lock + workflow     stale Node launcher binding → owner change required
O-I suite/native-protocol.json + mainline.json  stale revision pins → re-pinned
O-I current_main_install.rs (generic dispatch)  already compatible (follows the catalogue)
O-I cross-product.yml factory-proving actuation source-only fixture oracle at a pinned cut — not required
ai-kit caw-native-delivery.yml + cross-product.yml stale Node launcher build/bind → owner change required
ai-kit docs/implementation/CAW-NATIVE-DELIVERY.md  rides with the workflow change
ai-kit aikit-adapters actuation_* crates        already compatible (wire/CLI seam through a Runner abstraction)
Workcell runtime/cli (instance scan, detection intake) already compatible (intakes `actuation harness detect --json`)
Factory (contracts/canon only, no process invocation) already compatible
Rust-crate consumers (public types seam)        none exist; language-neutral operations remain the boundary
```

Required consumer changes were executed explicitly in the owning repositories,
not smuggled into Actuation:

- **O-I #245** (`feat/actuation-native-artifact`): the actuation surface
  follows the same locked-cargo install contract as the other five products
  (`target/release/actuation`, manifest `crates/actuation-cli/Cargo.toml`),
  re-pinned to `4af495f`; the current-main source test contract becomes
  `cargo test --workspace --locked` instead of the removed `npm test`;
  `native-protocol.json`/`mainline.json` re-pinned; the development-field cut
  and continuous-work source lock advance to the same main; the proving
  workflow builds and binds the native executable (its `setup-node` retired
  with the launcher); the development-field gate now runs the owner-declared
  `.oi/product.json` `build.command` before the owner-declared
  `verify.source_command`, and drops the removed Node register test
  (register integrity is asserted natively by
  `crates/actuation-cli/tests/register_integrity.rs`); the source-package
  fixture is restaged around a built artifact. Suite standing at open:
  `cargo test --locked --manifest-path cli/Cargo.toml` 151 passed / 0 failed.
- **ai-kit #300** (`ci/actuation-native-artifact`): both CAW workflows advance
  the Actuation pin to `4af495f`, build `actuation-cli` from that exact cut,
  and bind `AIKIT_CAW_ACTUATION_BIN` / `PATH` to the built binary; `setup-node`
  retired; the consumer test code itself needed no change (binary-path seam).
  Standing at open: the `caw_native_delivery` suite passes against the served
  binary; the `caw_task_dispatch` material phase fails with a workcell
  control-connection refusal whose only delta versus the green main run is the
  Actuation binary identity (launcher script → ELF binary) — diagnosis and
  local control experiment recorded on the PR; resolving it is joint
  ai-kit/Workcell work and is an explicit open R8 item.
- **Workcell / Factory**: no change required — Workcell consumes the served
  detection command and Factory correlates at the contract level; both stay
  wire-compatible with the served CLI without modification.

### Cross-product invariants and their native proofs

```text
RunRef != ActuationRef, RunRef != AgentSessionRef,
AgentSessionRef != Harness/process/provider id
    nominal reference types — crates/actuation-core/src/refs.rs (distinct
    newtypes; cross-assignment does not compile); no fabricated refs in a
    direct composition — crates/actuation-core/tests/constitutional.rs
    direct_constitutional_composition_requires_no_factory_or_body
Workcell material ref != Agent/Agency identity
    crates/actuation-adapters/tests/receipt_admission.rs
    target_or_material_replacement_does_not_change_enduring_correlations
Direct Agency has zero fabricated Factory ancestry
    constitutional.rs (above) + crates/actuation-stream/tests/actuality.rs
    direct_actuality_retains_its_world_actor_and_trace_without_factory_ancestry
External/native Harness has zero fabricated ancestry
    actuality.rs non_action_event_is_activity_and_factory_refs_remain_supplied_correlations
    + crates/actuation-adapters/tests/observation_native.rs
    optional_self_markers_are_names_and_ambiguity_never_picks_an_identity
Factory work can correlate real Journey/Run to the same Activity/Return
    actuality.rs factory-refs-remain-supplied-correlations (above)
Provider/material relocation preserves semantic identity
    crates/actuation-core/src/refs.rs AgencyIdentity::is_continuation_of
    (a new body is not an input to identity)
Model/provider choice remains AIKit responsibility
    survey: crates/aikit-adapters/src/actuation_model_routes.rs lives in ai-kit
Observed provider/body/usage actuality remains Actuation responsibility
    crates/actuation-adapters (observation, usage) and
    crates/actuation-stream usage occurrences
```

O:I dispatch now resolves the Rust artifact end to end: the owner's
`.oi/product.json` declares the build and entry, O:I's catalogue mirrors the
same contract, and the O:I lifecycle gates run the owner build before the
owner verify.

## R9 — pre-local convergence and JavaScript retirement

### Remaining JavaScript census (tracked files)

Before this tranche the tree held 90 tracked `.mjs`/`.js` files; 9 are removed
here, leaving 81, every one in a lawful class:

```text
class                        count  files
frozen-corpus verification     11   scripts/migration/{parity,scenario-parity,scenarios,
                                    verify-runtime-corpus,verify-stream-corpus,
                                    verify-adapter-corpus,verify-corpus,check-recorder,
                                    capture-record,source-lock,build-ledger}.mjs — the
                                    native gate and CI migration-oracle job execute these;
                                    they replay the frozen Node-written corpora against the
                                    native oracles (never recapture in a passing gate)
specimen-native research       63   experiments/ql-runtime/** (foundation, deep-ql,
                                    comparison/series1, prime, experiments/{native,pi,pydantic})
specimen-native research        4   experiments/native-research/adapters (dsh/pi hosts)
specimen-native research        3   experiments/epistemic-cultivation
```

Removed here (generic product tooling for the retired Node body; blobs
preserved in git history and named in `R8-R9-DISPOSITION.json`):

- `scripts/bundle.mjs`, `scripts/entry.mjs` — the esbuild bundler for the
  removed single-file `bin/actuation` launcher.
- `scripts/migration/verify-jsonl-interop.mjs` — imported the removed
  `contracts/` modules; the R4 cross-implementation interop gate ended at the
  cutover (R7 record), and the frozen Node-written stores remain the wire
  oracle, read natively.
- `scripts/migration/capture{,-loader,-register,-runtime,-scenarios}.mjs`,
  `scripts/migration/extra-cases.mjs` — the capture orchestrators whose
  subject (the served Node implementation) no longer exists; the recorder law
  itself (`capture-record.mjs`, CI-enforced via `check-recorder.mjs`) and the
  frozen corpora are retained.
- Locally, the git-ignored `node_modules/` residue of the removed
  `package.json` was deleted from the working copy (untracked; not a repo
  change).

### Documentation reconciliation

Stale implementation-path language was reconciled in place (no history
rewritten; the Node provenance stays named in the migration records):

- `docs/HARNESS-CAPABILITY.md` — capability descriptors live in
  `catalog/targets.json` via `crates/actuation-adapters::NativeCatalog`.
- `docs/HARNESS-REFERENCE.md` — the catalog declaration and the add-a-harness
  step now name the native catalog document.
- `docs/ACTUATION-STREAM.md` — the portable seam and the first-party durable
  store are owned by `crates/actuation-stream` (`stream.rs`, `journal.rs`,
  `store.rs`).
- `docs/REALISED-ACTUATION-LOOP.md` — `actuation.realised/v1` is implemented
  by `crates/actuation-runtime/src/realised.rs` with the frozen wire oracle.
- `docs/WORLD-BOUND-ROOT-AGENCY.md` and `docs/VISUAL-PRODUCT-UNDERSTANDING.md`
  — the agency contract is the native constitutional core
  (`crates/actuation-core`) with `fixtures/migration/oracle.json` as the
  frozen conformance evidence.

README, SYSTEM-PLACEMENT, the constitution/relation/activity documents and
both native Skills were reinspected and already describe the Rust product
(R7 reconciliation); nothing in the current docs presents the product as
harness detection/receipt tooling.

### Capability matrix and product ground (serialized refresh)

The generated account/matrix projections were refreshed from accepted code
through the serialized reconcile path only (plan → apply with review of the
six seed sections; change ref `actuation:#58 PR H (R8+R9 harmonisation)`;
transaction receipts under `.central/documentation-transactions/`):

- capability rows' `code_refs`/`test_refs` now resolve to the native sources
  (crates, `catalog/targets.json`, fixtures); every row's
  `maintenance.code_basis` re-digested against the current files;
- the maintenance discovery contract runs
  `./target/release/actuation capabilities --json` and `runtime_paths` cover
  `crates/**`, `catalog/**`, `schemas/**`, `fixtures/**`;
- the CLI observation rows record the native release-binary discovery of this
  tranche (21 command identities, 11 native contracts);
- the discovery prose and the command-table link in the account page name the
  native artifact;
- `.github/workflows/product-ground.yml` builds the served product and drops
  its Node setup, so the advisory drift check exercises the same native
  discovery locally and in CI.

Final product-ground standing on this tree: 0 errors; 21 discovered commands,
21 mapped; no exposure gaps. Human acceptance of the refreshed account is
explicitly not inferred — the account's own standing fields are unchanged.

### Executed verification

- `bash scripts/migration/verify-native.sh` — full native gate green on this
  tree: fmt, clippy (`--locked --workspace --all-targets -- -D warnings`),
  `cargo test --locked --workspace`, frozen parity corpora replayed against
  the native oracles (R2 119, R3 44, R4 213, R5 179, R6 44 pure parity cases;
  R3 29 / R4 / R5 corpus-integrity and effect-scenario lines), the 33 frozen
  cli command-surface scenarios through
  `examples/cli-scenario-oracle`, and the release-profile verify suite.
- O:I PR #245 branch: `cargo test --locked --manifest-path cli/Cargo.toml`
  151/0; `verify-mainline-snapshot.py` PASS.
- ai-kit PR #300 branch: workflow YAML validated; consumer suites unchanged
  and green on their own main.

## Receipt for the physical return (S7)

```text
basis revision          4af495f6509bb0dd779112a225d71a9df8145b6e (accepted main;
                        this PR's head is named by the PR itself)
artifact                kind cli; entry target/release/actuation;
                        platforms macos-arm64, linux-x86_64;
                        build cargo build --release --locked -p actuation-cli;
                        release law github-releases, actuation-v tags, sha256
                        checksums, github-artifact-attestation
                        (declared by .oi/product.json; mirrored by O:I surfaces)
store                   append-only JSONL, one file per stream
                        (crates/actuation-stream/src/store.rs); portable
                        contract actuation.stream/v1; frozen Node-written wire
                        oracle fixtures/migration/r4 (manifest sha256
                        121598f41376f72e5b565b0f0d9899bccdf458b42d97bfae272b5a86d2f6510d),
                        read natively by cargo tests and the shipped verify suite
contracts               actuation.cli/v1 + 10 served native contracts incl.
                        actuation.agency/v1, actuation.agency-actualisation/v1,
                        actuation.realised/v1, actuation.stream/v1,
                        actuation.activity/v1, actuation.model-usage/v1,
                        actuation.instantiation/v1, actuation.harness-detection/v1,
                        actuation.harness-capability/v1, and
                        oi.product-settings-disclosure/v2; public schema parse
                        over schemas/*.json in the native-cli gate
cross-product standing  O-I #245 and ai-kit #300 opened against exact accepted
                        mains; Workcell and Factory already compatible
                        (classification above)
```

## Not claimed

No live provider run, no owner-machine acceptance, no human acceptance —
those are the physical return tranche (R10 under O:I Development Field S7).
The consumer PRs' merge standing is stated at open time and must be re-read
from their repositories: O-I #245 carries one documented red job (the joined
Node witness needs a served-binary redesign — O:I owner decision, tracked as
O:I#246), and ai-kit #300 carries a red material phase in `caw_task_dispatch`
against the served binary (diagnosis on the PR) — both are explicit open R8
items, not silent regressions. The stale `~/.config/oi/catalogue.json` adopted
snapshot on owner machines predates this harmonisation until `oi catalogue
adopt` is re-run; and an npm-global `actuation` launcher installed from the
pre-R7 package may still exist on owner machines — both are physical machine
state for R10 to reconcile, not repository state. Whether O:I keeps or removes
its now-unreachable empty-build source-package code path is an O:I owner
decision, raised in #245, not taken here.
