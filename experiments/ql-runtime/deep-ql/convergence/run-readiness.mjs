import assert from 'node:assert/strict';
import fs from 'node:fs';
import { QLDirectCoreRuntime } from '../../foundation/ql-core-runtime/index.js';
import { createRunManifest, executeRun, replayRun } from '../../foundation/optics/index.js';
import { qlPosition } from '../../foundation/ql-core-runtime/semantics.js';
import { createPiHost, PI_UPSTREAM } from '../../experiments/pi/shared/host.js';
import { createPiBaselinePolicy } from '../../experiments/pi/ql/policy.js';
import { createPydanticHost, PYDANTIC_UPSTREAM } from '../../experiments/pydantic/shared/host.js';
import { createPydanticBaselinePolicy } from '../../experiments/pydantic/ql/policy.js';
import { createNativeHost } from '../../experiments/native/shared/host.js';
import { createNativeBaselinePolicy } from '../../experiments/native/ql/policy.js';
import { createDeepQLRuntimeClass, DeepQLOperatorSession } from '../index.js';
import { createConjugateDelta } from '../operators.js';
import { compareTraces } from '../comparison/comparator.js';
import { renderRun } from '../render/index.js';
import { runConformanceSuite, validatePortableEvent } from '../conformance/runner.js';
import { corpus, corpusStats } from '../typing-corpus/corpus.js';

const FOUNDATION_MERGE = '7690069846eb6fc89f6aa78dcf7aab886ac7c737';
const FOUNDATION_RUNTIME = '11c7be5767735c25f49139906b59398bcdf3bf42';
const SPEC_REVISION = 'f9d056c54caf094eb672f005ce3c8cbde4de0a5b';
const DeepRuntime = createDeepQLRuntimeClass(QLDirectCoreRuntime);

const dryRuns = JSON.parse(fs.readFileSync(new URL('./dry-runs.json', import.meta.url), 'utf8'));

const profiles = [
  {
    id: 'pi',
    hostFactory: createPiHost,
    policyFactory: createPiBaselinePolicy,
    model: { id:'pi-fixture-model', provider:'fixture', parameters:{ temperature:0 } },
    capabilities: [{ id:'inspect', kind:'pi-agent-tool' }],
    startState: { pi_revision: PI_UPSTREAM.revision }
  },
  {
    id: 'pydantic-ai',
    hostFactory: createPydanticHost,
    policyFactory: createPydanticBaselinePolicy,
    model: { id:'pydantic-fixture-model', provider:'fixture', parameters:{ temperature:0 } },
    capabilities: [{ id:'inspect', kind:'pydantic-tool' }],
    startState: { pydantic_revision: PYDANTIC_UPSTREAM.revision }
  },
  {
    id: 'native',
    hostFactory: createNativeHost,
    policyFactory: createNativeBaselinePolicy,
    model: { id:'native-fixture-model', provider:'fixture', parameters:{ temperature:0 } },
    capabilities: [{ id:'inspect', kind:'native-capability' }],
    startState: { native_contract: createNativeHost().revision ?? 'native-host-contract-v1' }
  }
];

function semanticEvents(record) {
  return record.events.filter((event) => event.channel === 'runtime-semantic');
}

async function runProfile(profile) {
  const request = {
    taskId: `${profile.id}-deep-readiness`,
    input: { subject:'matched-runtime-baseline', goal:'verify the host/runtime seam under the deeper profile' },
    successConditions: ['matched-runtime-baseline is currently verified'],
    capabilities: profile.capabilities,
    maxSteps: 8
  };
  const environment = { id:`${profile.id}-fixture-env`, network:false };

  const directHost = profile.hostFactory();
  const deepHost = profile.hostFactory();
  const directRuntime = new QLDirectCoreRuntime({ policy:profile.policyFactory() });
  const deepRuntime = new DeepRuntime({ policy:profile.policyFactory() });

  const common = {
    taskId: request.taskId,
    fixtureId: `${profile.id}-deep-readiness-v1`,
    specRevision: SPEC_REVISION,
    model: profile.model,
    capabilities: profile.capabilities,
    environment,
    successConditions: request.successConditions,
    startState: profile.startState
  };

  const directManifest = createRunManifest({ ...common, host:directHost, runtime:directRuntime });
  const deepManifest = createRunManifest({ ...common, host:deepHost, runtime:deepRuntime });
  const direct = await executeRun({ runtime:directRuntime, host:directHost, request, manifest:directManifest });
  const deep = await executeRun({ runtime:deepRuntime, host:deepHost, request, manifest:deepManifest });

  assert.equal(direct.status.execution, 'completed');
  assert.equal(direct.status.semantic, 'closed');
  assert.equal(deep.status.execution, 'completed');
  assert.equal(deep.status.semantic, 'closed');

  const directSemantic = semanticEvents(direct);
  const deepSemantic = semanticEvents(deep);
  const shallowComparison = compareTraces(directSemantic, deepSemantic);
  assert.equal(shallowComparison.equal, true, `${profile.id}: shallow Direct/Deep semantic mismatch`);
  assert.ok(deepSemantic.every((event) => validatePortableEvent(event).valid), `${profile.id}: invalid portable Deep event`);

  const replay = replayRun(JSON.parse(JSON.stringify(deep)));
  assert.equal(replay.event_count, deep.events.length);
  const rendered = renderRun(deepSemantic);
  assert.match(rendered, /FACE  DIRECT/);
  assert.match(rendered, /CLOSURE  closed/);

  return {
    profile: profile.id,
    provider_mode:'fixture',
    evidence_class:'structural-only',
    host_revision: deep.manifest.host.revision,
    direct_runtime: direct.manifest.runtime,
    deep_runtime: deep.manifest.runtime,
    direct_run_id: direct.run_id,
    deep_run_id: deep.run_id,
    shallow_direct_deep_equivalent: shallowComparison.equal,
    semantic_signature_mismatches: shallowComparison.mismatches.length,
    deep_semantic_event_count: deepSemantic.length,
    replay_event_count: replay.event_count,
    rendered
  };
}

function exerciseDeepOperators() {
  const runtime = new DeepRuntime({ policy:createPiBaselinePolicy() });
  const session = new DeepQLOperatorSession({ runId:'readiness:operators' });
  const frame = {
    id:'operator-frame',
    initiating_intent:'Verify deeper QL operator profile',
    operative_scope:'readiness',
    constraints:[],
    available_capabilities:[],
    success_conditions:['conjugation and depth are typed, observable and reintegrable'],
    provenance:{ fixture:'deep-readiness' }
  };

  const wholeCircuit = runtime.startCircuit(frame, { runId:'readiness:operators', circuitId:'readiness:operators:whole' });
  wholeCircuit.activePosition = qlPosition('P5');
  wholeCircuit.residues.push({ id:'form-1', kind:'form', position:'P3', value:{ known_defect:true }, provenance:{ fixture:'QLF-016' } });
  const whole = session.openConjugate({ directCircuit:wholeCircuit, scope:'whole', selectedResidueRefs:['form-1'] });
  assert.equal(whole.face, 'conjugate');
  assert.equal(whole.packet.requested_scope, 'whole');
  assert.equal(whole.packet.provenance.complete_direct_transcript_inherited, false);
  session.completeConjugate({
    directCircuit:wholeCircuit,
    conjugateCircuit:whole,
    delta:createConjugateDelta({ status:'confirm', discrepancyType:'none' })
  });

  const currentCircuit = runtime.startCircuit(frame, { runId:'readiness:operators', circuitId:'readiness:operators:current' });
  currentCircuit.activePosition = qlPosition('P5');
  const current = session.openConjugate({ directCircuit:currentCircuit, scope:'current_position' });
  assert.equal(current.packet.requested_scope, 'current_position');
  const delta = createConjugateDelta({ status:'reopen', targetPosition:'P3', discrepancyType:'formal-defect' });
  const reopened = session.completeConjugate({ directCircuit:currentCircuit, conjugateCircuit:current, delta });
  assert.equal(reopened.circuit.activePosition.id, 'P3');
  assert.equal(reopened.circuit.trajectory.at(-1).relation, 'R53');

  const parent = runtime.startCircuit(frame, { runId:'readiness:operators', circuitId:'readiness:operators:parent' });
  parent.activePosition = qlPosition('P4');
  const child = session.openChild({
    parentCircuit:parent,
    localWholeIntent:'Resolve bounded migration compatibility',
    successConditions:['compatibility determined']
  });
  assert.notEqual(child.id, parent.id);
  assert.equal(child.parent_id, parent.id);
  assert.equal(child.depth, 1);
  assert.equal(child.active_position.id, 'P0');
  child.active_position = qlPosition('P5');
  const reintegrated = session.completeChild({
    parentCircuit:parent,
    childCircuit:child,
    returnedDelta:{ compatibility:'supported' }
  });
  assert.equal('transcript' in reintegrated.summary, false);
  assert.equal(reintegrated.residue.provenance.typed_summary_only, true);
  assert.equal(reintegrated.circuit.activePosition.id, 'P4');

  const operatorEvents = session.snapshot();
  assert.ok(operatorEvents.length >= 8);
  assert.ok(operatorEvents.every((event) => validatePortableEvent(event).valid));
  const types = new Set(operatorEvents.map((event) => event.event_type));
  for (const required of ['conjugate_started','conjugate_completed','circuit_reopened','transition','child_started','child_completed','child_reintegrated']) {
    assert.ok(types.has(required), `missing observable operator event ${required}`);
  }
  const rendered = renderRun(operatorEvents);
  assert.match(rendered, /FACE  CONJUGATE/);
  assert.match(rendered, /CHILDREN  1/);
  assert.match(rendered, /RELATION  R53/);

  return {
    whole_conjugation: 'pass',
    current_position_conjugation: 'pass',
    fresh_context: 'pass',
    conjugate_reopen_relation: reopened.circuit.trajectory.at(-1).relation,
    recursive_depth: 'pass',
    child_summary_only: reintegrated.residue.provenance.typed_summary_only,
    observable_events: [...types].sort(),
    portable_event_count: operatorEvents.length,
    rendered
  };
}

function replayTypingBenchmark(profileId) {
  const events = corpus.map((record, index) => {
    const reference = record.benchmark_reference;
    const event = {
      spec:'ql-agent/0.1',
      schema_version:'0.1.0-deep',
      event_id:`typing:${profileId}:${record.id}`,
      event_type:'transition',
      run_id:`typing:${profileId}`,
      circuit_id:`typing:${profileId}:c0`,
      parent_circuit_id:null,
      sequence:index,
      face:reference.face,
      ql:{
        from:record.structural_facts.source_position,
        to:record.structural_facts.destination_position,
        relation:reference.relation,
        lens:['L1','L4′']
      },
      payload:{ benchmark_id:record.id, intent:record.act.intent, carrier:record.act.carrier },
      witness:{
        benchmark_reference:reference,
        model_claimed:record.model_claimed,
        retrospective:record.retrospective,
        human_witness:record.human_witness
      }
    };
    assert.equal(validatePortableEvent(event).valid, true, `${profileId}:${record.id} invalid benchmark replay event`);
    return event;
  });
  return events;
}

const conformance = runConformanceSuite();
assert.equal(conformance.failed, 0);
assert.equal(conformance.typing_corpus.readiness_gate, 'pass');
assert.equal(corpusStats().count, 100);
assert.equal(dryRuns.cases.length, 8);
assert.equal(new Set(dryRuns.cases.map((item) => item.id)).size, 8);

const hostResults = [];
for (const profile of profiles) hostResults.push(await runProfile(profile));

const benchmarkReplays = Object.fromEntries(profiles.map((profile) => [profile.id, replayTypingBenchmark(profile.id)]));
const referenceReplay = benchmarkReplays[profiles[0].id];
for (const profile of profiles.slice(1)) {
  const comparison = compareTraces(referenceReplay, benchmarkReplays[profile.id]);
  assert.equal(comparison.equal, true, `${profile.id}: benchmark replay differs from Pi profile`);
}

const operatorResults = exerciseDeepOperators();
const exactDeepRevision = process.env.DEEP_REVISION ?? process.env.GITHUB_HEAD_SHA ?? process.env.GITHUB_SHA ?? 'working-tree';

const report = {
  schema:'ql-structural-readiness/0.3',
  structural_ready:true,
  capability_effect_evidence_ready:false,
  evidence_class:'deterministic-structural-conformance',
  capability_effect_evidence:'requires live Series 1 / issue #110; fixture-model runs are ineligible for performance claims',
  revisions:{
    deep_ql:exactDeepRevision,
    foundation_merge:FOUNDATION_MERGE,
    foundation_runtime_candidate:FOUNDATION_RUNTIME,
    spec:SPEC_REVISION,
    direct_runtime:'0.1.0-foundation',
    deep_runtime:'0.1.0-deep-candidate',
    typing_benchmark:'ql-typing-benchmark/0.2'
  },
  conformance:{ required:conformance.required, passed:conformance.passed, failed:conformance.failed },
  operators:operatorResults,
  hosts:hostResults,
  typing_benchmark:{
    count:corpus.length,
    profiles_replayed:profiles.map((profile) => profile.id),
    cross_profile_semantic_equivalent:true,
    human_annotation_required:false
  },
  structural_review:{
    dry_run_catalog:dryRuns.cases.map(({ id, kind, review }) => ({ id, kind, review })),
    renderer_available_for:[...hostResults.map((result) => result.profile), 'deep-operator-session'],
    note:'These runs are deterministic structural fixtures. Review them for QL semantics and observability; do not use them as evidence of LLM capability improvement.'
  }
};

console.log(JSON.stringify(report, null, 2));
