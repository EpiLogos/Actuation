import test from 'node:test';
import assert from 'node:assert/strict';
import { QLDirectCoreRuntime, UnsupportedQLOperatorError } from '../../foundation/ql-core-runtime/index.js';
import { qlPosition } from '../../foundation/ql-core-runtime/semantics.js';
import { createDeepQLRuntimeClass } from '../index.js';
import { createConjugatePacket, createConjugateDelta, reintegrateConjugateDelta, openChildCircuit, closeChildCircuit, reintegrateChildSummary } from '../operators.js';
import { REQUIRED_QLC_IDS, runConformanceSuite, validatePortableEvent } from '../conformance/runner.js';
import { corpus, corpusStats, agreementMetrics } from '../typing-corpus/corpus.js';
import { renderRun } from '../render/index.js';
import { compareTraces } from '../comparison/comparator.js';

const direct = (active = 'P5', closed = false) => ({
  id: 'd', depth: 0, face: 'direct',
  frame: { id: 'f', initiating_intent: 'i', success_conditions: [] },
  active_position: qlPosition(active), residues: [], trajectory: [],
  closure_state: closed ? 'closed' : 'open', children: [], conjugates: []
});

test('all 61 stable QLC IDs pass substantive checks', () => {
  const r = runConformanceSuite();
  assert.equal(REQUIRED_QLC_IDS.length, 61);
  assert.equal(r.failed, 0);
  assert.equal(r.passed, 61);
});

test('QLF-016 reconstructs fresh conjugate context and R53 reopens open direct P5', () => {
  const d = direct();
  const p = createConjugatePacket({ directCircuit: d, scope: 'whole' });
  assert.equal(p.provenance.complete_direct_transcript_inherited, false);
  const x = createConjugateDelta({ status: 'reopen', targetPosition: 'P3' });
  const reintegrated = reintegrateConjugateDelta({ directCircuit: d, delta: x });
  assert.equal(reintegrated.circuit.active_position.id, 'P3');
  assert.equal(reintegrated.circuit.trajectory.at(-1).relation, 'R53');
});

test('positive closure cannot be retroactively reopened by conjugation', () => {
  assert.throws(() => reintegrateConjugateDelta({
    directCircuit: direct('P5', true),
    delta: createConjugateDelta({ status: 'reopen', targetPosition: 'P3' })
  }));
});

test('QLF-015 child closes independently and parent reintegrates typed summary without transcript', () => {
  const p = direct('P4');
  const c = openChildCircuit({ parentCircuit: p, localWholeIntent: 'local' });
  c.active_position = qlPosition('P5');
  const s = closeChildCircuit({ childCircuit: c, returnedDelta: { x: 1 } });
  const r = reintegrateChildSummary({ parentCircuit: p, summary: s });
  assert.equal(r.residue.provenance.typed_summary_only, true);
  assert.equal('transcript' in s, false);
});

test('frozen Direct Core rejects deeper operators while Deep profile exposes the reserved seam', () => {
  const base = new QLDirectCoreRuntime({ policy: {} });
  assert.throws(() => base.openConjugate(), UnsupportedQLOperatorError);
  assert.throws(() => base.openChild(), UnsupportedQLOperatorError);

  const DeepRuntime = createDeepQLRuntimeClass(QLDirectCoreRuntime);
  const deep = new DeepRuntime({ policy: {} });
  assert.equal(deep.openConjugate({ directCircuit: direct() }).face, 'conjugate');
  assert.equal(deep.openChild({ parentCircuit: direct('P4'), localWholeIntent: 'local' }).active_position.id, 'P0');
});

test('typing benchmark is meaningful, stable, provenance-bearing and does not invent human judgement', () => {
  const s = corpusStats();
  assert.equal(s.count, 100);
  assert.equal(s.stable_ids, true);
  assert.equal(s.benchmark_provenance, true);
  assert.equal(s.human_witnesses, 0);
  assert.ok(corpus.every((record) => record.title && !record.title.startsWith('Deterministic semantic typing act')));
  assert.ok(corpus.every((record) => record.human_witness === null));
  assert.equal(agreementMetrics().claimed_human, null);
  assert.ok(agreementMetrics().claimed_benchmark.position_exact_agreement < 1);
});

test('invalid and unsupported semantic claims fail explicitly', () => {
  const badRelation = { spec: 'ql-agent/0.1', schema_version: 'x', event_id: 'e', event_type: 'transition', run_id: 'r', circuit_id: 'c', sequence: 0, face: 'direct', ql: { relation: 'R99' }, payload: {}, witness: {} };
  const badFace = { ...badRelation, ql: { relation: 'R11' }, face: 'sideways' };
  assert.equal(validatePortableEvent(badRelation).valid, false);
  assert.equal(validatePortableEvent(badFace).valid, false);
});

test('renderer and comparator remain portable and preserve visible disagreement', () => {
  const first = { spec: 'ql-agent/0.1', schema_version: 'x', event_id: 'e0', event_type: 'run_started', run_id: 'r', circuit_id: 'c', parent_circuit_id: null, sequence: 0, face: 'direct', ql: { to: 'P0', lens: ['L1', 'L4′'] }, payload: {}, witness: {} };
  const second = { ...first, event_id: 'e1', sequence: 1, event_type: 'transition', ql: { from: 'P0', to: 'P3', relation: 'R03' }, witness: { claimed: 'P0', retrospective: 'P3' } };
  const rendered = renderRun([first, second]);
  assert.match(rendered, /ACTIVE  P3/);
  assert.match(rendered, /RELATION  R03/);
  assert.equal(compareTraces([first, second], [first, second]).equal, true);
  assert.equal(compareTraces([first], [first, second]).equal, false);
});
