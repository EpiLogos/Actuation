#!/usr/bin/env bash
# Deterministic native gates. No provider or owner-machine acceptance is
# inferred from this build environment. Since the R7 cutover the served product
# is the native executable; since the R11 JavaScript retirement the frozen
# migration corpora are replayed by the native gate
# (crates/actuation-migration-gate), which reads the same Node-written corpora
# byte-for-byte and asserts the same drift laws the retired .mjs executors did.
# Evidence file names are unchanged.
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
# The native gate replays the frozen corpora against the wire oracles:
# R2/R3/R4/R5/R6 pure parity, R3/R4/R5 corpus integrity, the effect/store and
# detection scenario families, and the R7 CLI command-surface scenarios. The
# served executable certifies this build afterwards.
target/debug/actuation-migration-gate parity --evidence "$evidence"
# R7: the release-profile executable certifies this build.
cargo build --locked --release -p actuation-cli
./target/release/actuation verify --json | tee "$evidence/R7-native-verify.json"
