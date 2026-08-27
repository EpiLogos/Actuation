# ActuationStream

`ActuationStream` is the canonical portable relation for the **ordered attributable material through which a situated Actuation unfolds**.

It exists because `RealisedActuation` can identify that an Agent/Agency is really acting through a body/session, while Surfaces and harnesses expose the act as an evolving sequence of messages, model results, capability/tool operations, world observations, interruptions, artifacts, evidence and Returns. Without a common Stream ref, each Surface can only expose its own provider-local trace and O:I cannot truthfully encounter the same Agency across Cradle, terminal, messaging gateway and other Surfaces.

The core relation is:

```text
Agency / Actuation
    ↓ embodied through
runtime body / HarnessComposition
    ↓ continuous through
AgentSession
    ↓ unfolds as
ActuationStream
    ↓ projected through
Surface(s)
```

The distinctions remain real:

```text
Agent ≠ Agency ≠ Actuation
ActuationStream ≠ AgentSession
ActuationStream ≠ Execution
ActuationStream ≠ Harness-native trace
ActuationStream ≠ Factory Run/trace
ActuationStream ≠ Workcell event log
```

A Stream may correlate all of those objects through refs. It does not absorb their native semantics.

## Identity and continuity

A Stream has its own stable `stream_ref` and explicitly correlates an `actuation_ref`, governing `agency_ref` and `agent_session_ref`. Those identities are distinct.

A Stream is not a UI conversation. Surface attachment and loss are projection/material facts. A Telegram Surface can disconnect while the same Stream continues through Cradle or a harness-native Surface.

The portable event sequence is contiguous and monotonic from `1`. `cursor.last_sequence` and `cursor.next_sequence` describe the exact portable sequence and allow deterministic replay/resume. A Stream may span multiple model turns and Executions while the same situated Actuation/AgentSession continuity remains valid.

Replacing or forking an AgentSession is a separate lineage decision. Session/context refinement belongs in AIKit's AgentSession/Context condition; a new context revision does not silently rewrite prior Stream material. A future continuation/fork contract can cite Stream sequence positions without changing Stream history.

## Attribution

Each event can attribute the material to a locus, Agency, Agent or Participant. This permits one Stream to carry multi-locus material while preserving the governing Agency identity of the containing Stream.

Provider-native traces remain addressable through `native_trace_ref`. Portable projection need not flatten richer target traces.

## Disclosure and hidden model material

`ActuationStream` never requires or fabricates hidden model chain-of-thought.

Portable events may carry disclosed text in `content`. Material that is intentionally not projected can use `disclosure: reference-only` and remain addressable through native/resource/evidence refs without inlining its content. This is a disclosure boundary, not a claim that provider-private internals are available.

## Return

A `return` event must correlate an explicit `return_ref`. Return therefore remains an Actuation relation which can include artifacts/evidence and does not collapse into the final assistant text.

## Portable read / replay / subscribe seam

`contracts/actuation-stream.mjs` supplies:

- validation of Stream identity, lifecycle, ordering and event attribution;
- `actuationStreamReadModel(..., { afterSequence, limit })` for cursor-based read/replay;
- immutable append/close helpers;
- `ActuationStreamJournal` as a small executable reference for append/read/replay/subscribe behaviour.

The Journal is a conformance/reference body, not a requirement that every runtime store its Stream in JavaScript memory. A Workcell service, harness adapter, gateway or other provider may implement persistence/subscription differently while preserving the portable contract.

## O:I gateway / Cradle consequence

The first-party Agency Gateway and Cradle should consume this canonical Stream rather than promote AIKit provider `ConnectionSignal` or a platform-native message trace into a second event ontology.

```text
Telegram / Slack / CLI / Cradle / harness-native Surface
        ↕ projection + attribution
ActuationStream
        ↕
AgentSession / Agency / Actuation
```

This is the semantic seam required for one situated Agency to be encountered through multiple embodied communication contexts and for AIKit to disclose the live ecology of addressable AgentSessions without equating a session with any single Surface.

## L1 bounded reading

Within the current six-product O:I whole, `Actuation ↔ L1.1 material cause` remains a bounded MEF reading: the constituted body/Stream is the material through which the act becomes actual. It is not a universal assignment for every Actuation object in every context.
