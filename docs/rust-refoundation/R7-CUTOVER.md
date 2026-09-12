# R7 — native Rust CLI, packaging and owner cutover

Basis: branch `act-rust/r7-cli-cutover` fresh from the R6-accepted main
`1a21db9c2230a3756a7f79681542a3981a208dcf` (per issue #58, "Merge before R7").
The frozen per-source classifications remain in `docs/rust-refoundation/ledger.json`
and are not rewritten here; this record states their executed standing at the
cutover.

## What became the product

`crates/actuation-cli` is the served executable (`target/release/actuation`,
binary name `actuation`). One Rust-owned command descriptor table
(`src/dispatch.rs`) drives help, the capabilities listing, dispatch and the
tests; the human renderers (`src/render.rs`) and the JSON envelopes reproduce
the served semantics of the retired Node CLI. The Wave 5 settings disclosure
was ported natively (`src/system.rs`) with the same canonical-body digest law,
its provenance paths now naming the real native sources.

`actuation verify --json` no longer spawns a test runner: it executes a
compiled-in deterministic suite of fifteen checks over the shipped product's
own application surfaces — read-model goldens, durable store lifecycle, the
bundled harness catalog, the disclosure digest law, and an embedded R4 frozen
wire store (the Node-written snapshot, sha256-published in
`fixtures/migration/r4/manifest.json`) read through the native store. An empty
suite cannot pass; the suite table is itself asserted. The binary requires no
Node runtime and no repository.

## How parity was proven before the MJS was removed

The cutover law (do not delete the MJS oracle until everything passes from the
Rust binary) was executed in one PR, in commit order:

1. The CLI was built and the frozen `cli` scenario corpus — 33 cases captured
   from the served Node CLI covering every read model in JSON and human form,
   refusals, harness commands and exit codes — was run against the Rust
   executable through a new native scenario runner
   (`crates/actuation-cli/examples/cli-scenario-oracle.rs`): 33/33 green.
2. `scripts/migration/verify-native.sh` was restructured to the native-only
   gate and run green with the MJS tree still present: R2 119, R3 44,
   R4 213, R5 179, R6 44 pure parity cases, the R4/R5 effect scenarios, and
   the new R7 lines (cli scenario parity, release-profile verify).
3. Only then were `package.json`, `bin/actuation`, `cli/`, `contracts/` and
   `detection/` removed (74 files), and the full gate was re-run green on the
   removal commit.

## Gate restructure

- `native-cli.yml` (branch-required, check name preserved) is now the Rust
  product gate: full native script, the R7 gate commands, self-certification
  of the served version against the workspace version and of the served
  revision against the build sha, packaging parity, public schema parse.
- `native-rust` (rust-refoundation.yml matrix, ubuntu + macos) runs the same
  verification script across both release targets. The migration-oracle job
  keeps the unique obligations: the recorder law and corpus integrity
  (`verify-corpus.mjs`); the MJS-evaluated parity modes ended with the oracle.
- The R4 live Node↔Rust interop step ended with the second implementation;
  the frozen Node-written store snapshots remain the wire oracle and are read
  natively by cargo tests and by the shipped binary's verify suite.
- The R5 explicit-corrections evidence moved to a native parity test
  (`crates/actuation-adapters/tests/parity.rs`) asserting the three correction
  laws against the frozen scenarios.
- `ProjectCentral/tests/register.integrity.mjs` was ported to
  `crates/actuation-cli/tests/register_integrity.rs` (committed-register view);
  the README command-documentation law moved to
  `crates/actuation-cli/tests/readme_law.rs`.

## Packaging, release and O:I

- `.oi/product.json`: artifact entry `target/release/actuation`, platforms
  `macos-arm64` and `linux-x86_64`, locked cargo build command, verify via the
  release binary. The O:I `oi actuation` dispatch path is unchanged in shape:
  same lifecycle schema, same install/update/remove modes, same release law
  (github-releases, `actuation-v` tags, sha256 checksums, artifact
  attestation).
- `prelocal-build.yml` produces release artifacts per platform with sha256
  checksums and GitHub artifact attestation, and proves installation
  conformance by serving the packaged binary from a clean directory
  (no repository, no Node) through `--version` and `verify --json`.
- The release profile is `lto = "thin"`, `codegen-units = 1`,
  `strip = "symbols"`.
- README install/use now describes the native build and release artifacts;
  both native Skills name the native sources, commands and verification.
- `crates/actuation-research/src/application.rs`: the stale R6
  `acceptance_pending` items were reconciled to the R8/R9 horizon and
  owner-machine acceptance.

## Executed verification

- `cargo fmt --all --check`; `cargo clippy --locked --workspace --all-targets
  -- -D warnings`; `cargo test --locked --workspace` green including the new
  CLI crate suites (dispatch/help/routing, system disclosure laws, verify
  suite, register integrity, README law).
- Full native gate green at the removal commit: frozen parity R2–R6, effect
  scenarios, 33/33 cli command-surface scenarios, release-binary verify.
- Provider evidence, owner-machine evidence and human acceptance remain
  explicitly not assessed and are not R7 scope.

## Not claimed

No live provider run, no owner-machine acceptance, no human acceptance. The
experiment trees keep their specimen-native oracles (foundation, deep-ql,
prime, epistemic-cultivation); their JavaScript retirement is the R9
convergence, and consumer harmonisation is R8 per the Wayfinder.
