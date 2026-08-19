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

The public portable seam is `actuation.realised/v1`, implemented by `contracts/realised-actuation.mjs` and `contracts/realised-actuation-v1.schema.json`.

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
