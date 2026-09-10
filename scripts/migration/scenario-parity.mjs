import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { evaluateScenario, compareScenario } from './scenarios.mjs';
const argv = process.argv.slice(2);
const path = argv.shift() ?? 'fixtures/migration/scenarios.json';
const corpus = JSON.parse(readFileSync(path, 'utf8'));
assert.equal(corpus.schema, 'actuation.migration-scenarios/v1');
const delimiter = argv.indexOf('--');
const selection = delimiter < 0 ? argv : argv.slice(0, delimiter);
const rows = corpus.cases.filter(row => !selection.length || selection.includes(row.kind));
assert.ok(rows.length, 'zero selected scenarios is not conformance');
let actual;
if (delimiter >= 0) {
  const command = argv.slice(delimiter + 1);
  assert.ok(command.length, 'missing native scenario command');
  const run = spawnSync(command[0], command.slice(1), { encoding: 'utf8', timeout: 180000, maxBuffer: 64 * 1024 * 1024,
    input: rows.map(({ id, kind, input }) => JSON.stringify({ id, kind, input })).join('\n') + '\n' });
  assert.equal(run.status, 0, run.error?.message ?? run.stderr);
  actual = run.stdout.trim().split('\n').map(line => JSON.parse(line));
} else actual = rows.map(row => ({ id: row.id, value: evaluateScenario(row) }));
assert.equal(actual.length, rows.length, 'scenario results must be total and ordered');
let failed = 0;
for (let index = 0; index < rows.length; index++) {
  try { assert.equal(actual[index].id, rows[index].id); compareScenario(actual[index].value, rows[index].expected); }
  catch (error) { failed++; console.error(rows[index].id, error.message); }
}
console.log(JSON.stringify({ schema: 'actuation.scenario-parity/v1', cases: rows.length, failed, status: failed ? 'failed' : 'ok', evidence_class: 'D' }));
if (failed) process.exitCode = 1;
