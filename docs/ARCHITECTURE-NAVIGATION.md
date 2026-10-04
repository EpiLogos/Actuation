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

## Local governing authority

At inspected Source `185f972d685a33bf7a0703dff205f9566c387046`,
`actuation authority issue/resolve/revoke` dispatch to the
[local authority owner](../crates/actuation-cli/src/authority.rs).
[AuthorityStore](../crates/actuation-stream/src/authority_store.rs) retains
separate append-only issue/revoke JSONL records under `--store`,
`ACTUATION_AUTHORITY_STORE` or `~/.actuation/authority`. A directory lock covers
read/check/append; file opens refuse symlink redirection and appends sync data.
Resolution reads the stored governing binding/grant and checks revocation,
expiry, holder, allowed World, operations and exact requested bounds. It returns
admitted/refused/unavailable with an assembled request only when admitted.
Actualisation is the next semantic operation; runtime launch, physical
permissions and Central source mutation keep their own owners. This separation
prevents a saved plan, participation or copied JSON grant from silently becoming
acting authority. The record is local owner material, not a provider credential
or independently authenticated claim about a remote requester.

## Position tenure and presence

`actuation occupancy claim/read/list/presence/verify/release` reaches the
[CLI](../crates/actuation-cli/src/occupancy.rs) and
[Position ledger](../crates/actuation-stream/src/occupancy_store.rs), under
`--store`, `ACTUATION_OCCUPANCY_STORE` or `~/.actuation/occupancy`. Each Position's
SHA-256-named JSONL records tenure-began, tenure-ended and presence. The current
occupant is the single open generation; explicit vacancy/current-generation
expectations are checked under the append lock. Supersession and release retain
history; old generations cannot verify as current. Torn or ambiguous ledgers
refuse instead of choosing the newest line or silently repairing history.
Claim returns tenure, generation, occupancy and environment bindings; identities
and placement refs are supplied by the caller. Presence is a protocol statement,
not proof of a live process. Tenure is distinct from Workcell physical custody,
Agent enduring identity and metagency grants. Consumers must apply their own
current-generation/authority gates before effects; this ledger does not grant
those powers. The [real executable tests](../crates/actuation-cli/tests/occupancy_command.rs)
exercise initial claim, handover, stale generations, release, races and corruption.

## Configuration and harness intake

`config-contribution` and `config validate/plan/apply/reset` route through
[commands](../crates/actuation-cli/src/commands.rs) to the
[configuration owner](../crates/actuation-cli/src/configuration.rs). The bare
contribution discloses declared code settings; validation answers truthfully in
its document. This Source mints no new settings plans and performs no settings
mutation: plan/apply/reset return structured owner errors with nonzero exits,
after addressing, scope, schema and authority checks. Existing executed receipt
keys can replay as no-op; the historical receipt ledger is not evidence that a
fresh setting changed. [Configuration tests](../crates/actuation-cli/tests/config_plane.rs)
use the real binary in an isolated home to verify those distinctions.

`harness capability validate` and `config-contribution capability` share
[descriptor intake](../crates/actuation-adapters/src/contribution.rs): schema,
known slug and declared-gap closure. Validation emits named checks; contribution
emits exact supplied-byte digest, provenance and an owner landing instruction.
Neither mutates the compiled catalogue. This boundary lets an outside contributor
return a reviewable descriptor without silently changing what a resident binary
can detect, install or invoke. [CLI intake tests](../crates/actuation-cli/tests/capability_intake.rs)
keep admitted gap filling distinct from shadowing refusal and malformed input.

## Bounded Nara session actor

`actuation nara serve` owns a bounded JSON-lines
[ephemeral actor](../crates/actuation-cli/src/nara_session.rs) over the native
[Nara binding](../crates/actuation-runtime/src/nara.rs). Constitution admits the
body relation, canonical AgentSession and QL dialogue context; the pipe retains
response state and supports context, listen/response/complete, interruption,
reconnect and close. Correlated request IDs cannot be reused; invalid or stale
operations preserve the prior admitted binding. Limits are 256 KiB per complete
line, 1 MiB per reply and 65,536 admitted request identifiers. A fresh pipe starts
unconstituted and needs its basis resupplied; close/EOF does not destroy the
canonical AgentSession or persist this actor state.

The actor performs no audio or provider I/O. Interruption evidence is explicitly
caller-reported and scoped to playback/provider response; native cancellation
standing and actual transport effects stay distinct, including unsupported
interruption. Reconnect preserves the actor's one canonical AgentSession while
changing an admitted body/context. This boundary retains the richer coordinate,
Expression, private personal basis and session relations without turning a
transport socket into the enduring Nara or accepting a generic profile page.
The [real process tests](../crates/actuation-cli/tests/nara_actor.rs) verify flush,
request/turn currency, interruption scope, reconnect preservation and framing;
contract specimens do not prove a live microphone, provider, saved personal
world or completed Nara/Epii answer.

The new operation relations above are inspected at `185`; their named CLI tests
actually pass on both hosts in [the `50` refoundation run](https://github.com/EpiLogos/Actuation/actions/runs/37219849074),
and the cited handler/store/test bytes are identical between those cuts. The
separate authority-store revocation library case passes on Linux only; macOS
does not reach those later stream-store unit tests after its research failure.
Other macOS research failures and its unreached controlled process gate remain open.
The existing capability identities, domain seed, standing and diagram meanings
are retained; these mappings do not add numerical/domain decisions or H credit.
