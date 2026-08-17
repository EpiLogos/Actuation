import test from 'node:test';
import assert from 'node:assert/strict';
import { RuntimeRegistry, RUN_STATUS } from '../runtime-contract/index.js';
import { POSITIONS, RESIDUE_KIND_BY_POSITION, qlPosition, qlRelation } from '../ql-core-runtime/semantics.js';
import {
  createCircuit,
  absorbReturn,
  establishDifference,
  interpretReturn,
  applyTransition,
  createDetermination,
  createClosureVerdict,
  createReentryDelta,
  closeCircuit,
  createReentry
} from '../ql-core-runtime/circuit.js';

const inertHost = {
  async callModel() {},
  async executeCapability() {},
  async receiveExternalInput() { return null; },
  async readContext() { return {}; }
};
const observer = { emit() {} };

test('shared registry selects two runtimes without QL concepts in the contract', async () => {
  const make = (id) => ({ id, version: 'test', async run() { return { status: RUN_STATUS.COMPLETED, runtime: id }; } });
  const registry = new RuntimeRegistry().register(make('classic')).register(make('ql-core'));
  assert.deepEqual(registry.list().map((entry) => entry.id), ['classic', 'ql-core']);
  assert.equal((await registry.run('classic', { taskId: 't' }, inertHost, observer)).runtime, 'classic');
  assert.equal((await registry.run('ql-core', { taskId: 't' }, inertHost, observer)).runtime, 'ql-core');
});

test('4+2 structure, all Rij relations and six residue responsibilities are representable', () => {
  assert.equal(qlPosition('P0').structuralKind, 'implicate');
  assert.equal(qlPosition('P5').structuralKind, 'implicate');
  for (const id of ['P1','P2','P3','P4']) assert.equal(qlPosition(id).structuralKind, 'explicate');
  const relations = new Set();
  for (const from of POSITIONS) for (const to of POSITIONS) relations.add(qlRelation(from, to).id);
  assert.equal(relations.size, 36);
  assert.deepEqual(Object.values(RESIDUE_KIND_BY_POSITION), ['frame','material','effect','form','evaluation','determination']);
});

test('returned difference must exist before semantic destination and Rij transition', () => {
  const circuit = createCircuit({ id: 'frame:1', initiating_intent: 'prove recurrence', success_conditions: [] }, { runId: 'r' });
  const act = { id: 'a1', carrier: { kind: 'internal_control' } };
  const returned = absorbReturn({ act, rawResult: 'candidate form', operationSuccess: true });
  assert.throws(() => interpretReturn(circuit, returned, { destination: 'P3' }), /difference must be established/i);
  establishDifference(returned, { kind: 'form-disclosed' });
  const interpretation = interpretReturn(circuit, returned, { destination: 'P3' });
  const transition = applyTransition(circuit, interpretation);
  assert.equal(transition.relation, 'R03');
  assert.equal(circuit.active_position.id, 'P3');
});

test('P5 determination, typed reopening and positive closure remain distinct', () => {
  const circuit = createCircuit({ id: 'frame:2', initiating_intent: 'close positively', success_conditions: ['done'] }, { runId: 'r2' });
  circuit.active_position = qlPosition('P5');
  const determination = createDetermination(circuit, { synthesis: 'candidate', requested_outcome: 'close' });
  const reopen = createClosureVerdict({ status: 'reopen', destination: 'P3', task_success: 'false' });
  assert.equal(reopen.reopening_relation, 'R53');
  assert.equal(circuit.closure_state, 'open');

  const close = createClosureVerdict({ status: 'close', task_success: 'true' });
  const closure = closeCircuit(circuit, determination, close);
  assert.equal(closure.closed_at_position, 'P5');
  assert.equal(circuit.closure_state, 'closed');
  // Closure is created before any re-entry material exists; the linkage slot
  // is only filled after the delta is derived from the closed circuit.
  assert.equal(closure.reentry_delta_ref, null);
  const delta = createReentryDelta(circuit, determination, { achieved_artifact_refs: ['artifact:A'], unresolved_refs: ['question:Q'] });
  const reentry = createReentry(circuit, delta, closure);
  assert.equal(closure.reentry_delta_ref, null);
  assert.equal(reentry.closure_ref, closure.id);
  assert.equal(reentry.renewed_frame.inherited_delta, delta.id);
  assert.deepEqual(delta.achieved_artifact_refs, ['artifact:A']);
  assert.deepEqual(delta.unresolved_refs, ['question:Q']);
});
