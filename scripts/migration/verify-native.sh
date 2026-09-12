#!/usr/bin/env bash
# Deterministic native gates. No provider or owner-machine acceptance is
# inferred from this build environment. Since the R7 cutover the served product
# is the native executable; the frozen migration corpora remain the historical
# wire oracle and are replayed against the native oracles below.
set -euo pipefail
cd "$(dirname "$0")/../.."
evidence="${1:-native-evidence}"
mkdir -p "$evidence"
git rev-parse HEAD > "$evidence/source-commit.txt"
git write-tree > "$evidence/index-tree.txt"
if git diff --quiet && git diff --cached --quiet; then
  printf 'clean\n' > "$evidence/worktree-state.txt"
else
  printf 'candidate-working-tree; not evidence for HEAD alone\n' > "$evidence/worktree-state.txt"
fi
rustc -Vv > "$evidence/rustc.txt"
cargo -V > "$evidence/cargo.txt"
cargo fmt --all -- --check 2>&1 | tee "$evidence/fmt.log"
cargo clippy --locked --workspace --all-targets -- -D warnings 2>&1 | tee "$evidence/clippy.log"
cargo test --locked --workspace 2>&1 | tee "$evidence/tests.log"
cargo build --locked --examples --bins
node scripts/migration/parity.mjs --phase R2 -- target/debug/examples/constitutional-oracle | tee "$evidence/R2-parity.json"
node scripts/migration/verify-runtime-corpus.mjs | tee "$evidence/R3-corpus-integrity.json"
node scripts/migration/parity.mjs --phase R3 -- target/debug/examples/runtime-oracle | tee "$evidence/R3-parity.json"
node scripts/migration/verify-stream-corpus.mjs | tee "$evidence/R4-corpus-integrity.json"
node scripts/migration/parity.mjs --phase R4 -- target/debug/examples/stream-oracle | tee "$evidence/R4-parity.json"
node scripts/migration/scenario-parity.mjs fixtures/migration/scenarios.json store fold filename -- target/debug/examples/store-oracle | tee "$evidence/R4-scenario-parity.json"
# The R4 cross-implementation interop gate ends at the cutover: the frozen
# Node-written store snapshots (fixtures/migration/r4/) are now read natively
# by cargo tests and by the shipped binary's own verify suite.
node scripts/migration/verify-adapter-corpus.mjs | tee "$evidence/R5-corpus-integrity.json"
node scripts/migration/parity.mjs --phase R5 -- target/debug/examples/adapter-oracle | tee "$evidence/R5-parity.json"
node scripts/migration/scenario-parity.mjs fixtures/migration/scenarios.json catalog probe secret -- target/debug/examples/observation-oracle | tee "$evidence/R5-scenario-parity.json"
node scripts/migration/parity.mjs --phase R6 -- target/debug/examples/research-oracle | tee "$evidence/R6-parity.json"
# R7: the served CLI itself answers the frozen command-surface scenarios.
ACTUATION_CLI_SCENARIO_BIN=target/debug/actuation \
  node scripts/migration/scenario-parity.mjs fixtures/migration/scenarios.json cli -- target/debug/examples/cli-scenario-oracle | tee "$evidence/R7-cli-scenario-parity.json"
# R7: the release-profile executable certifies this build.
cargo build --locked --release -p actuation-cli
./target/release/actuation verify --json | tee "$evidence/R7-native-verify.json"
