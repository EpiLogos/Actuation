import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, statSync, symlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';

import {
  EPISTEMIC_ACCESS_KINDS,
  EPISTEMIC_RECORD_KINDS,
  EPISTEMIC_RECORD_SCHEMA,
  validateEpistemicRecord,
} from '../records.mjs';
import { listEpistemicRecords, persistEpistemicRecord, readEpistemicRecord } from '../store.mjs';

const timestamp = '2026-09-09T09:10:00Z';

function access(overrides = {}) {
  return Object.fromEntries(EPISTEMIC_ACCESS_KINDS.map((kind) => [kind, overrides[kind] ?? {
    state: 'not-assessed',
    reason: 'No provider or experiment evidence was supplied for this record.',
  }]));
}

function base(kind, index) {
  const provenances = [
    { kind: 'source', source_refs: ['source:corpus-a'] },
    { kind: 'human', actor_ref: 'person:owner' },
    { kind: 'agent', actor_ref: 'agent:researcher' },
    { kind: 'transformed', source_refs: ['source:trace-a'], derivation_refs: ['method:refraction-a'] },
    { kind: 'synthetic', generator_ref: 'generator:controlled-a' },
  ];
  return {
    schema: EPISTEMIC_RECORD_SCHEMA,
    record_kind: kind,
    record_ref: `actuation:epistemic:${index}`,
    recorded_at: timestamp,
    provenance: provenances[index % provenances.length],
    access: access(),
    subject_refs: [`subject:${index}`],
    method_refs: [],
    coordinate_refs: [],
    derivation_refs: [],
    evidence_refs: [],
  };
}

function fixtures() {
  const records = [];
  records.push({
    ...base('EpistemicCorpus', 0),
    payload: { corpus_ref: 'corpus:a', revision_ref: 'revision:1', item_refs: ['item:1'] },
  });
  records.push({
    ...base('EpistemicAnnotation', 1),
    payload: { annotation_ref: 'annotation:a', annotation_kind: 'relational', content_ref: 'content:a', source_span_refs: ['span:a'] },
  });
  records.push({
    ...base('DisclosureTrace', 2),
    payload: { trace_ref: 'trace:a', condition_ref: 'condition:control', step_refs: ['step:1'] },
  });
  records.push({
    ...base('InteriorObservation', 3),
    model_ref: 'model:open-a',
    checkpoint_ref: 'checkpoint:sha256-a',
    method_refs: ['method:hidden-state-read'],
    coordinate_refs: ['layer:4/token:7'],
    evidence_refs: ['evidence:observation-a'],
    access: access({ internal_read: { state: 'available', method_refs: ['method:hidden-state-read'], evidence_refs: ['evidence:capability-a'] } }),
    payload: { observation_ref: 'observation:a', observed_quantity: 'activation-vector', coordinate_ref: 'layer:4/token:7' },
  });
  records.push({
    ...base('InteriorIntervention', 4),
    model_ref: 'model:open-a',
    checkpoint_ref: 'checkpoint:sha256-a',
    method_refs: ['method:activation-patch'],
    coordinate_refs: ['layer:4/token:7'],
    evidence_refs: ['evidence:intervention-a'],
    access: access({ internal_write: { state: 'available', method_refs: ['method:activation-patch'], evidence_refs: ['evidence:capability-b'] } }),
    payload: { intervention_ref: 'intervention:a', operation: 'activation-patch', before_ref: 'state:before', after_ref: 'state:after' },
  });
  records.push({
    ...base('CultivationRun', 5),
    model_ref: 'model:open-a',
    checkpoint_ref: 'checkpoint:sha256-a',
    method_refs: ['method:retrieval-condition'],
    coordinate_refs: ['run:condition-a'],
    payload: { run_ref: 'run:a', input_corpus_ref: 'corpus:a', condition_ref: 'condition:retrieval', result_state_ref: 'state:declared-output' },
  });
  records.push({
    ...base('EpistemicEvaluation', 6),
    payload: { evaluation_ref: 'evaluation:a', criteria_refs: ['criterion:provenance-retention'], evaluation_state: 'declared' },
  });
  records.push({
    ...base('StructuralFinding', 7),
    payload: { finding_ref: 'finding:a', statement: 'Candidate relation awaiting empirical evidence.', finding_state: 'hypothesis' },
  });
  return records;
}

test('all eight experimental record kinds persist and round-trip through the real store', () => {
  const store = mkdtempSync(join(tmpdir(), 'actuation-epistemic-'));
  const records = fixtures();
  assert.deepEqual(records.map((record) => record.record_kind), EPISTEMIC_RECORD_KINDS);
  for (const record of records) {
    const persisted = persistEpistemicRecord(store, record);
    assert.equal(persisted.deduplicated, false);
    assert.deepEqual(readEpistemicRecord(store, record.record_ref), validateEpistemicRecord(record));
    assert.equal(statSync(persisted.pathname).mode & 0o777, 0o600);
  }
  assert.deepEqual(listEpistemicRecords(store).map((record) => record.record_kind).sort(), [...EPISTEMIC_RECORD_KINDS].sort());
});

test('same record is idempotent while conflicting reuse of identity is refused', () => {
  const store = mkdtempSync(join(tmpdir(), 'actuation-epistemic-'));
  const record = fixtures()[0];
  persistEpistemicRecord(store, record);
  assert.equal(persistEpistemicRecord(store, record).deduplicated, true);
  const conflict = structuredClone(record);
  conflict.payload.revision_ref = 'revision:2';
  assert.throws(() => persistEpistemicRecord(store, conflict), /conflicting epistemic record_ref/);
});

test('provider-neutral access declarations cannot turn absence into availability', () => {
  const record = fixtures()[0];
  record.access.internal_read = { state: 'available', method_refs: [], evidence_refs: [] };
  assert.throws(() => validateEpistemicRecord(record), /available requires method_refs and evidence_refs/);

  const unavailable = fixtures()[0];
  unavailable.access.learning = { state: 'unavailable' };
  assert.throws(() => validateEpistemicRecord(unavailable), /unavailable requires a reason/);

  const unknown = fixtures()[0];
  unknown.access.behavioural.provider_token = 'secret-shaped-value';
  assert.throws(() => validateEpistemicRecord(unknown), /provider_token is not declared/);
});

test('provenance kinds retain their distinct evidence obligations', () => {
  const transformed = fixtures()[0];
  transformed.provenance = { kind: 'transformed', source_refs: ['source:a'] };
  assert.throws(() => validateEpistemicRecord(transformed), /requires source_refs and derivation_refs/);

  const human = fixtures()[0];
  human.provenance = { kind: 'human' };
  assert.throws(() => validateEpistemicRecord(human), /human provenance requires actor_ref/);

  const synthetic = fixtures()[0];
  synthetic.provenance = { kind: 'synthetic', actor_ref: 'agent:a' };
  assert.throws(() => validateEpistemicRecord(synthetic), /synthetic provenance requires generator_ref/);
});

test('model-interior records require exact model coordinates and evidenced access', () => {
  const observation = fixtures()[3];
  observation.access.internal_read = { state: 'unavailable', reason: 'Provider exposes behavioural access only.' };
  assert.throws(() => validateEpistemicRecord(observation), /requires evidenced internal_read access/);

  const intervention = fixtures()[4];
  intervention.access.internal_write = { state: 'not-assessed', reason: 'No write probe was run.' };
  assert.throws(() => validateEpistemicRecord(intervention), /requires evidenced internal_write or causal access/);
});

test('stored corruption and record-ref path attacks fail closed', () => {
  const store = mkdtempSync(join(tmpdir(), 'actuation-epistemic-'));
  const record = fixtures()[0];
  record.record_ref = '../../outside';
  const persisted = persistEpistemicRecord(store, record);
  assert.equal(persisted.pathname.startsWith(`${store}/`), true);
  assert.equal(readFileSync(persisted.pathname, 'utf8').includes('../../outside'), true);
  writeFileSync(persisted.pathname, '{"schema":', 'utf8');
  assert.throws(() => readEpistemicRecord(store, record.record_ref), SyntaxError);
});

test('record files cannot be substituted by symlinks', () => {
  const store = mkdtempSync(join(tmpdir(), 'actuation-epistemic-'));
  const outside = join(mkdtempSync(join(tmpdir(), 'actuation-epistemic-outside-')), 'record.json');
  const record = fixtures()[0];
  writeFileSync(outside, `${JSON.stringify(record)}\n`, 'utf8');
  const target = join(store, `${encodeURIComponent(record.record_ref)}.json`);
  symlinkSync(outside, target);
  assert.throws(() => readEpistemicRecord(store, record.record_ref), /not a regular file/);
  assert.throws(() => persistEpistemicRecord(store, record), /not a regular file/);
});

test('supported and refuted findings require evidence while hypotheses remain non-claims', () => {
  const finding = fixtures()[7];
  finding.payload.finding_state = 'supported';
  assert.throws(() => validateEpistemicRecord(finding), /requires evidence_refs/);
  finding.evidence_refs = ['evidence:replication-a'];
  assert.equal(validateEpistemicRecord(finding).payload.finding_state, 'supported');
});
