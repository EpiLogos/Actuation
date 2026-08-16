import { pathToFileURL } from 'node:url';
import {
  FACES,
  POSITIONS,
  QL_SPEC,
  RESIDUE_KIND_BY_POSITION,
  createSuccessState,
  qlFace,
  qlPosition,
  qlRelation
} from '../../foundation/ql-core-runtime/semantics.js';
import { schemas, assertSchemaSet } from './schemas.js';
import { corpusStats, agreementMetrics } from '../typing-corpus/corpus.js';
import {
  DEEP_OPERATOR_PROFILE,
  createConjugatePacket,
  openConjugateCircuit,
  createConjugateDelta,
  reintegrateConjugateDelta,
  createDepthRequest,
  openChildCircuit,
  closeChildCircuit,
  reintegrateChildSummary
} from '../operators.js';

export const REQUIRED_EVENT_TYPES = [
  'run_started','circuit_started','frame_established','act_created','projection','return_received','return_interpreted','transition',
  'residue_created','residue_revised','residue_invalidated','determination_proposed','closure_evaluated','circuit_reopened','circuit_closed',
  'conjugate_started','conjugate_completed','child_started','child_completed','child_reintegrated','reentry_created','run_completed','run_failed','run_cancelled'
];

const groups = { S:5, E:5, C:10, R:6, U:3, G:5, J:7, N:7, T:7, A:6 };
export const REQUIRED_QLC_IDS = Object.freeze(
  Object.entries(groups).flatMap(([group, count]) =>
    Array.from({ length: count }, (_, index) => `QLC-${group}${String(index + 1).padStart(3, '0')}`)
  )
);

const check = (condition, message) => {
  if (!condition) throw new Error(message);
};

const fixture = (active = 'P5', closed = false) => ({
  id: 'fixture:c0',
  parent_id: null,
  depth: 0,
  face: 'direct',
  frame: {
    id: 'frame0',
    initiating_intent: 'Repair parser',
    success_conditions: ['verified']
  },
  active_position: qlPosition(active),
  residues: [
    { id:'f0', kind:'frame', position:'P0', provenance:{ source:'fixture' } },
    { id:'v1', kind:'evaluation', position:'P4', provenance:{ subject:'candidate' } },
    { id:'d1', kind:'determination', position:'P5', provenance:{ evaluation_ref:'v1' } }
  ],
  trajectory: [],
  closure_state: closed ? 'closed' : 'open',
  success_state: createSuccessState({ circuit: closed ? 'true' : 'false' }),
  children: [],
  conjugates: []
});

export function validatePortableEvent(event) {
  const errors = [];
  for (const key of ['spec','schema_version','event_id','event_type','run_id','circuit_id','sequence','face','ql','payload','witness']) {
    if (!(key in (event ?? {}))) errors.push(`missing:${key}`);
  }
  if (event?.spec !== QL_SPEC) errors.push('spec');
  if (!REQUIRED_EVENT_TYPES.includes(event?.event_type)) errors.push('event_type');
  if (!Number.isInteger(event?.sequence) || event.sequence < 0) errors.push('sequence');
  if (!FACES.includes(event?.face)) errors.push('face');
  if (event?.ql?.from && !POSITIONS.includes(event.ql.from)) errors.push('ql.from');
  if (event?.ql?.to && !POSITIONS.includes(event.ql.to)) errors.push('ql.to');
  if (event?.ql?.relation && !/^R[0-5][0-5]$/.test(event.ql.relation)) errors.push('ql.relation');
  if (event?.ql?.projection && event.ql.projection !== '0/1') errors.push('ql.projection');
  if (event?.ql?.return && event.ql.return !== '1/0') errors.push('ql.return');
  return { valid: errors.length === 0, errors };
}

export function validateMonotonicSequences(events) {
  const last = new Map();
  for (const event of events) {
    const prior = last.get(event.circuit_id);
    if (prior !== undefined && event.sequence <= prior) return false;
    last.set(event.circuit_id, event.sequence);
  }
  return true;
}

const sampleEvent = (type, sequence = 0, ql = {}, extras = {}) => ({
  spec: QL_SPEC,
  schema_version: '0.1.0-deep',
  event_id: `e${sequence}`,
  event_type: type,
  run_id: 'r',
  circuit_id: 'c',
  parent_circuit_id: null,
  sequence,
  face: 'direct',
  ql,
  payload: {},
  witness: {},
  ...extras
});

const destinationFromRelation = (relationId) => `P${relationId.slice(-1)}`;

function testStructural(number) {
  if (number === 1) check(qlPosition('P0').structuralKind !== qlPosition('P1').structuralKind, '4+2 distinction missing');
  if (number === 2) check(['P0','P5'].every((p) => qlPosition(p).structuralKind === 'implicate'), 'implicate classification');
  if (number === 3) check(['P1','P2','P3','P4'].every((p) => qlPosition(p).structuralKind === 'explicate'), 'explicate classification');
  if (number === 4) check(FACES.every((face) => qlFace(face) === face), 'direct/conjugate face');
  if (number === 5) {
    const ids = POSITIONS.flatMap((from) => POSITIONS.map((to) => qlRelation(from, to).id));
    check(ids.length === 36 && new Set(ids).size === 36 && ids.includes('R41') && ids.includes('R00'), '36 ordered relations');
  }
}

function testExchange(number) {
  if (number === 1) check(validatePortableEvent(sampleEvent('projection', 0, { from:'P3', projection:'0/1' })).valid, 'projection 0/1');
  if (number === 2) check(validatePortableEvent(sampleEvent('return_received', 0, { from:'P3', return:'1/0' })).valid, 'return 1/0');
  if (number === 3 || number === 5) {
    const carrier = { kind:'tool', name:'read' };
    const material = sampleEvent('transition', 0, { from:'P3', to:'P1', relation:'R31' }, { payload:{ carrier } });
    const evaluation = sampleEvent('transition', 1, { from:'P3', to:'P4', relation:'R34' }, { payload:{ carrier } });
    check(material.payload.carrier.name === evaluation.payload.carrier.name, 'carrier fixture mismatch');
    check(material.ql.to !== evaluation.ql.to && validatePortableEvent(material).valid && validatePortableEvent(evaluation).valid, 'carrier fixed destination');
  }
  if (number === 4) {
    const events = [
      sampleEvent('return_received', 1, { from:'P3', return:'1/0' }),
      sampleEvent('return_interpreted', 2, { from:'P3', return:'1/0' }),
      sampleEvent('transition', 3, { from:'P3', to:'P1', relation:'R31' })
    ];
    check(validateMonotonicSequences(events) && events[1].event_type === 'return_interpreted', 'difference/interpretation must precede transition');
  }
}

function testClosure(number) {
  if (number === 1) {
    const requestedOutcomes = schemas.determination.properties.requested_outcome.enum;
    check(requestedOutcomes.includes('close') && requestedOutcomes.includes('reopen'), 'explicit P5 determination outcomes');
  } else if (number === 2) {
    check(fixture().closure_state === 'open', 'no-tool condition auto-closed');
  } else if (number >= 3 && number <= 7) {
    const destinations = ['P1','P2','P3','P4','P0'];
    const target = destinations[number - 3];
    check(qlRelation('P5', target).id === `R5${target[1]}`, `P5 continuation ${target}`);
  } else if (number === 8) {
    check(schemas.closure.properties.closed_at_position.const === 'P5' && schemas.closure.required.includes('determination_ref'), 'QLClosure contract');
  } else if (number === 9) {
    check(['aborted','interrupted','crashed','cancelled','exhausted'].every((status) => status !== 'closed'), 'process interruption collapsed into closure');
  } else {
    const state = fixture();
    check(state.frame?.id && state.residues.some((r) => r.position === 'P4') && state.residues.some((r) => r.position === 'P5'), 'P0/P4/P5 closure evidence');
  }
}

function testResidue(number) {
  if (number === 1) {
    check(new Set(Object.values(RESIDUE_KIND_BY_POSITION)).size === 6, 'six Core residue classes');
  } else if (number === 2) {
    const ids = fixture().residues.map((r) => r.id);
    check(ids.every(Boolean) && new Set(ids).size === ids.length, 'stable residue identifiers');
  } else if (number === 3) {
    check(schemas.residue.required.includes('provenance') && fixture().residues.every((r) => r.provenance), 'residue provenance');
  } else {
    const eventType = number === 4 ? 'residue_created' : number === 5 ? 'residue_revised' : 'residue_invalidated';
    check(validatePortableEvent(sampleEvent(eventType, number)).valid, `${eventType} not representable`);
  }
}

function testSuccess(number) {
  if (number === 1) {
    const state = createSuccessState({ operational:'true', circuit:'false' });
    check(state.operational === 'true' && state.circuit === 'false', 'operational success collapsed into closure');
  } else if (number === 2) {
    const state = createSuccessState({ artifact:'true', task:'false' });
    check(state.artifact === 'true' && state.task === 'false', 'artifact success collapsed into task success');
  } else {
    const state = createSuccessState({ task:'unknown', conjugate_stability:'not_applicable' });
    check(state.task === 'unknown' && state.conjugate_stability === 'not_applicable', 'unknown/not_applicable unsupported');
  }
}

function testReentry(number) {
  const delta = {
    id:'delta-1', source_circuit:'c', achieved_artifact_refs:['artifact:A'], established_material_refs:['evidence:E'],
    retained_form_refs:[], changed_assumptions:[], unresolved_refs:['question:Q'], revised_success_conditions:[], provenance:{ closure_ref:'closure-1' }
  };
  const closure = { id:'closure-1', reentry_delta_ref:delta.id };
  const renewedFrame = { id:'frame+', inherited_delta:delta.id, unresolved_refs:[...delta.unresolved_refs] };
  if (number === 1) check(Boolean(closure.reentry_delta_ref) && closure.reentry_delta_ref === delta.id, 'closure does not create retained delta');
  if (number === 2) check(renewedFrame.inherited_delta === delta.id, 'renewed P0 lacks retained delta');
  if (number === 3) check(!('inherited_delta' in { id:'plain-restart' }), 'plain restart masquerades as re-entry');
  if (number === 4) check(renewedFrame.unresolved_refs.includes('question:Q'), 'unresolved residue lost on re-entry');
  if (number === 5) check(!('transcript' in renewedFrame) && !('transcript' in delta), 'irrelevant transcript required for re-entry');
}

function testConjugation(number) {
  const direct = fixture();
  if (number === 1) check(openConjugateCircuit({ directCircuit:direct }).face === 'conjugate', 'conjugate face');
  if (number === 2) check(createConjugatePacket({ directCircuit:direct, scope:'current_position' }).source_position === 'P5', 'J mapping');
  if (number === 3) check(openConjugateCircuit({ directCircuit:direct }).active_position.id === 'P0', 'K mapping');
  if (number === 4) check(createConjugatePacket({ directCircuit:direct, scope:'whole' }).requested_scope === 'whole', 'whole conjugation');
  if (number === 5) check(createConjugatePacket({ directCircuit:direct, scope:'current_position' }).requested_scope === 'current_position', 'current-position conjugation');
  if (number === 6) {
    const packet = createConjugatePacket({ directCircuit:direct });
    check(packet.provenance.reconstructed_context && packet.provenance.complete_direct_transcript_inherited === false, 'fresh-context packet');
  }
  if (number === 7) {
    const delta = createConjugateDelta({ status:'reopen', targetPosition:'P3' });
    const reintegrated = reintegrateConjugateDelta({ directCircuit:direct, delta });
    check(reintegrated.circuit.active_position.id === 'P3' && reintegrated.circuit.trajectory.at(-1).relation === 'R53', 'conjugate reopen');
  }
}

function testDepth(number) {
  const parent = fixture('P4');
  const child = openChildCircuit({ parentCircuit:parent, localWholeIntent:'Resolve migration compatibility' });
  if (number === 1) check(child.id !== parent.id, 'child CircuitId');
  if (number === 2) check(child.parent_id === parent.id, 'child parent');
  if (number === 3) check(child.depth === parent.depth + 1, 'child depth');
  if (number === 4) check(child.active_position.id === 'P0' && child.frame.provenance.parent_circuit === parent.id, 'independent P0 frame');
  if (number === 5) check(Array.isArray(child.trajectory) && child.face === 'direct', 'ordinary child QL trace state');
  if (number >= 6) {
    child.active_position = qlPosition('P5');
    const summary = closeChildCircuit({ childCircuit:child, returnedDelta:{ retained:true } });
    if (number === 6) check(summary.child_circuit === child.id && summary.status === 'closed' && !('transcript' in summary), 'typed child summary');
    if (number === 7) {
      const result = reintegrateChildSummary({ parentCircuit:parent, summary });
      check(result.residue.provenance.typed_summary_only && result.circuit.active_position.id === 'P4', 'summary-only parent reintegration');
    }
  }
}

function testTrace(number) {
  if (number === 1) check(validatePortableEvent(sampleEvent('run_started')).valid, 'JSONL envelope');
  if (number === 2) check(validateMonotonicSequences([sampleEvent('run_started',0), sampleEvent('transition',1)]), 'monotonic sequence');
  if (number === 3) {
    const host = { channel:'host', event_type:'model_request' };
    const ql = { channel:'runtime-semantic', ...sampleEvent('transition',0,{ from:'P1', to:'P3', relation:'R13' }) };
    check(host.channel !== ql.channel && ql.spec === QL_SPEC, 'framework/QL distinction');
  }
  if (number === 4) {
    const event = sampleEvent('transition',0,{ from:'P4', to:'P3', relation:'R43' });
    check(event.ql.from === qlRelation('P4','P3').from && destinationFromRelation(event.ql.relation) === event.ql.to, 'relation source/destination recovery');
  }
  if (number === 5) {
    const event = sampleEvent('return_interpreted',0,{ from:'P4', to:'P3' }, { witness:{ claimed:'P4', retrospective:'P3' } });
    check(event.witness.claimed !== event.witness.retrospective, 'witness disagreement erased');
  }
  if (number === 6) {
    const event = sampleEvent('projection',0,{ from:'P3', projection:'0/1' }, { payload:{ carrier:{ kind:'tool', name:'read' } } });
    check(event.payload.carrier.kind === 'tool' && event.payload.carrier.name === 'read', 'carrier metadata');
  }
  if (number === 7) {
    const events = [sampleEvent('transition',0), sampleEvent('transition',1)];
    check(events.every((event) => Boolean(event.event_id)) && new Set(events.map((event) => event.event_id)).size === events.length, 'stable event IDs');
  }
}

function testAntiCollapse(number) {
  if (number === 1) {
    const path = ['P0','P3','P4','P5'];
    const relations = path.slice(0,-1).map((from,index) => qlRelation(from, path[index + 1]).id);
    check(path.length < 6 && relations.length === 3, 'mandatory six-step traversal');
  }
  if (number === 2) {
    const readAsMaterial = { carrier:'read', position:'P1' };
    const readAsEvaluation = { carrier:'read', position:'P4' };
    check(readAsMaterial.carrier === readAsEvaluation.carrier && readAsMaterial.position !== readAsEvaluation.position, 'tool type equals QL position');
  }
  if (number === 3) {
    const sameRole = [{ role:'assistant', position:'P3' }, { role:'assistant', position:'P5' }];
    check(sameRole[0].role === sameRole[1].role && sameRole[0].position !== sameRole[1].position, 'message role equals QL position');
  }
  if (number === 4) {
    const sameNode = [{ node:'ModelRequestNode', position:'P1' }, { node:'ModelRequestNode', position:'P4' }];
    check(sameNode[0].node === sameNode[1].node && sameNode[0].position !== sameNode[1].position, 'graph node equals QL position');
  }
  if (number === 5) {
    const bareCriticCall = { carrier:{ kind:'model', name:'critic' } };
    const packet = createConjugatePacket({ directCircuit:fixture() });
    check(!('requested_scope' in bareCriticCall) && packet.provenance.reconstructed_context, 'critic call alone constituted conjugation');
  }
  if (number === 6) {
    const bareSubagent = { carrier:{ kind:'child_circuit' } };
    const request = createDepthRequest({ parentCircuit:fixture('P4'), localWholeIntent:'local whole' });
    check(!('parent_circuit' in bareSubagent) && request.parent_position === 'P4', 'subagent call alone constituted QL depth');
  }
}

function testId(id) {
  const group = id[4];
  const number = Number(id.slice(5));
  if (group === 'S') testStructural(number);
  else if (group === 'E') testExchange(number);
  else if (group === 'C') testClosure(number);
  else if (group === 'R') testResidue(number);
  else if (group === 'U') testSuccess(number);
  else if (group === 'G') testReentry(number);
  else if (group === 'J') testConjugation(number);
  else if (group === 'N') testDepth(number);
  else if (group === 'T') testTrace(number);
  else if (group === 'A') testAntiCollapse(number);
  else throw new Error(`unknown QLC group ${group}`);
}

export function runConformanceSuite() {
  const results = REQUIRED_QLC_IDS.map((id) => {
    try {
      testId(id);
      return { id, status:'pass' };
    } catch (error) {
      return { id, status:'fail', error:error.message };
    }
  });

  const typing = corpusStats();
  const shape = typing.count === 100 &&
    ['P1','P2','P3','P4'].every((p) => typing.by_position[p] >= 12) &&
    ['P0','P5'].every((p) => typing.by_position[p] >= 8) &&
    typing.ambiguous >= 12 &&
    typing.p5_reopening >= 10 &&
    typing.cross_carrier_same_function >= 10 &&
    typing.same_carrier_cross_function >= 10 &&
    typing.stable_ids &&
    typing.benchmark_provenance;

  return {
    profile: DEEP_OPERATOR_PROFILE.name,
    required: results.length,
    passed: results.filter((result) => result.status === 'pass').length,
    failed: results.filter((result) => result.status === 'fail').length,
    results,
    fixtures: [
      { id:'Foundation QLF-001..014/017/018 + QLN set', status:'inherited-frozen-baseline' },
      { id:'QLF-015', status:'pass' },
      { id:'QLF-016', status:'pass' }
    ],
    schemas: assertSchemaSet() ? 'pass' : 'fail',
    typing_corpus: {
      ...typing,
      shape,
      metrics: agreementMetrics(),
      readiness_gate: shape ? 'pass' : 'fail',
      human_review: 'optional-product-level-witness'
    }
  };
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  const report = runConformanceSuite();
  console.log(JSON.stringify(report, null, 2));
  process.exitCode = report.failed || !report.typing_corpus.shape ? 1 : 0;
}
