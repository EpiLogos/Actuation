#!/usr/bin/env node
// General-sweep driver: 12 entries x 7 calls (full_text x2, nameonly x1,
// resonant x1, being/becoming/knowing x1 each) = 84 calls. 200ms between
// calls. Fail-closed: non-zero exit or invalid JSON -> retry once, then
// write a .failed.json record and move on. Resumable: existing valid output
// files are skipped.
import { spawnSync } from 'node:child_process';
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { join } from 'node:path';

const DIR = '/Users/admin/.cache/actuation/analysis/general-sweep';
const CORPUS = '/Users/admin/Central/Work/Actuation/experiments/ql-runtime/comparison/series1/corpus/mef-general-corpus.md';
const INSTRUMENT = '/Users/admin/.cache/actuation/jev-reflect.mjs';
const NAMEONLY = join(DIR, 'jev-reflect-nameonly.mjs');
const DELAY_MS = 200;
const TIMEOUT_MS = 300000;

function parseCorpus() {
  const text = readFileSync(CORPUS, 'utf8');
  const lines = text.split('\n');
  const entries = [];
  let cur = null;
  for (const line of lines) {
    const m = line.match(/^## C(\d+)\. (.+)$/);
    if (m) {
      if (cur) entries.push(cur);
      cur = { id: `C${m[1]}`, nn: m[1].padStart(2, '0'), title: m[2], lines: [] };
    } else if (cur) {
      cur.lines.push(line);
    }
  }
  if (cur) entries.push(cur);
  return entries.map(({ id, nn, title, lines: ls }) => ({
    id, nn, title,
    subject: ls.join('\n').replace(/^\n+/, '').replace(/\s+$/, ''),
  }));
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

function runCall(script, mode, subject) {
  const input = JSON.stringify({ mode, subject });
  const t0 = Date.now();
  const r = spawnSync('node', [script], { input, encoding: 'utf8', timeout: TIMEOUT_MS });
  const wall_ms = Date.now() - t0;
  if (r.error) return { ok: false, wall_ms, error: `spawn: ${r.error.message}${r.signal ? ` signal=${r.signal}` : ''}` };
  if (r.status !== 0) return { ok: false, wall_ms, error: `exit ${r.status}: ${(r.stderr || '').slice(0, 400)}` };
  try {
    const parsed = JSON.parse(r.stdout);
    if (!parsed || parsed.mode !== mode || !parsed.reading) throw new Error('missing reading');
    return { ok: true, wall_ms, out: parsed };
  } catch (e) {
    return { ok: false, wall_ms, error: `invalid JSON: ${e.message}; stdout head: ${r.stdout.slice(0, 200)}` };
  }
}

const entries = parseCorpus();
writeFileSync(join(DIR, 'corpus-meta.json'), JSON.stringify(entries.map(({ id, nn, title, subject }) => ({ id, nn, title, subject })), null, 2));

const jobs = [];
for (const e of entries) {
  jobs.push({ e, mode: 'full_text', script: INSTRUMENT, file: `c${e.nn}-full_text-1.json` });
  jobs.push({ e, mode: 'full_text', script: INSTRUMENT, file: `c${e.nn}-full_text-2.json` });
  jobs.push({ e, mode: 'full_text', script: NAMEONLY, file: `c${e.nn}-nameonly-1.json` });
  for (const mode of ['resonant', 'being', 'becoming', 'knowing']) {
    jobs.push({ e, mode, script: INSTRUMENT, file: `c${e.nn}-${mode}-1.json` });
  }
}

const log = [];
let done = 0, failed = 0, skipped = 0;
for (const job of jobs) {
  const outPath = join(DIR, job.file);
  if (existsSync(outPath)) {
    try { JSON.parse(readFileSync(outPath, 'utf8')); skipped++; console.log(`SKIP ${job.file}`); continue; } catch { /* stale, rerun */ }
  }
  await sleep(DELAY_MS);
  let result = runCall(job.script, job.mode, job.e.subject);
  if (!result.ok) {
    console.log(`RETRY ${job.file}: ${result.error}`);
    await sleep(DELAY_MS);
    result = runCall(job.script, job.mode, job.e.subject);
  }
  if (result.ok) {
    writeFileSync(outPath, JSON.stringify({ entry: job.e.id, title: job.e.title, subject_chars: result.out.subject_chars, mode: job.mode, reading: result.out.reading, latency_ms: result.out.latency_ms, wall_ms: result.wall_ms }, null, 2));
    done++;
    console.log(`OK   ${job.file} (${result.wall_ms}ms)`);
  } else {
    failed++;
    writeFileSync(outPath.replace(/\.json$/, '.failed.json'), JSON.stringify({ entry: job.e.id, mode: job.mode, error: result.error, wall_ms: result.wall_ms }, null, 2));
    console.log(`FAIL ${job.file}: ${result.error}`);
  }
  log.push({ file: job.file, ok: result.ok, error: result.ok ? null : result.error, wall_ms: result.wall_ms });
}
writeFileSync(join(DIR, 'run-log.json'), JSON.stringify({ total: jobs.length, done, failed, skipped, log }, null, 2));
console.log(`\nDONE: ${done} ok, ${failed} failed, ${skipped} skipped (pre-existing) of ${jobs.length} jobs`);
