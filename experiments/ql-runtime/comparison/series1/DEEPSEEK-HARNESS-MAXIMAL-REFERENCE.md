# Series 1 v0.1 — DeepSeek Harness maximal-reference host

Status: normative Series 1 amendment for issue #139  
Scope: add one host (`dsh`) without adding a loop condition or changing Classic / Direct QL / Deep QL semantics.

## 1. Frozen upstream reference

This amendment was derived from the published upstream repositories at the following revisions and MUST be revised explicitly if those sources change:

- `deepseek-ai/deepseek-harness` `master`: `47f943859bef60e4160492346772ded9b24f765a`
- published DeepSeek Harness packages: `0.1.0-rc.5`
- `cordiverse/paper` `main`: `948a07b369c62adb3b12e102458be5c18dfb69b9`

The DSH source revision was re-checked when #139 implementation began. It was still the live `master` head.

## 2. Source-derived integration seam

The Series 1 DSH host uses public DeepSeek Harness composition seams; it does not patch DSH loop internals.

The upstream contracts establishing the seam are:

1. `docs/architecture.md`: the harness is a Cordis plugin tree; model adapters, session log, agent registry and loop are replaceable composition services.
2. `packages/core/agent/README.md`: the `Agent` interface has zero dependency on the concrete loop and explicitly states: **replace the loop by implementing `Agent` and registering via `ctx.agents.register()`**.
3. `packages/core/agent-loop/README.md`: `ReactLoopAgent` and its driver are package-private; behavior beyond the default loop belongs in plugins; UI observes durable `session/event` plus live `agent/*` events.
4. `packages/llm/llm/README.md`: `ctx.llm.stream()` is the provider-neutral public direct-call surface.
5. `packages/llm/llm-deepseek/README.md`: the official adapter owns provider route `deepseek-official`, resolves `DEEPSEEK_API_KEY` per request and supports `deepseek-v4-flash` without adding adapter-authored prompt prose.
6. `packages/core/session/README.md`: `Session` is append-only; `SessionStore` owns live session identity; model-visible history is a derived surface rather than the whole event log.
7. `packages/core/session/src/known-event-types.ts`: rc.5 persistence knows the event families generated in the upstream repository. Downstream event-type registration is explicitly deferred; unknown persisted events are safe only when their envelope carries `ignorable: true`.
8. `packages/session/session-persistence-jsonl/README.md`: JSONL persistence mirrors the SessionEvent stream without adding model-visible input and exposes an awaited durability checkpoint through `SessionStore.flush()`.
9. `docs/cookbook/adding-a-conversation-node.md`: a Web Client plugin can render replayable producer-owned event families through `ConversationNodeDefinition`, but the Host must first possess a persistence-safe event contract.

The Series 1 adapter therefore treats the frozen LoopRuntime as the **replacement DSH Agent driver**. It does not run LoopRuntime inside DSH's concrete `ReactLoopAgent`, and it does not let DSH's default loop make a second set of recurrence/closure decisions.

## 3. Composition

The host-local composition is fixed as:

```text
Cordis Context
├── dsh-llm                    public LlmRuntime service
├── dsh-llm-deepseek           official `deepseek-official` adapter
├── dsh-session                SessionStore
├── dsh-session-persistence    persistence seam
├── dsh-session-persistence-jsonl
├── dsh-agent                  AgentRegistry
└── Series1 DSH adapter
    ├── replacement Agent identity bound to the Series 1 Run
    ├── frozen LoopRuntime driver (classic | ql-direct | ql-deep)
    ├── portable Series1Workspace capability bridge
    └── read-only evidence/inspection projection
```

The composition intentionally omits DSH's model-facing shell, filesystem, web, skill and default agent-loop tools. The candidate capability surface remains exactly the portable Series 1 contract:

- `list_files`
- `read_file`
- `write_file`
- `run_tests`

The task workspace receives no DSH web/search/browser capability and no DSH filesystem/tool schema. Only the provider boundary has network access.

## 4. Candidate request boundary

Every candidate model call travels through DSH's real `ctx.llm.stream()` and official DeepSeek adapter, but it receives the same frozen Series 1 model request assembled by `LiveRuntimeHost`:

```text
system = Series 1 common response/control system text
user   = exact host-built Series 1 prompt payload
model  = deepseek-v4-flash
sampling parameter = temperature 0
```

The DSH host does **not** derive this request from `Session.deriveMessages()`. This is deliberate. DSH SessionEvents are retained inspection evidence and are not an additional candidate context channel. No DSH identity prose, deployment persona, project instructions, observer state, review reference, QL semantic state or Web UI state is prepended to the candidate request.

This is the central non-leak invariant:

> DSH observes the Series 1 run; it does not teach the candidate which runtime condition is running.

## 5. Replacement Agent identity

For each Series 1 condition-run the adapter creates one DSH Session and registers a custom Agent with the same id. The custom Agent exists so DSH's native registry/session/inspection surfaces have a real target identity while the actual driver remains the frozen LoopRuntime.

Its public driving methods (`followup`, `steer`, `inject`, `send`) are not used for benchmark execution and reject observer-originated attempts. Runtime cancellation remains the portable LoopRuntime/AbortSignal contract. DSH Agent status or turn completion never creates a QLClosure.

This preserves the distinction:

```text
DSH process/session/turn state != LoopRuntime Run state != QLClosure
```

## 6. Native evidence and the rc.5 downstream-event constraint

Portable evidence remains canonical. DSH augments it with native session/trajectory evidence; it never replaces it.

### 6.1 Candidate Session — native core events only

The live candidate Session records only upstream rc.5 core event families whose semantics the DSH persistence/runtime already owns. Model calls are bracketed with native `turn/*` and `step/*` events and retain the provider request header/context, user message, raw assistant chunks, assembled assistant message and termination reason.

This gives a genuine replayable DSH trajectory for the provider/model interaction without inventing a new hidden prompt source. The Series 1 candidate is still driven from the portable request; the Session surface is evidence, not subsequent model input.

### 6.2 Portable trace — complete cross-host runtime truth

The portable run record continues to retain every host capability event and every QL semantic/operator event with canonical `record_index` ordering. These are the cross-host experiment semantics and remain complete even when a target host has no native event vocabulary for one of them.

### 6.3 QL inspection sidecar and optional ignorable inspection Session

rc.5's generated `KNOWN_SESSION_EVENT_TYPES` explicitly does not provide a registration surface for downstream plugin event families. The Series 1 adapter therefore MUST NOT append a made-up QL event through a core DSH event name or pretend that a live downstream event is persistence-native when it is not.

The adapter instead builds a read-only QL inspection projection from the portable events. Each item preserves:

- `run_id`;
- `portable_record_index`;
- `portable_channel`;
- `portable_event_type`;
- the complete sanitized portable event.

Where the published rc.5 Session constructor/persistence path accepts an independently seeded inspection Session whose unknown events explicitly carry `ignorable: true`, the adapter may retain the same projection as a **separate post-run inspection Session**. That Session is not the candidate Session, is never used to derive model history, and is evidence-only. This path is evidence-eligible only after a real rc.5 structural test proves replay/persistence acceptance.

If that test does not hold, the inspection projection remains a portable/DSH-adapter sidecar and the candidate Session remains the sole DSH-native durable log. The absence of a downstream persistence registration seam is then recorded as an upstream rc.5 limitation rather than concealed by abusing an existing event type.

### 6.4 Alignment

Every DSH provider call is assigned an ordinal and the native Session seq range that brackets it. The portable host `model_requested` / `model_returned` events are aligned by the same call ordinal after execution. Capability and QL-only events remain aligned through their portable `record_index` and surrounding model-call ranges.

Evidence can therefore navigate:

```text
portable model call ordinal N
        ↕ exact recorded alignment
DSH SessionEvent seq range A..B

portable QL/capability event record_index K
        ↕ chronological run relation
portable trace + DSH inspection projection
```

Before evidence is declared complete, the adapter awaits the DSH SessionStore persistence flush barrier.

## 7. Composition fingerprint

Every DSH condition record carries the same `host_composition_fingerprint` for a given implementation revision. The fingerprint binds at least:

- DSH upstream revision;
- package version `0.1.0-rc.5`;
- exact mounted service/plugin list;
- official DeepSeek adapter route;
- candidate capability contract digest;
- inspection projection schema version;
- DSH Series 1 adapter implementation revision;
- persistence mode.

The fingerprint MUST NOT contain secrets, runtime condition identity or mutable session/run ids.

A mismatch between Classic / Direct / Deep invalidates the matched set before interpretation.

## 8. Human inspection and QL observational view

The DSH lane exposes two read models:

1. DSH's native candidate Session/trajectory for the real model/session chronology;
2. the Series 1 QL inspection projection for runtime semantics that are not native DSH events.

A Web Client `ConversationNodeDefinition`/trajectory component may consume the evidence-only inspection Session when the rc.5 ignorable-event path is proven. Otherwise the same read model is rendered by the Series 1 review surface alongside links/coordinates into the native DSH Session. In either case the component contract is observational: it consumes retained evidence and has no Agent-driving or model-context authority.

The QL read model exposes, where the underlying portable events contain them:

- selected LoopRuntime condition (Pass B only);
- Circuit id/current position and historical transitions;
- `Rij` relation events;
- π / ρ and difference evidence;
- P5 candidate determination separately from positive closure;
- continuation/reopen/re-entry facts;
- depth, conjugation and square-modulation events;
- portable event index beside the aligned DSH model-call/session range.

The view MUST NOT:

- append candidate Session surface messages;
- call Agent `inject`, `steer`, `followup` or `send`;
- alter model requests;
- expose human-only reviewReference material to the candidate;
- synthesize a winner or scalar quality score;
- treat event volume as quality.

Pass A review remains generated by the portable masked renderer. DSH native/QL inspection is Pass-B/unmasked evidence because its runtime/QL vocabulary can reveal condition identity.

## 9. Failure and cancellation semantics

DSH transport/provider/session failures are execution evidence. They do not count as negative QL closure and do not silently trigger a different runtime path.

Distinct facts remain distinct:

- provider request failed;
- provider request aborted;
- DSH session persistence failed;
- DSH turn ended;
- LoopRuntime execution ended;
- QL Circuit remained open;
- positive QLClosure occurred.

No fixture or stub DSH provider is evidence eligible. Structural tests may use pure projection functions without a model call, but `--live` readiness requires the pinned published DSH packages, the real `deepseek-official` adapter, and `DEEPSEEK_API_KEY`.

## 10. Series 1 role

`dsh` is a fourth **host**, not a fourth **condition**.

The comparison remains:

```text
same DSH composition + same DeepSeek model + same exact task
vary only:
  classic
  ql-direct
  ql-deep
```

No benchmark task, Direct Core rule, Deep operator rule, success condition or human-review protocol is amended by adding this host.
