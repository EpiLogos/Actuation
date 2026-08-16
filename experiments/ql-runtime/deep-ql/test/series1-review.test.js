import test from 'node:test';
import assert from 'node:assert/strict';
import { REQUIRED_HELD_CONSTANTS, SERIES1_SCHEMA } from '../../comparison/series1/contract.mjs';
import { assessComparisonEvidence, assertNoConfiguredSecrets, buildMaskMapping, sanitizeEvidence } from '../../comparison/series1/evidence.mjs';
import { renderMaskedReview, renderMaskMapping, renderUnmaskedReview } from '../../comparison/series1/review.mjs';

function record(condition, runtimeId) {
  const events = [
    { record_index: 0, channel: 'host', event_type: 'model_requested', payload: { purpose: condition === 'classic' ? 'classic-turn' : 'ql-control', input: { system: `runtime ${condition}`, prompt: 'internal model prompt' } } },
    { record_index: 1, channel: 'host', event_type: 'model_returned', payload: { purpose: condition === 'classic' ? 'classic-turn' : 'ql-control', output: { content: condition === 'classic' ? 'ordinary return' : '', control: condition === 'classic' ? null : { position: 'P5' }, usage: { total_tokens: 3 } } } },
    { record_index: 2, channel: 'host', event_type: 'capability_requested', payload: { name: 'read_file', args: { path: 'fact.txt' } } },
    { record_index: 3, channel: 'host', event_type: 'capability_returned', payload: { name: 'read_file', ok: true, result: { ok: true, path: 'fact.txt', content: 'grounded fact' } } },
    { record_index: 4, channel: 'runtime-semantic', event_type: 'ql_position_entered', ql: { position: 'P5' }, payload: { runtime: runtimeId } }
  ];
  return {
    schema: SERIES1_SCHEMA,
    benchmark_revision: 'bench-rev',
    task_revision: 'task-rev',
    runner_revision: 'runner-rev',
    review_contract_revision: 'review-rev',
    host_revision: 'host-rev',
    host: { id: 'native', revision: 'host-rev' },
    condition,
    repetition: 0,
    runtime: { id: runtimeId, version: '0.1' },
    model: { provider: 'deepseek', id: 'deepseek-v4-flash', parameters: { temperature: 0 } },
    prompt: 'According to fact.txt, report the fact.',
    success_conditions: ['Use the file.', 'Do not edit.'],
    starting_workspace: { 'fact.txt': 'grounded fact\n' },
    final_workspace: { 'fact.txt': 'grounded fact\n' },
    outcome: 'The fact is grounded fact.',
    verification: { protocol: 'workspace unchanged', observations: { workspaceUnchanged: true }, objective_checks_pass: true },
    execution_status: 'completed',
    semantic_status: condition === 'classic' ? 'not_applicable' : 'closed',
    elapsed_ms: 12,
    model_calls: 1,
    capability_calls: 1,
    total_tokens: 3,
    model_cost: null,
    record: { events },
    host_native_evidence: { kind: 'trajectory-reference', ref: `native:${condition}:session` }
  };
}

function manifest() {
  const held = Object.fromEntries(REQUIRED_HELD_CONSTANTS.map((field) => [field, true]));
  held.valid = true;
  held.mismatches = [];
  return {
    schema: SERIES1_SCHEMA,
    benchmark: 'series1-v0.1-human-review',
    benchmark_revision: 'bench-rev',
    runner_revision: 'runner-rev',
    review_contract_revision: 'review-rev',
    host: { id: 'native', revision: 'host-rev', real_framework_path: 'fixture host for structural rendering test' },
    model: { provider: 'deepseek', id: 'deepseek-v4-flash', parameters: { temperature: 0 } },
    determination: 'pending-human-review',
    held_constant: held,
    task: { id: 'S1-RESTRAINT-001', category: 'bounded-restraint', revision: 'task-rev' },
    review: {
      prompt: 'According to fact.txt, report the fact.',
      success_conditions: ['Use the file.', 'Do not edit.'],
      verification_protocol: ['Check the workspace is unchanged.'],
      focus: ['restraint', 'evidence use'],
      human_reference: ['__HUMAN_ONLY_EXPECTED_REFERENCE__']
    },
    process_summary: [{ repetition: 0, conditions: {} }],
    records: [record('classic', 'classic'), record('ql-direct', 'ql-core'), record('ql-deep', 'ql-deep')]
  };
}

test('one deterministic Series 1 record renders separate masked and unmasked human-review passes', () => {
  const evidence = manifest();
  const masked = renderMaskedReview(evidence);
  const unmasked = renderUnmaskedReview(evidence);
  const mapping = renderMaskMapping(evidence);

  assert.match(masked, /PASS A/);
  assert.match(masked, /Candidate [ABC]/);
  assert.match(masked, /Complete starting workspace/);
  assert.match(masked, /capability requested/);
  assert.match(masked, /grounded fact/);
  assert.match(masked, /Objective verification evidence/);
  assert.match(masked, /Complete final workspace/);
  assert.doesNotMatch(masked, /\bclassic\b/i);
  assert.doesNotMatch(masked, /ql-direct|ql-deep|ql-core|runtime-semantic|ql_position_entered|\bP5\b/i);
  assert.doesNotMatch(masked, /__HUMAN_ONLY_EXPECTED_REFERENCE__/);

  assert.match(unmasked, /PASS B/);
  assert.match(unmasked, /classic/);
  assert.match(unmasked, /ql-direct/);
  assert.match(unmasked, /ql-deep/);
  assert.match(unmasked, /runtime-semantic/);
  assert.match(unmasked, /ql_position_entered/);
  assert.match(unmasked, /__HUMAN_ONLY_EXPECTED_REFERENCE__/);
  assert.match(unmasked, /trajectory-reference/);

  assert.match(mapping, /ql-series1-mask-map\/0\.1/);
  assert.match(mapping, /classic/);
  assert.match(mapping, /ql-direct/);
  assert.match(mapping, /ql-deep/);
  assert.deepEqual(buildMaskMapping(evidence), buildMaskMapping(evidence));
});

test('review assessment marks asymmetric or incomplete evidence invalid instead of treating it as candidate failure', () => {
  const evidence = manifest();
  const valid = assessComparisonEvidence(evidence);
  assert.equal(valid.valid, true);

  evidence.held_constant.start_state = false;
  evidence.held_constant.valid = false;
  evidence.held_constant.mismatches = [{ field: 'start_state', distinct_values: 2 }];
  delete evidence.records[2].final_workspace;
  const invalid = assessComparisonEvidence(evidence);
  assert.equal(invalid.valid, false);
  assert.ok(invalid.reasons.some((reason) => reason.includes('start_state')));
  assert.ok(invalid.reasons.some((reason) => reason.includes('final_workspace')));
  assert.match(renderMaskedReview(evidence), /INVALID \/ INCOMPLETE/);
  assert.match(renderUnmaskedReview(evidence), /INVALID \/ INCOMPLETE/);
});

test('credential-shaped fields and configured secret values are redacted from JSON and review projections', () => {
  const secret = '__SERIES1_TEST_SECRET_VALUE__';
  const raw = {
    credential_contract: 'DEEPSEEK_API_KEY',
    api_key: secret,
    nested: { authorization: `Bearer ${secret}`, text: `provider error accidentally echoed ${secret}` }
  };
  const safe = sanitizeEvidence(raw, { secrets: [secret] });
  assert.equal(safe.credential_contract, 'DEEPSEEK_API_KEY');
  assert.equal(safe.api_key, '[REDACTED]');
  assert.equal(safe.nested.authorization, '[REDACTED]');
  assert.doesNotMatch(JSON.stringify(safe), new RegExp(secret));
  assertNoConfiguredSecrets(safe, { secrets: [secret] });
  assert.throws(() => assertNoConfiguredSecrets(raw, { secrets: [secret] }), /configured credential value/);
});
