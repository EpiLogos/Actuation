import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { BASE_REVISION } from './source-lock.mjs';
const oracle = JSON.parse(readFileSync('fixtures/migration/oracle.json', 'utf8'));
const scenarios = JSON.parse(readFileSync('fixtures/migration/scenarios.json', 'utf8'));
const ledger = JSON.parse(readFileSync('docs/rust-refoundation/ledger.json', 'utf8'));
assert.equal(oracle.source_revision, BASE_REVISION);
assert.equal(scenarios.source_revision, BASE_REVISION);
assert.equal(ledger.source_revision, BASE_REVISION);
assert.ok(oracle.cases.length >= 500, 'missing substantive public oracle');
assert.ok(Object.keys(oracle.coverage).length >= 50, 'public operation coverage shrank');
assert.ok(scenarios.cases.length >= 60, 'missing effect/store/CLI oracle');
assert.ok(ledger.entries.length >= 200, 'missing exact whole-tree ledger');
for (const corpus of [oracle, scenarios]) assert.equal(new Set(corpus.cases.map(row => row.id)).size, corpus.cases.length, 'duplicate case identity');
for (const prefix of ['contracts/agency.mjs#', 'contracts/agency-actualisation.mjs#', 'contracts/realised-actuation.mjs#', 'contracts/actuation-stream.mjs#', 'contracts/activity.mjs#', 'contracts/model-usage.mjs#', 'contracts/instantiation.mjs#', 'contracts/harness-detection.mjs#', 'contracts/harness-capability.mjs#', 'contracts/request-correlation.mjs#', 'contracts/secret-detection.mjs#', 'experiments/epistemic-cultivation/', 'experiments/ql-runtime/prime/']) assert.ok(oracle.cases.some(row => row.operation.startsWith(prefix)), `unrepresented family ${prefix}`);
for (const kind of ['catalog', 'probe', 'secret', 'fold', 'filename', 'store', 'cli']) assert.ok(scenarios.cases.some(row => row.kind === kind), `unrepresented scenario kind ${kind}`);
for (const row of ledger.entries) {
  assert.match(row.source_blob, /^[a-f0-9]{40}$/);
  assert.ok(row.target && row.reason && row.phase && row.disposition);
  if (row.path.startsWith('experiments/')) assert.ok(['A', 'B', 'C', 'D'].includes(row.classification), `unclassified experiment ${row.path}`);
}
for (const line of readFileSync('fixtures/migration/SHA256SUMS', 'utf8').trim().split('\n')) {
  const [expected, path] = line.split(/\s+/);
  assert.equal(createHash('sha256').update(readFileSync(path)).digest('hex'), expected, `frozen file changed: ${path}`);
}
console.log(JSON.stringify({ schema: 'actuation.corpus-integrity/v1', source_revision: BASE_REVISION, pure_cases: oracle.cases.length, scenario_cases: scenarios.cases.length, source_files: ledger.entries.length, evidence_class: 'D', status: 'ok' }));
