import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { evaluateScenario, compareScenario } from './scenarios.mjs';
import { capabilityDescriptorBySlug } from '../../detection/catalog.mjs';
const corpus = JSON.parse(readFileSync(process.argv[2], 'utf8'));
const destination = process.argv[3];
if (!destination) throw new Error('usage: capture-scenarios.mjs <oracle.json> <new-scenarios.json>');
const seed = operation => structuredClone(corpus.cases.find(row => row.operation === operation && row.expected.ok)?.args ?? (() => { throw new Error(`missing seed ${operation}`); })());
const rows = [];
function add(id, kind, input, assertion) {
  const scenario = { id, kind, input };
  const expected = evaluateScenario(scenario);
  assertion?.(expected);
  compareScenario(evaluateScenario(scenario), expected);
  rows.push({ ...scenario, expected });
}
add('catalog-all-native-kinds', 'catalog', {});
const identity = { stream_ref: "stream:oracle/雪 !'()*", actuation_ref: 'actuation:oracle', agency_ref: 'agency:oracle', agent_session_ref: 'session:oracle', world_binding_ref: 'binding:oracle', provenance: ['source:oracle'] };
const started_at = '2026-09-10T00:00:00.000Z';
const ended_at = '2026-09-10T00:01:00.000Z';
const capability = capabilityDescriptorBySlug('zcode');
const native_event = capability.native_events.find(e => e.event !== 'custom').native_name;
const custom = capability.native_events.find(e => e.event === 'custom').native_name;
const open = { label: 'open', operation: 'openDurableStream', args: { ...identity, started_at }, expect_ok: true };
const occurrence = { stream_ref: identity.stream_ref, harness: 'zcode', native_event, event_ref: 'event:oracle:1', observed_at: started_at, content: 'visible boundary', native_trace_ref: 'native:opaque:1', metadata: { caller_standing: 'unknown' } };
const record = { label: 'record', operation: 'recordBoundaryOccurrence', args: occurrence, expect_ok: true, append_only: true };
const close = { label: 'close', operation: 'closeDurableStream', args: { stream_ref: identity.stream_ref, state: 'closed', ended_at }, expect_ok: true };
const usage = seed('contracts/model-usage.mjs#validateModelUsageObservation')[0];
usage.actuation_ref = identity.actuation_ref;
usage.correlation = { ...usage.correlation, agency_ref: identity.agency_ref, agent_session_ref: identity.agent_session_ref };
const recordUsage = { label: 'usage', operation: 'recordModelUsageObservation', args: { stream_ref: identity.stream_ref, event_ref: 'event:usage:1', observation: usage }, expect_ok: true, append_only: true };
const actions = [open,
  { ...open, label: 'idempotent-open', unchanged: true },
  { ...open, label: 'identity-conflict', args: { ...open.args, agency_ref: 'agency:imposter' }, expect_ok: false, unchanged: true },
  record,
  { ...record, label: 'duplicate-event', expect_ok: false, unchanged: true, append_only: false },
  { ...record, label: 'unknown-native-event', args: { ...occurrence, event_ref: 'event:2', native_event: 'invented-event' }, expect_ok: false, unchanged: true, append_only: false },
  { ...record, label: 'declared-custom', args: { ...occurrence, event_ref: 'event:custom', native_event: custom } },
  recordUsage,
  { ...recordUsage, label: 'usage-idempotence', unchanged: true, append_only: false },
  { ...recordUsage, label: 'usage-conflict', args: { ...recordUsage.args, observation: { ...usage, observed_at: ended_at } }, expect_ok: false, unchanged: true, append_only: false },
  { label: 'cursor', operation: 'replayDurableStream', args: { stream_ref: identity.stream_ref, afterSequence: 1, limit: 1 }, expect_ok: true, unchanged: true },
  close,
  { ...record, label: 'closed-append-refusal', args: { ...occurrence, event_ref: 'event:after-close' }, expect_ok: false, unchanged: true, append_only: false },
  { ...close, label: 'closed-close-refusal', expect_ok: false, unchanged: true },
  { ...recordUsage, label: 'closed-dedup-no-reopening', unchanged: true, append_only: false },
  { label: 'reopen-read-only', operation: 'openDurableStream', args: identity, expect_ok: true, unchanged: true },
];
add('durable-identity-cursor-usage-lifecycle', 'store', { actions });
for (const state of ['interrupted', 'cancelled']) add(`durable-${state}`, 'store', { actions: [open, record, { ...close, args: { ...close.args, state } }] });
add('durable-implicit-Direct-open', 'store', { actions: [{ ...record, args: { ...occurrence, identity }, append_only: false }] });
const durable = rows.find(r => r.id === 'durable-identity-cursor-usage-lifecycle');
const populated = Object.values(durable.expected[6].files)[0];
for (const [name, raw, ok] of [
  ['valid-open', populated, true], ['valid-terminal', Object.values(durable.expected[11].files)[0], true],
  ['empty', '', false], ['invalid-header', 'garbled\n', false],
  ['torn-tail', populated + '{"event_ref":', false], ['interior-hole', populated.replace('\n', '\n\n'), false],
  ['gap', populated.replace('"sequence":1', '"sequence":5'), false],
  ['duplicate-ref', populated.replace('event:custom', 'event:oracle:1'), false],
  ['derived-header-cursor', populated.replace('"last_sequence":0,"next_sequence":1', '"last_sequence":500,"next_sequence":900'), true],
]) add(`fold-${name}`, 'fold', { raw }, actual => assert.equal(actual.ok, ok, name));
for (const ref of [identity.stream_ref, '../escape', '/', 'a:b', 'a%20b', '', null, 3]) add(`filename-${JSON.stringify(ref)}`, 'filename', { ref });
const spec = extra => ({ schema: 'actuation.harness-detection/v1', document: 'descriptor', slug: 'fixture', native_kind: 'harness', probe: { executable: { names: ['fixture'] } }, provenance: { authored_by: 'R1 synthetic fixture', catalog_revision: 1 }, ...extra });
const absent = { now: started_at, descriptors: [spec({})] };
add('probe-absent-not-unavailable', 'probe', absent, x => assert.equal(x.result.value.harnesses[0].state, 'not-installed'));
add('probe-errors-not-absence', 'probe', { ...absent, effects: { resolveExecutable: { value: { error: 'permission denied' } } } }, x => assert.equal(x.result.value.harnesses[0].state, 'unavailable'));
const presentEffects = { resolveExecutable: { value: { found: true, path: '/fixture/harness' } }, statProbe: { value: { exists: true, isDir: false, size: 4, mtimeMs: 123.6 } }, hashProbe: { value: { ok: true, sha256: 'a'.repeat(64) } }, versionProbe: { value: { ok: true, version: 'fixture 1.0.0' } } };
add('probe-present-no-version-unrequested', 'probe', { ...absent, effects: presentEffects }, x => assert.ok(!x.calls.some(c => c.effect === 'versionProbe')));
add('probe-version-explicit', 'probe', { ...absent, effects: presentEffects, versions: true }, x => assert.ok(x.calls.some(c => c.effect === 'versionProbe')));
add('probe-config-only', 'probe', { ...absent, descriptors: [spec({ probe: { 'config-dir': { path: '~/.fixture' } } })], effects: { statProbe: { value: { exists: true, isDir: true } } }, versions: true }, x => { assert.equal(x.result.value.harnesses[0].receipts.executable_is, 'config-dir'); assert.ok(!x.calls.some(c => c.effect === 'versionProbe')); });
add('probe-all-catalogue-absent', 'probe', { now: started_at }, x => assert.equal(x.result.value.harnesses.length, 12));
const markers = ['CLAUDECODE', 'CODEX_CI'];
for (const count of [0, 1, 2]) add(`self-${count}-identity-matches`, 'probe', { now: started_at, mode: 'self', descriptors: markers.map((name, i) => spec({ slug: `fixture-${i}`, probe: { env: { any_of: [name] } } })), effects: { envProbe: { by_argument: Object.fromEntries(markers.map((name, i) => [JSON.stringify([name]), { ok: true, matched: i < count ? { [name]: 'redacted' } : {} }])) } } }, x => { assert.equal(x.result.value.ambiguity, count > 1); assert.equal(x.result.value.resolved !== null, count === 1); });
for (const kind of ['absent', 'unavailable', 'fingerprint']) {
  const effects = kind === 'absent' ? {} : kind === 'unavailable' ? Object.fromEntries(['env', 'file-pattern', 'vault-item', 'cli-presence'].map(name => [name, { value: { ok: false, reason: 'permission denied' } }])) : { env: { value: { ok: true, matched: { UNDECLARED_TEST_TOKEN: { fingerprint_sha256: 'b'.repeat(64), byte_length: 13 } } } } };
  add(`secrets-${kind}`, 'secret', { effects }, x => assert.ok(!JSON.stringify(x.value).includes('actual-secret-value')));
}
for (const [name, route, operation] of [
  ['agency', ['agency'], 'contracts/agency.mjs#agencyReadModel'],
  ['actualise', ['agency', 'actualise'], 'contracts/agency-actualisation.mjs#actualiseAgency'],
  ['realised', ['realised'], 'contracts/realised-actuation.mjs#realisedActuationReadModel'],
  ['stream', ['stream'], 'contracts/actuation-stream.mjs#actuationStreamReadModel'],
  ['activity', ['activity'], 'contracts/activity.mjs#validateActivity'],
  ['usage', ['usage'], 'contracts/model-usage.mjs#validateModelUsageObservation'],
  ['instantiation', ['instantiation'], 'contracts/instantiation.mjs#instantiationReceipt'],
]) {
  const input = seed(operation)[0];
  for (const json of [true, false]) add(`cli-${name}-${json ? 'json' : 'human'}`, 'cli', { argv: [...route, '-', ...(json ? ['--json'] : [])], stdin: JSON.stringify(input) }, x => assert.equal(x.code, 0));
  add(`cli-${name}-refusal`, 'cli', { argv: [...route, '-', '--json'], stdin: '{}' }, x => assert.equal(x.code, 2));
}
for (const argv of [['--version'], ['capabilities', '--json'], ['contract', 'list', '--json'], ['harness', 'catalog', '--json'], ['harness', 'capability', '--json'], ['harness', 'capability', 'claude-code', '--json'], ['harness', 'capability', 'undeclared', '--json'], ['harness', 'detect', '--only', 'undeclared', '--json'], ['harness', 'invented'], ['invented'], ['stream', 'replay'], ['stream', 'replay', identity.stream_ref, '--after', '-1', '--store', '$STORE', '--json']]) add(`cli-${argv.join(' ')}`, 'cli', { argv });
writeFileSync(destination, JSON.stringify({ schema: 'actuation.migration-scenarios/v1', source_revision: corpus.source_revision, evidence_class: 'D', cases: rows }, null, 2) + '\n', { flag: 'wx' });
console.log(`Captured ${rows.length} effect/store/CLI scenarios; ${createHash('sha256').update(JSON.stringify(rows)).digest('hex')}`);
