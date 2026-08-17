import test from 'node:test';
import assert from 'node:assert/strict';

import {
  createCircuit,
  createDetermination,
  createClosureVerdict,
  createReentryDelta,
  closeCircuit,
  createReentry
} from '../ql-core-runtime/circuit.js';
import { qlPosition } from '../ql-core-runtime/semantics.js';
import { closureSummaryForResult } from '../optics/index.js';

test('published re-entry helpers reject pre-closure derivation', () => {
  const circuit = createCircuit({
    id: 'frame:closure-order',
    initiating_intent: 'enforce closure before re-entry',
    success_conditions: ['closed first']
  }, { runId: 'run:closure-order' });
  circuit.active_position = qlPosition('P5');

  const determination = createDetermination(circuit, {
    synthesis: 'candidate',
    evidence_refs: ['ver:current'],
    requested_outcome: 'close'
  });

  assert.throws(
    () => createReentryDelta(circuit, determination),
    /only be derived after positive QLClosure/i
  );

  const verdict = createClosureVerdict({ status: 'close', task_success: 'true' });
  const closure = closeCircuit(circuit, determination, verdict);
  const delta = createReentryDelta(circuit, determination);
  const reentry = createReentry(circuit, delta, closure);

  assert.equal(circuit.closure_state, 'closed');
  assert.equal(reentry.closure_ref, closure.id);
  assert.equal(reentry.delta_ref, delta.id);
});

test('closure optics retain supporting Run and evidence references', () => {
  const result = {
    status: 'completed',
    determination: {
      evidence_refs: ['ver:current', 'artifact:evidence']
    },
    closure: {
      id: 'c0:closure',
      circuit_id: 'c0',
      evaluation_refs: ['evaluation:1'],
      success_state: { task: 'true', circuit: 'true' }
    }
  };

  const summary = closureSummaryForResult(result, 'run:verified');
  assert.deepEqual(summary.run_refs, ['run:verified']);
  assert.deepEqual(summary.evidence_refs, ['ver:current', 'artifact:evidence']);
  assert.deepEqual(summary.evaluation_refs, ['evaluation:1']);
});
