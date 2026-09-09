# Actuation capability matrix

Status: agent-generated candidate, 2026-09-06. [Whole-account seed](actuation.html#whole) · [CSV](capability-matrix.csv).

This matrix gives the account stable addresses for actual powers, intended results and the evidence supporting them. Contract validation, a public CLI operation, and a complete live user journey have different implementation statuses. Follow each capability back to its governing account unit before changing its intended meaning.




## Seed × field contribution

[View declarations](capability-matrix.json) · [Editable CSV](capability-matrix.csv). Select a populated cell for its source and capability links. Unassessed cells carry no assertion.

| Seed | O:I whole | Central | AIKit | Software Factory | Workcell | Quaternal Logic |
| --- | --- | --- | --- | --- | --- | --- |
| [Why?](actuation.html#whole/why) | [relation](#field-q0-S) | Unassessed | Unassessed | Unassessed | Unassessed | Unassessed |
| [What?](actuation.html#whole/what) | [6 capabilities](#field-q1-S) | Unassessed | Unassessed | Unassessed | Unassessed | Unassessed |
| [How?](actuation.html#whole/how) | [3 capabilities](#field-q2-S) | Unassessed | Unassessed | [4 capabilities](#field-q2-S3) | Unassessed | Unassessed |
| [Who / Whereby?](actuation.html#whole/whereby) | [2 capabilities](#field-q3-S) | [2 capabilities](#field-q3-S0) | [2 capabilities](#field-q3-S2) | Unassessed | [2 capabilities](#field-q3-S4) | Unassessed |
| [Where / When?](actuation.html#whole/context) | [3 capabilities](#field-q4-S) | Unassessed | Unassessed | Unassessed | Unassessed | Unassessed |
| [Why-For?](actuation.html#whole/purpose) | [1 capabilities](#field-q5-S) | Unassessed | Unassessed | Unassessed | Unassessed | Unassessed |

<details>
<summary>Read the field contributions and their capability links</summary>

<a id="field-q0-S"></a>

### Why? → O:I whole

Actuation addresses a gap between asking AI to work and understanding the agency that actually acts. A person needs to know who is acting, in which World, with what purpose and permission, and how the experience of that work returns. As work moves between models, harnesses, sessions and collaborating Agents, these relationships should stay inspectable. Actuation gives them an explicit form so that authority can remain answerable to consequence.



[Source account passage](actuation.html#whole/why) · placement: agent-inference.

<a id="field-q1-S"></a>

### What? → O:I whole

Actuation is a CLI and a set of portable software contracts for identifying, inspecting and studying technological agency. Its current product detects available harnesses and reads structured accounts of situated Agency, realised acting loops, their unfolding Streams, semantic Activity and model-bearing instantiation. Its wider purpose is to make the constitution and management of agency an explicit part of an O:I World, from an ordinary coding session to a bounded composition of Agents.

[discovery](#cap-actuation-discovery) · [world binding](#cap-actuation-world-binding) · [realised](#cap-actuation-realised) · [activity](#cap-actuation-activity) · [instantiation](#cap-actuation-instantiation) · [model usage](#cap-actuation-model-usage)

[Source account passage](actuation.html#whole/what) · placement: agent-inference.

<a id="field-q2-S"></a>

### How? → O:I whole

Actuation relates an enduring Agent to a situated Agency and an explicit WorldBinding. Receipts identify the acting condition and its model, harness and material relations. Ordered Streams preserve attributable events; Activity makes those events understandable; Return carries results, evidence and difference back to the governing relation. Portable contracts let native runtimes supply these facts while preserving their own richer records.

[determination](#cap-actuation-determination) · [return](#cap-actuation-return) · [stream](#cap-actuation-stream)

[Source account passage](actuation.html#whole/how) · placement: agent-inference.

<a id="field-q2-S3"></a>

### How? → Software Factory

Factory and source-owning products carry their respective recognition and mutation procedures.

[return](#cap-actuation-return) · [stream](#cap-actuation-stream) · [activity](#cap-actuation-activity) · [model usage](#cap-actuation-model-usage)

[Source account passage](actuation.html#q2/follow) · placement: agent-inference.

<a id="field-q3-S"></a>

### Who / Whereby? → O:I whole

Actuation serves people shaping their relationship with AI, Agents working within declared bounds, and developers connecting harnesses and interfaces. Its identity and authority contracts make collaboration inspectable: an existing Agent can receive delegated work, a new Agent can have explicit lineage, and an independent participant can retain its own standing. Researchers can then study how the constitution of agency changes the work it produces.

[metagency](#cap-actuation-metagency) · [continuity](#cap-actuation-continuity)

[Source account passage](actuation.html#whole/whereby) · placement: agent-inference.

<a id="field-q3-S0"></a>

### Who / Whereby? → Central

Central provides durable authored ground and the project or personal World in which agency is situated.

[activity](#cap-actuation-activity) · [instantiation](#cap-actuation-instantiation)

[Source account passage](actuation.html#q3/integration) · placement: agent-inference.

<a id="field-q3-S2"></a>

### Who / Whereby? → AIKit

AIKit resolves the body, Context, Skills, models, sessions and Surfaces for an operative locus.

[activity](#cap-actuation-activity) · [instantiation](#cap-actuation-instantiation)

[Source account passage](actuation.html#q3/integration) · placement: agent-inference.

<a id="field-q3-S4"></a>

### Who / Whereby? → Workcell

Workcell materialises processes, services, storage and lifecycle.

[activity](#cap-actuation-activity) · [instantiation](#cap-actuation-instantiation)

[Source account passage](actuation.html#q3/integration) · placement: agent-inference.

<a id="field-q4-S"></a>

### Where / When? → O:I whole

Actuation begins wherever a model acts in a persistent working World, including an ordinary directory-bound coding harness. The same semantic account can accompany a richer body, another Surface or a different machine. Observations belong to their actual time and environment. Continuity and change remain attributable as sessions, models and material conditions evolve.

[harness detection](#cap-actuation-harness-detection) · [harness self](#cap-actuation-harness-self) · [research](#cap-actuation-research)

[Source account passage](actuation.html#whole/context) · placement: agent-inference.

<a id="field-q5-S"></a>

### Why-For? → O:I whole

Actuation is for an O:I World in which agency can be deliberately constituted, encountered and revised. Central supplies durable authored ground; AIKit provisions the operative body and context; Workcell supplies material execution; Factory commissions developmental work; O:I presents the whole. Actuation gives that work an intelligible actor and a return path. Its research programme investigates how models, contexts and ways of acting can be cultivated, while keeping experiment results distinct from the intentions that motivate them.

[verification](#cap-actuation-verification)

[Source account passage](actuation.html#whole/purpose) · placement: agent-inference.

</details>


## Capability index

| Capability | Operation | Current standing |
|---|---|---|
| [Discovery](#cap-actuation-discovery) | actuation capabilities; actuation contract list | observed read-only CLI JSON discovery on this checkout, 2026-09-06 |
| [World Binding](#cap-actuation-world-binding) | actuation agency; validateWorldBinding; isRootAgency | contract-tested: selected 36-test run, 2026-09-06 |
| [Metagency](#cap-actuation-metagency) | validateMetagencyGrant; agencyReadModel | contract-tested; public creation/configuration mutation not established |
| [Determination](#cap-actuation-determination) | validateDetermination; validateDeterminationLineage | contract-tested; commissioning journey remains design commitment |
| [Return](#cap-actuation-return) | validateReturn; agencyReadModel | contract-tested: selected 36-test run, 2026-09-06 |
| [Realised](#cap-actuation-realised) | actuation realised; realisedActuationReadModel | contract-tested: selected 36-test run, 2026-09-06 |
| [Continuity](#cap-actuation-continuity) | continuityDelta | contract-tested: selected 36-test run, 2026-09-06 |
| [Stream](#cap-actuation-stream) | actuation stream; actuation stream usage; actuationStreamReadModel; ActuationStreamJournal | contract-tested including durable model-usage attachment and replay deduplication, 2026-09-09 |
| [Activity](#cap-actuation-activity) | actuation activity; activityFromActuationStream; activityFromStreamEvent | contract-tested with model-usage refs and reversible Stream evidence, 2026-09-09 |
| [Model Usage](#cap-actuation-model-usage) | actuation usage; actuation stream usage; modelUsageFromClaudeCodeTranscript; recordModelUsageObservation | contract-tested in the 152-test native run: live-shaped Claude Code evidence adapter; provider, latency and cost remain unavailable when not reported, 2026-09-09 |
| [Harness Detection](#cap-actuation-harness-detection) | actuation harness catalog; actuation harness detect | observed catalog and machine-local detection with version probes, 2026-09-06 |
| [Harness Self](#cap-actuation-harness-self) | actuation harness self | observed self query was unresolved: no environment marker matched, 2026-09-06 |
| [Instantiation](#cap-actuation-instantiation) | actuation instantiation; actuation instantiation record; attachDetectionEvidence | receipt contracts tested; recorder CLI source-inspected, not live exercised |
| [Research](#cap-actuation-research) | Pinned QL runtime experiment and comparison procedures | experimental; no live capability-effect result claimed by this pass |
| [Verification](#cap-actuation-verification) | actuation verify; node --test selected native contract suites | 36 selected contract tests passed 2026-09-06; full verify command not run |

## cap-actuation-discovery

**Need:** A caller needs the native command and contract surface.

**Operation:** actuation capabilities; actuation contract list.

**Outcome:** Discoverable routes and contract versions for this checkout.

**Implementation:** Observed read-only CLI JSON discovery on this checkout, 2026-09-06; the generated account remains a review candidate.

**Governing account:** [Actuation · q1/product](actuation.html#q1/product) · `[[actuation:doc:product]]`.

**Source:** [README.md](../../README.md).

**Code:** [cli/commands.mjs](../../cli/commands.mjs).

**Tests:** [cli/actuation.test.mjs](../../cli/actuation.test.mjs).

## cap-actuation-world-binding

**Need:** A reader needs to know which Agent and Agency act in which World and scope.

**Operation:** actuation agency; validateWorldBinding; isRootAgency.

**Outcome:** Validated situated identity and positional root reading.

**Implementation:** contract-tested: selected 36-test run, 2026-09-06.

**Governing account:** [Actuation · q1/agency](actuation.html#q1/agency) · `[[actuation:doc:agency]]`.

**Source:** [docs/WORLD-BOUND-ROOT-AGENCY.md](../../docs/WORLD-BOUND-ROOT-AGENCY.md).

**Code:** [contracts/agency.mjs](../../contracts/agency.mjs).

**Tests:** [contracts/agency.test.mjs](../../contracts/agency.test.mjs).

## cap-actuation-metagency

**Need:** A governing actor needs explicit authority to act on agency.

**Operation:** validateMetagencyGrant; agencyReadModel.

**Outcome:** Inspectable granted operations, authority and bound World.

**Implementation:** contract-tested; public creation/configuration mutation not established.

**Governing account:** [Actuation · q3/boundaries](actuation.html#q3/boundaries) · `[[actuation:doc:boundaries]]`.

**Source:** [docs/WORLD-BOUND-ROOT-AGENCY.md](../../docs/WORLD-BOUND-ROOT-AGENCY.md).

**Code:** [contracts/agency.mjs](../../contracts/agency.mjs).

**Tests:** [contracts/agency.test.mjs](../../contracts/agency.test.mjs).

## cap-actuation-determination

**Need:** Participants need explicit origin, delegated autonomy and recursive lineage.

**Operation:** validateDetermination; validateDeterminationLineage.

**Outcome:** Distinct delegation, derivation, self-differentiation and federation relations.

**Implementation:** contract-tested; commissioning journey remains design commitment.

**Governing account:** [Actuation · q2/commission](actuation.html#q2/commission) · `[[actuation:doc:commission]]`.

**Source:** [../github-recovery-mirror/mirror/Actuation/issues/1.comments.json](../../../github-recovery-mirror/mirror/Actuation/issues/1.comments.json).

**Code:** [contracts/agency.mjs](../../contracts/agency.mjs).

**Tests:** [contracts/agency.test.mjs](../../contracts/agency.test.mjs).

## cap-actuation-return

**Need:** A governing decision needs attributable difference from completed or interrupted work.

**Operation:** validateReturn; agencyReadModel.

**Outcome:** Return preserves provenance and distinct received, recognition and mutation states.

**Implementation:** contract-tested: selected 36-test run, 2026-09-06.

**Governing account:** [Actuation · q2/follow](actuation.html#q2/follow) · `[[actuation:doc:follow]]`.

**Source:** [docs/WORLD-BOUND-ROOT-AGENCY.md](../../docs/WORLD-BOUND-ROOT-AGENCY.md).

**Code:** [contracts/agency.mjs](../../contracts/agency.mjs).

**Tests:** [contracts/agency.test.mjs](../../contracts/agency.test.mjs).

## cap-actuation-realised

**Need:** A person needs to inspect the acting condition already present.

**Operation:** actuation realised; realisedActuationReadModel.

**Outcome:** An evidence-bearing reading of Agent, Agency, WorldBinding, recurrence and body refs.

**Implementation:** contract-tested: selected 36-test run, 2026-09-06.

**Governing account:** [Actuation · q1/observation](actuation.html#q1/observation) · `[[actuation:doc:observation]]`.

**Source:** [docs/REALISED-ACTUATION-LOOP.md](../../docs/REALISED-ACTUATION-LOOP.md).

**Code:** [contracts/realised-actuation.mjs](../../contracts/realised-actuation.mjs).

**Tests:** [contracts/realised-actuation.test.mjs](../../contracts/realised-actuation.test.mjs).

## cap-actuation-continuity

**Need:** A reader needs to distinguish body change from semantic identity change.

**Operation:** continuityDelta.

**Outcome:** Attributed differences between two realised receipts.

**Implementation:** contract-tested: selected 36-test run, 2026-09-06.

**Governing account:** [Actuation · q3/identities](actuation.html#q3/identities) · `[[actuation:doc:identities]]`.

**Source:** [docs/REALISED-ACTUATION-LOOP.md](../../docs/REALISED-ACTUATION-LOOP.md).

**Code:** [contracts/realised-actuation.mjs](../../contracts/realised-actuation.mjs).

**Tests:** [contracts/realised-actuation.test.mjs](../../contracts/realised-actuation.test.mjs).

## cap-actuation-stream

**Need:** Surfaces need to read and resume attributable unfolding work.

**Operation:** actuation stream; actuation stream usage; actuationStreamReadModel; ActuationStreamJournal; recordModelUsageObservation.

**Outcome:** Ordered event reads, append, replay, subscription and closure, with idempotent attachment of typed model-usage evidence.

**Implementation:** contract-tested including the durable store, native usage normalization, replay deduplication and partial usage retained before cancellation, 2026-09-09; cross-surface deployment is not established.

**Governing account:** [Actuation · q2/follow](actuation.html#q2/follow) · `[[actuation:doc:follow]]`.

**Source:** [docs/ACTUATION-STREAM.md](../../docs/ACTUATION-STREAM.md).

**Code:** [contracts/actuation-stream.mjs](../../contracts/actuation-stream.mjs).

**Tests:** [contracts/actuation-stream.test.mjs](../../contracts/actuation-stream.test.mjs).

## cap-actuation-activity

**Need:** A person needs a meaningful summary with a route back to trace evidence.

**Operation:** actuation activity; activityFromActuationStream; activityFromStreamEvent.

**Outcome:** Semantic subject, phase and outcome with reversible Stream and model-usage references.

**Implementation:** contract-tested with usage refs and raw Stream evidence reconstruction in the 149-test native run, 2026-09-09.

**Governing account:** [Actuation · q1/observation](actuation.html#q1/observation) · `[[actuation:doc:observation]]`.

**Source:** [docs/ACTIVITY.md](../../docs/ACTIVITY.md).

**Code:** [contracts/activity.mjs](../../contracts/activity.mjs).

**Tests:** [contracts/activity.test.mjs](../../contracts/activity.test.mjs).

## cap-actuation-model-usage

**Need:** Consumers need measurable model use correlated with Agency, Activity and Return without parsing provider logs or inventing cost.

**Operation:** actuation usage; actuation stream usage; validateModelUsageObservation; modelUsageFromClaudeCodeTranscript; recordModelUsageObservation.

**Outcome:** Provider-truthful model, token, cache, timing and cost observations attached to ActuationStream and referenced by Activity.

**Implementation:** Contract-tested in the 152-test native run on 2026-09-09. The Claude Code adapter is grounded in a live native transcript record shape and preserves exact model/token/cache facts while leaving provider route, latency and cost `not-reported` where that record supplies none. Live cross-surface aggregation is not established.

**Governing account:** [Actuation · q1/observation](actuation.html#q1/observation) · `[[actuation:doc:observation]]`.

**Source:** [docs/ACTUATION-STREAM.md](../../docs/ACTUATION-STREAM.md) · [docs/ACTIVITY.md](../../docs/ACTIVITY.md).

**Code:** [contracts/model-usage.mjs](../../contracts/model-usage.mjs) · [contracts/model-usage-v1.schema.json](../../contracts/model-usage-v1.schema.json) · [contracts/actuation-stream-store.mjs](../../contracts/actuation-stream-store.mjs) · [contracts/actuation-stream-v1.schema.json](../../contracts/actuation-stream-v1.schema.json) · [contracts/activity.mjs](../../contracts/activity.mjs) · [contracts/activity-v1.schema.json](../../contracts/activity-v1.schema.json) · [cli/commands.mjs](../../cli/commands.mjs) · [cli/actuation.mjs](../../cli/actuation.mjs) · [cli/surface.mjs](../../cli/surface.mjs).

**Tests:** [contracts/model-usage.test.mjs](../../contracts/model-usage.test.mjs) · [contracts/actuation-stream-store.test.mjs](../../contracts/actuation-stream-store.test.mjs) · [contracts/activity.test.mjs](../../contracts/activity.test.mjs) · [cli/stream-store.test.mjs](../../cli/stream-store.test.mjs).

## cap-actuation-harness-detection

**Need:** A caller needs evidence of locally available harness candidates.

**Operation:** actuation harness catalog; actuation harness detect.

**Outcome:** Catalogued candidates and detected, absent or unavailable results with receipts.

**Implementation:** Observed read-only catalog and machine-local detection with version probes, 2026-09-06. The result is scoped to this machine and observation time.

**Governing account:** [Actuation · q4/environment](actuation.html#q4/environment) · `[[actuation:doc:environment]]`.

**Source:** [README.md](../../README.md).

**Code:** [detection/detect.mjs](../../detection/detect.mjs) · [detection/catalog.mjs](../../detection/catalog.mjs).

**Tests:** [detection/detect.test.mjs](../../detection/detect.test.mjs) · [contracts/harness-detection.test.mjs](../../contracts/harness-detection.test.mjs).

## cap-actuation-harness-self

**Need:** An actor needs to identify its current harness where environment markers permit.

**Operation:** actuation harness self.

**Outcome:** Resolved marker identity or explicit ambiguity with local detection context.

**Implementation:** Observed read-only self query, 2026-09-06. It returned no matching environment marker and `resolved: null`; this is not a positive self-identification.

**Governing account:** [Actuation · q4/environment](actuation.html#q4/environment) · `[[actuation:doc:environment]]`.

**Source:** [README.md](../../README.md).

**Code:** [detection/self.mjs](../../detection/self.mjs).

**Tests:** [detection/self.test.mjs](../../detection/self.test.mjs).

## cap-actuation-instantiation

**Need:** An integration needs to preserve model, engine, surface and material relationships.

**Operation:** actuation instantiation; actuation instantiation record; attachDetectionEvidence.

**Outcome:** Portable receipt with separated access/placement facts and optional detected harness binding or JSONL recording.

**Implementation:** receipt contracts tested; recorder CLI source-inspected, not live exercised.

**Governing account:** [Actuation · q1/product](actuation.html#q1/product) · `[[actuation:doc:product]]`.

**Source:** [README.md](../../README.md).

**Code:** [contracts/instantiation.mjs](../../contracts/instantiation.mjs) · [cli/commands.mjs](../../cli/commands.mjs).

**Tests:** [contracts/instantiation.test.mjs](../../contracts/instantiation.test.mjs) · [cli/actuation.test.mjs](../../cli/actuation.test.mjs).

## cap-actuation-research

**Need:** Researchers need attributable comparisons of agency and harness conditions.

**Operation:** Pinned QL runtime experiment and comparison procedures.

**Outcome:** Experimental records that can support critique, replay and later accepted findings.

**Implementation:** experimental; no live capability-effect result claimed by this pass.

**Governing account:** [Actuation · q4/research](actuation.html#q4/research) · `[[actuation:doc:research]]`.

**Source:** [docs/QL-RUNTIME-MIGRATION.md](../../docs/QL-RUNTIME-MIGRATION.md) · [../github-recovery-mirror/mirror/Actuation/issues/26.json](../../../github-recovery-mirror/mirror/Actuation/issues/26.json).

**Code:** [experiments/ql-runtime/prime/run.mjs](../../experiments/ql-runtime/prime/run.mjs).

**Tests:** [experiments/ql-runtime/prime/test/prime-structural.test.mjs](../../experiments/ql-runtime/prime/test/prime-structural.test.mjs).

## cap-actuation-verification

**Need:** Developers need to check portable contract behaviour before integration.

**Operation:** actuation verify; node --test selected native contract suites.

**Outcome:** A bounded native test result and discoverable verification command.

**Implementation:** 36 selected contract tests passed 2026-09-06; full verify command not run.

**Governing account:** [Actuation · q5/verification](actuation.html#q5/verification) · `[[actuation:doc:verification]]`.

**Source:** [README.md](../../README.md).

**Code:** [cli/commands.mjs](../../cli/commands.mjs).

**Tests:** [contracts/agency.test.mjs](../../contracts/agency.test.mjs) · [contracts/realised-actuation.test.mjs](../../contracts/realised-actuation.test.mjs) · [contracts/actuation-stream.test.mjs](../../contracts/actuation-stream.test.mjs) · [contracts/activity.test.mjs](../../contracts/activity.test.mjs) · [contracts/instantiation.test.mjs](../../contracts/instantiation.test.mjs).

## CLI command catalog

The command inventory was observed with `node ./bin/actuation capabilities --json` on 2026-09-06. It emitted `actuation.cli/v1`, version `0.2.0`, and these exact command identities. The catalog describes the inspected checkout; it does not adopt the generated account or establish a broader runtime deployment.

<!-- cli-catalog:start -->
| CLI identity | Capability |
| --- | --- |
| `activity.read` | [cap.actuation.activity](#cap-actuation-activity) |
| `agency.actualise` | [cap.actuation.determination](#cap-actuation-determination) · [cap.actuation.metagency](#cap-actuation-metagency) · [cap.actuation.return](#cap-actuation-return) · [cap.actuation.world-binding](#cap-actuation-world-binding) |
| `agency.read` | [cap.actuation.metagency](#cap-actuation-metagency) · [cap.actuation.return](#cap-actuation-return) · [cap.actuation.world-binding](#cap-actuation-world-binding) |
| `capabilities` | [cap.actuation.discovery](#cap-actuation-discovery) |
| `contract.list` | [cap.actuation.discovery](#cap-actuation-discovery) |
| `harness.capability` | [cap.actuation.discovery](#cap-actuation-discovery) |
| `harness.catalog` | [cap.actuation.harness-detection](#cap-actuation-harness-detection) |
| `harness.detect` | [cap.actuation.harness-detection](#cap-actuation-harness-detection) |
| `harness.self` | [cap.actuation.harness-self](#cap-actuation-harness-self) |
| `instantiation.read` | [cap.actuation.instantiation](#cap-actuation-instantiation) |
| `instantiation.record` | [cap.actuation.instantiation](#cap-actuation-instantiation) |
| `realised.read` | [cap.actuation.realised](#cap-actuation-realised) |
| `stream.close` | [cap.actuation.stream](#cap-actuation-stream) |
| `stream.open` | [cap.actuation.stream](#cap-actuation-stream) |
| `stream.read` | [cap.actuation.stream](#cap-actuation-stream) |
| `stream.record` | [cap.actuation.stream](#cap-actuation-stream) |
| `stream.replay` | [cap.actuation.stream](#cap-actuation-stream) |
| `stream.usage` | [cap.actuation.model-usage](#cap-actuation-model-usage) |
| `usage.read` | [cap.actuation.model-usage](#cap-actuation-model-usage) |
| `verify` | [cap.actuation.verification](#cap-actuation-verification) |
<!-- cli-catalog:end -->

## Evidence and reconciliation

The documentation pass ran `node --test contracts/agency.test.mjs contracts/realised-actuation.test.mjs contracts/actuation-stream.test.mjs contracts/activity.test.mjs contracts/instantiation.test.mjs` on 2026-09-06: 36 passed, zero failed. This selection tests real exported contract operations. CLI discovery, catalog, local versioned detection and self-detection were executed read-only on 2026-09-06. The resulting detection observation is machine-local: all 12 catalog descriptors were detected, while self-detection had no matching environment marker and resolved no harness. Referenced detection and CLI tests remain trace targets, not represented as executed here. No live model invocation, harness deployment or public Agency-creation journey was tested.

The CLI discovery response identified checkout revision `7c59dbcb295faf42e1cb85d873efe57d85af86b7`; the product account remains a generated candidate rather than an adoption of that revision. The repository had pre-existing CLI/detection edits outside this documentation work. The HTML provenance pins inspected file contents. The September 2 wayfinder correction asks for a public create/bind operation; the inspected command table exposes readers, detection and receipt recording, so this matrix does not mark commissioning as a complete runtime capability.

An overview change prompts review of its expansion and linked capabilities. An implementation limitation pressures the affected capability, contract or development work. It does not silently rewrite the product purpose. The account's seed remains an Agent-recovered candidate until human adoption.

## Preserved directed relation field

The `suite-relations` view uses the same CSV contract as `product-field`. Its H/A identifiers retain the human-facing and agent-facing orientations of the six products. Source-defined relation readings and annotations remain attached to each determination in `extensions`; coverage is independent of implementation status. The manifest declares the selected scope and axis meaning.

The full MD includes exactly the CSV below, including its fourteen capability records. One source convention therefore supports readable inspection and structured consumption without a separate relationship spreadsheet.

## Full CSV

```csv
id,record_type,view_id,row_id,column_id,capability_refs,need,operation,outcome,implementation_status,standing,source_refs,code_refs,test_refs,account_ref,relation,coverage,extensions,question
cap.actuation.discovery,capability,,,,[],A caller needs the native command and contract surface.,actuation capabilities; actuation contract list,Discoverable routes and contract versions for this checkout.,"observed public CLI JSON discovery on this checkout, 2026-09-09: 20 command identities and 10 native contracts",agent-inference,README.md,cli/commands.mjs,cli/actuation.test.mjs,actuation.html#q1/product,,,"{""basis"": ""Source-recovered capability account; original verification limits retained in Markdown."", ""cli_commands"": [""capabilities"", ""contract.list"", ""harness.capability""], ""cli_exposure"": {""kind"": ""direct"", ""reason"": ""The CLI emits its command identities and native contract versions as JSON.""}, ""cli_observation"": {""argv"": [""node"", ""./bin/actuation"", ""capabilities"", ""--json""], ""executed_at"": ""2026-09-09"", ""result_scope"": ""Working-tree CLI surface: actuation.cli/v1, version 0.2.0, 20 command identities and 10 native contracts including agency.actualise and actuation.agency-actualisation/v1."", ""revision_basis"": ""feat/public-agency-actualisation based on 7925f69c99a068fc064eeec6518efea9bb7aa265""}, ""converted_from_sha256"": ""0a76352c705f30682c115a1347bfa3a1e5300b25a6ba1cf542b1b11da5175369"", ""maintenance"": {""change_refs"": [""actuation:#30"", ""actuation:#31"", ""actuation:#33"", ""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5"", ""github-recovery-mirror/mirror/Actuation/issues/1.json"", ""actuation:#39"", ""actuation:#44"", ""wiki-continuity:model-route-join""], ""code_basis"": {""cli/commands.mjs"": ""6a2e6030ff3a5146ef5b9b428240d610a6fd46b35be847595dfb70123cb272a9""}, ""updated_at"": ""2026-09-09""}, ""last_reconciled_at"": ""2026-09-09T17:53:41.060067+00:00""}",
cap.actuation.world-binding,capability,,,,[],A reader needs to know which Agent and Agency act in which World and scope.,actuation agency; actuation agency actualise; validateWorldBinding; isRootAgency,Validated situated identity and positional root reading; exact WorldBinding retained by authority-gated actualisation.,contract-tested read and public actualisation binding; no material provisioning claimed,agent-inference,docs/WORLD-BOUND-ROOT-AGENCY.md,contracts/agency.mjs;contracts/agency-actualisation.mjs;cli/commands.mjs,contracts/agency.test.mjs;cli/agency-actualisation.test.mjs,actuation.html#q1/agency,,,"{""basis"": ""Source-recovered capability account; original verification limits retained in Markdown."", ""cli_commands"": [""agency.read"", ""agency.actualise""], ""cli_exposure"": {""kind"": ""direct"", ""reason"": ""The CLI reads supplied Agency relations and actualises a supplied exact WorldBinding only through the metagency authority gate.""}, ""converted_from_sha256"": ""0a76352c705f30682c115a1347bfa3a1e5300b25a6ba1cf542b1b11da5175369"", ""maintenance"": {""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5"", ""github-recovery-mirror/mirror/Actuation/issues/1.json"", ""actuation:#44"", ""wiki-continuity:model-route-join""], ""code_basis"": {""contracts/agency.mjs"": ""91f48560a2773fd93abeeaf77a6321bd04f7954e5c670d58f26d083709e2f5a5"", ""contracts/agency-actualisation.mjs"": ""ecc64e6886e9cb16dcf9e2f094a4d0d3d420b9ca2edf7dee0341febe0545262e"", ""cli/commands.mjs"": ""6a2e6030ff3a5146ef5b9b428240d610a6fd46b35be847595dfb70123cb272a9""}, ""updated_at"": ""2026-09-09""}, ""last_reconciled_at"": ""2026-09-09T17:53:41.060067+00:00""}",
cap.actuation.metagency,capability,,,,[],A governing actor needs explicit authority to act on agency.,actuation agency actualise; actualiseAgency; validateMetagencyGrant; agencyReadModel,Inspectable grants and an authority-gated semantic actualisation receipt in the exact bound World.,contract-tested public determination/actualisation; configuration and material provisioning remain outside this operation,agent-inference,docs/WORLD-BOUND-ROOT-AGENCY.md,contracts/agency.mjs;contracts/agency-actualisation.mjs;cli/commands.mjs,contracts/agency.test.mjs;cli/agency-actualisation.test.mjs,actuation.html#q3/boundaries,,,"{""basis"": ""Source-recovered capability account; original verification limits retained in Markdown."", ""cli_commands"": [""agency.read"", ""agency.actualise""], ""cli_exposure"": {""kind"": ""direct"", ""reason"": ""The public actualisation command exercises a matching explicit MetagencyGrant; visibility, capability and participation refs cannot replace it.""}, ""converted_from_sha256"": ""0a76352c705f30682c115a1347bfa3a1e5300b25a6ba1cf542b1b11da5175369"", ""maintenance"": {""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5"", ""github-recovery-mirror/mirror/Actuation/issues/1.json"", ""actuation:#44"", ""wiki-continuity:model-route-join""], ""code_basis"": {""contracts/agency.mjs"": ""91f48560a2773fd93abeeaf77a6321bd04f7954e5c670d58f26d083709e2f5a5"", ""contracts/agency-actualisation.mjs"": ""ecc64e6886e9cb16dcf9e2f094a4d0d3d420b9ca2edf7dee0341febe0545262e"", ""cli/commands.mjs"": ""6a2e6030ff3a5146ef5b9b428240d610a6fd46b35be847595dfb70123cb272a9""}, ""updated_at"": ""2026-09-09""}, ""last_reconciled_at"": ""2026-09-09T17:53:41.060067+00:00""}",
cap.actuation.determination,capability,,,,[],"Participants need explicit origin, delegated autonomy and recursive lineage.",actuation agency actualise; actualiseAgency; validateDetermination; validateDeterminationLineage,"Public bounded actualisation preserving distinct self-differentiation, delegation, derivation and federation lineage.",contract-tested public determination/actualisation; physical rematerialisation proof remains outstanding,agent-inference,docs/WORLD-BOUND-ROOT-AGENCY.md;README.md,contracts/agency.mjs;contracts/agency-actualisation.mjs;cli/commands.mjs;contracts/agency-actualisation-v1.schema.json,contracts/agency.test.mjs;cli/agency-actualisation.test.mjs,actuation.html#q2/commission,,,"{""basis"": ""Source-recovered capability account; original verification limits retained in Markdown."", ""cli_commands"": [""agency.actualise""], ""cli_exposure"": {""kind"": ""direct"", ""reason"": ""The CLI actualises the supplied determination after exact authority, binding, bounds, lineage, identity and Return checks.""}, ""converted_from_sha256"": ""0a76352c705f30682c115a1347bfa3a1e5300b25a6ba1cf542b1b11da5175369"", ""maintenance"": {""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5"", ""github-recovery-mirror/mirror/Actuation/issues/1.json"", ""actuation:#44"", ""wiki-continuity:model-route-join""], ""code_basis"": {""contracts/agency.mjs"": ""91f48560a2773fd93abeeaf77a6321bd04f7954e5c670d58f26d083709e2f5a5"", ""contracts/agency-actualisation.mjs"": ""ecc64e6886e9cb16dcf9e2f094a4d0d3d420b9ca2edf7dee0341febe0545262e"", ""cli/commands.mjs"": ""6a2e6030ff3a5146ef5b9b428240d610a6fd46b35be847595dfb70123cb272a9"", ""contracts/agency-actualisation-v1.schema.json"": ""3732c1ec149038e4e3872c229cf8ba46814f04c5f26b6b328c528420609e8713""}, ""updated_at"": ""2026-09-09""}, ""last_reconciled_at"": ""2026-09-09T17:53:41.060067+00:00""}",
cap.actuation.return,capability,,,,[],A governing decision needs attributable difference from completed or interrupted work.,actuation agency; actuation agency actualise; validateReturn; agencyReadModel,Return provenance and state remain distinct; actualisation preserves the Return relation without recognising it or mutating source.,contract-tested relation preservation; Return recognition and world/source mutation are explicitly not performed,agent-inference,docs/WORLD-BOUND-ROOT-AGENCY.md,contracts/agency.mjs;contracts/agency-actualisation.mjs;cli/commands.mjs,contracts/agency.test.mjs;cli/agency-actualisation.test.mjs,actuation.html#q2/follow,,,"{""basis"": ""Source-recovered capability account; original verification limits retained in Markdown."", ""cli_commands"": [""agency.read"", ""agency.actualise""], ""cli_exposure"": {""kind"": ""composed"", ""reason"": ""The actualisation receipt preserves the required Return relation and explicitly performs no recognition or source mutation; Return records remain readable through agency.read.""}, ""converted_from_sha256"": ""0a76352c705f30682c115a1347bfa3a1e5300b25a6ba1cf542b1b11da5175369"", ""maintenance"": {""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5"", ""github-recovery-mirror/mirror/Actuation/issues/1.json"", ""actuation:#44"", ""wiki-continuity:model-route-join""], ""code_basis"": {""contracts/agency.mjs"": ""91f48560a2773fd93abeeaf77a6321bd04f7954e5c670d58f26d083709e2f5a5"", ""contracts/agency-actualisation.mjs"": ""ecc64e6886e9cb16dcf9e2f094a4d0d3d420b9ca2edf7dee0341febe0545262e"", ""cli/commands.mjs"": ""6a2e6030ff3a5146ef5b9b428240d610a6fd46b35be847595dfb70123cb272a9""}, ""updated_at"": ""2026-09-09""}, ""last_reconciled_at"": ""2026-09-09T17:53:41.060067+00:00""}",
cap.actuation.realised,capability,,,,[],A person needs to inspect the acting condition already present.,actuation realised; realisedActuationReadModel,"An evidence-bearing reading of Agent, Agency, WorldBinding, recurrence and body refs.","contract-tested: selected 36-test run, 2026-09-06",agent-inference,docs/REALISED-ACTUATION-LOOP.md,contracts/realised-actuation.mjs,contracts/realised-actuation.test.mjs,actuation.html#q1/observation,,,"{""basis"": ""Source-recovered capability account; original verification limits retained in Markdown."", ""cli_commands"": [""realised.read""], ""cli_exposure"": {""kind"": ""direct"", ""reason"": ""The CLI reads a supplied realised-actuation record.""}, ""converted_from_sha256"": ""0a76352c705f30682c115a1347bfa3a1e5300b25a6ba1cf542b1b11da5175369"", ""maintenance"": {""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5"", ""github-recovery-mirror/mirror/Actuation/issues/1.json""], ""code_basis"": {""contracts/realised-actuation.mjs"": ""c1ca31f78acce5db5bf334a445036e79fb20ee15aa18f58099b8e6f60e48b839""}, ""updated_at"": ""2026-09-06""}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
cap.actuation.continuity,capability,,,,[],A reader needs to distinguish body change from semantic identity change.,continuityDelta,Attributed differences between two realised receipts.,"contract-tested: selected 36-test run, 2026-09-06",agent-inference,docs/REALISED-ACTUATION-LOOP.md,contracts/realised-actuation.mjs,contracts/realised-actuation.test.mjs,actuation.html#q3/identities,,,"{""basis"": ""Source-recovered capability account; original verification limits retained in Markdown."", ""cli_commands"": [], ""cli_exposure"": {""kind"": ""library"", ""reason"": ""continuityDelta is a library operation; the CLI emits no continuity command.""}, ""converted_from_sha256"": ""0a76352c705f30682c115a1347bfa3a1e5300b25a6ba1cf542b1b11da5175369"", ""maintenance"": {""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5"", ""github-recovery-mirror/mirror/Actuation/issues/1.json""], ""code_basis"": {""contracts/realised-actuation.mjs"": ""c1ca31f78acce5db5bf334a445036e79fb20ee15aa18f58099b8e6f60e48b839""}, ""updated_at"": ""2026-09-06""}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
cap.actuation.stream,capability,,,,[],Surfaces need to read and resume attributable unfolding work.,actuation stream; actuation stream open; actuation stream record; actuation stream replay; actuation stream close; actuationStreamReadModel; ActuationStreamJournal; openDurableStream; recordBoundaryOccurrence; replayDurableStream; closeDurableStream,"Ordered event reads, append, replay, subscription and closure in the reference implementation; durable append-only JSONL store with descriptor-validated boundary occurrence recording.","contract-tested incl. the durable store (17 tests, 2026-09-06); harness-boundary forwarding not wired yet; cross-surface deployment not exercised",agent-inference,docs/ACTUATION-STREAM.md,contracts/actuation-stream.mjs;contracts/actuation-stream-store.mjs;cli/commands.mjs,contracts/actuation-stream.test.mjs;contracts/actuation-stream-store.test.mjs;cli/stream-store.test.mjs,actuation.html#q2/follow,,,"{""basis"": ""Source-recovered capability account; original verification limits retained in Markdown."", ""cli_commands"": [""stream.read"", ""stream.open"", ""stream.record"", ""stream.replay"", ""stream.close""], ""cli_exposure"": {""kind"": ""direct"", ""reason"": ""The CLI reads a supplied Actuation Stream and records boundary occurrences into the durable descriptor-validated store.""}, ""converted_from_sha256"": ""0a76352c705f30682c115a1347bfa3a1e5300b25a6ba1cf542b1b11da5175369"", ""maintenance"": {""change_refs"": [""actuation:#33"", ""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5"", ""github-recovery-mirror/mirror/Actuation/issues/1.json"", ""actuation:#39"", ""actuation:#44"", ""wiki-continuity:model-route-join""], ""code_basis"": {""contracts/actuation-stream.mjs"": ""0437bc3bf619bc1cc513ac9f22e086481f6f13a76883f9561bf2bd9f6bc58399"", ""contracts/actuation-stream-store.mjs"": ""e8152e10b95b07f8414fa7baded9863e6c76bae9564d08d24994a3852d553d2f"", ""cli/commands.mjs"": ""6a2e6030ff3a5146ef5b9b428240d610a6fd46b35be847595dfb70123cb272a9""}, ""updated_at"": ""2026-09-09""}, ""last_reconciled_at"": ""2026-09-09T17:53:41.060067+00:00""}",
cap.actuation.activity,capability,,,,[],A person needs a meaningful summary with a route back to trace evidence.,actuation activity; activityFromActuationStream; activityFromStreamEvent,"Semantic subject, phase and outcome with reversible Stream references.","contract-tested: selected 36-test run, 2026-09-06",agent-inference,docs/ACTIVITY.md,contracts/activity.mjs,contracts/activity.test.mjs,actuation.html#q1/observation,,,"{""basis"": ""Source-recovered capability account; original verification limits retained in Markdown."", ""cli_commands"": [""activity.read""], ""cli_exposure"": {""kind"": ""direct"", ""reason"": ""The CLI reads and validates a supplied Activity record.""}, ""converted_from_sha256"": ""0a76352c705f30682c115a1347bfa3a1e5300b25a6ba1cf542b1b11da5175369"", ""maintenance"": {""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5"", ""github-recovery-mirror/mirror/Actuation/issues/1.json"", ""actuation:#39""], ""code_basis"": {""contracts/activity.mjs"": ""c6a499d19247e9d39ec0244c20ce6e1fb05e25ba83dc73fbef58ec9cbf5924bc""}, ""updated_at"": ""2026-09-08""}, ""last_reconciled_at"": ""2026-09-08T23:08:58.074153+00:00""}",
cap.actuation.model-usage,capability,,,,[],"Consumers need measurable model use correlated with Agency, Activity and Return without parsing provider logs or inventing cost.",actuation usage; actuation stream usage; validateModelUsageObservation; modelUsageFromClaudeCodeTranscript; recordModelUsageObservation,"Provider-truthful model, token, cache, timing and cost observations attached to ActuationStream and referenced by Activity, recorded either by normalizing one native transcript adapter or by validating an already-normalized actuation.model-usage/v1 observation straight through — both adapters share the identical stream-consistency, dedup and append-only recording path.","contract-tested: 192 native tests passed 2026-09-09; Claude Code transcript adapter grounded in a live native record shape; the observation adapter accepts pre-normalized evidence from any reporter without a second recording path — it is one-way telemetry, never read back as an availability or catalogue source; no provider cost or latency is claimed where the record did not supply it; an unrecognised adapter is refused by name rather than silently accepted",agent-inference,docs/ACTUATION-STREAM.md;docs/ACTIVITY.md,contracts/model-usage.mjs;contracts/actuation-stream-store.mjs;contracts/activity.mjs;cli/commands.mjs;cli/actuation.mjs;cli/surface.mjs;contracts/activity-v1.schema.json;contracts/actuation-stream-v1.schema.json;contracts/model-usage-v1.schema.json,contracts/model-usage.test.mjs;contracts/actuation-stream-store.test.mjs;contracts/activity.test.mjs;cli/stream-store.test.mjs,actuation.html#q1/observation,,,"{""basis"": ""Actuation #39 implementation reconciled against executable contracts and native tests."", ""cli_commands"": [""usage.read"", ""stream.usage""], ""cli_exposure"": {""kind"": ""direct"", ""reason"": ""The CLI validates portable usage observations and normalizes one supported Claude Code native transcript event into the durable Stream.""}, ""maintenance"": {""change_refs"": [""actuation:#39"", ""actuation:#44"", ""wiki-continuity:model-route-join""], ""code_basis"": {""contracts/model-usage.mjs"": ""dd4143c5cf94ea65bd785e9067fd1bc3ad5dac76880361442377000b02ba368a"", ""contracts/actuation-stream-store.mjs"": ""e8152e10b95b07f8414fa7baded9863e6c76bae9564d08d24994a3852d553d2f"", ""contracts/activity.mjs"": ""c6a499d19247e9d39ec0244c20ce6e1fb05e25ba83dc73fbef58ec9cbf5924bc"", ""cli/commands.mjs"": ""6a2e6030ff3a5146ef5b9b428240d610a6fd46b35be847595dfb70123cb272a9"", ""cli/actuation.mjs"": ""54ffa145993a0a7fd8bb5e84baff9bb08ea299a5841bd71cc631b54f54cb280d"", ""cli/surface.mjs"": ""fff47f1d9647ce24b08e29475e601d83ce6aaa24acbd4f9ad9168854c451c856"", ""contracts/activity-v1.schema.json"": ""9c18f3909889b5c31c7a848603d7cdb9a2b8c3ed7ce30a6a8f60dc4aac91a083"", ""contracts/actuation-stream-v1.schema.json"": ""5fb12e60d064012c06780f5d40e7b3bd057ac3c7f3841c03a6381a12d8cee9ff"", ""contracts/model-usage-v1.schema.json"": ""42215b3f06dffe5bfba53b0f51db6400d5b8739098c4fb1275f7a006579615cb""}, ""updated_at"": ""2026-09-09""}, ""last_reconciled_at"": ""2026-09-09T17:50:08.535039+00:00""}",
cap.actuation.harness-detection,capability,,,,[],A caller needs evidence of locally available harness candidates.,actuation harness catalog; actuation harness detect,"Catalogued candidates and detected, absent or unavailable results with receipts; each entry carries its declared native_kind, and a model-provider facet may carry a typed inventory of provider-native model ids read from the declared service endpoint. The harness-capability contract (catalog r6) additionally declares each dispatch-relevant harness's optional model_dispatch: which provider(s) it natively binds to, how a model is named on the harness's own surface (config-key, cli-flag or env-var), and whether a credential is required and hinted — never which models a provider serves, which stays the provider's own catalogue.","observed read-only catalog and machine-local detection with version probes, 2026-09-06; catalog r5 adds native_kind on every detection entry and a service-backed model inventory facet (Ollama), verified live 2026-09-09; catalog r6 adds the optional model_dispatch block to actuation.harness-capability/v1 - claude-code binds provider:anthropic and codex binds provider:openai (both selector config-key model, credential required), zcode declares kind:none because no model-provider binding is evidenced anywhere in this repo; contract-tested: 192 native tests passed 2026-09-09; result is environment-scoped",agent-inference,README.md,detection/detect.mjs;detection/catalog.mjs;contracts/harness-detection.mjs;contracts/harness-detection-v1.schema.json;detection/probes.mjs;detection/harnesses/ollama.mjs;contracts/harness-capability.mjs;contracts/harness-capability-v1.schema.json;detection/capabilities/claude-code.mjs;detection/capabilities/codex.mjs;detection/capabilities/zcode.mjs,detection/detect.test.mjs;contracts/harness-detection.test.mjs;contracts/harness-capability.test.mjs;detection/capabilities.test.mjs,actuation.html#q4/environment,,,"{""basis"": ""Source-recovered capability account; original verification limits retained in Markdown."", ""cli_commands"": [""harness.catalog"", ""harness.detect""], ""cli_exposure"": {""kind"": ""direct"", ""reason"": ""The CLI emits the declared catalog and executes local harness detection.""}, ""cli_observation"": {""additional_probe"": {""argv"": [""node"", ""./bin/actuation"", ""harness"", ""detect"", ""--versions"", ""--json""], ""executed_at"": ""2026-09-06T14:04:42Z"", ""result_scope"": ""Machine-local observation: all 12 catalog descriptors were detected; version receipts were emitted where their descriptor declares a version probe.""}, ""argv"": [""node"", ""./bin/actuation"", ""harness"", ""catalog"", ""--json""], ""catalog_revision"": 2, ""executed_at"": ""2026-09-06T14:04:42Z"", ""result_scope"": ""Catalog r2 emitted 12 declared descriptors.""}, ""converted_from_sha256"": ""0a76352c705f30682c115a1347bfa3a1e5300b25a6ba1cf542b1b11da5175369"", ""maintenance"": {""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5"", ""github-recovery-mirror/mirror/Actuation/issues/1.json"", ""wiki-continuity:model-catalogue-route-join"", ""wiki-continuity:model-route-join""], ""code_basis"": {""detection/catalog.mjs"": ""52647f8953bfd9783b7447eb6c6b58c5c815469dded842d2d453cc566b0d5fee"", ""detection/detect.mjs"": ""299a00864b41d11226dbd40273121e08b5929d791165dcba59750ec35e3b402f"", ""contracts/harness-detection.mjs"": ""6892048e5fcfede6c18967dd418101b59d0583c3f5e4d4130461be236c0bf56b"", ""contracts/harness-detection-v1.schema.json"": ""f62f89dc92c700e4a0647b8e241401dbad2046a47308fc4a0c087cf2dbb0bd4d"", ""detection/probes.mjs"": ""eaf0169c798c5e41cec374150051d33b197f6e09e5b1127bd72dae6dd6c5000a"", ""detection/harnesses/ollama.mjs"": ""bd94e2a0d065d5e5f69f89fa2f94eb5ea66c79adefb5b72296c00c01f2eef673"", ""contracts/harness-capability.mjs"": ""861a4ca76151377df0800cc7898f61551df9f4fed7c6b003979277ea1ce8c844"", ""contracts/harness-capability-v1.schema.json"": ""48a465834c59049cc8291d335a92aa5ae4aee74b9ed662c4784e8b62dcde4c2b"", ""detection/capabilities/claude-code.mjs"": ""aaf0e58272ce72746338cc096a8711719e70d7135a1912548da0f29de549ea24"", ""detection/capabilities/codex.mjs"": ""0352fd0e8d729d8c8f3f07cbc16a5e7b8ce72e4b1fc56648f7ea05883cd907e5"", ""detection/capabilities/zcode.mjs"": ""4577fb9a0f8ad741d78445caacf53c2740342305910621dccf515350e9adfb2f""}, ""updated_at"": ""2026-09-09""}, ""last_reconciled_at"": ""2026-09-09T17:50:08.535039+00:00""}",
cap.actuation.harness-self,capability,,,,[],An actor needs to identify its current harness where environment markers permit.,actuation harness self,Resolved marker identity or explicit ambiguity with local detection context.,"observed read-only self query, 2026-09-06; resolved null because no environment marker matched",agent-inference,README.md,detection/self.mjs,detection/self.test.mjs,actuation.html#q4/environment,,,"{""basis"": ""Source-recovered capability account; original verification limits retained in Markdown."", ""cli_commands"": [""harness.self""], ""cli_exposure"": {""kind"": ""direct"", ""reason"": ""The CLI emits its environment-marker self-detection reading.""}, ""cli_observation"": {""argv"": [""node"", ""./bin/actuation"", ""harness"", ""self"", ""--json""], ""catalog_revision"": 2, ""executed_at"": ""2026-09-06T14:04:42Z"", ""result_scope"": ""Local self query returned matched: [], resolved: null, ambiguity: false; it did not identify this running harness.""}, ""converted_from_sha256"": ""0a76352c705f30682c115a1347bfa3a1e5300b25a6ba1cf542b1b11da5175369"", ""maintenance"": {""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5"", ""github-recovery-mirror/mirror/Actuation/issues/1.json""], ""code_basis"": {""detection/self.mjs"": ""4e21366797b81edd944c16ed219fa74e11fa78292a0a6f95b67bc0e3b33b3c72""}, ""updated_at"": ""2026-09-06""}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
cap.actuation.instantiation,capability,,,,[],"An integration needs to preserve model, engine, surface and material relationships.",actuation instantiation; actuation instantiation record; attachDetectionEvidence,Portable receipt with separated access/placement facts and optional detected harness binding or JSONL recording.,"receipt contracts tested; recorder CLI source-inspected, not live exercised",agent-inference,README.md,contracts/instantiation.mjs;cli/commands.mjs,contracts/instantiation.test.mjs;cli/actuation.test.mjs,actuation.html#q1/product,,,"{""basis"": ""Source-recovered capability account; original verification limits retained in Markdown."", ""cli_commands"": [""instantiation.read"", ""instantiation.record""], ""cli_exposure"": {""kind"": ""direct"", ""reason"": ""The CLI reads instantiation receipts and exposes an explicit recorder route; this pass did not execute the mutating recorder.""}, ""converted_from_sha256"": ""0a76352c705f30682c115a1347bfa3a1e5300b25a6ba1cf542b1b11da5175369"", ""maintenance"": {""change_refs"": [""actuation:#33"", ""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5"", ""github-recovery-mirror/mirror/Actuation/issues/1.json"", ""actuation:#39"", ""actuation:#44"", ""wiki-continuity:model-route-join""], ""code_basis"": {""cli/commands.mjs"": ""6a2e6030ff3a5146ef5b9b428240d610a6fd46b35be847595dfb70123cb272a9"", ""contracts/instantiation.mjs"": ""c4b6552f410dd8876761159477816ed8253eea881ef70f7926d9bc898167d47c""}, ""updated_at"": ""2026-09-09""}, ""last_reconciled_at"": ""2026-09-09T17:53:41.060067+00:00""}",
cap.actuation.research,capability,,,,[],Researchers need attributable comparisons of agency and harness conditions.,Pinned QL runtime experiment and comparison procedures,"Experimental records that can support critique, replay and later accepted findings.",experimental; no live capability-effect result claimed by this pass,agent-inference,docs/QL-RUNTIME-MIGRATION.md;../github-recovery-mirror/mirror/Actuation/issues/26.json,experiments/ql-runtime/prime/run.mjs,experiments/ql-runtime/prime/test/prime-structural.test.mjs,actuation.html#q4/research,,,"{""basis"": ""Source-recovered capability account; original verification limits retained in Markdown."", ""cli_commands"": [], ""cli_exposure"": {""kind"": ""intent"", ""reason"": ""Research procedures are experimental and no CLI research command is emitted.""}, ""converted_from_sha256"": ""0a76352c705f30682c115a1347bfa3a1e5300b25a6ba1cf542b1b11da5175369"", ""maintenance"": {""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5"", ""github-recovery-mirror/mirror/Actuation/issues/26.json"", ""actuation:#46"", ""actuation:#44""], ""code_basis"": {""experiments/ql-runtime/prime/run.mjs"": ""3efdc1da4db79a5c243f122dfcfeaff00ad56db740438944d0e7dd1fb97e9bce""}, ""updated_at"": ""2026-09-09""}, ""last_reconciled_at"": ""2026-09-09T09:12:46.650796+00:00""}",
cap.actuation.verification,capability,,,,[],Developers need to check portable contract behaviour before integration.,actuation verify; node --test selected native contract suites,A bounded native test result and discoverable verification command.,"192 full native tests passed 2026-09-09, including the public actualisation Wayfinder conformance cases and the new harness-capability model_dispatch and stream-usage observation-adapter coverage; physical rematerialisation remains untested",agent-inference,README.md,cli/commands.mjs,contracts/agency.test.mjs;contracts/realised-actuation.test.mjs;contracts/actuation-stream.test.mjs;contracts/activity.test.mjs;contracts/instantiation.test.mjs;cli/agency-actualisation.test.mjs,actuation.html#q5/verification,,,"{""basis"": ""Source-recovered capability account; original verification limits retained in Markdown."", ""cli_commands"": [""verify""], ""cli_exposure"": {""kind"": ""direct"", ""reason"": ""The CLI executes all discovered native suites; this change observed status ok across 18 test files and 164 passing tests.""}, ""converted_from_sha256"": ""0a76352c705f30682c115a1347bfa3a1e5300b25a6ba1cf542b1b11da5175369"", ""maintenance"": {""change_refs"": [""actuation:#33"", ""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5"", ""github-recovery-mirror/mirror/Actuation/issues/1.json"", ""actuation:#39"", ""actuation:#44"", ""wiki-continuity:model-route-join""], ""code_basis"": {""cli/commands.mjs"": ""6a2e6030ff3a5146ef5b9b428240d610a6fd46b35be847595dfb70123cb272a9""}, ""updated_at"": ""2026-09-09""}, ""last_reconciled_at"": ""2026-09-09T17:53:41.060067+00:00""}",
H0->H1,relation,suite-relations,H0,H1,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,01:grounded-agency,H,"{""src_product"": ""Central"", ""dst_product"": ""Actuation"", ""ql"": ""A1"", ""cf_view"": ""CF2"", ""seam"": ""01:grounded-agency"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Central#24(PR);EpiLogos/Actuation#4"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
H0->A1,relation,suite-relations,H0,A1,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,01:grounded-agency,H,"{""src_product"": ""Central"", ""dst_product"": ""Actuation"", ""ql"": ""D2-transform"", ""cf_view"": ""CF2"", ""seam"": ""01:grounded-agency"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Central#24(PR);EpiLogos/Actuation#4"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
H1->H0,relation,suite-relations,H1,H0,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,01:grounded-agency,H,"{""src_product"": ""Actuation"", ""dst_product"": ""Central"", ""ql"": ""A1"", ""cf_view"": ""CF2"", ""seam"": ""01:grounded-agency"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Central#24(PR);EpiLogos/Actuation#4"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
H1->H1,relation,suite-relations,H1,H1,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,self:Actuation,I,"{""src_product"": ""Actuation"", ""dst_product"": ""Actuation"", ""ql"": """", ""cf_view"": ""CF2"", ""seam"": ""self:Actuation"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/Actuation#4"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
H1->H2,relation,suite-relations,H1,H2,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,12:agency-operative-body,H,"{""src_product"": ""Actuation"", ""dst_product"": ""AIKit"", ""ql"": ""B1"", ""cf_view"": ""CF3"", ""seam"": ""12:agency-operative-body"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/ai-kit#58(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
H1->H3,relation,suite-relations,H1,H3,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,13:agency-development,S,"{""src_product"": ""Actuation"", ""dst_product"": ""Factory"", ""ql"": """", ""cf_view"": ""CF4"", ""seam"": ""13:agency-development"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/agent-system-design#142(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
H1->H4,relation,suite-relations,H1,H4,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,14:agency-embodiment,H,"{""src_product"": ""Actuation"", ""dst_product"": ""Workcell"", ""ql"": ""C2"", ""cf_view"": ""CF5-field"", ""seam"": ""14:agency-embodiment"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/Workcell#18(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
H1->H5,relation,suite-relations,H1,H5,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,15:agency-ql,S,"{""src_product"": ""Actuation"", ""dst_product"": ""QL"", ""ql"": """", ""cf_view"": """", ""seam"": ""15:agency-ql"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/QL-MEF#19(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
H1->A0,relation,suite-relations,H1,A0,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,01:grounded-agency,H,"{""src_product"": ""Actuation"", ""dst_product"": ""Central"", ""ql"": ""D2-require"", ""cf_view"": ""CF2"", ""seam"": ""01:grounded-agency"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Central#24(PR);EpiLogos/Actuation#4"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
H1->A1,relation,suite-relations,H1,A1,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,conjugation:Actuation,H,"{""src_product"": ""Actuation"", ""dst_product"": ""Actuation"", ""ql"": ""D1"", ""cf_view"": ""CF2"", ""seam"": ""conjugation:Actuation"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/Actuation#4"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
H1->A2,relation,suite-relations,H1,A2,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,12:agency-operative-body,H,"{""src_product"": ""Actuation"", ""dst_product"": ""AIKit"", ""ql"": ""D2-transform"", ""cf_view"": ""CF3"", ""seam"": ""12:agency-operative-body"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/ai-kit#58(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
H1->A3,relation,suite-relations,H1,A3,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,13:agency-development,S,"{""src_product"": ""Actuation"", ""dst_product"": ""Factory"", ""ql"": """", ""cf_view"": ""CF4"", ""seam"": ""13:agency-development"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/agent-system-design#142(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
H1->A4,relation,suite-relations,H1,A4,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,14:agency-embodiment,H,"{""src_product"": ""Actuation"", ""dst_product"": ""Workcell"", ""ql"": ""D2-complete"", ""cf_view"": ""CF5-field"", ""seam"": ""14:agency-embodiment"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/Workcell#18(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
H1->A5,relation,suite-relations,H1,A5,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,15:agency-ql,S,"{""src_product"": ""Actuation"", ""dst_product"": ""QL"", ""ql"": """", ""cf_view"": """", ""seam"": ""15:agency-ql"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/QL-MEF#19(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
H2->H1,relation,suite-relations,H2,H1,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,12:agency-operative-body,H,"{""src_product"": ""AIKit"", ""dst_product"": ""Actuation"", ""ql"": ""B1"", ""cf_view"": ""CF3"", ""seam"": ""12:agency-operative-body"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/ai-kit#58(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
H2->A1,relation,suite-relations,H2,A1,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,12:agency-operative-body,H,"{""src_product"": ""AIKit"", ""dst_product"": ""Actuation"", ""ql"": ""D2-require"", ""cf_view"": ""CF3"", ""seam"": ""12:agency-operative-body"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/ai-kit#58(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
H3->H1,relation,suite-relations,H3,H1,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,13:agency-development,S,"{""src_product"": ""Factory"", ""dst_product"": ""Actuation"", ""ql"": """", ""cf_view"": ""CF4"", ""seam"": ""13:agency-development"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/agent-system-design#142(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
H3->A1,relation,suite-relations,H3,A1,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,13:agency-development,S,"{""src_product"": ""Factory"", ""dst_product"": ""Actuation"", ""ql"": """", ""cf_view"": ""CF4"", ""seam"": ""13:agency-development"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/agent-system-design#142(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
H4->H1,relation,suite-relations,H4,H1,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,14:agency-embodiment,H,"{""src_product"": ""Workcell"", ""dst_product"": ""Actuation"", ""ql"": ""C2"", ""cf_view"": ""CF5-field"", ""seam"": ""14:agency-embodiment"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/Workcell#18(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
H4->A1,relation,suite-relations,H4,A1,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,14:agency-embodiment,H,"{""src_product"": ""Workcell"", ""dst_product"": ""Actuation"", ""ql"": ""D2-complete"", ""cf_view"": ""CF5-field"", ""seam"": ""14:agency-embodiment"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/Workcell#18(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
H5->H1,relation,suite-relations,H5,H1,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,15:agency-ql,S,"{""src_product"": ""QL"", ""dst_product"": ""Actuation"", ""ql"": """", ""cf_view"": """", ""seam"": ""15:agency-ql"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/QL-MEF#19(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
H5->A1,relation,suite-relations,H5,A1,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,15:agency-ql,S,"{""src_product"": ""QL"", ""dst_product"": ""Actuation"", ""ql"": """", ""cf_view"": """", ""seam"": ""15:agency-ql"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/QL-MEF#19(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
A0->H1,relation,suite-relations,A0,H1,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,01:grounded-agency,H,"{""src_product"": ""Central"", ""dst_product"": ""Actuation"", ""ql"": ""D2-require.inverse"", ""cf_view"": ""CF2"", ""seam"": ""01:grounded-agency"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Central#24(PR);EpiLogos/Actuation#4"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
A0->A1,relation,suite-relations,A0,A1,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,01:grounded-agency,H,"{""src_product"": ""Central"", ""dst_product"": ""Actuation"", ""ql"": ""D3:A1"", ""cf_view"": ""CF2"", ""seam"": ""01:grounded-agency"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Central#24(PR);EpiLogos/Actuation#4"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
A1->H0,relation,suite-relations,A1,H0,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,01:grounded-agency,H,"{""src_product"": ""Actuation"", ""dst_product"": ""Central"", ""ql"": ""D2-transform.inverse"", ""cf_view"": ""CF2"", ""seam"": ""01:grounded-agency"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Central#24(PR);EpiLogos/Actuation#4"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
A1->H1,relation,suite-relations,A1,H1,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,conjugation:Actuation,H,"{""src_product"": ""Actuation"", ""dst_product"": ""Actuation"", ""ql"": ""D1.inverse"", ""cf_view"": ""CF2"", ""seam"": ""conjugation:Actuation"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/Actuation#4"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
A1->H2,relation,suite-relations,A1,H2,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,12:agency-operative-body,H,"{""src_product"": ""Actuation"", ""dst_product"": ""AIKit"", ""ql"": ""D2-require.inverse"", ""cf_view"": ""CF3"", ""seam"": ""12:agency-operative-body"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/ai-kit#58(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
A1->H3,relation,suite-relations,A1,H3,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,13:agency-development,S,"{""src_product"": ""Actuation"", ""dst_product"": ""Factory"", ""ql"": """", ""cf_view"": ""CF4"", ""seam"": ""13:agency-development"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/agent-system-design#142(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
A1->H4,relation,suite-relations,A1,H4,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,14:agency-embodiment,H,"{""src_product"": ""Actuation"", ""dst_product"": ""Workcell"", ""ql"": ""D2-complete.inverse"", ""cf_view"": ""CF5-field"", ""seam"": ""14:agency-embodiment"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/Workcell#18(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
A1->H5,relation,suite-relations,A1,H5,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,15:agency-ql,S,"{""src_product"": ""Actuation"", ""dst_product"": ""QL"", ""ql"": """", ""cf_view"": """", ""seam"": ""15:agency-ql"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/QL-MEF#19(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
A1->A0,relation,suite-relations,A1,A0,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,01:grounded-agency,H,"{""src_product"": ""Actuation"", ""dst_product"": ""Central"", ""ql"": ""D3:A1"", ""cf_view"": ""CF2"", ""seam"": ""01:grounded-agency"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Central#24(PR);EpiLogos/Actuation#4"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
A1->A1,relation,suite-relations,A1,A1,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,self:Actuation,I,"{""src_product"": ""Actuation"", ""dst_product"": ""Actuation"", ""ql"": """", ""cf_view"": ""CF2"", ""seam"": ""self:Actuation"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/Actuation#4"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
A1->A2,relation,suite-relations,A1,A2,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,12:agency-operative-body,H,"{""src_product"": ""Actuation"", ""dst_product"": ""AIKit"", ""ql"": ""D3:B1"", ""cf_view"": ""CF3"", ""seam"": ""12:agency-operative-body"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/ai-kit#58(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
A1->A3,relation,suite-relations,A1,A3,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,13:agency-development,S,"{""src_product"": ""Actuation"", ""dst_product"": ""Factory"", ""ql"": """", ""cf_view"": ""CF4"", ""seam"": ""13:agency-development"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/agent-system-design#142(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
A1->A4,relation,suite-relations,A1,A4,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,14:agency-embodiment,H,"{""src_product"": ""Actuation"", ""dst_product"": ""Workcell"", ""ql"": ""D3:C2"", ""cf_view"": ""CF5-field"", ""seam"": ""14:agency-embodiment"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/Workcell#18(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
A1->A5,relation,suite-relations,A1,A5,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,15:agency-ql,S,"{""src_product"": ""Actuation"", ""dst_product"": ""QL"", ""ql"": """", ""cf_view"": """", ""seam"": ""15:agency-ql"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/QL-MEF#19(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
A2->H1,relation,suite-relations,A2,H1,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,12:agency-operative-body,H,"{""src_product"": ""AIKit"", ""dst_product"": ""Actuation"", ""ql"": ""D2-transform.inverse"", ""cf_view"": ""CF3"", ""seam"": ""12:agency-operative-body"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/ai-kit#58(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
A2->A1,relation,suite-relations,A2,A1,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,12:agency-operative-body,H,"{""src_product"": ""AIKit"", ""dst_product"": ""Actuation"", ""ql"": ""D3:B1"", ""cf_view"": ""CF3"", ""seam"": ""12:agency-operative-body"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/ai-kit#58(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
A3->H1,relation,suite-relations,A3,H1,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,13:agency-development,S,"{""src_product"": ""Factory"", ""dst_product"": ""Actuation"", ""ql"": """", ""cf_view"": ""CF4"", ""seam"": ""13:agency-development"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/agent-system-design#142(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
A3->A1,relation,suite-relations,A3,A1,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,13:agency-development,S,"{""src_product"": ""Factory"", ""dst_product"": ""Actuation"", ""ql"": """", ""cf_view"": ""CF4"", ""seam"": ""13:agency-development"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/agent-system-design#142(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
A4->H1,relation,suite-relations,A4,H1,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,14:agency-embodiment,H,"{""src_product"": ""Workcell"", ""dst_product"": ""Actuation"", ""ql"": ""D2-complete.inverse"", ""cf_view"": ""CF5-field"", ""seam"": ""14:agency-embodiment"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/Workcell#18(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
A4->A1,relation,suite-relations,A4,A1,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,14:agency-embodiment,H,"{""src_product"": ""Workcell"", ""dst_product"": ""Actuation"", ""ql"": ""D3:C2"", ""cf_view"": ""CF5-field"", ""seam"": ""14:agency-embodiment"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/Workcell#18(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
A5->H1,relation,suite-relations,A5,H1,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,15:agency-ql,S,"{""src_product"": ""QL"", ""dst_product"": ""Actuation"", ""ql"": """", ""cf_view"": """", ""seam"": ""15:agency-ql"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/QL-MEF#19(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
A5->A1,relation,suite-relations,A5,A1,[],,,,,agent-inference,O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR),,,,15:agency-ql,S,"{""src_product"": ""QL"", ""dst_product"": ""Actuation"", ""ql"": """", ""cf_view"": """", ""seam"": ""15:agency-ql"", ""defined_in"": ""O-I:docs/CANONICAL-PRODUCT-FIELD.md|QL-MEF#19(PR)"", ""tracked_by"": ""EpiLogos/O-I#29;EpiLogos/Actuation#1;EpiLogos/QL-MEF#19(PR)"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
rel.actuation.q0.S,relation,product-field,q0,S,[],,,,,agent-inference,ProjectCentral/user/actuation.html#whole-why,,,actuation.html#whole/why,"Actuation addresses a gap between asking AI to work and understanding the agency that actually acts. A person needs to know who is acting, in which World, with what purpose and permission, and how the experience of that work returns. As work moves between models, harnesses, sessions and collaborating Agents, these relationships should stay inspectable. Actuation gives them an explicit form so that authority can remain answerable to consequence.",,"{""basis"": ""Exact overview seed text. Capability links follow their existing governing expanded account units; cell placement is editorial inference."", ""seed_ref"": ""actuation:seed:q0"", ""source_unit"": ""whole-why"", ""seed_sha256"": ""5e3f1dc43daf0e04a6c8045514cc0de8c3c6cf820678fc57fcb054a831c4f67e"", ""source_standing"": ""agent-inference"", ""reconciled_change_ref"": ""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",Why?
rel.actuation.q1.S,relation,product-field,q1,S,"[""cap.actuation.discovery"", ""cap.actuation.world-binding"", ""cap.actuation.realised"", ""cap.actuation.activity"", ""cap.actuation.instantiation"", ""cap.actuation.model-usage""]",,,,,agent-inference,ProjectCentral/user/actuation.html#whole-what,,,actuation.html#whole/what,"Actuation is a CLI and a set of portable software contracts for identifying, inspecting and studying technological agency. Its current product detects available harnesses and reads structured accounts of situated Agency, realised acting loops, their unfolding Streams, semantic Activity and model-bearing instantiation. Its wider purpose is to make the constitution and management of agency an explicit part of an O:I World, from an ordinary coding session to a bounded composition of Agents.",,"{""basis"": ""Exact overview seed text. Capability links follow their existing governing expanded account units; cell placement is editorial inference."", ""seed_ref"": ""actuation:seed:q1"", ""source_unit"": ""whole-what"", ""seed_sha256"": ""0e8b1ab545cbda254836a7c24ea3fc596d110776ef62ff90095bf917f92f7af4"", ""source_standing"": ""agent-inference"", ""reconciled_change_ref"": ""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5"", ""maintenance"": {""updated_at"": ""2026-09-08"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5"", ""actuation:#39""]}, ""last_reconciled_at"": ""2026-09-08T23:08:58.074153+00:00""}",What?
rel.actuation.q2.S,relation,product-field,q2,S,"[""cap.actuation.determination"", ""cap.actuation.return"", ""cap.actuation.stream""]",,,,,agent-inference,ProjectCentral/user/actuation.html#whole-how,,,actuation.html#whole/how,"Actuation relates an enduring Agent to a situated Agency and an explicit WorldBinding. Receipts identify the acting condition and its model, harness and material relations. Ordered Streams preserve attributable events; Activity makes those events understandable; Return carries results, evidence and difference back to the governing relation. Portable contracts let native runtimes supply these facts while preserving their own richer records.",,"{""basis"": ""Exact overview seed text. Capability links follow their existing governing expanded account units; cell placement is editorial inference."", ""seed_ref"": ""actuation:seed:q2"", ""source_unit"": ""whole-how"", ""seed_sha256"": ""bf8f87c91459a438d4b39c3852afcaf9c8774cfa492683328baf0f4dc3512b99"", ""source_standing"": ""agent-inference"", ""reconciled_change_ref"": ""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",How?
rel.actuation.q3.S,relation,product-field,q3,S,"[""cap.actuation.metagency"", ""cap.actuation.continuity""]",,,,,agent-inference,ProjectCentral/user/actuation.html#whole-whereby,,,actuation.html#whole/whereby,"Actuation serves people shaping their relationship with AI, Agents working within declared bounds, and developers connecting harnesses and interfaces. Its identity and authority contracts make collaboration inspectable: an existing Agent can receive delegated work, a new Agent can have explicit lineage, and an independent participant can retain its own standing. Researchers can then study how the constitution of agency changes the work it produces.",,"{""basis"": ""Exact overview seed text. Capability links follow their existing governing expanded account units; cell placement is editorial inference."", ""seed_ref"": ""actuation:seed:q3"", ""source_unit"": ""whole-whereby"", ""seed_sha256"": ""ed84e4337a88344419fa5ab8519560930d6ba4e102d91c1d399273cde900a93b"", ""source_standing"": ""agent-inference"", ""reconciled_change_ref"": ""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",Who / Whereby?
rel.actuation.q4.S,relation,product-field,q4,S,"[""cap.actuation.harness-detection"", ""cap.actuation.harness-self"", ""cap.actuation.research""]",,,,,agent-inference,ProjectCentral/user/actuation.html#whole-context,,,actuation.html#whole/context,"Actuation begins wherever a model acts in a persistent working World, including an ordinary directory-bound coding harness. The same semantic account can accompany a richer body, another Surface or a different machine. Observations belong to their actual time and environment. Continuity and change remain attributable as sessions, models and material conditions evolve.",,"{""basis"": ""Exact overview seed text. Capability links follow their existing governing expanded account units; cell placement is editorial inference."", ""seed_ref"": ""actuation:seed:q4"", ""source_unit"": ""whole-context"", ""seed_sha256"": ""95ed77bfb6e424213d1f025ee51cee752f0337ef960ff39422ccbefe5dc3bb4a"", ""source_standing"": ""agent-inference"", ""reconciled_change_ref"": ""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",Where / When?
rel.actuation.q5.S,relation,product-field,q5,S,"[""cap.actuation.verification""]",,,,,agent-inference,ProjectCentral/user/actuation.html#whole-purpose,,,actuation.html#whole/purpose,"Actuation is for an O:I World in which agency can be deliberately constituted, encountered and revised. Central supplies durable authored ground; AIKit provisions the operative body and context; Workcell supplies material execution; Factory commissions developmental work; O:I presents the whole. Actuation gives that work an intelligible actor and a return path. Its research programme investigates how models, contexts and ways of acting can be cultivated, while keeping experiment results distinct from the intentions that motivate them.",,"{""basis"": ""Exact overview seed text. Capability links follow their existing governing expanded account units; cell placement is editorial inference."", ""seed_ref"": ""actuation:seed:q5"", ""source_unit"": ""whole-purpose"", ""seed_sha256"": ""1645c2ddf52e972ea1e658d137b280a17318a8bac5977035aea05ba3dfcd4625"", ""source_standing"": ""agent-inference"", ""reconciled_change_ref"": ""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",Why-For?
rel.actuation.q3.S0,relation,product-field,q3,S0,"[""cap.actuation.activity"", ""cap.actuation.instantiation""]",,,,,agent-inference,ProjectCentral/user/actuation.html#q3-integration,,,actuation.html#q3/integration,Central provides durable authored ground and the project or personal World in which agency is situated.,,"{""basis"": ""Exact account sentence; cell placement is editorial inference."", ""source_unit"": ""q3-integration"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
rel.actuation.q3.S2,relation,product-field,q3,S2,"[""cap.actuation.activity"", ""cap.actuation.instantiation""]",,,,,agent-inference,ProjectCentral/user/actuation.html#q3-integration,,,actuation.html#q3/integration,"AIKit resolves the body, Context, Skills, models, sessions and Surfaces for an operative locus.",,"{""basis"": ""Exact account sentence; cell placement is editorial inference."", ""source_unit"": ""q3-integration"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
rel.actuation.q2.S3,relation,product-field,q2,S3,"[""cap.actuation.return"", ""cap.actuation.stream"", ""cap.actuation.activity"", ""cap.actuation.model-usage""]",,,,,agent-inference,ProjectCentral/user/actuation.html#q2-follow,,,actuation.html#q2/follow,Factory and source-owning products carry their respective recognition and mutation procedures.,,"{""basis"": ""Exact account sentence; cell placement is editorial inference."", ""source_unit"": ""q2-follow"", ""maintenance"": {""updated_at"": ""2026-09-08"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5"", ""actuation:#39""]}, ""last_reconciled_at"": ""2026-09-08T23:08:58.074153+00:00""}",
rel.actuation.q3.S4,relation,product-field,q3,S4,"[""cap.actuation.activity"", ""cap.actuation.instantiation""]",,,,,agent-inference,ProjectCentral/user/actuation.html#q3-integration,,,actuation.html#q3/integration,"Workcell materialises processes, services, storage and lifecycle.",,"{""basis"": ""Exact account sentence; cell placement is editorial inference."", ""source_unit"": ""q3-integration"", ""maintenance"": {""updated_at"": ""2026-09-06"", ""change_refs"": [""codex:thread:01a07608-d2ec-7b10-9713-74c445adf8a5""]}, ""last_reconciled_at"": ""2026-09-06T14:30:23.797672+00:00""}",
```
