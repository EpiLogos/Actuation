// JSONL is a temporary conformance transport, not a new public product API.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
const root = new URL('../../', import.meta.url);
const argv = process.argv.slice(2);
function option(name, fallback) {
  const index = argv.indexOf(name);
  if (index < 0) return fallback;
  const value = argv[index + 1];
  if (!value || value.startsWith('--')) throw new Error(`${name} requires a value`);
  argv.splice(index, 2);
  return value;
}
const path = option('--corpus', fileURLToPath(new URL('fixtures/migration/oracle.json', root)));
const phase = option('--phase', 'all');
const corpus = JSON.parse(readFileSync(path, 'utf8'));
assert.equal(corpus.schema, 'actuation.migration-oracle/v1');
const phases = {
  R2: ['contracts/agency.mjs'],
  R3: ['contracts/agency-actualisation.mjs', 'contracts/realised-actuation.mjs'],
  R4: ['contracts/actuation-stream.mjs', 'contracts/activity.mjs', 'contracts/model-usage.mjs', 'contracts/request-correlation.mjs'],
  R5: ['contracts/harness-capability.mjs', 'contracts/harness-detection.mjs', 'contracts/instantiation.mjs', 'contracts/secret-detection.mjs'],
  R6: ['experiments/'],
};
if (phase !== 'all' && !(phase in phases)) throw new Error(`unknown phase ${phase}`);
const rows = corpus.cases.filter(row => phase === 'all' || phases[phase].some(prefix => row.operation.startsWith(prefix)));
assert.ok(rows.length, `zero ${phase} cases is not conformance`);
let results;
if (argv[0] === '--') {
  const command = argv.slice(1);
  if (!command.length) throw new Error('missing native oracle command');
  const run = spawnSync(command[0], command.slice(1), {
    cwd: fileURLToPath(root), encoding: 'utf8', timeout: 120000, maxBuffer: 64 * 1024 * 1024,
    input: rows.map(({ id, operation, args }) => JSON.stringify({ id, operation, args })).join('\n') + '\n',
  });
  assert.equal(run.status, 0, `native oracle process failed: ${run.error ?? run.stderr}`);
  results = run.stdout.trim().split('\n').map(line => JSON.parse(line));
} else {
  assert.equal(argv.length, 0, `unexpected argument ${argv[0]}`);
  results = [];
  for (const row of rows) {
    const [path, name] = row.operation.split('#');
    if (!/^(contracts|experiments)\/[\w./-]+\.mjs$/.test(path) || path.includes('..') || !/^\w+$/.test(name)) throw new Error(`invalid oracle operation ${row.operation}`);
    const module = await import(new URL(path, root));
    if (typeof module[name] !== 'function') throw new Error(`missing oracle operation ${row.operation}`);
    try { results.push({ id: row.id, ok: true, value: JSON.parse(JSON.stringify(await module[name](...structuredClone(row.args)))) }); }
    catch (error) { results.push({ id: row.id, ok: false, error: { name: error.name, message: error.message } }); }
  }
}
assert.equal(results.length, rows.length, 'oracle must answer every case exactly once');
const failures = [];
for (let index = 0; index < rows.length; index++) {
  const row = rows[index];
  const actual = results[index];
  try {
    assert.equal(actual.id, row.id, 'response identity/order drift');
    assert.equal(actual.ok, row.expected.ok, 'accept/refuse drift');
    if (actual.ok) assert.deepEqual(actual.value, row.expected.value, 'semantic JSON drift');
  } catch (error) { failures.push({ operation: row.operation, id: row.id, error: error.message, expected: row.expected, actual }); }
}
if (failures.length) {
  console.error(JSON.stringify(failures, null, 2));
  process.exitCode = 1;
}
console.log(JSON.stringify({ schema: 'actuation.migration-parity/v1', phase, cases: rows.length, failed: failures.length, status: failures.length ? 'failed' : 'ok', evidence_class: 'D' }));
