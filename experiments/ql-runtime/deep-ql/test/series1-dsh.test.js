import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { HOSTS, REQUIRED_HELD_CONSTANTS, SERIES1_CAPABILITY_CONTRACT, SERIES1_SCHEMA, assertLiveManifest, stableDigest } from '../../comparison/series1/contract.mjs';
import { DSH_INSPECTION_SCHEMA, DSH_PACKAGE_VERSION, DSH_PLUGIN_TREE, DSH_PROVIDER_ROUTE, DSH_UPSTREAM_REVISION, buildDshInspectionProjection, buildIgnorableInspectionSeed, dshCompositionFingerprint } from '../../comparison/series1/dsh.mjs';
import { Series1Workspace, createLiveHost } from '../../comparison/series1/host.mjs';

test('DeepSeek Harness is a fourth Series 1 host, never a fourth loop condition', () => {
  assert.deepEqual(HOSTS, ['pi', 'pydantic-ai', 'native', 'dsh']);
  assert.equal(DSH_PROVIDER_ROUTE, 'deepseek-official');
  assert.equal(DSH_PACKAGE_VERSION, '0.1.0-rc.5');
  assert.equal(DSH_UPSTREAM_REVISION, '47f943859bef60e4160492346772ded9b24f765a');
  assert.ok(DSH_PLUGIN_TREE.includes('@deepseek-ai/dsh-session-persistence-jsonl'));
});

test('DSH composition fingerprint is stable and condition/run independent', () => {
  const first = dshCompositionFingerprint();
  const second = dshCompositionFingerprint();
  assert.equal(first, second);
  assert.equal(first.length, 64);
  assert.notEqual(first, stableDigest({ condition: 'classic' }));
});

test('DSH inspection projection is read-only evidence derived from portable chronology', () => {
  const events = [
    { record_index: 0, channel: 'host', event_type: 'model_requested', payload: { provider: 'dsh-deepseek-official' } },
    { record_index: 1, channel: 'runtime-semantic', event_type: 'ql_position_entered', ql: { position: 'P5' }, payload: { position: 'P5' } },
    { record_index: 2, channel: 'runtime-semantic', event_type: 'conjugate_started', payload: { depth: 1 } },
    { record_index: 3, channel: 'host', event_type: 'model_returned', payload: { provider: 'dsh-deepseek-official' } }
  ];
  const projection = buildDshInspectionProjection(events, {
    runId: 'run-x',
    condition: 'ql-deep',
    modelCalls: [{ ordinal: 0, dsh_seq_start: 0, dsh_seq_end: 8, portable_model_requested_record_index: 0, portable_model_returned_record_index: 3 }]
  });
  assert.equal(projection.schema, DSH_INSPECTION_SCHEMA);
  assert.equal(projection.read_only, true);
  assert.equal(projection.candidate_context_authority, false);
  assert.equal(projection.portable_events.length, events.length);
  assert.deepEqual(projection.ql.positions, ['P5']);
  assert.equal(projection.ql.operators.length, 1);
  assert.equal('send' in projection, false);
  assert.equal('inject' in projection, false);
  assert.equal('followup' in projection, false);
  assert.equal('steer' in projection, false);

  const seed = buildIgnorableInspectionSeed(projection, { time: 0 });
  assert.equal(seed.length, events.length);
  assert.ok(seed.every((event, index) => event.type === 'series1/portable-event' && event.seq === index && event.ignorable === true));
});

test('DSH Web UI contribution registers only read-model and renderer surfaces', async () => {
  const source = await fs.readFile(fileURLToPath(new URL('../../comparison/series1/dsh-ui-client.tsx', import.meta.url)), 'utf8');
  assert.match(source, /ctx\.conversationEvents\.register\(series1QLInspectionDefinition\)/);
  assert.match(source, /ctx\.slots\.inject\('conversation\.chat\.node'/);
  assert.doesNotMatch(source, /ctx\.(?:agents|sessions)\./);
  assert.doesNotMatch(source, /\.append\s*\(/);
  assert.doesNotMatch(source, /\.callModel\s*\(/);
  assert.doesNotMatch(source, /\.executeCapability\s*\(/);
  assert.doesNotMatch(source, /\.(?:followup|steer|send)\s*\(/);
});

test('DSH host reuses the exact portable candidate capability contract and carries target revision/composition', () => {
  const workspace = new Series1Workspace(process.cwd());
  assert.equal(stableDigest(workspace.describe()), stableDigest(SERIES1_CAPABILITY_CONTRACT));
  const provider = { id: 'dsh-deepseek-official', compositionFingerprint: dshCompositionFingerprint(), complete: async () => ({ content: '', capabilityCalls: [] }) };
  const host = createLiveHost({ hostId: 'dsh', provider, workspace });
  assert.equal(host.id, 'dsh');
  assert.equal(host.compositionFingerprint, provider.compositionFingerprint);
  assert.match(host.revision, new RegExp(DSH_UPSTREAM_REVISION));
  assert.match(host.realFrameworkPath, /programmatic Cordis composition/);
});

test('DSH live evidence is rejected when its composition fingerprint is absent', () => {
  const held = Object.fromEntries(REQUIRED_HELD_CONSTANTS.map((field) => [field, true]));
  held.valid = true;
  held.mismatches = [];
  const base = {
    schema: SERIES1_SCHEMA,
    provider_mode: 'live',
    fixture_provider: false,
    host: { id: 'dsh', revision: `deepseek-ai/deepseek-harness@${DSH_UPSTREAM_REVISION}`, real_framework_path: 'real DSH' },
    model: { provider: 'deepseek', id: 'deepseek-v4-flash', parameters: { temperature: 0 } },
    conditions: ['classic', 'ql-direct', 'ql-deep'],
    held_constant: held,
    determination: 'pending-human-review',
    review: { prompt: 'frozen prompt', focus: ['inspect'] },
    records: [{ benchmark_revision: 'b', task_revision: 't', runner_revision: 'r', review_contract_revision: 'v', host_revision: 'h', host: { id: 'dsh' } }]
  };
  assert.throws(() => assertLiveManifest(base), /host_composition_fingerprint is required for dsh/);
  base.records[0].host_composition_fingerprint = dshCompositionFingerprint();
  assert.doesNotThrow(() => assertLiveManifest(base));
});
