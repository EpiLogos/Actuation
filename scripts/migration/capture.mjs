// Reproducible R1 discovery tool, not the authority of the frozen expectation.
// The committed corpus is authoritative. Recapturing never silently replaces it.
import { mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { lockOriginalSources } from './source-lock.mjs';
const root = fileURLToPath(new URL('../../', import.meta.url));
const destination = process.argv[2];
if (!destination) throw new Error('usage: node scripts/migration/capture.mjs <new-corpus.json>');
lockOriginalSources(root);
const canonical = value => Array.isArray(value) ? value.map(canonical) : value && typeof value === 'object' ? Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])])) : value;
const digest = value => createHash('sha256').update(JSON.stringify(canonical(value))).digest('hex');
const directory = mkdtempSync(join(tmpdir(), 'actuation-oracle-'));
try {
  const tests = ['contracts', 'detection', 'cli'].flatMap(dir => readdirSync(join(root, dir)).filter(name => name.endsWith('.test.mjs')).sort().map(name => `${dir}/${name}`));
  tests.push('experiments/ql-runtime/prime/test/prime-structural.test.mjs');
  const run = spawnSync(process.execPath, ['--import', './scripts/migration/capture-register.mjs', '--test', ...tests], {
    cwd: root, encoding: 'utf8', maxBuffer: 32 * 1024 * 1024,
    env: { ...process.env, ACTUATION_ORACLE_CAPTURE_DIR: directory },
  });
  process.stdout.write(run.stdout ?? '');
  process.stderr.write(run.stderr ?? '');
  if (run.status !== 0) throw new Error(`instrumented oracle tests failed (${run.status})`);
  const capturedRows = () => readdirSync(directory).filter(file => file.endsWith('.jsonl')).sort().flatMap(file => readFileSync(join(directory, file), 'utf8').trim().split('\n').filter(Boolean).map(line => JSON.parse(line)));
  const seeds = join(directory, 'seeds.json');
  writeFileSync(seeds, JSON.stringify(capturedRows()));
  const extra = spawnSync(process.execPath, ['--import', './scripts/migration/capture-register.mjs', './scripts/migration/extra-cases.mjs', seeds], { cwd: root, encoding: 'utf8', env: { ...process.env, ACTUATION_ORACLE_CAPTURE_DIR: directory } });
  process.stdout.write(extra.stdout ?? '');
  process.stderr.write(extra.stderr ?? '');
  if (extra.status !== 0) throw new Error(`explicit adversarial corpus failed (${extra.status})`);
  const rows = capturedRows();
  const unique = new Map(rows.map(row => [digest(row), row]));
  const cases = [...unique.entries()].map(([id, row]) => ({ id, ...row })).sort((a, b) => a.operation.localeCompare(b.operation) || a.id.localeCompare(b.id));
  const coverage = {};
  for (const row of cases) {
    coverage[row.operation] ??= { accepted: 0, rejected: 0 };
    coverage[row.operation][row.expected.ok ? 'accepted' : 'rejected']++;
  }
  const sources = Object.fromEntries([...new Set(cases.map(row => row.operation.split('#')[0]))].sort().map(path => [path, createHash('sha256').update(readFileSync(join(root, path))).digest('hex')]));
  const corpus = {
    schema: 'actuation.migration-oracle/v1',
    source_revision: '1c862c6bf58478adf6842a090214906dd2337001',
    evidence_class: 'D',
    capture: { node: process.version, tests, source_sha256: sources },
    comparison: { success: 'semantic-json', failure: 'rejection; diagnostic retained; CLI exit/status separately frozen' },
    coverage, cases,
  };
  if (cases.length === 0) throw new Error('refusing an empty oracle');
  mkdirSync(new URL('../../fixtures/migration/', import.meta.url), { recursive: true });
  writeFileSync(destination, JSON.stringify(corpus, null, 2) + '\n', { flag: 'wx' });
  console.log(`Captured ${cases.length} immutable JSON cases across ${Object.keys(coverage).length} operations; ${digest(cases)}`);
} finally { rmSync(directory, { recursive: true, force: true }); }
