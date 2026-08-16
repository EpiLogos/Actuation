import assert from 'node:assert/strict';
import { createRunManifest, executeRun } from '../../foundation/optics/index.js';
import { qlPosition } from '../../foundation/ql-core-runtime/semantics.js';
import { DeepQLOperatorSession } from '../operator-session.js';
import { createConjugateDelta } from '../operators.js';
import { validatePortableEvent } from '../conformance/runner.js';
import { renderRun } from '../render/index.js';

const residueKind = Object.freeze({ P1:'material', P2:'effect', P3:'form', P4:'evaluation' });

function arrivals(circuit, position) {
  return circuit.trajectory.filter((transition) => transition.to === position).length;
}

function destinationFor(scenarioId, circuit) {
  const from = circuit.activePosition.id;
  if (scenarioId === 'QLDR-001') {
    return { P0:'P3', P3:'P4', P4:'P5' }[from];
  }
  if (scenarioId === 'QLDR-002') {
    return { P0:'P1', P1:'P2', P2:'P4', P4:'P5' }[from];
  }
  if (scenarioId === 'QLDR-003') {
    if (from === 'P0') return 'P1';
    if (from === 'P1') return 'P3';
    if (from === 'P3') return arrivals(circuit, 'P3') <= 1 ? 'P2' : 'P4';
    if (from === 'P2') return 'P3';
    if (from === 'P4') return 'P5';
  }
  if (scenarioId === 'QLDR-004') {
    return { P0:'P2', P2:'P4', P4:'P5' }[from];
  }
  if (scenarioId === 'QLDR-005') {
    if (from === 'P0') return arrivals(circuit, 'P0') ? 'P3' : 'P4';
    if (from === 'P4') return arrivals(circuit, 'P4') >= 2 ? 'P5' : 'P0';
    if (from === 'P3') return 'P4';
  }
  if (scenarioId === 'QLDR-006') {
    return { P0:'P1', P1:'P4', P4:'P5' }[from];
  }
  if (scenarioId === 'QLDR-007' || scenarioId === 'QLDR-008') {
    return { P0:'P4', P4:'P5' }[from];
  }
  throw new Error(`${scenarioId}: no review destination from ${from}`);
}

function createReviewPolicy(scenario) {
  return {
    nextAct({ circuit }) {
      const from = circuit.activePosition.id;
      if (from === 'P5') return null;
      if (scenario.id === 'QLDR-004' && from === 'P2') {
        return {
          intent:'Exercise a failing exterior operation and retain its difference.',
          carrier:{ kind:'capability', name:'missing-review-capability', args:{ scenario:scenario.id } }
        };
      }
      if (from === 'P1' && ['QLDR-002','QLDR-003','QLDR-006'].includes(scenario.id)) {
        return {
          intent:`Acquire current evidence for ${scenario.id}.`,
          carrier:{ kind:'capability', name:'inspect', args:{ subject:scenario.id } }
        };
      }
      return {
        intent:`${scenario.kind}: enact ${from} responsibility.`,
        carrier:{ kind:'model' }
      };
    },

    establishDifference({ returned }) {
      return returned.raw_result;
    },

    interpret({ circuit, difference, returned }) {
      const from = circuit.activePosition.id;
      const destination = destinationFor(scenario.id, circuit);
      const create = [];
      const kind = residueKind[from];
      if (kind) {
        const value = kind === 'evaluation'
          ? {
              scenario:scenario.id,
              subject:scenario.id,
              current:scenario.id !== 'QLDR-006' || arrivals(circuit, 'P4') >= 2,
              assessment:'whole-relative review evidence'
            }
          : kind === 'effect'
            ? { scenario:scenario.id, operation_success:returned.operation_success, difference }
            : { scenario:scenario.id, difference };
        create.push({ kind, position:from, value, provenance:{ fixture:scenario.id, review:true } });
      }
      return {
        destination,
        rationale:`${scenario.id}: ${from} discloses ${destination} as the next responsibility.`,
        witness:{
          claimed_position:from,
          observed_position:destination,
          structural_facts:{ scenario:scenario.id, carrier_failure:returned.operation_success === false }
        },
        residueDelta:{ create }
      };
    },

    proposeDetermination({ circuit }) {
      const evaluations = circuit.residues.filter((entry) => entry.kind === 'evaluation' && !entry.invalidated);
      return {
        synthesis:`${scenario.id} candidate is ready for whole-relative closure evaluation.`,
        claimed_adequacy:'candidate',
        claimed_subject:scenario.id,
        claimed_state:'reviewed',
        evidence_refs:evaluations.map((entry) => entry.id),
        evaluation_refs:evaluations.map((entry) => entry.id),
        unresolved_refs:[],
        requested_outcome:'close'
      };
    },

    evaluateClosure({ evaluations }) {
      if (scenario.id === 'QLDR-006' && !evaluations.some((entry) => entry.value?.current === true)) {
        return {
          status:'reopen',
          destination:'P4',
          task_success:'false',
          rationale:'The only verification is stale; revisit P4 for current subject-matched evidence.'
        };
      }
      return {
        status:'close',
        task_success:'true',
        rationale:'The representative dry run has current whole-relative evidence for its stated review purpose.'
      };
    },

    createReentryDelta() {
      return {
        achieved_artifact_refs:[`review:${scenario.id}`],
        changed_assumptions:[],
        unresolved_refs:[],
        opened_questions:[],
        provenance:{ fixture:scenario.id, kind:'product-review' }
      };
    }
  };
}

function operatorFrame(profileId, scenarioId) {
  return {
    id:`${profileId}:${scenarioId}:frame`,
    initiating_intent:`Exercise ${scenarioId} deeper operator semantics for ${profileId}.`,
    operative_scope:'product-review',
    constraints:[],
    available_capabilities:[],
    success_conditions:['operator behaviour is typed, observable and reintegrable'],
    provenance:{ profile:profileId, scenario:scenarioId }
  };
}

function recordOperatorReview({ scenario, profile, runtime }) {
  if (!['QLDR-007','QLDR-008'].includes(scenario.id)) return null;
  const runId = `review:${profile.id}:${scenario.id}:operators`;
  const session = new DeepQLOperatorSession({ runId });
  const frame = operatorFrame(profile.id, scenario.id);

  if (scenario.id === 'QLDR-007') {
    const parent = runtime.startCircuit(frame, { runId, circuitId:`${runId}:parent` });
    parent.activePosition = qlPosition('P4');
    const child = session.openChild({
      parentCircuit:parent,
      localWholeIntent:'Resolve one bounded compatibility question.',
      successConditions:['compatibility determined']
    });
    child.active_position = qlPosition('P5');
    const result = session.completeChild({
      parentCircuit:parent,
      childCircuit:child,
      returnedDelta:{ compatibility:'supported', profile:profile.id }
    });
    assert.equal(result.residue.provenance.typed_summary_only, true);
  } else {
    const wholeDirect = runtime.startCircuit(frame, { runId, circuitId:`${runId}:whole` });
    wholeDirect.activePosition = qlPosition('P5');
    const whole = session.openConjugate({ directCircuit:wholeDirect, scope:'whole' });
    session.completeConjugate({
      directCircuit:wholeDirect,
      conjugateCircuit:whole,
      delta:createConjugateDelta({ status:'confirm', discrepancyType:'none' })
    });

    const currentDirect = runtime.startCircuit(frame, { runId, circuitId:`${runId}:current` });
    currentDirect.activePosition = qlPosition('P5');
    const current = session.openConjugate({ directCircuit:currentDirect, scope:'current_position' });
    session.completeConjugate({
      directCircuit:currentDirect,
      conjugateCircuit:current,
      delta:createConjugateDelta({ status:'reopen', targetPosition:'P3', discrepancyType:'formal-defect' })
    });
  }

  const events = session.snapshot();
  assert.ok(events.every((event) => validatePortableEvent(event).valid));
  return {
    event_count:events.length,
    event_types:[...new Set(events.map((event) => event.event_type))].sort(),
    rendered:renderRun(events)
  };
}

async function runHostReview({ scenario, profile, DeepRuntime, specRevision }) {
  const runtime = new DeepRuntime({ policy:createReviewPolicy(scenario) });
  const host = profile.hostFactory();
  const request = {
    taskId:`${profile.id}-${scenario.id.toLowerCase()}`,
    input:{ scenario:scenario.id, prompt:scenario.prompt },
    successConditions:scenario.review,
    capabilities:profile.capabilities,
    maxSteps:12
  };
  const manifest = createRunManifest({
    taskId:request.taskId,
    fixtureId:`${scenario.id}-${profile.id}`,
    host,
    runtime,
    specRevision,
    model:profile.model,
    capabilities:profile.capabilities,
    environment:{ id:`${profile.id}-${scenario.id}-review-env`, network:false },
    successConditions:request.successConditions,
    startState:{ ...profile.startState, review_scenario:scenario.id }
  });
  const record = await executeRun({ runtime, host, request, manifest });
  assert.equal(record.status.execution, 'completed', `${profile.id}/${scenario.id}: execution did not complete`);
  assert.equal(record.status.semantic, 'closed', `${profile.id}/${scenario.id}: semantic circuit did not close`);
  const semantic = record.events.filter((event) => event.channel === 'runtime-semantic');
  assert.ok(semantic.every((event) => validatePortableEvent(event).valid), `${profile.id}/${scenario.id}: invalid portable event`);

  if (scenario.id === 'QLDR-004') {
    assert.ok(semantic.some((event) => event.event_type === 'return_received' && event.payload?.returned?.operation_success === false));
  }
  if (scenario.id === 'QLDR-006') {
    assert.ok(semantic.some((event) => event.event_type === 'circuit_reopened' && event.ql?.relation === 'R54'));
  }

  return {
    profile:profile.id,
    scenario:scenario.id,
    kind:scenario.kind,
    run_id:record.run_id,
    execution:record.status.execution,
    semantic:record.status.semantic,
    trajectory:semantic.filter((event) => event.event_type === 'transition').map((event) => event.ql?.relation),
    semantic_event_count:semantic.length,
    rendered:renderRun(semantic),
    operator_review:recordOperatorReview({ scenario, profile, runtime })
  };
}

export async function runProductReviewSet({ profiles, scenarios, DeepRuntime, specRevision }) {
  const records = [];
  for (const profile of profiles) {
    for (const scenario of scenarios) {
      records.push(await runHostReview({ scenario, profile, DeepRuntime, specRevision }));
    }
  }
  assert.equal(records.length, profiles.length * scenarios.length);
  return records;
}
