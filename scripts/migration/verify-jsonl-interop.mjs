// Fresh files cross actual Node/Rust process boundaries. Frozen examples are
// inputs, never replacement answers. No provider or owner-machine acceptance.
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync, mkdtempSync, rmSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import * as nodeStore from '../../contracts/actuation-stream-store.mjs';
import { validateActuationStream } from '../../contracts/actuation-stream.mjs';
const binary = resolve(process.argv[2] ?? 'target/debug/examples/store-oracle');
const corpus = JSON.parse(readFileSync(new URL('../../fixtures/migration/scenarios.json', import.meta.url)));
const scenario = corpus.cases.find(row => row.id === 'durable-identity-cursor-usage-lifecycle');
const args = label => structuredClone(scenario.input.actions.find(row => row.label === label).args);
let checks = 0, request = 0;
function same(actual, expected, message) { assert.deepEqual(actual, expected, message); checks++; }
function native(root, operation, args, ok = true) {
  const id = ++request;
  const process = spawnSync(binary, [], {
    input: JSON.stringify({ id, kind: 'command', input: { root, operation, args } }) + '\n',
    encoding: 'utf8', timeout: 15000, maxBuffer: 16 * 1024 * 1024,
  });
  assert.equal(process.status, 0, `Rust test transport failed: ${process.error ?? process.stderr}`);
  const result = JSON.parse(process.stdout);
  assert.equal(result.id, id);
  assert.equal(result.value.ok, ok, `${operation}: ${JSON.stringify(result.value)}`);
  checks++;
  return result.value.value;
}
const roots = [];
function fresh() { const root = mkdtempSync(join(tmpdir(), 'actuation-cross-reader-')); roots.push(root); return root; }
try {
  // Node owns the first write; Rust must fold it, extend it, and keep all facts.
  const root = fresh(), opening = args('open'), ref = opening.stream_ref;
  const path = join(root, nodeStore.streamFileName(ref));
  nodeStore.openDurableStream({ root, ...opening });
  nodeStore.recordBoundaryOccurrence({ root, ...args('record') });
  same(native(root, 'loadDurableStream', { stream_ref: ref }), nodeStore.loadDurableStream({ root, stream_ref: ref }), 'Node write -> Rust read');
  const beforeRust = readFileSync(path, 'utf8');
  native(root, 'recordModelUsageObservation', args('usage'));
  assert.ok(readFileSync(path, 'utf8').startsWith(beforeRust)); checks++;
  same(native(root, 'loadDurableStream', { stream_ref: ref }), validateActuationStream(nodeStore.loadDurableStream({ root, stream_ref: ref })), 'Rust append -> Node validates');
  const beforeNode = readFileSync(path, 'utf8');
  nodeStore.recordBoundaryOccurrence({ root, ...args('declared-custom') });
  assert.ok(readFileSync(path, 'utf8').startsWith(beforeNode)); checks++;
  same(native(root, 'loadDurableStream', { stream_ref: ref }), nodeStore.loadDurableStream({ root, stream_ref: ref }), 'Node extends Rust-written evidence');
  for (const page of [{afterSequence:0,limit:0},{afterSequence:1,limit:1},{afterSequence:99},{}]) {
    same(native(root, 'replayDurableStream', {stream_ref:ref,...page}), nodeStore.replayDurableStream({root,stream_ref:ref,...page}), 'cursor replay is portable');
  }
  const tail = readFileSync(path,'utf8').split('\n').slice(1).join('\n');
  native(root, 'closeDurableStream', {stream_ref:ref,state:'closed',ended_at:'2026-09-10T10:00:00Z'});
  same(readFileSync(path,'utf8').split('\n').slice(1).join('\n'), tail, 'close changes header only');
  same(nodeStore.loadDurableStream({root,stream_ref:ref}).lifecycle.state, 'closed');
  same(native(root,'loadDurableStream',{stream_ref:ref}),nodeStore.loadDurableStream({root,stream_ref:ref}));
  const closed = readFileSync(path,'utf8');
  native(root, 'recordModelUsageObservation', args('usage')); // same exact fact: idempotent, not resumed
  assert.throws(() => nodeStore.recordBoundaryOccurrence({root,...args('record'),event_ref:'event:after-close'})); checks++;
  native(root,'recordBoundaryOccurrence',{...args('record'),event_ref:'event:after-close'},false);
  same(readFileSync(path,'utf8'),closed,'terminal refusal and dedup never rewrite');

  // Rust owns the first write; Node must consume ordinary JSONL with no migration.
  const rustRoot = fresh();
  native(rustRoot,'openDurableStream',opening);
  native(rustRoot,'recordBoundaryOccurrence',args('record'));
  same(native(rustRoot,'loadDurableStream',{stream_ref:ref}),nodeStore.loadDurableStream({root:rustRoot,stream_ref:ref}),'Rust creates -> Node reads');
  nodeStore.recordModelUsageObservation({root:rustRoot,...args('usage')});
  nodeStore.closeDurableStream({root:rustRoot,stream_ref:ref,state:'cancelled',ended_at:'2026-09-10T10:00:00Z'});
  same(native(rustRoot,'loadDurableStream',{stream_ref:ref}),nodeStore.loadDurableStream({root:rustRoot,stream_ref:ref}),'Node extends/closes Rust-created store');

  // Every persisted snapshot originally captured by R1 is independently readable
  // by both implementations, including lifecycle transitions and usage records.
  const unique = new Set();
  for (const row of corpus.cases.filter(row => row.kind === 'store')) {
    for (const step of row.expected) for (const raw of Object.values(step.files)) unique.add(raw);
  }
  const fixtureRoot = fresh();
  for (const raw of unique) {
    const expected = nodeStore.foldStreamFile(raw);
    writeFileSync(join(fixtureRoot,nodeStore.streamFileName(expected.stream_ref)),raw);
    same(native(fixtureRoot,'loadDurableStream',{stream_ref:expected.stream_ref}),expected,'R1 Node-created snapshot');
  }

  // Invalid state is a visible refusal, never a tail silently dropped by either reader.
  const corruptRoot = fresh();
  const openRaw = JSON.stringify({schema:'actuation.stream/v1',...opening,lifecycle:{state:'open'},cursor:{last_sequence:0,next_sequence:1},events:[]})+'\n';
  for (const suffix of ['{"event_ref":', '\n', '{"event_ref":"event:gap","kind":"world-observation","sequence":2}\n']) {
    const p = join(corruptRoot,nodeStore.streamFileName(ref)), raw=openRaw+suffix;
    writeFileSync(p,raw);
    assert.throws(()=>nodeStore.loadDurableStream({root:corruptRoot,stream_ref:ref})); checks++;
    native(corruptRoot,'loadDurableStream',{stream_ref:ref},false);
    same(readFileSync(p,'utf8'),raw,'refusal preserves evidence bytes');
  }
  for(const dir of [root,rustRoot,corruptRoot]) assert.ok(readdirSync(dir).every(name=>name.endsWith('.jsonl')));
  console.log(JSON.stringify({schema:'actuation.jsonl-interop/v1',phase:'R4',checks,frozen_snapshots:unique.size,rust_processes:request,status:'ok',evidence_class:'D',scope:'fresh serial Node/Rust cross-reader conformance; no concurrent mixed writers, P/M/H'}));
} finally { for (const root of roots) rmSync(root,{recursive:true,force:true}); }
