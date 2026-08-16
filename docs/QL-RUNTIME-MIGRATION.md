# QL Agent Runtime migration

**Status:** active migration plan  
**Source repository:** `EpiLogos/agent-system-design`  
**Source Wayfinder:** issue `#94` — QL Agent Runtime Experiments  
**Source integration PR:** `#130` — `ql/deep-runtime`  
**Pinned source head:** `a654c62f68b82236061986d9215b23257fe53b17`

## 1. Why this moves

The QL Agent Runtime programme began inside Software Factory because Factory provided an immediate real agentic proving ground. That location was useful for development but is no longer the correct ownership boundary.

The experiment's own established seam is already generic:

```text
AGENT HOST / HARNESS
  model + capabilities + context/session
             │
             ▼
        LOOP RUNTIME
  classic | ql-direct | ql-deep
```

The programme intentionally varies recurrence while keeping host/body/model/task variables observable. That is research into first-class agent/runtime actuation, not a definition of Factory developmental semantics.

Actuation is therefore the canonical developmental home for the programme from this migration onward.

## 2. What is being imported

The migration imports the **entire `ql-agent-experiments` tree at the pinned PR #130 head**, not only PR #130's current diff.

That is required because PR #130 builds on a frozen Direct Core and earlier host/foundation work already present on its branch/base history. Copying only its 51 changed files would strand the Deep layer without the programme it amends.

The imported tree includes, where present at the pinned source head:

- the QL agent specification and development protocol;
- Loop Runtime foundation and frozen Direct Core material;
- Classic / Direct / Deep runtime implementations;
- Pi, Pydantic AI, Native and DeepSeek Harness host work;
- conjugation, recursive depth, pairing/square and typing-corpus work;
- deterministic structural/conformance evidence;
- Series 1 benchmark and review tooling;
- the current structural/live workflows adapted to the Actuation path.

## 3. Evidence status is preserved

Migration MUST NOT upgrade the evidential status of the programme.

At the pinned source state:

```text
structural / conformance evidence     present
live Series 1 capability runs         0
capability-effect determination       unclaimed
```

The migration therefore preserves the source programme's explicit discipline: negative, mixed and null results are first-class; deterministic convergence is not proof that QL improves an LLM; host richness must not be misattributed to recurrence.

## 4. Source history remains authoritative provenance

This repository initially imports a pinned snapshot rather than rewriting 119 commits of experimental history.

The original review/provenance remains available in Factory issue #94 and draft PR #130. The Actuation copy records the exact source commit so results can be traced back byte-for-byte at the migration boundary.

After the Actuation import is accepted, new runtime-experiment development should occur here. Factory should retain only a relocation/consumer note and any Factory-specific integration adapters that are genuinely developmental-system concerns.

## 5. Path transition

Canonical path after migration:

```text
experiments/ql-runtime/
```

Former source path:

```text
ql-agent-experiments/
```

Imported CI workflows are rewritten to target the new path. Internal references that name the old repository path are mechanically rewritten where they describe local paths; historical prose may continue to mention the former location when doing so is provenance rather than an executable path.

## 6. Ownership after migration

Actuation owns the experiment body and generic Loop Runtime research seam.

Software Factory MAY:

- consume an Actuation runtime/profile;
- request an AgenticComposition for developmental work;
- provide Factory tasks, evidence contracts, or integration fixtures as experiment inputs;
- test whether a recurrence profile improves Factory work.

Software Factory MUST NOT:

- redefine the generic Loop Runtime as Factory Core;
- make QL recurrence mandatory for Factory correctness;
- duplicate the experiment body under a second canonical directory;
- treat a successful Actuation experiment as proof of Factory semantics or vice versa.

## 7. Relationship to the new Actuation ontology

The imported runtime programme predates `AgenticComposition` and the generic Actuation relation introduced here. It is therefore preserved first and interpreted second.

The immediate research bridge is:

```text
Actuation semantics
  governing locus + world + intent
  differentiated agentic loci
  relation/bounds/return
             │
             ▼
Loop Runtime experiment
  recurrence + closure + re-entry
  conjugation + depth
  observable host/body boundary
```

A later amendment should test explicitly where the current QL circuit is:

- recurrence *within one AgenticLocus*;
- a semantic projection over multiple loci;
- or both at different recursion depths.

That question MUST be answered experimentally and formally. The move to Actuation does not silently equate a six-position QL circuit with six independent Agents.

## 8. Acceptance gates

The relocation is complete when:

1. Actuation contains the pinned complete experiment corpus and adapted workflows.
2. Existing deterministic tests/conformance run from the new path without Factory-local assumptions.
3. Source provenance and evidence status are recorded.
4. Factory issue/PR state points to Actuation as the new canonical home and stops advertising the runtime as Factory-owned future work.
5. Any remaining Factory adapter is demonstrably Factory-specific rather than a duplicate runtime.
6. New Series 1/live evidence, when executed, is recorded against Actuation revisions.

Until those gates are met, this document describes an active relocation rather than declaring historical Factory work deleted or invalid.
