# Series 1 Benchmark v0.1 — matched human-review experiment

**Status:** normative for Series 1 v0.1  
**Parent programme:** #94  
**Live experiment:** #110  
**Executable corpus:** `comparison/series1/tasks.mjs`  
**Runner:** `comparison/series1/run.mjs`  
**Review renderer:** `comparison/series1/render-review.mjs`

## 1. Purpose

Series 1 v0.1 is a small controlled benchmark for **experiencing and comparing agent behaviour**, not an automated leaderboard.

The primary question is:

> When the same model is given the same prompt, files, capabilities, verification contract and execution budget, how does Classic recurrence differ from Direct QL and Deep QL in understanding, action, recovery, restraint and fulfilment of intent?

The benchmark is intentionally prior to a formal eval programme. Its first job is to make the behavioural consequences of the three loop conditions inspectable enough that a human reviewer can determine what is actually different and which differences appear useful, harmful, neutral, ceremonial or invalidating.

Series 1 v0.1 MUST NOT manufacture a scalar winner, use deterministic QL conformance as a proxy for product capability, or introduce a judge model as the final arbiter. Automated evals, repeated trials and ablations belong to later phases after the first live behaviour has been inspected.

## 2. Experimental variable

The only intended independent variable inside a matched comparison set is the loop condition:

```text
classic
ql-direct
ql-deep
```

The benchmark is executed through three real host paths:

```text
pi
pydantic-ai
native
```

The primary experimental unit is:

```text
one task × one host × one repetition
  ├─ classic
  ├─ ql-direct
  └─ ql-deep
```

The primary comparison is therefore **within one host for one task and repetition**. Cross-host comparison is secondary because Pi, Pydantic AI and Native are themselves different host/framework conditions.

A full exploratory v0.1 pass is:

```text
6 tasks × 3 hosts × 1 repetition × 3 loop conditions = 54 condition-runs
```

One repetition is sufficient for Phase A exploratory review. It is not sufficient for population-level or variance-sensitive claims.

## 3. Canonical provider/model contract

Series 1 v0.1 is provider-specific by design:

```text
provider       deepseek
candidate      deepseek-v4-flash
credential     DEEPSEEK_API_KEY
API base       https://api.deepseek.com
```

The candidate model and model parameters MUST be identical across Classic, Direct QL and Deep QL inside a matched set.

`QL_SERIES1_API_KEY` is not part of this benchmark and MUST NOT be introduced. Pi, Pydantic AI and Native use the same provider-native `DEEPSEEK_API_KEY` contract.

The default candidate temperature is `0` where the provider/host path exposes that parameter. If any host cannot faithfully apply a declared model parameter, that discrepancy MUST be recorded and the affected cross-host comparison treated accordingly.

## 4. Held-constant contract

For a matched Classic / Direct / Deep set, all of the following MUST be identical:

1. provider and candidate model identity;
2. candidate model parameters;
3. exact task prompt bytes;
4. exact success/constraint text supplied to the agent;
5. complete starting workspace bytes and paths;
6. capability names, descriptions and argument contracts;
7. task-specific objective verification protocol;
8. maximum recurrence/step budget;
9. host profile and pinned host revision;
10. network policy;
11. benchmark task definition revision;
12. runner/review-contract revision relevant to the run.

The task workspace has no internet/retrieval capability. Only the model-provider boundary has network access. A condition MUST NOT receive a different skill, hidden hint, verifier, source file, tool, retrieval path or corrective intervention.

Condition order rotates across repetitions. With one exploratory repetition, the recorded order is still evidence and MUST be preserved.

The manifest MUST record stable digests for at least:

- task prompt + success conditions;
- complete starting workspace;
- capability contract;
- verification protocol;
- model identity + parameters;
- execution budget.

A mismatch in any required held constant invalidates that matched comparison set. The run may be retained as diagnostic evidence but MUST NOT be treated as a valid Classic/Direct/Deep comparison.

## 5. Capability surface

All six tasks use the same portable workspace capability contract unless a future benchmark version explicitly changes it:

```text
list_files(path?)
read_file(path)
write_file(path, content)
run_tests(files?)
```

Capabilities are confined to the isolated task root. Path escape is forbidden. No shell, web search, repository API, package installation, arbitrary command execution or external filesystem access is available to the candidate agent.

This capability surface is intentionally modest. The benchmark is testing loop behaviour over ordinary inspection, local research, file mutation and verification, not breadth of tool integration.

## 6. Executable corpus authority and freeze

`tasks.mjs` is the executable fixture body. It contains the exact prompts, starting files, success conditions, objective verification functions, human-only review references and task-specific review focus.

`BENCHMARK-V0.1.md` defines the experimental meaning and invariants. The two MUST NOT drift silently.

Before the first formal exploratory dispatch, the v0.1 corpus MUST be frozen by recording:

- the git revision containing this specification;
- the git revision containing `tasks.mjs`;
- a stable digest for every task definition;
- a stable digest for every generated starting workspace;
- the capability-contract digest;
- the runner/review renderer revisions.

Any substantive change to a task prompt, fixture, verifier, review reference, capability contract or execution semantics after live evidence exists creates a new benchmark revision or clearly versioned amendment. Old and new evidence MUST NOT be pooled without noting the change.

## 7. Universal observation protocol

Every live condition-run MUST preserve enough information to reconstruct what the candidate experienced and what it did.

Required record:

- task ID, category and benchmark revision;
- exact benchmark prompt and success/constraint text;
- full starting workspace snapshot;
- exact host, runtime, model and specification revisions;
- every model input, including system text and request/control payload;
- every full returned model envelope or control decision;
- every capability call with arguments;
- every complete capability result or error;
- ordered host events;
- ordered runtime/QL semantic events;
- Deep operator events where present;
- final answer/outcome;
- full final workspace snapshot;
- task-specific objective verification output;
- elapsed time;
- model-call count;
- input/output/total tokens where exposed;
- provider cost where exposed;
- capability-call count;
- execution status and semantic/closure status as distinct fields.

Run termination MUST NOT be silently represented as QL Closure. Fixture providers are ineligible for this evidence path.

## 8. Failure and invalidation discipline

Series 1 is allowed to fail. Failures are evidence when they reflect the tested condition; they are experiment defects when the comparison contract itself breaks.

### 8.1 Retain as behavioural evidence

Retain and review, rather than erase:

- candidate tool misuse;
- candidate failure to inspect needed evidence;
- incorrect edits;
- unnecessary edits;
- premature stopping;
- pointless recurrence;
- model/provider refusal produced under the same valid conditions;
- runtime exhaustion caused by the tested loop behaviour;
- a failed tool call requested by the candidate;
- negative/null outcomes.

### 8.2 Mark comparison invalid

A matched set is invalid for comparative determination if any condition has a material experimental asymmetry such as:

- different prompt/success-condition bytes;
- different starting workspace;
- different model identity or parameters;
- different capabilities;
- different verifier;
- different step budget;
- benchmark/reference information leaked to one candidate condition;
- host/provider integration defect that prevents one condition from receiving the same exterior affordances;
- incomplete trace capture that makes the run materially unreconstructable;
- human intervention in only one condition.

Invalid runs remain useful for debugging but are rerun after the defect is fixed.

## 9. Human-only material and leakage boundary

`reviewReference` material is for the reviewer only. It MUST NOT be included in the candidate model prompt, candidate-readable workspace, controller prompt, tool result or any other candidate-visible context.

The same rule applies to any future expected-answer notes or review annotations.

For local research and epistemic tasks, the candidate must work only from the frozen task corpus. The human reference exists to make later source-checking efficient; it is not an answer key supplied to the agent.

## 10. Human review protocol

Series 1 v0.1 records:

```text
determination = pending-human-review
```

Objective verification supports the review but does not decide which loop condition is better.

The reviewer considers at least:

1. **Prompt apprehension** — what did the agent understand, miss, invent or preserve?
2. **Constraint preservation** — did it respect explicit boundaries and avoid solving a different task?
3. **Epistemic conduct** — did it distinguish evidence, inference, uncertainty, contradiction and stale information appropriately?
4. **Action selection** — were tool calls relevant, ordered and sufficient? Did it inspect before acting when warranted?
5. **Agency / skill use** — did it use procedures as aids to judgement rather than ceremony? Did it know when not to act?
6. **Difference and recovery** — how did it react when evidence contradicted its working view or an action failed?
7. **Closure / stopping** — did it stop at the right point, prematurely, or continue pointlessly?
8. **Output / artifact** — did the final answer or workspace fulfil the actual intent?
9. **Friction** — unnecessary model calls, tool calls, verbosity, edits, latency or complexity.
10. **QL-specific behaviour** — when unblinded, did recurrence, reopening, conjugation, recursive depth or square modulation visibly help, harm or merely decorate the run?

### 10.1 Two-pass review

Where practical, the review bundle SHOULD support two passes:

**Pass A — behaviour first, condition-masked.** Compare prompt, ordinary model/tool chronology, workspace diff, verifier evidence and final output with the three conditions relabelled neutrally. The purpose is not perfect experimental blinding; it is to reduce immediate expectation bias while assessing ordinary work quality and friction.

**Pass B — QL mechanics unmasked.** Reveal Classic / Direct QL / Deep QL identities and inspect the QL semantic/operator trace alongside the behaviour. Determine whether the additional recurrence actually explains a useful difference, a harmful difference, no meaningful difference, or mere ceremony.

The mapping used for Pass A must be recoverable but should not be displayed inside the masked review itself.

### 10.2 Review outcome vocabulary

The human review may use qualitative outcomes such as:

```text
preferred
roughly-equivalent
worse
interesting-but-inconclusive
invalid-comparison
```

These are reviewer determinations, not automatic scores. A reviewer SHOULD include a short rationale and identify the concrete trace/output evidence behind the judgement.

No benchmark-wide synthetic winner is required in v0.1. It is acceptable for Direct to look better on one task, Deep worse on another and all three roughly equivalent elsewhere.

## 11. Objective verification philosophy

Every task has one frozen objective verifier shared by all three loop conditions.

The verifier may establish facts such as:

- tests pass or fail;
- an export was preserved;
- a required artifact exists;
- protected source files remained unchanged;
- the workspace remained byte-identical;
- stale status text was actually corrected;
- implementation files were unnecessarily changed.

The verifier MUST NOT convert those facts into a capability winner or a prose-quality score in v0.1.

A verifier failure is important evidence but does not remove the need to inspect why it happened.

## 12. Benchmark tasks

The corpus deliberately spans different kinds of agent work so QL is not evaluated only on coding or only on abstract dialogue.

### 12.1 `S1-CODE-001` — code repair and verification

**Prompt**

> Inspect the workspace and fix `buildIndex` so records are keyed by their normalized id and, when multiple records normalize to the same id, the latest record wins. Preserve the existing public exports, add no dependency, avoid unrelated changes, and run the tests before you finish.

The workspace contains `ids.js`, `index.js`, `index.test.js` and `package.json`. The defect is small but requires inspection, reuse of an existing helper, mutation and verification.

**What this probes:** causal diagnosis, inspect-before-edit behaviour, API/constraint preservation, minimal change, verification and stopping after current evidence.

**Objective verifier:** unchanged tests; public exports preserved; `normalizeId` still used rather than duplicated; support files unchanged.

### 12.2 `S1-RESEARCH-001` — local-file research

**Prompt**

> Using only the files in this workspace, prepare a concise research note answering: (1) which execution surfaces are currently supported, (2) what evidence establishes that, (3) which older statement is superseded, and (4) what remains unresolved about comparing runs. Cite the filename supporting every material claim. Do not edit the workspace and do not import outside knowledge.

The frozen source corpus contains current architecture, dated validation evidence, an earlier archived planning assumption and a future possibility that is not yet supported.

**What this probes:** source discovery, relevance selection, temporal/source discrimination, local grounding, citation discipline, synthesis and resistance to importing outside facts.

**Objective verifier:** complete workspace remains byte-identical. Semantic correctness is reviewed against the source corpus and human-only reference, not auto-scored.

### 12.3 `S1-EPISTEMIC-001` — understanding under incomplete evidence

**Prompt**

> Using only the evidence files in this workspace, give the best current explanation for runs where the task result succeeds but telemetry upload fails. Separate what is directly observed, what is inferred, and what is still open. State at least one piece of evidence that would materially weaken or falsify your current explanation. Do not edit any file.

The evidence packet supports a plausible current explanation but deliberately leaves one latency observation unresolved.

**What this probes:** observation/inference separation, explanatory synthesis, uncertainty calibration, falsifiability, contradiction handling and resistance to premature closure.

**Objective verifier:** complete workspace remains byte-identical. Epistemic quality is reviewed manually against the frozen evidence packet.

### 12.4 `S1-SKILL-001` — procedural skill with judgement

**Prompt**

> Complete the incoming request using the local `SKILL.md`. Apply the procedure with judgement: perform the steps that are relevant, do not manufacture work merely because a step is optional, preserve the source notes, and create the requested `deliverable.md`.

The local skill contains a conditional contradiction-resolution step. Relevant notes are coherent; one unrelated social note is present.

**What this probes:** skill discovery, procedure-following without ritualism, relevant evidence selection, conditional judgement, artifact creation and preservation of source material.

**Objective verifier:** `deliverable.md` exists; `SKILL.md`, request and source notes are byte-identical. Writing quality and procedural judgement remain human-reviewed.

### 12.5 `S1-AGENCY-001` — truthful state and justified action

**Prompt**

> Make this workspace truthful and ready. Inspect the current state, verify actual behaviour before deciding what to change, preserve the public API, and make only the narrowest justified correction. Leave STATUS.md describing the verified current state and the evidence you used.

The executable implementation is already correct. `STATUS.md` is stale and falsely says the implementation is broken. The correct response is therefore not “repair code because a status file says so”; it is to establish current evidence and make the descriptive state truthful.

**What this probes:** verify-before-edit agency, stale-evidence handling, evidence authority, narrowest justified action, truthful state maintenance and stopping once the actual inconsistency is resolved.

**Objective verifier:** tests pass; implementation remains byte-identical; public API remains intact; stale broken-state claim is removed from `STATUS.md`.

### 12.6 `S1-RESTRAINT-001` — trivial bounded request

**Prompt**

> According to `fact.txt`, what is the preferred review format? Answer in one sentence. Do not edit anything.

`fact.txt` contains the complete answer.

**What this probes:** bounded directness, minimal tool use, anti-ceremony and correct stopping. This task is intentionally hostile to unnecessary recurrence.

**Objective verifier:** workspace remains byte-identical. The human reviewer checks that the one-sentence answer is grounded in `fact.txt` and observes any unnecessary calls/recurrence.

## 13. Task-selection rationale

The six tasks cover complementary agent behaviours:

```text
CODE       enact and verify a causal change
RESEARCH   discover and synthesise local evidence
EPISTEMIC  understand without overstating certainty
SKILL      apply procedure with contextual judgement
AGENCY     decide whether action is needed at all
RESTRAINT  answer correctly with minimal ceremony
```

The benchmark is not intended to be comprehensive. It is deliberately small enough that a human can inspect every run in depth.

A task should remain in v0.1 only if it reveals something meaningfully different about recurrence while preserving a clean matched comparison. Additional domains should be introduced in later versions rather than making Phase A too large to review closely.

## 14. Host interpretation

The three host paths are real implementations, not three votes on the same result:

- **Pi** — pinned Pi model/provider stack and host adapter;
- **Pydantic AI** — pinned Pydantic AI model path and bridge;
- **Native** — minimal repository-owned DeepSeek-compatible transport.

Within-host Classic/Direct/Deep comparison is primary because host semantics are then held constant.

If the same loop condition behaves differently across hosts, record that as host/runtime interaction rather than averaging it away.

## 15. Execution phases

### Phase A0 — freeze and preflight

Before live capability evidence:

- freeze benchmark/spec/task digests;
- prove held-constant enforcement;
- prove review-reference non-leakage;
- prove full trace/workspace capture;
- prove the review bundle can render valid records;
- configure `DEEPSEEK_API_KEY` only in the execution environment, never in artifacts.

### Phase A1 — bounded live smoke

Use a small number of live runs to validate provider/host integration and evidence capture, not to draw capability conclusions. Prefer `S1-RESTRAINT-001` because failures are easy to inspect, then `S1-CODE-001` because it exercises read/write/test behaviour.

A smoke run that exposes an experiment defect is fixed and rerun. It is not promoted into the benchmark determination.

### Phase A2 — exploratory human-review benchmark

Run all six tasks through all three hosts with one repetition and matched Classic/Direct/Deep conditions. Retain machine JSON plus human-readable review bundles.

Conduct the two-pass human review and record task/host-level determinations and experiment-defect flags.

### Phase B — repeated evidence

Only after Phase A2 shows that the benchmark and harness are trustworthy, repeat matched trials to observe model variance and stability of the qualitative findings.

The number of repetitions is chosen after seeing Phase A behaviour; v0.1 does not pretend the correct statistical design is already known.

### Phase C — formal evals and ablations

Only after the behaviour is understood should the programme define scalar or model-judge evals, operator ablations, statistical aggregation, larger corpora or stronger claims.

Possible later questions include Direct vs Deep operator ablations, depth/conjugation invocation value, cost/latency trade-offs and whether any qualitative advantage survives repeated trials.

Do not skip Phase A because automated scoring is convenient.

## 16. Acceptance / freeze criteria for Series 1 v0.1

Series 1 v0.1 is complete only when:

1. this benchmark contract and executable corpus are revision-pinned;
2. all required held constants are machine-checked;
3. review-reference leakage is prevented/tested;
4. live preflight passes against the real provider and all intended host paths;
5. bounded smoke runs demonstrate usable complete evidence capture;
6. all six tasks complete one exploratory matched Classic/Direct/Deep set through all three intended host paths, or any omitted/invalid set is explicitly identified and rerun;
7. complete JSON evidence and human-readable review artifacts are retained;
8. objective verifier results remain observations rather than automatic winners;
9. human review records concrete differences in understanding, action, recovery, closure, output and friction;
10. QL-specific semantic/operator behaviour is inspected after ordinary behaviour review;
11. experiment defects are separated from candidate-agent failures;
12. the programme records what, if anything, warrants Phase B repetition or Phase C formal eval/ablation work.

## 17. Non-goals

Series 1 v0.1 does not attempt to establish:

- a universal QL benchmark;
- statistically generalisable superiority;
- a production model leaderboard;
- cross-provider generality;
- a final cost/quality frontier;
- proof that every Deep operator is beneficial;
- proof that a structurally conformant QL runtime improves model capability.

The legitimate output of this benchmark may be mixed, null or negative. If Classic is cleaner on trivial work, Direct helps some evidence-heavy tasks, Deep adds friction, or host effects dominate the signal, that is a valid result.

The benchmark exists to make those facts visible before the programme decides what to formalise next.
