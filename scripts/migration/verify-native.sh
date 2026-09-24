#!/usr/bin/env bash
# Deterministic native gates. No provider or owner-machine acceptance is
# inferred from this build environment. Since the R7 cutover the served product
# is the native executable. The frozen Node-era migration corpora and their
# native replay gate were retired in cleanup/retire-node-oracle-2026-09-22:
# the product's own contract, conformance and verify suites are the standing
# acceptance. Evidence file names are unchanged.
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
# R7: the release-profile executable certifies this build.
cargo build --locked --release -p actuation-cli
./target/release/actuation verify --json | tee "$evidence/R7-native-verify.json"
