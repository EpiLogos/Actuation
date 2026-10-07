# Actuation

Actuation is a Rust command-line tool and set of libraries, `actuation`. It records which agent holds which position, on whose authority and within what bounds, and the events and results reported for each act. It keeps those records in append-only files that other tools and people can inspect. It holds O:I's **agency** facet: who acts, for whom, within what bounds, and how results come back.

Actuation does not launch or run agents. It is the place where an agent's standing to act is stated, checked and recorded, while harnesses, Workcell and AIKit do the running.

## What it does today

Three words carry the design, and Actuation keeps them apart:

- **Agent**: an enduring identity, such as "the reviewer".
- **Agency**: that Agent placed for a particular act: in which world, with what authority and bounds.
- **Body**: the session, process, model and harness that carry the act this time. Changing the body does not create a new Agent.

**Works now** (installed `actuation` 0.2.1; `actuation verify` passes its 15 compiled-in checks):

- **Authority.** `actuation authority issue | resolve | revoke` keeps an append-only local record of who holds an explicit grant, who issued it, and which worlds it covers. Grants are never implied by saving, preparing or starting something.
- **Agency admission.** `actuation agency actualise` takes one complete request and refuses it unless the grant matches the exact governing Agency and world binding and covers the requested bounds. The receipt keeps identity, lineage, authority and the return path. It states plainly that it performed no materialisation, no Factory recognition and no source change. Workcell calls this before admitting a run.
- **Who holds a position.** `actuation occupancy claim | release | verify | presence | read | list` keeps a ledger of who occupies a stable World Position defined by Central, for example `central:position:project:Factory:factory-sensing-guardian`. Every claim gets a new generation that is never reused. To replace an occupant you must name its current generation, and ambiguous or corrupt ledgers are refused, never resolved by "newest wins". AIKit's launcher, Factory and O:I all use this today.
- **Activity records.** `actuation stream open | record | replay | close` keeps an append-only JSONL record of attributed events: messages, tool calls, refusals, delegations, evidence and returned results. The stream records what was reported. It does not perform the action. `actuation stream usage` reads Claude Code transcripts and Codex `exec --json` output into token and usage observations, and never stores prompt or response content.
- **Harness detection.** `actuation harness catalog | detect | capability` declares 24 known harnesses, proves which are installed on this machine, and says which hook events each accepts. AIKit and Central both use this as their harness census.
- **Agency Gateway** (`actuation-gateway`, a separate binary). A Unix-socket protocol through which surfaces attach to agent sessions and one agent may invoke another under policy. Refusals become stream events. Factory uses it to deliver attempts.
- **Disclosure to O:I.** `actuation system` and `actuation config-contribution`.

**Limits, stated plainly:**

- Identities are references supplied by the caller. Actuation checks and records them; it never mints an Agent.
- The authority record is local material protected by file access. It is not a cryptographically authenticated claim.
- Actuation does not decide whether a returned result is accepted; that judgement belongs to whoever owns the world being changed. There is no CLI command to create a return record directly.
- `actuation harness self` cannot yet identify most harnesses, including Claude Code, because only two catalogue entries declare an environment probe.
- The configuration plane only discloses settings. `config plan | apply | reset` always refuse.
- The `revision` field in `actuation capabilities --json` currently reports the git HEAD of whatever directory you run it in, not the revision the binary was built from.
- The Nara speech actor (`actuation nara serve`) performs no audio or provider I/O.
- Research runs exist as recorded evidence, but no capability effect is claimed.

## How it fits O:I

O:I gives each facet of an agent's world its own product. Actuation holds **agency**. It meets the other facets only through command-line and JSON contracts. No other suite product is a crate dependency in either direction.

| Neighbouring facet | Where they meet |
|---|---|
| Ground / Central | World Positions (`central:position:<world>:<slug>`) are defined by Central and carried here verbatim. Central's harness connector calls `actuation harness detect` and `harness capability`. |
| Capability / AIKit | AIKit reads harness detection and capability as its candidates and calls `agency actualise`. Its launcher (`aikit inhabit`) claims occupancy with `--expect-vacant`. Actuation answers *what* has been instantiated as agency; AIKit answers *how* it is provisioned. |
| Development / Software Factory | Factory delivers attempts through the Agency Gateway (`actuation.gateway/v1`). Its custody of work is gated on current occupancy, and its telemetry mutations need a local-authority admission. A Factory Commission never grants Actuation authority. |
| Environment / Workcell | Workcell runs `actuation agency actualise` and requires `status: actualised` before it admits a run. An occupancy claim can name the Workcell where the body sits. |
| Reflection / QL | Research only: the research crate binds an exact `ql` binary, and the Nara pipe admits QL's dialogue context. Generic Actuation needs no QL. |
| O:I | `oi agent roster` and `oi agent participation` read occupancy. `oi actuation …` dispatches to `actuation`. |

**In the Cradle.** The O:I desktop shows who is acting, under what authority and within what bounds, and the path by which results return. The Cradle's kernel reads Actuation's permission and occupancy state as one of its owner readings.

## Install and quick start

Through O:I (ordinary route):

```sh
oi install actuation
actuation verify
```

From source (developer route; stable Rust, no Node runtime):

```sh
git clone https://github.com/EpiLogos/Actuation
cd Actuation
cargo build --release --locked -p actuation-cli     # binary at target/release/actuation
```

GitHub releases carry `macos-arm64` and `linux-x86_64` archives with sha256 checksums and artifact attestations. Note that the `actuation-v0.2.1` release archive was built before the occupancy, Nara and capability-intake commands landed on `main`, which still reports version 0.2.1. For the full command surface below, build from source or install through `oi`. `oi` reports that the suite's builds "have not passed physical acceptance".

First commands:

```sh
actuation harness detect                 # which agent harnesses exist on this machine
actuation harness capability claude-code # what hook events that harness accepts
actuation occupancy list                 # who currently holds which World Position
actuation agency actualise --schema      # a filled example admission request
actuation capabilities --json            # the full command surface and contract versions
```

## Command reference

`actuation --help` prints every command with one line on what it does. Per-command `--help` is not supported.

```text
actuation capabilities [--json]
actuation nara serve
actuation contract list [--json]
actuation agency [file|-] [--json]
actuation agency actualise <file|-> [--schema] [--json]
actuation authority issue [--store <dir>] [--now <ts>] [file|-] [--json]
actuation authority resolve [--store <dir>] [--now <ts>] [file|-] [--json]
actuation authority revoke <authority_source_ref> [--reason <text>] [--store <dir>] [--now <ts>] [--json]
actuation realised [file|-] [--json]
actuation occupancy claim --position <ref> --agent <ref> --agency <ref> [--agent-session <ref>] [--session-space <ref>] [--harness-composition <ref>] [--model <ref>] [--workcell <ref>] [--gateway-address <addr>] --reason <text> [--expect-vacant | --expect-generation <generation>] [--kind initial|handover|fresh|adopt] [--store <dir>] [--json]
actuation occupancy release --position <ref> --generation <generation> --reason <text> [--store <dir>] [--json]
actuation occupancy verify --position <ref> --generation <generation> [--store <dir>] [--json]
actuation occupancy presence --position <ref> --generation <generation> --presence active|idle|away|offline [--attention <text>] [--store <dir>] [--json]
actuation occupancy read --position <ref> [--store <dir>] [--json]
actuation occupancy list [--store <dir>] [--json]
actuation stream [file|-] [--json]
actuation stream open [--store <dir>] [file|-] [--json]
actuation stream record [--store <dir>] [file|-] [--json]
actuation stream usage [--store <dir>] [file|-] [--json] [adapter: claude-code-transcript|codex-exec-jsonl|observation]
actuation stream replay <stream_ref> [--after <n>] [--limit <n>] [--store <dir>] [--json]
actuation stream close <stream_ref> [--state closed|interrupted|cancelled] [--ended-at <ts>] [--store <dir>] [--json]
actuation activity [file|-] [--json]
actuation usage [file|-] [--json]
actuation instantiation [file|-] [--json]
actuation instantiation record [--allow-unattributed] [--out <file>] [file|-] [--json]
actuation harness catalog [--json]
actuation harness detect [--only <slugs>] [--versions] [--json]
actuation harness self [--json]
actuation harness capability [<slug>] [--json]
actuation harness capability validate <file|-> [--json]
actuation system [--json]
actuation config-contribution [--json]
actuation config-contribution capability <file|-> [--json]
actuation config validate [--json] [--setting <setting_ref>] [--scope <compact>] [--value <json> | --value-file <path|->]
actuation config plan [--json] [--setting <setting_ref>] [--scope <compact>] [--value <json> | --value-file <path|->]
actuation config apply [--json] [--plan-file <path|->] [--changeset <id>]
actuation config reset [--json] [--setting <setting_ref>] [--scope <compact>] [--changeset <id>]
actuation verify [--json]
```

`nara serve` owns a bounded JSON-lines pipe. Each newline-terminated request
uses `schema: "actuation.nara-session-request/v1"`, a unique `request_ref`, and
`operation`. Replies use `actuation.nara-session-response/v1`, echo the request
reference, and carry `ok` plus the native binding `reading` or an `error`.
The actor flushes every reply before reading again; EOF or `close` releases
this ephemeral actor, without destroying the canonical AIKit conversation or
claiming provider cancellation. A line is limited to 256 KiB, a reply to 1 MiB,
and an actor to 65,536 distinct request references. Oversized or unterminated
lines end the actor with status 2. Malformed requests and reused references
are refused without changing its binding. The pipe owner must inspect state
after uncertain delivery; request references are not a retry protocol.

The pipe owner admits the existing `actuation.speech-constitution/v1` and
`ql.nara-dialogue-context/v1` as `constitution` and `dialogue_context` in
`constitute`, alongside `allowed_action_refs` and `denied_action_refs` arrays.
The operations are `read`, `context` (`dialogue_context`), `listen`, `response`
(`response_ref`), `complete` (`response_ref`), `interrupt`, `reconnect`, and
`close`. Completion must name the current response. Reconnect supplies a fresh
constitution and context plus `change_ref`, `reason`, nonempty `evidence_refs`,
and RFC3339 `at`; this pipe retains its original AgentSession.

Interrupt requires `interruption_ref`, the current `response_ref`, `reason`,
nonempty `evidence_refs`, RFC3339 `at`, and `effect` equal to `playback-stopped`
or `provider-cancelled`. The caller supplies evidence of an already observed
transport effect, never just a stop request. The native interruption receipt
is accompanied by a separately attributed `transport_effect`: stopping local
playback does not claim provider cancellation. The actor does not authenticate
external evidence, open a microphone, stop audio, invoke a model, or provide a
network endpoint; its parent broker owns admission and those actual effects.
Contract-process tests establish native lifecycle behavior, not live voice.

The command surface, help text and capabilities listing are all derived from
one Rust-owned command table (`crates/actuation-cli/src/dispatch.rs`); parity
is enforced by the crate tests and the frozen command-surface scenarios, so
the CLI cannot lie about itself. `harness catalog` declares what can be
detected, `harness detect` proves it live on this machine, and `--versions`
enriches receipts by executing detected binaries with their declared
`version_args` (opt-in: some version probes are slow or prompt the
keychain; failures are disclosed, never folded into detection state).
`config-contribution` emits Actuation's configuration contribution
(`oi.configuration-contribution/v1`): the settings Actuation genuinely owns
as configuration — its declared agency constitution (determination kinds,
WorldBinding constraint categories, metagency operations, derivation and
federation authority rules), its Return modes and its durable stream store
selection. `config-contribution capability <file|->` is the public intake of
harness capability descriptors: it validates a descriptor that fills a
declared capability gap and mints a receipt the owner lands into the bundled
catalog (see docs/HARNESS-CAPABILITY.md). Every contributed subject is
declared code, not applied
configuration: `writable` is false, and `plan`/`apply`/`reset` are
structurally unavailable. The four config verbs implement the frozen
owner-native transport (`config validate|plan|apply|reset --json`), and they
preserve Actuation's authority law end to end: discoverability is not
authority, O:I root position confers no Actuation permission, a mutation of
the authority constitution is refused as `not_authorised` (it would be a
metagency configure-agency act under an explicit grant, which the transport
carries no channel for), every other mutation is refused as
`unsupported_setting`, and failures exit non-zero with an
`oi.config-error/v1` document on stdout — never a silent success. The frozen
argument grammar is `config validate --json --setting <setting_ref>
[--scope <compact>] (--value <json> | --value-file <path|->)` for validate
and plan (scopes default to this machine; compact form is `kind:ref`), `config
apply --json (--plan-file <path|->) [--changeset <id>]`, and `config reset
--json --setting <setting_ref> [--scope <compact>] [--changeset <id>]`.
An executed idempotency key (owner, changeset, setting, scope, plan digest)
replays as outcome `no_op` naming the original receipt instead of
re-executing.
`instantiation record --out <file>` appends bound receipts as JSONL.
`occupancy` keeps the tenure ledger of stable World Positions (the
addresses Central defines as `central:position:<world>:<slug>`; see the
[World inhabitation contract v1](https://github.com/EpiLogos/O-I/blob/main/docs/contracts/WORLD-INHABITATION-V1.md) §2).
A Position is an address, not an Agent, Agency, AgentSession or composition
locus: it survives every change of occupant, model, harness and Workcell.
Each occupancy is a tenure with its own generation
(`actuation:generation:<uuid>`, per-Position ordinal max + 1), appended to
one JSONL ledger per Position under `$ACTUATION_OCCUPANCY_STORE` (default
`~/.actuation/occupancy/`) inside one exclusive lock. `claim` on a vacant
Position needs no expectation (or `--expect-vacant`); on an occupied one it
must name the current generation with `--expect-generation`, and then
supersedes it in the same write. A superseded or released generation can no
longer `verify`, report `presence` or `release`. The current occupant is the
single open tenure: two open tenures or an unreadable line are refused as
`occupancy.ambiguous` / `occupancy.corrupt`, never resolved newest-wins.
Every refusal exits 2 with `{ok:false,error:{code,fact,consequence,action}}`
under `--json` (three lines on stderr otherwise), naming the current holder
and the exact next lawful command. `claim` returns the `OI_POSITION_REF` and
`OI_OCCUPANT_GENERATION` values a launcher stamps into the body's environment.
`agency actualise` accepts one complete semantic request — the same envelope
`actuation authority resolve` assembles; `--schema` prints a filled example — and
fails closed unless
its `MetagencyGrant` matches the exact governing Agency and WorldBinding,
authorises determination (and, for derivation, actualisation), and covers the
declared bounds. Its receipt preserves Agent/Agency identity, WorldBinding,
lineage, authority and Return while explicitly performing no materialisation,
Factory recognition or source mutation.
`stream usage` accepts one declared adapter document and writes only its
typed usage observation into the durable Stream: `claude-code-transcript`
normalizes a native Claude Code transcript assistant event,
`codex-exec-jsonl` normalizes one bounded Codex `exec --json` invocation,
and `observation` takes an already-normalized `actuation.model-usage/v1`
observation straight through — all three adapters validate before anything
is recorded, and all three share the exact same stream-consistency, dedup
and append-only path. Provider message/request identity, model, tokens and cache
facts are retained where the native record supplies them; prompt and
response content are never persisted. Missing provider, latency or cost
evidence remains explicitly `not-reported`. An undeclared adapter is refused
by name rather than silently accepted.

---

## The model behind the commands

### Agent, Agency and composition

Actuation preserves several identities that are easy to collapse in implementation:

```text
Agent
    enduring semantic identity

Agency
    that Agent situated for an act relative to world, role, authority,
    capability, stance and context

AgentSession
    replaceable runtime/session continuity

Execution
    one concrete act

Harness / body
    an operational constitution through which the act is realised
```

Changing a process, model, harness, session, machine or Workcell does not by itself create a new enduring Agent.

The primary generic relation is **Actuation**:

> An Actuation is the whole-preserving relation by which a situated Agency determines a bounded plurality of agentic loci, governs how those loci may operate and interrelate, and admits their returned difference into the world from which the determination arose.

An Actuation establishes an `AgenticComposition`: a semantic plurality of Agents and/or Agencies related as one operative whole for a purpose in a world.

The determining position is a relation, not a `ManagerAgent`, `MasterAgent` or `WorkerAgent` species. Any authorised locus may recursively govern another composition while remaining governed relative to a wider one.


### Different ways an Other can enter

Actuation distinguishes at least four determination relations because they carry different identity and authority consequences:

- **self-differentiation** — several situated Agencies of one enduring Agent;
- **delegation** — an existing Agent acts under a bounded delegated Agency;
- **derivation** — a new enduring Agent is explicitly created with lineage and authority provenance;
- **federation** — an independently grounded Agent participates without being re-described as a derivative of the governing Agent.

Federation matters because a shared field can contain genuine Others. Composition must not acquire the right to rewrite their origin merely because they participate.


### Determination, labour and Return

Actuation models agency as a circuit rather than a one-way command tree.

```text
purpose / intention
      ↓
determination
      ↓
bounded delegated autonomy
      ↓
encounter / labour / interaction
      ↓
difference + evidence + dissent
      ↓
Return
      ↓
reconstituted governing world
      ↺
```

The determining locus and the locus encountering actuality can be separated. One may set purpose and authority while another meets resistance, error, contingency and unexpected possibility in the world.

That separation is why a downward authority path is not enough. **If evidence and difference cannot travel back into the locus that governs what happens next, command becomes insulated from consequence.** A full Actuation therefore requires an admissible Return path, or an explicitly declared autonomous termination, and preserves attributable returned material before synthesis can erase who discovered what.

The same relation makes dissent and refusal structurally important. A returned disagreement or refusal can be evidence about bounds, world state or governing assumptions rather than simply a failed worker result.


### Metagency and root agency

**Metagency** is an Agency's capacity to take agency itself as an object of action: to establish, bind, configure, relate, suspend, recall, replace or reintegrate agentic loci inside its world.

A `RootAgency` is not a superior species. It is an Agency whose `WorldBinding` is the enclosing Objective Internality for the relevant scope. Rootness is positional and scoped; the same grammar can recur inside nested worlds and compositions.


### Current generic relation

The current constitutional shorthand is:

```text
Actuation(g, W, I) = C⟨g, L, D, Γ, B, R⟩
```

where:

- `g` is the governing/determining situated Agency;
- `W` is its operative world/scope;
- `I` is the purpose under which agency is differentiated;
- `L` is the participating agentic loci;
- `D` records determination and lineage;
- `Γ` records dependence, coordination, review, isolation and other inter-locus relations;
- `B` records authority, capability, resource and world bounds;
- `R` records how attributable returned difference can re-enter the governing world.

The tuple is useful because it makes the relation inspectable. It is not the reason Actuation exists; the reason is the human and technical need to know how agency was constituted and how consequence can revise it.


### What changes when agency becomes first-class

A model invocation can be treated as an opaque worker process, or it can be situated inside an explicit account of agency.

Actuation opens the latter possibility. A human or agent can ask:

- **Who can shape this agency?** What authored purpose, identity and determination produced the situated actor?
- **Who can exercise it?** Which Agent or Agency actually holds the capacity and authority to act?
- **Who sets its conditions?** Which bounds, capabilities, resources, worlds and delegated powers constrain the act?
- **Who observes it?** Which evidence and provenance survive the work rather than disappearing into a final answer?
- **Who can refuse?** Does an independently grounded or delegated locus retain meaningful bounds rather than becoming ambient subordinate process state?
- **Who learns from its operation?** Which actor or world receives the returned difference?
- **What comes back to the world that bore the consequences?** Can resistance, error, dissent, failure and unforeseen possibility revise the governing determination?

These are political questions in the literal architectural sense: they concern the distribution of power, authority, visibility and consequence. They do not disappear because the actors are software.


## Relation to the wider O:I field

**O:I** is the whole field in which differentiated technological worlds can be composed and selectively related. An O:I `SharedField` is not an `AgenticComposition`: disclosure and participation do not automatically confer determination or execution authority.

**Central** remains the persistent authored personal ground and natural residence of world-bound agents. Actuation can describe agency in that world without becoming the owner of the person's Control source.

**AIKit** resolves the operative body and horizon of a locus — Context, capabilities, models, `HarnessComposition`, `AgentSession` and Surfaces. That body realises an actor; it is not the semantic plurality of actors.

**Software Factory** owns developmental Project/Run/evidence/candidate/Recognition semantics. Factory can commission or consume an Actuation when development requires first-class agentic composition, while Actuation does not become a development workflow.

**Workcell** supplies processes, services, storage, network relations and lifecycle. Material placement can change without changing Actuation identity.

**Quaternal Logic** can formally refract Actuation and can supply optional recurrence or bimba/pratibimba profiles. Generic Actuation remains number-neutral and does not require QL terminology as software ontology.

## Reference runtimes and experiments

Actuation develops portable contracts against real runtimes and harnesses so the ontology is pressured by actual implementation rather than protected by toy examples.

The current maximal reference harness is the pinned public **DeepSeek Harness (DSH)** used by the QL Runtime proving body. It is a conformance specimen, not a suite-wide dependency or the source of Actuation semantics. See [`docs/HARNESS-REFERENCE.md`](docs/HARNESS-REFERENCE.md).

The QL Agent Runtime experimental programme was migrated from Software Factory with its evidential status intact. At the migration boundary there were **zero claimed live Series 1 capability runs**. Live runs have since been recorded as evidence (for example `fixtures/research/series1/2026-09-13-glm-native-S1-RESTRAINT-001.json`), but no capability effect is claimed, so structural/conformance evidence must not be rewritten as a capability-effect result.

Current research also studies model-bearing agency and epistemic cultivation. The accepted experiment-local record floor validates and persists declared research artifacts and their access/provenance conditions; it does not claim that a provider exposed model-interior access or that any empirical result occurred. Observations should return into the constitution explicitly rather than being promoted by prose alone.

## Repository map

- [`docs/ACTUATION-CONSTITUTION.md`](docs/ACTUATION-CONSTITUTION.md) — constitutional purpose, determination modes, authority, Return and product boundary.
- [`docs/ACTUATION-RELATION.md`](docs/ACTUATION-RELATION.md) — the one↔many↔return relation and recursive composition grammar.
- [`docs/ARCHITECTURE-NAVIGATION.md`](docs/ARCHITECTURE-NAVIGATION.md) — each concern mapped to its native source, CLI operation and tests.
- [`docs/WORLD-BOUND-ROOT-AGENCY.md`](docs/WORLD-BOUND-ROOT-AGENCY.md), [`docs/REALISED-ACTUATION-LOOP.md`](docs/REALISED-ACTUATION-LOOP.md), [`docs/ACTUATION-STREAM.md`](docs/ACTUATION-STREAM.md), [`docs/ACTIVITY.md`](docs/ACTIVITY.md) — the WorldBinding, realised-loop, stream and activity contracts.
- [`docs/SYSTEM-PLACEMENT.md`](docs/SYSTEM-PLACEMENT.md) — placement across the wider O:I field.
- [`docs/HARNESS-REFERENCE.md`](docs/HARNESS-REFERENCE.md) — maximal-reference harness policy and portability boundary.
- [`docs/HARNESS-CAPABILITY.md`](docs/HARNESS-CAPABILITY.md) — `actuation.harness-capability/v1`: what a dispatch-relevant harness is, including optional `model_dispatch`.
- [`docs/EPISTEMIC-CULTIVATION-AND-MODEL-INTERIOR-RESEARCH.md`](docs/EPISTEMIC-CULTIVATION-AND-MODEL-INTERIOR-RESEARCH.md) — research specification for epistemic cultivation and graded model-interior access.
- [`docs/MODEL-BEARING-AGENCY-RESEARCH-AND-MATERIALISATION.md`](docs/MODEL-BEARING-AGENCY-RESEARCH-AND-MATERIALISATION.md) — model-bearing agency and materialisation research ground.
- [`schemas/actuation.v0.schema.json`](schemas/actuation.v0.schema.json) — language-neutral experimental `AgenticComposition` contract.
- [`docs/QL-RUNTIME-MIGRATION.md`](docs/QL-RUNTIME-MIGRATION.md) — provenance and acceptance rules for the migrated QL runtime experiments.
- [`catalog/targets.json`](catalog/targets.json) — the versioned harness catalog bundled into the executable; `actuation harness detect` proves which operative bodies exist on this machine (`actuation.harness-detection/v1`).
- [`crates/`](crates/) — the native Rust workspace: `actuation-core` (constitutional semantics), `actuation-runtime` (the acting relation), `actuation-stream` (actuality and durable streams), `actuation-adapters` (boundary observation, instantiation, usage), `actuation-research` (first-class research), `actuation-cli` (the served executable), `actuation-gateway` (the first-party Agency Gateway encounter plane; see its [README](crates/actuation-gateway/README.md)) (the retired `actuation-migration-gate` is no longer a workspace member).
- [`fixtures/research/`](fixtures/research/) — live-evidence parity receipts and recorded specimen-conformance fixtures (the retired JavaScript experiment tree's live function now runs in `crates/actuation-research`; see [`docs/rust-refoundation/R11-JS-RETIREMENT.md`](docs/rust-refoundation/R11-JS-RETIREMENT.md)).

The Actuation Wayfinder in the issue tracker records current development state. Main and accepted evidence determine present implementation truth; open PRs are not silently described here as completed capability.



---

## Background

O:I stands for Objective : Internality. It names the means through which a life knows and acts within a world: memory, language, tools, permissions and other people. Those means are internal because every act proceeds through them, and objective because each can be examined and changed. Actuation makes one of those means, the standing under which an agent acts, something that can be stated, checked and revised. The idea is developed in the essay [*Confronting the Limit: Determination, Subjectivity and Mind as Objective Internality*](https://oi.epi-logos.org/essay/).
