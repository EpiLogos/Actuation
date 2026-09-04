# Prime recursive relational Agency experiment

This tranche makes Prime Agent a **real Actuation research harness**, not a reference diagram.

It uses Prime's native RLM recursion and Continual Harness to test the open Actuation question already carried by #1:

> does QL recurrence operate inside one locus, across an AgenticComposition, recursively at both levels, and can useful returned difference condition later Agency composition?

## Source lock

`source-lock.json` currently pins:

- Actuation base `bd8927b54ac17016be6f879994f736e453f2881c`;
- Prime Agent stable `v0.9.1` / `81ae3cb34d27d38ee37f9e205a1e73694993b344`;
- QL-MEF accepted main `cddd97d3e7717954256a46f482bd569fa7448870`;
- optional harmonic current-development carrier QL-MEF #81 `42d36ed75fd9cf8a70bcbabc5dca766cc51b6811`.

Re-source-lock deliberately when those products move. Do not silently accept drift.

## What is being tested

Prime child agents are treated as **materially realised acting loci**. The generic semantics remain Actuation's:

```text
governing Agency
    ↓ determination / differentiation
child acting locus
    ↓ bounded situated Actuation
Return
    ↓
parent reconstitution
```

In relational conditions the root and its Prime children receive the same Python-backed `ql-relational` faculty. Prime inherits installed Skills into child sessions, so QL/MEF/Wiki intelligence can genuinely recur through the tree rather than being held only by the parent.

The faculty consumes QL-MEF. It does not duplicate its kernel, MEF, Wiki or harmonic semantics in Actuation.

## Conditions

```text
P0 prime-native        RLM_MAX_DEPTH=1
   Prime RLM control; QL skill absent.

P2 prime-relational    RLM_MAX_DEPTH=1
   Root + inherited children can use executable QL/MEF/Wiki faculty.

P3 prime-relational-return  RLM_MAX_DEPTH=1
   P2 + explicit returned-difference/reconstitution envelope.

P4 prime-recursive-field    RLM_MAX_DEPTH=2
   P3 + descendant recursion when the live Prime runtime admits deeper child creation.

P5 prime-continual          RLM_MAX_DEPTH=2
   P3/P4 field + one explicit post-trajectory RPC refine invocation.
```

Prime inheritance makes a "QL root but deliberately dumb children" condition artificial, so it is not manufactured here.

## QL relational faculty

`skills/ql-relational/` is a project Python-backed Prime Skill.

It exposes:

```python
await ql_relational.capabilities()
await ql_relational.kernel_apply(...)
await ql_relational.mef_lenses()
await ql_relational.context_frames()
await ql_relational.vak_locate(...)
await ql_relational.negotiate(...)
await ql_relational.wiki_refract(...)
await ql_relational.constellation_contract()
await ql_relational.harmonic_search(...)
await ql_relational.harmonic_snapshot(...)
ql_relational.return_envelope(...)
```

The deterministic constellation/Return source comes directly from QL-MEF `docs/wiki-structural-contract-v2.md`. Wiki refraction executes the real `ql-wiki-refraction` binary. Harmonic search stays source-relative: accepted V3 source is available on main, while executable harmonic material is admitted only when the checkout is the pinned #81 head and `QL_PRIME_HARMONIC=1`.

Each call appends a provenance record to the experiment evidence log.

## Install Prime

Use the pinned stable release for matched runs. Prime's own stable installer currently is:

```bash
curl -fsSL https://app.primeintellect.ai/prime-agent/install.sh | sh
prime-agent --version
```

Configure the provider/model through Prime normally, then set their explicit experiment identity:

```bash
export QL_PRIME_PROVIDER=openai
export QL_PRIME_MODEL='<configured model id>'
export QL_MEF_ROOT=/path/to/QL-MEF
```

## Run

From the Actuation checkout:

```bash
node experiments/ql-runtime/prime/run.mjs \
  --condition prime-relational-return \
  --task S1-CODE-001 \
  --output /tmp/prime-p3-code.json
```

The runner reuses the frozen Series 1 task setup and verification code. Available Series 1 tasks therefore remain directly comparable while Prime's native recursion is exercised instead of replacing it with `LoopRuntime`.

Control:

```bash
node experiments/ql-runtime/prime/run.mjs \
  --condition prime-native \
  --task S1-CODE-001 \
  --output /tmp/prime-p0-code.json
```

Recursive field:

```bash
node experiments/ql-runtime/prime/run.mjs \
  --condition prime-recursive-field \
  --task S1-RESEARCH-001 \
  --output /tmp/prime-p4-research.json
```

Prime has a real `RLM_MAX_DEPTH` runtime contract. The runner sets it explicitly from the selected condition: depth 1 for P0/P2/P3 and depth 2 for P4/P5. The manifest records the requested cap and what child/session evidence actually appeared; it does not claim grandchild recursion merely because P4 was requested.

Two Prime-native mechanical acceptance tasks supplement the matched Series 1 corpus:

```text
PRIME-COMPOSITION-001
  requires at least two differentiated child Agencies over distinct source regions,
  then root reconstitution.

PRIME-RECURSIVE-001
  deliberately exercises root → child → grandchild → child → root Return at depth 2.
```

These tasks establish that the recursive material path actually works. Use the unchanged Series 1 tasks for matched cognition/performance comparisons.

Continual:

```bash
node experiments/ql-runtime/prime/run.mjs \
  --condition prime-continual \
  --task S1-AGENCY-001 \
  --output /tmp/prime-p5-agency.json
```

P5 performs exactly one explicit RPC `refine` after the task trajectory completes and records Prime's refinement result. This is deliberate experiment authority, not ambient self-modification.

## Harmonic development run

Check out QL-MEF PR #81 at the source-locked head, then:

```bash
export QL_MEF_ROOT=/path/to/QL-MEF-at-42d36ed75fd9cf8a70bcbabc5dca766cc51b6811
export QL_PRIME_HARMONIC=1

node experiments/ql-runtime/prime/run.mjs \
  --condition prime-recursive-field \
  --task S1-EPISTEMIC-001 \
  --output /tmp/prime-p4-harmonic.json
```

That run is explicitly **current-development research**, not evidence that the harmonic implementation is accepted QL-MEF main.

## Evidence

The manifest retains:

- exact Prime release/version;
- exact expected/observed QL-MEF source state;
- condition/task/model/provider;
- complete before/after task workspace;
- Series 1 verifier result;
- Prime RPC event stream and session stats;
- discovered child/session identifiers and lineage edges where exposed;
- every QL relational operation digest and QL revision;
- final outcome;
- P5 refinement result.

No run is scored as "QL wins" by construction. The first physical campaign should compare process and outcome across P0/P2/P3/P4/P5 and inspect whether relational operations actually changed apprehension, delegation, recovery, Return and later harness composition.

## Structural verification

This repository CI does not pretend to execute a credentialed Prime model run. It verifies the experiment apparatus:

```bash
cd experiments/ql-runtime/prime
npm test
python3 -m py_compile skills/ql-relational/src/ql_relational/__init__.py
```

Live Prime/QL runs remain explicit physical/provider evidence.
