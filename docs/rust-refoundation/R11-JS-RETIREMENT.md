# R11 — JavaScript research/experiment layer retirement

**Date:** 2026-09-13.
**Authorship:** owner commission via coordinator, 2026-09-13. The owner
explicitly commissioned this campaign, superseding the R9 convergence
record's preservation decision (see "Supersession" below).

## Supersession

This record supersedes the R9 pre-local convergence record's JavaScript
retention decision — `R8-R9-HARMONISATION.md` ("R9 — pre-local convergence
and JavaScript retirement", the "Remaining JavaScript census" classification
of 81 tracked files into "lawful classes" preserved in place) and its machine
record `R8-R9-DISPOSITION.json`. R9 preserved the JavaScript tree because the
live research instrument, the migration gate executors, the specimen-native
SDK oracles and the live-provider evidence path ran only there, and
`crates/actuation-research/src/application.rs` declared live-provider QL
research evidence "deep-runtime/series1-live unexercised" in Rust.

The owner reversed that decision on 2026-09-13 by commission: the JavaScript
layer is retired and every live function it carried now runs in Rust. The R9
classifications in `ledger.json` are frozen history and are not rewritten;
this record states the executed supersession. Nothing else in the R9 record is
reopened — the consumer harmonisation (O-I #245, ai-kit #300), the served
native product and the frozen corpora stand as recorded there.

## What moved into Rust, and how it is proven

1. **Live research harness (Phase 1).** The experiment families are expressed
   and run by `crates/actuation-research`: the frozen JSON twins are the
   declared sources (`experiments/native-research/tasks.json` for the task
   corpus; `experiments/ql-runtime/prime/conditions.json` for the Prime
   condition catalogue, now digest-pinned in `prime.rs` and no longer bound to
   `conditions.mjs`); `prime_run.rs`, `comparison.rs`, `execution.rs`,
   `readiness.rs` and `sdk.rs` run Prime, Deep-runtime and Series 1 trials and
   emit the same `ql-series1-run/0.3` and `actuation.prime-recursive-
   experiment/v1` records. Offline parity against the recorded 2026-09-13
   live runs (GLM series1 native host; Prime P0/P3) is asserted by
   `tests/live_evidence_parity.rs` over tracked receipts under
   `fixtures/research/`: the prompt and success-constraint digests reproduce
   byte-for-byte, the held-constant determination reproduces exactly, mask
   mapping is deterministic, and the live-composed Prime prompts reproduce
   byte-for-byte from the JSON twin. **Live-provider evidence custody has
   moved to Rust.** The first live validation run collected by the Rust
   harness is an explicit owner-gated step (`ql-series1-live.yml`, dispatch
   only; declared in `actuation capabilities --json` → `acceptance_pending`).
2. **Migration gate (Phase 2).** The frozen Node-written corpora are replayed
   by `crates/actuation-migration-gate` — R2 119 / R3 44 / R4 213 / R5 179 /
   R6 44 pure parity cases through the same wire oracles, the R3/R4/R5
   corpus-integrity laws (including the r4 exact-bytes law), the effect/store
   and detection scenario families, and the R7 CLI command-surface scenarios.
   Evidence file names are unchanged. `verify-native.sh` keeps its path and
   asserts the same drift laws; `cargo fmt/clippy/test` laws unchanged.
3. **Specimen conformance (Phase 3).** The specimen-native JavaScript hosts
   (`experiments/native-research/adapters/pi.mjs`, `dsh.mjs`) are replaced by
   recorded-fixture conformance: fixtures captured from the real pinned
   specimens (`@earendil-works/pi-ai@0.84.1` from the public npm registry;
   public `deepseek-harness` source at `47f9438…` built from source with the
   pinned pnpm bootstrap), with pins asserted against the workflow
   declarations so a pin bump without re-capture fails CI. The pydantic
   specimen keeps its live lane (retained Python adapter executes the pinned
   source with controlled responses, `research-sdk.yml`).
4. **CI (Phase 4).** Every job that ran JS now runs the Rust-native
   equivalents with preserved workflow/job names. The one branch-required
   check, `native-cli` (workflow "Native CLI"), is unchanged as a job
   identity. No `setup-node`, `npm` or `.mjs` invocation remains in any
   workflow.
5. **Records and pins (Phase 5).** The capability-matrix research pins moved
   from `experiments/ql-runtime/prime/run.mjs` +
   `prime-structural.test.mjs` to the native research crate artefacts with
   new sha256 digests (the retired files' digests remain named as
   historical provenance). README no longer presents the retired tree as the
   proving body.

## Specimen fixture regeneration procedure

The fixtures under `fixtures/research/specimens/` are versioned evidence:
each names its upstream source, the pinned revisions, the capture date and
the pre-retirement adapter blob it was captured through
(`ece2478649d58ed210eadd2e2bac00af9e5ced0d`). To re-capture after bumping a
pin:

1. Recover the retired capture drivers from git history:
   `git show ece2478:experiments/native-research/adapters/pi.mjs` (and
   `dsh.mjs`), plus the retired tests (`adapters/test/*.test.mjs`) that show
   the scripted-model injection points.
2. **pi:** `npm install --ignore-scripts --no-save --no-package-lock
   @earendil-works/pi-ai@<new-pin>` into a scratch directory, copy the
   recovered `pi.mjs` beside it, run a capture driver equivalent to the
   retired `test/pi.test.mjs` (real catalogue preflight; controlled model
   for the completion shape; standalone JSONL envelope frames), and record
   the output shape as `pi.json`.
3. **dsh:** check out `deepseek-ai/deepseek-harness` at the new pin, build
   `pnpm@10.15.1 install --frozen-lockfile && pnpm run build:lib:host`
   (with `COREPACK_ENABLE_PROJECT_SPEC=0`), project the workspace packages
   into `adapters/node_modules` (the projection snippet lives in the
   pre-R11 `ql-series1-dsh.yml` at the campaign-base commit), then capture
   `dsh.json` through the recovered `dsh.mjs` as in (2).
4. Update the fixture, keep the schema `actuation.specimen-conformance/v1`,
   and update the pin assertions in
   `crates/actuation-research/tests/specimen_conformance.rs` in the same
   commit. CI fails whenever the fixture pins and the workflow pins disagree.
5. **pydantic:** no fixture — the live lane re-captures nothing; it executes
   the newly pinned source directly in CI.

## Recorder law after the retirement

`check-recorder.mjs` unit-tested the JS capture tool (`capture-record.mjs`)
whose subject was the removed Node implementation; with the JS retirement the
tool and its test are retired together. The underlying law — **never
recapture expectations in a passing gate** — holds structurally in Rust: the
frozen corpora are sha256-pinned (`fixtures/migration/SHA256SUMS`,
`R3-SHA256SUMS`, `r5/SHA256SUMS`) and the native gate is read-only over them.
The `corpus` gate output carries the same
`actuation.corpus-integrity/v1` schema with a `gate` provenance field naming
the native gate.

## Deviations and honest limits

- **Live pinned-DSH and pi execution left CI.** The retired JS hosts were the
  only in-repo way to execute those JS/Node specimen SDKs; Rust cannot drive
  them without Node. Their guarantee is carried by recorded fixtures
  (captured at the exact pinned revisions, with regeneration documented
  above); the pydantic lane stays live-specimen.
- **Runner-local identities.** The recorded live runs bind
  `task_corpus_revision` to the sha256 of the retired `tasks.mjs` and carry
  runner-local digest identities (task revision, start state, verification
  protocol, capability contract, execution budget) over equivalent content;
  the tracked receipts label exactly which digests the Rust twin reproduces
  byte-for-byte and which are runner-local, and why.
- **Series 1 live hosts.** The multi-host matched comparison (pi/pydantic-ai/
  dsh/native JS hosts) is now expressed through the harness's supplied-body
  seam: live collection rides explicit body processes under owner gating;
  the harness itself provides no provider transport and claims no provider
  evidence.
- Pre-existing stale `.mjs` references elsewhere in the capability matrix
  (rows for capabilities retired at the R7 cutover) predate this campaign and
  were not rewritten here beyond the research row this campaign pins.

## Executed verification (this record's evidence)

- `cargo build --release --locked` green; `cargo test --locked --workspace`
  green (32 suites, 212+ tests, including the new parity and conformance
  suites) at each phase commit.
- `bash scripts/migration/verify-native.sh` green with the native gate: all
  frozen parity/scenario corpora replay green with identical evidence
  filenames, and `actuation verify --json` certifies the release build.
- Specimen fixtures: captured locally from the pinned specimens; the pinned
  `dsh.test.mjs` and `pi.test.mjs` suites were run green against the same
  specimens before capture.
- Deletion: 49 tracked `.mjs` + 33 tracked `.js` (all under the retired
  trees) removed with their orphaned companions; `git status` afterwards
  shows exactly the pre-campaign untracked owner evidence.

## Not claimed

No live provider run was initiated by this campaign; no provider evidence,
owner-machine acceptance or human acceptance is claimed. The upstream Series
1 findings (#79 draft PR, #80) are not modified by this campaign; superseding
them is an owner decision to draft separately.
