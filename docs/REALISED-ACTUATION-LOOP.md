# Realised Actuation / acting-loop contract

Status: implementation contract for Actuation #16.

Actuation begins at the acting loop that has actually been instantiated and develops outward through its architectural constitution. The ordinary directory-bound coding harness is therefore a valid collapsed Actuation when a model is actually acting in a persistent working world. It does not become an Actuation only after AIKit provisions it.

The product boundary is deliberately simple:

```text
Actuation
  WHAT has been instantiated as agency here?
  acting loop · Agency · WorldBinding · recurrence · body relation · Return

AIKit
  HOW is that Actuation provisioned here?
  Context · Skills/Methods · capabilities · models · harness binding · session · Surfaces · native projection
```

The public portable seam is `actuation.realised/v1`, implemented natively in `crates/actuation-runtime/src/realised.rs`, with the frozen wire oracle in `fixtures/migration/` preserving the public contract.

## What the receipt means

A `RealisedActuation` receipt is an observed/read-model account of an existing acting condition. It is not a runtime daemon, harness wrapper, or configuration source. It identifies the enduring `Agent`, situated `Agency`, `WorldBinding`, and Actuation separately from externally owned body facts such as harness, session, process, model condition, and Workcell material binding.

The contract therefore preserves:

```text
Agent
  != Agency
  != Actuation
  != Harness
  != AgentSession
  != process
  != model condition
  != Workcell material binding
  != one loop implementation
```

Changing body, session, process, model condition or material binding is attributable change in the realised condition; it does not silently mint a new Agent.

## Collapsed and articulated cases

The collapsed valid case can contain only:

```text
persistent working world
+ Agent / Agency / WorldBinding
+ observed acting recurrence
+ evidence that the model is actually acting
```

No HarnessComposition, SessionSpace, Workcell binding, QL provider, or AIKit installation is required merely to make that condition valid.

A richer case can add opaque externally owned refs for a harness, AgentSession, process, model condition, material binding, participating loci, `ActuationStream`, and `Return`. Rich target evidence remains native; Actuation records only the portable semantic relation and stable refs needed to attribute the act.

## Observation and degradation

`acting=true` is required. Model availability by itself is not realised Actuation.

An `observed` receipt must cite evidence. Targets that cannot disclose a faculty report it as unsupported or degraded; the contract does not invent cancellation, subagent, stream, lifecycle, or hidden-reasoning semantics to make harnesses look uniform.

`stream_ref` is a relation to Actuation #15. The realised loop is not the `ActuationStream`: the loop/body is the acting condition, while the stream is the attributable unfolding evidence where a target exposes it.

## AIKit handoff

AIKit may consume the stable refs and observed faculties from this receipt when deciding how to provision the same Actuation. AIKit can then project Skills, Methods, ContextSources, models, tools, lifecycle hooks, Surfaces and compact orientation through the target's native faculties.

That changes the effective operative world without moving Agent/Actuation semantic ownership into AIKit. A later AIKit activation observation is evidence about how the target received that provisioning; it is not evidence that the Actuation only began to exist at activation time.


## Exact native turn deadline port

The optional `actuation.native-turn-deadline/cooperative/v1` port restricts an already admitted turn. A forwarded `actuation.native-turn-bound/v1` condition grants no Agency, Source, provider or Factory authority. The host admits under its existing native Task, Agency and audience checks; Factory retains the original dispatch clock (before preparation), immutable total duration/current intent and total-wall Return guard. Host admission never replenishes the original total allowance and never certifies a forwarded Factory epoch.

`NativeTurnBound::canonical_bytes` is the sole condition basis: UTF-8 JSON of the fixed-order nested arrays defined in that method, without map ordering, an admission or a self-digest. Consumers compute `blake3-v1:` followed by 64 lowercase hexadecimal BLAKE3 characters. Legacy omitted conditions remain omitted; strict unknown condition fields refuse. Actual admission binds that digest to the current owner clock incarnation/tick, exact delivery, connection generation and native prompt token. Queue time consumes the same live allowance; replay returns the first receipt; lost/restarted clock continuity refuses before Prompt.

The existing event drain drives `NativeTurnDeadline::step`. The same native port must check the whole selector before adapter permission mutation and its cancellation burst. Requested or written cancellation, provider-observed cancellation, useful late completion and actual owned-process retirement are separate facts. This port cannot terminate a shared body. A required owned-process condition is unsupported before Prompt. Provider non-response and a blocked existing transport write remain uncertain; this cooperative port does not establish a hard process lifetime.

These are additive v0.x Rust source APIs. Existing LocusDriver default cancellation and its requirement for an actually observed Cancelled phase are unchanged. Native compiler/platform/provider gates and genuine current/previous caller checks are required before adoption; source definitions are not execution evidence.
