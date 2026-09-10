# R5 — Native observation at Actuation's boundary

## Meaning, source and standing

This is serial PR E of #58, based on accepted and reverified R4 main
`0ac4bbb47cc37a7ac3c1db42b88a39ffd6bb5b30` (tree
`269950f7017c586a28acfc1dbc0c190ed38a7f49`). It follows the founding positions,
Actuation constitution and R0 disposition. Actuation observes the body actually
present, its exposed relations, and attributable native occurrences. It does not
choose a model, provision a HarnessComposition, host AIKit sessions, allocate a
Workcell, decide Factory Recognition, or create QL formal semantics.

A declaration and an observation are different sources. `NativeCatalog` loads
versioned target data without executing anything. `ProbeEffects` supplies typed
observations in an explicit environment. `run_detection` produces the accepted
public detection reading. `resolve_self` relates marker names to the same-run
body observation; it cannot infer an enduring Agent or determining authority.
A provider-native inventory is an observed offering, not an AIKit selection.

The Node CLI continues to serve the product until R7. This tranche introduces
ordinary native library operations, not a second public CLI or daemon. PR #62's
separate settings-disclosure work is not overwritten or adopted by implication.

## Native ownership

`actuation-adapters` owns catalogue loading, target/capability/secret wire
admission, existing-body observation, instantiation receipt readback and native
usage translation. Lossless private record wrappers retain the current open
extension and null/omission window. The admitted record cannot be mutated behind
its validator. Execution uses explicit effects and typed results, not fixture
lookup or the accidental structure of MJS handlers.

`catalog/targets.json` contains the exact declared facts previously exported by
12 harness descriptors, three capability descriptors and three secret-source
descriptors. The target catalogue remains revision 6 and secret catalogue
revision 1: the facts did not change merely because their representation moved.
A declared target may have no known dispatch capability; Pi's absent capability
is not inferred from another target. New target data needs no generic-core code.
The native catalogue implements the existing `BoundaryCatalogue` port, retaining
**the capability's own provenance revision**, not substituting the later whole
catalogue revision into an observed stream occurrence.

R4's Claude/Codex usage codecs moved to `actuation-adapters::usage`; they were not
copied. `actuation-stream` continues to own observations, units, correlation,
deduplication and persistence. Its pure conformance transport moved beside the
codecs so the production dependency remains adapters -> stream -> core, never
stream -> adapters. Existing R4 persistence and frozen-snapshot tests remain in
the stream crate and execute unchanged in meaning.

`InstantiationReceipt::read` preserves the explicit legacy schema window.
`require_correlation` checks exact Actuation/Agency/WorldBinding request identity;
a well-shaped receipt for another request is refused. The narrow historical
`attach_detection_evidence` operation remains available for compatibility but
is not full receipt admission. Production callers should first admit the typed
receipt and use its `with_detection` method. None of these operations claims that
receipt construction has executed a determination.

## Bounded real mechanisms

Native effects accept an explicit home, working directory, search path and
private environment snapshot. Plain detection resolves executable metadata but
does not run it. Version probing is opt-in. Owned probes use argument vectors,
not shell interpolation, with time/output budgets, private file-backed capture,
and Unix process-group termination. A held-open descendant output descriptor
cannot make a timeout wait indefinitely. Only the observed version line is
returned; provider error output is not copied into receipts.

Service and inventory probes use bounded native HTTP(S), with no automatic
redirect or ambient proxy credential propagation. Inventories are limited to
4 MiB and are read only after that descriptor's own same-run service presence
observation. Malformed/failed reads stay unavailable, not an empty offering.
Native directory counts do not become provider model identities.

Fingerprint effects observe existing supplied secret sources only. Central owns
secret reference/governance and AIKit owns credential resolution for dispatch.
These effects neither retrieve credentials for an act nor migrate private
Control. Environment values and file contents produce SHA-256, byte length and
location only. Hidden directories are traversed within the documented bounds;
dependency/build directories and symlinks are not followed. A single secret file
is capped at 64 MiB; incomplete or unreadable material is unavailable. Vault
metadata retains only the existing item/vault labels. Authentication failure is
not treated as item absence and raw fields/stderr are not emitted. No ordinary
verification reads the user's private Control or personal credentials.

## Explicit corrections, not a rewritten oracle

The immutable R1 oracle is unchanged. Three additional original-source
regressions are frozen in `fixtures/migration/r5/corrections.json`, with exact
source blobs and SHA-256. The read-only gate replays them against the unchanged
original source and native tests assert the corrected relation independently:

1. Executable absence plus present configuration no longer turns the text
   `not found on PATH` into an executable receipt or a version-probe argument.
2. A successful environment marker cannot turn a failed presence observation
   into proof of absence. The result remains unavailable.
3. Service presence without a captureable same-run executable/configuration
   receipt is disclosed as unavailable, not a malformed receipt or invented path.

The native secret scanner also refuses to mark a multi-probe declaration fully
verified when another required probe failed or was truncated. Positive evidence
does not make unobserved material known. These are tested honesty corrections;
none is asserted to have been a reproduced production incident. Existing schema
names/versions, catalog facts and frozen expected results are unchanged.

## Executable acceptance

Run from the exact source checkout with its supported Rust toolchain and Node
reference runtime during migration:

```sh
bash scripts/migration/verify-native.sh /tmp/actuation-native-evidence
npm test
node bin/actuation verify --json
node scripts/migration/verify-corpus.mjs
node scripts/migration/parity.mjs
node scripts/migration/scenario-parity.mjs
bash scripts/verify-native-skills.sh
```

The native gate covers formatting and locked all-target Clippy with warnings
denied; 84 integration tests plus the identity compile-fail doctest; all 119 R2,
44 R3, 213 R4 and 179 R5 pure cases; 29 original-loop cases; 21 durable scenarios;
57 bidirectional Node/Rust file assertions; 13 catalogue/probe/secret effect
scenarios; and exact descriptor-data/source-correction integrity. Native tests
exercise controlled real files, executable processes, process-group timeout,
loopback HTTP, same-run inventory, partial/failure paths, fingerprints, context
readback and the catalogue-to-durable-stream join.

Original Node's 219 tests, 599 pure cases, 67 scenarios, 84 research structural
tests and native Skills remain active. The two maintained Linux/macOS native
jobs must pass on the final readable PR source before merge. Accepted main must
then be re-read and reverified before R6 begins. Source transport capsules and
any temporary writable importer must be absent from the accepted diff.

This is D and explicitly scoped cross-implementation C evidence. Controlled
HTTP bodies, scripts and vault-shaped responses are specimens, not live provider
P, owner-machine M or human H acceptance. No physical installation, genuine
provider invocation, personal migration or S7/R10 acceptance is claimed.

## Remaining sequence

Generic MJS still serves the pre-cutover CLI/reference and unmigrated research
consumers; it is not relabelled specimen-native. R6 migrates active reusable
research apparatus and keeps actual specimen-language boundaries explicit. R7
owns the public native CLI/package cutover. The final consumer/acceptance tranche
must inspect current native owner operations and merge only demonstrated needs.
