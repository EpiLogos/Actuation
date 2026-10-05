---
role: architecture
standing: agent-inference
scope: Actuation native operations and composed O:I consumer boundaries
updated: 2026-10-04
---
# Actuation architecture navigation

This is an implementation-facing navigation companion for O:I #65/#220 and the
existing documentation programme. It recovers native owners and successors;
it does not adopt a new design or claim the whole running experience complete.
Inspected native checkout revision: `e5225dc29861e9c1d9d5b799da37655aafdf9d02`. Active repair source may advance
that cut; identify the file revision before relying on its returned result.

## Governing source and successors

- [ACTUATION-CONSTITUTION](ACTUATION-CONSTITUTION.md)
- [WORLD-BOUND-ROOT-AGENCY](WORLD-BOUND-ROOT-AGENCY.md)
- [ACTUATION-STREAM](ACTUATION-STREAM.md)
- [ACTIVITY](ACTIVITY.md)
- [Native research](rust-refoundation/R6-RESEARCH.md) and its
  [JavaScript retirement successor](rust-refoundation/R11-JS-RETIREMENT.md)


Directory names are routes, not authority. Target design, amendment, historical
baseline, current implementation and observed result keep their own standing.

| Concern | Public operation / entry | Native source | Boundary and lifecycle |
| --- | --- | --- | --- |
| Agency / WorldBinding | Native World-bound root agency / instantiation | `crates/actuation-core/src/agency.rs`; `crates/actuation-adapters/src/instantiation.rs` | Agency and attributable stream identity are independent of provider sockets or Factory attempts. |
| Position occupancy | Native claim/release/verify/presence | `crates/actuation-stream/src/occupancy_store.rs` | Append-only generations with expected-vacant/current-generation gates; Factory consumes current occupancy for custody. |
| Stream / activity / Return | Native stream and activity reads | `crates/actuation-stream/src/store.rs`; `activity.rs` | Durable attributable events; owner Return is not merely a desktop connection signal. |
| Research / comparison | Separate `actuation-research` JSON executable | `crates/actuation-research/src/bin/actuation-research.rs`; `prime_run.rs`; `comparison.rs` | R11 replaces the retired JavaScript drivers. `tests/live_evidence_parity.rs` reproduces retained live-run receipt digests and prompts offline; first Rust-collected provider acceptance remains pending. |

## Diagram and consumer relation

The maintained suite companion is
`source:project:O-I:docs/architecture/world-ownership.md`; its editable diagram is
`source:project:O-I:docs/architecture/world-ownership.mmd`. The O:I architecture entry
contains six question-specific companions, full-size rendered SVGs, an indexed
basis for every arrow, exact source hashes and independent navigation evidence.
Resolve that source in the current O:I checkout before substituting a cached or
historical copy. Its solid arrows are inspected relations, not installed
acceptance; proposed joins remain explicitly proposed.

Existing capability records link this companion through the optional
`extensions.documentation` protocol. These links change discoverability, not
capability IDs, coordinate placements or source authority. The research row's
retired code/test references now name the R11 successors while retaining its
experimental status and historical digests.

## Verification and open joins

Read each native test and its actual runner conditions, then the corresponding
dated Return. A test definition is not an executed result; a process/receipt is
not human Recognition. The suite's architecture verification records real
Mermaid rendering, source/link checks and the fresh-agent navigation task.
The four repair lanes continue to own their code, installed replay and open
architectural decisions. Preserve a missing join as missing until that proof
or decision is returned.

## Current selected-handoff source relation — 4 October 2026

This is the retained `ac036` inspection before the later hosted qualification.
Its pending statements belong to that cut; the current evidence and remaining
process boundary are stated in the successor below.

Actuation Source `ac03605c47c66c0f0a1a574e871d0451917164cb` includes the
[selected native handoff consumer](EPI-LOGOS-AGENT-ARCHITECTURE.md#10-native-now-continuation--2026-09-22-follow-up).
The inherited [`ql-relational` SDK](../experiments/ql-runtime/prime/skills/ql-relational/src/ql_relational/__init__.py)
returns through `projectcentral.now.return` and consumes the owner's optional
`read_path` through `central.files.read`. Project ID-only reads retain
`projectcentral.now.inspect`; root ID-only reads are unavailable.
`central.now.read` selects an allocated clearing NOWRef and is not an alias for
this ordinary handoff. Central owns the record identity, bytes and Source
binding. A successful selected reading retains the owner revision and binding;
the SDK has no automatic resend path. Reader final cause/freshness and production
process capture remain pending.

The current owner-process and seven-case joined SDK qualification remain
pending; their [capture](../experiments/ql-runtime/prime/tests/central_now_handoff_capture.rs)
and [native driver](../experiments/ql-runtime/prime/tests/central_now_handoff_native.py)
are definitions, not executed acceptance. Central
[`1224bb7cd67243c37cd6c2676f2b77ba862f2ca4` architecture guidance](https://github.com/EpiLogos/Central/blob/1224bb7cd67243c37cd6c2676f2b77ba862f2ca4/docs/ARCHITECTURE-NAVIGATION.md)
and its separate hosted learning-reader result do not qualify this joined
handoff relation, installed operation or human Recognition. No diagram arrow
or Epi numerical/domain mapping is changed by this source reading.

## Native process ownership and qualified handoff

Inspected process Source is Actuation
`50ac3991dc7546d3268af27e06c39d8837cf4299`. The research
[process module](../crates/actuation-research/src/process.rs) owns spawning,
held private capture, direct-child wait and process-group retirement. The
[SDK](../crates/actuation-research/src/sdk.rs) consumes that result and keeps a
completed reply distinct from confirmed retirement. Core
[wire errors](../crates/actuation-core/src/wire.rs) share original typed causes
across clones; core performs no process operation. Public failure projections
retain selected scalar observation/cause facts; private stdout/stderr remain an
explicit owner access. Capture bounds are checked over held files after the
activity: they do not establish a hard while-running disk/RSS limit or control
escaped descendants. These boundaries keep causal and material evidence without
publishing a private body or pretending that outer completion proves retirement.

[Hosted refoundation run 37219849074](https://github.com/EpiLogos/Actuation/actions/runs/37219849074)
at this Source passes Linux format, clippy, whole tests and all thirteen original
named native cases (twelve research, one core; zero failed or ignored). Its
retained case record explicitly withholds full scalar-observation emission and
installed/provider/Original/H credit. macOS compiles, then the whole research
libtest returns 101 with 116 passed, 15 failed and zero ignored; its separate
controlled thirteen-case gate is not reached. The process-retirement join on
macOS remains a real failed boundary, not an architectural choice or a completed
cross-platform power.

The [joined handoff run 37218103369](https://github.com/EpiLogos/Actuation/actions/runs/37218103369)
uses Actuation `b91b3fbe0e1f397846276d7dff6274fd15ab4960`, current Central
`f596baecb469e07ef0f704a03b45d38454b01369` and an attributed example over
historical AIKit `900bce05483c6fd39cfd320ebf95c2e2b5aa4d29`. Each hosted platform
executes the exact eleven Root cases and seven unchanged SDK cases: capture exit
zero, seven passed, zero skipped and 33 native calls. Actual source archives,
compiler receipts and four retained compiled-image byte hashes are joined;
before-operation custody and original SDK/driver digests are preserved. macOS's
non-UTF8 Root case retains its genuine EILSEQ prerequisite limitation. This
optional capture harness supplies no production SDK-capture repair, current
AIKit session, installed Factory undertaking or human acceptance. Earlier failed
capture/format/compile cuts remain historical results; they are not relabelled.
