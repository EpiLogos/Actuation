#!/usr/bin/env bash
# Deterministic native and temporary cross-implementation gates. No provider
# or owner-machine acceptance is inferred from this build environment.
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
cargo build --locked --examples
node scripts/migration/parity.mjs --phase R2 -- target/debug/examples/constitutional-oracle | tee "$evidence/R2-parity.json"
node scripts/migration/verify-runtime-corpus.mjs | tee "$evidence/R3-corpus-integrity.json"
node scripts/migration/parity.mjs --phase R3 -- target/debug/examples/runtime-oracle | tee "$evidence/R3-parity.json"
node scripts/migration/verify-stream-corpus.mjs | tee "$evidence/R4-corpus-integrity.json"
node scripts/migration/parity.mjs --phase R4 -- target/debug/examples/stream-oracle | tee "$evidence/R4-parity.json"
node scripts/migration/scenario-parity.mjs fixtures/migration/scenarios.json store fold filename -- target/debug/examples/store-oracle | tee "$evidence/R4-scenario-parity.json"
node scripts/migration/verify-jsonl-interop.mjs | tee "$evidence/R4-jsonl-interop.json"
