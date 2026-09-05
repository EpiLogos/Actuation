# Actuation Harness Reference Principle

**Status:** foundation contract, v0.1  
**Scope:** reference-harness policy for Actuation semantics and conformance

## Purpose

Actuation owns generic semantics for Agent, situated Agency, Actuation, AgenticComposition, world binding, determination/lineage, return, and the relation between semantic agency and an operative body. Those abstractions must not be designed only against a minimal toy harness.

Actuation therefore maintains a **maximal reference harness**: a deliberately demanding real harness composition used to expose missing scope, lifecycle, identity, session, persistence, model-routing, observation, delegation, cancellation, and composition requirements in the generic contracts.

The current maximal reference harness is the pinned public **DeepSeek Harness (DSH)** source used by the QL Runtime proving body. Its exact experimental integration and evidence boundary are specified in [`../experiments/ql-runtime/comparison/series1/DEEPSEEK-HARNESS-MAXIMAL-REFERENCE.md`](../experiments/ql-runtime/comparison/series1/DEEPSEEK-HARNESS-MAXIMAL-REFERENCE.md).

## Constitutional rule

> **The maximal reference harness informs the breadth of Actuation's portable abstractions; it does not become their ontology.**

A root Actuation primitive is stronger when it can be embodied cleanly in a harness with rich agent/session/composition semantics. It is not stronger merely because it names DSH classes or assumes one provider's process topology.

Accordingly:

- Actuation MUST exercise its root contracts against the maximal reference harness where the relevant seam exists.
- DSH-specific identities, event names, plugin classes, provider routes, package topology, and persistence details MUST remain in adapters, conformance fixtures, evidence, or harness-specific profiles.
- A conforming Agent, Agency, Actuation, AgenticComposition, WorldBinding, or return relation MUST remain expressible without DSH.
- AIKit, Factory, Workcell, Central, O:I, QL-MEF, and external implementations are not required to adopt DSH merely because Actuation uses it as the maximal reference.
- Alternative harnesses and future DSH forks/derivatives MAY target the same portable Actuation seams without becoming semantically subordinate to the pinned reference implementation.

## Harness detection (detection-first ground)

Actuation owns WHAT operative bodies exist here and what they mean. The
catalog (`detection/`) declares the harnesses this product can detect — one
descriptor module per harness, mostly data (probe spec, adaptation facets,
provenance) — and `actuation harness detect` proves them live into
`actuation.harness-detection/v1` records. The law of that contract:

- a harness is **detected** only when a probe proved presence, with receipts captured in the same run;
- **unavailable** always carries a reason and is never silently read as absence;
- **not-installed** requires probe evidence of absence — "could not run" never collapses into "ran and found nothing".

Adding a harness is one descriptor module plus one import in
`detection/catalog.mjs` and a `CATALOG_REVISION` bump: a small mechanical
step that an LLM can perform against an upstream release, with the contract
tests as the gate.

## Why a maximal target matters

Minimal harnesses are useful for proving portability but are weak discovery environments for the full agentic problem. A richer reference can force the architecture to confront, at minimum:

- enduring Agent identity versus situated Agency;
- semantic identity versus AgentSession and process lifetime;
- replaceable harness/body composition;
- provider/model routing without provider identity becoming Agent identity;
- lifecycle and cancellation;
- durable session/evidence continuity;
- scoped composition and nested agency;
- ownership and initiator attribution;
- observation/read models that do not silently acquire driving authority;
- persistence and restart boundaries;
- provenance across body, session, model, host, and semantic actuation changes.

These pressures are useful even when a smaller runtime implements only a strict subset. The resulting generic grammar should describe both the maximal case and the collapsed case without bifurcating into separate ontologies.

## Reference versus default

"Maximal reference" does **not** mean:

- Actuation is a DSH wrapper;
- every agent must run in DSH;
- DSH is the only accepted harness;
- AIKit must resolve DSH for every AgentSession;
- Workcell must host DSH-specific services;
- Factory must regain ownership of the migrated runtime experiment;
- Central's resident personal agent is relocated into this repository.

It means that when Actuation develops a portable semantic or embodiment seam and DSH exposes a relevant richer case, the reference implementation should prove that seam there rather than leave the difficult case theoretical.

## Pinning and evolution

Conformance is reproducible only against an exact upstream revision. The QL Runtime experiment therefore clones and builds the pinned public DSH repository in CI rather than depending on an ambient global installation or mutable package-manager state.

A newer upstream revision, fork, or derivative may become the reference only through an explicit conformance update recording:

1. source identity and immutable revision;
2. public seams relied upon;
3. adapter changes;
4. structural and live acceptance evidence;
5. any newly exposed portable requirements;
6. any DSH-specific concerns deliberately kept out of the generic Actuation ontology.

This makes the reference harness an architectural pressure test and evidence source, not an accidental source of semantic coupling.
