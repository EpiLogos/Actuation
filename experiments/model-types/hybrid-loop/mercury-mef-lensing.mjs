#!/usr/bin/env node
// E-MT-LENS-MERCURY — the actual Mercury test: MEF lens analysis.
//
// Not a speed benchmark. The question this programme has for Mercury is
// whether a diffusion LLM's iteration-on-output makes it good at WIDE lensing:
// reading one subject through all 12 MEF lenses in a single pass, at parallel
// scale. Three measurements, same corpus as Jev's E-MT-1 lens readings
// (100 acts from the retained Series-1 runs), so readings are comparable
// across instruments:
//
//   A width-in-one-go   one call per act: all-12-lens mass distribution as
//                       strict JSON. Parse rate, standing-pair mass (L1+L4'),
//                       latency, tokens, cost.
//   B parallel scale    the same 100 calls at concurrency 10: aggregate
//                       throughput (the multi-analysis use case).
//   C depth-in-one-go   one full circuit as subject: a single call producing
//                       per-lens prose analysis for all 12 lenses. Does one
//                       diffusion pass hold coherence across the whole lens
//                       manifold? Completeness measured, not scored for truth.
//
// Standing law: MEF has no ground truth by design (a manifold of disclosure,
// not a bucket taxonomy). Every reading here is agreement-of-readings with the
// runtime's standing refraction pair (L1+L4'), semantic-stochastic, promotion
// none. Credential: INCEPTION_API_KEY env or login keychain; fails closed.

import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

const API = 'https://api.inceptionlabs.ai/v1/chat/completions';
const MODEL = 'mercury-2.5';
const LENSES = ['L0', "L0'", 'L1', "L1'", 'L2', "L2'", 'L3', "L3'", 'L4', "L4'", 'L5', "L5'"];
const PAIR = ['L1', "L4'"];
const CONCURRENCY = 10;
const VENDOR_IN = 0.20 / 1e6, VENDOR_OUT = 0.75 / 1e6; // vendor-claimed rates

let key = process.env.INCEPTION_API_KEY;
if (!key) {
  try {
    key = execFileSync('security', ['find-generic-password', '-s', 'INCEPTION_API_KEY', '-w'],
      { stdio: ['ignore', 'pipe', 'ignore'] }).toString().trim();
  } catch { /* fall through */ }
}
if (!key) {
  console.error(JSON.stringify({ refusal: 'missing-credential' }));
  process.exit(3);
}

// same corpus as E-MT-1 mef-lens-refraction states
const jevDir = resolve(import.meta.dirname, '../jev');
const acts = readFileSync(resolve(jevDir, 'states.jsonl'), 'utf8').trim().split('\n')
  .map((l) => JSON.parse(l))
  .filter((s) => s.question_id === 'mef-lens-refraction')
  .map((s) => s.state);

function lensPrompt(subject, successConditions, deep) {
  const base = `Subject: ${subject}
Task success conditions: ${successConditions.join(' | ')}
Read this subject through each of the 12 MEF lenses: ${LENSES.join(', ')}.
Lenses are readings taken now, never identities assigned to the subject.
Return STRICT JSON only, no prose outside the JSON:
{"masses": {${LENSES.map((l) => `"${l}": <0.0-1.0 mass>`).join(', ')}}, "primary": "<lens>", "confidence": <0.0-1.0>${deep ? ', "reading": {"<lens>": "<2-3 sentence reading through that lens>" for EVERY lens}' : ''}}
Masses need not sum to 1; give your honest distribution.`;
  return base;
}

async function call(prompt, maxTokens = 900) {
  const t0 = Date.now();
  const res = await fetch(API, {
    method: 'POST',
    headers: { 'content-type': 'application/json', authorization: `Bearer ${key}` },
    body: JSON.stringify({ model: MODEL, messages: [{ role: 'user', content: prompt }], max_tokens: maxTokens, reasoning_effort: 'low' }),
  });
  const wall_ms = Date.now() - t0;
  if (!res.ok) return { ok: false, status: res.status, wall_ms, detail: (await res.text()).slice(0, 200) };
  const doc = await res.json();
  return { ok: true, wall_ms, usage: doc.usage ?? {}, content: doc.choices?.[0]?.message?.content ?? '' };
}

function parseMasses(content) {
  try {
    const m = content.match(/\{[\s\S]*\}/);
    if (!m) return null;
    const doc = JSON.parse(m[0]);
    const masses = doc.masses ?? {};
    const got = LENSES.filter((l) => typeof masses[l] === 'number');
    if (got.length < 6) return null;
    return { masses, primary: doc.primary ?? null, confidence: doc.confidence ?? null, lenses_covered: got.length };
  } catch { return null; }
}

async function pool(items, n, fn) {
  const results = new Array(items.length);
  let i = 0;
  await Promise.all(Array.from({ length: n }, async () => {
    while (i < items.length) { const mine = i++; results[mine] = await fn(items[mine], mine); }
  }));
  return results;
}

// ---- A + B: width-in-one-go at parallel scale
const tAll = Date.now();
const readings = await pool(acts, CONCURRENCY, async (st, i) => {
  const r = await call(lensPrompt(st.subject, st.task_success_conditions ?? [], false));
  const parsed = r.ok ? parseMasses(r.content) : null;
  const pairMass = parsed ? PAIR.reduce((s, l) => s + (parsed.masses[l] ?? 0), 0) : null;
  process.stderr.write(`#${i} ${r.wall_ms}ms parsed=${!!parsed} pair=${pairMass?.toFixed(2) ?? '-'}\n`);
  return { i, ok: r.ok, wall_ms: r.wall_ms, usage: r.usage, parsed, pair_mass: pairMass,
           content_chars: r.content?.length ?? 0, primary_agrees_runtime: null };
});
const wallAll = Date.now() - tAll;

const parsedReadings = readings.filter((r) => r.parsed);
const failures = readings.filter((r) => !r.ok);
const tokIn = readings.reduce((s, r) => s + (r.usage?.prompt_tokens ?? 0), 0);
const tokOut = readings.reduce((s, r) => s + (r.usage?.completion_tokens ?? 0), 0);
const lat = readings.map((r) => r.wall_ms).sort((a, b) => a - b);
const genSec = readings.reduce((s, r) => s + r.wall_ms, 0) / 1000;

// ---- C: depth-in-one-go on one full circuit (the richest retained subject)
const depthSubject = {
  subject: `A completed circuit: intent "Make this workspace truthful and ready - inspect current state, verify actual behaviour before deciding what to change, preserve the public API, make only the narrowest justified correction." Residues: P1 read of STATUS.md falsely claiming the implementation broken; P1 test-suite run showing all tests pass; P2 verification of actual behaviour; P3 narrow correction: STATUS.md rewritten to record the verified state with evidence; P4 evaluation that the change is the narrowest justified correction and nothing else changed. Determination: workspace made truthful - stale broken-state claim removed, STATUS.md now describes verified current state with the evidence used.`,
  successConditions: ['STATUS.md describes the verified current state with evidence', 'implementation unchanged', 'public API intact'],
};
const depth = await call(lensPrompt(depthSubject.subject, depthSubject.successConditions, true), 1600);
const depthParsed = depth.ok ? (await Promise.resolve(depth)).content : '';
const depthDoc = (() => { try { const m = depthParsed.match(/\{[\s\S]*\}/); return m ? JSON.parse(m[0]) : null; } catch { return null; } })();
const depthReading = depthDoc?.reading ?? {};
const depthCovered = LENSES.filter((l) => typeof depthReading[l] === 'string' && depthReading[l].trim().length > 20).length;

const evidence = {
  schema: 'actuation.model-types-mercury-mef-lensing/v1',
  created: new Date().toISOString(),
  corpus: 'same 100 mef-lens-refraction act states as E-MT-1 (jev/states.jsonl)',
  standing: 'agreement-of-readings only; MEF has no ground truth by design; semantic-stochastic, promotion none',
  A_width: {
    n: acts.length,
    api_failures: failures.length,
    parse_rate: parsedReadings.length / acts.length,
    mean_lens_mass_on_standing_pair_L1_L4prime: parsedReadings.length
      ? parsedReadings.reduce((s, r) => s + r.pair_mass, 0) / parsedReadings.length : null,
    jev_same_corpus_reference: 0.870,
    mean_lens_covered_per_reading: parsedReadings.length
      ? parsedReadings.reduce((s, r) => s + r.parsed.lenses_covered, 0) / parsedReadings.length : null,
    latency_ms_p50: lat[Math.floor(lat.length / 2)],
    tokens_in_out: [tokIn, tokOut],
    cost_usd_at_vendor_rates: Number((tokIn * VENDOR_IN + tokOut * VENDOR_OUT).toFixed(5)),
  },
  B_parallel: {
    concurrency: CONCURRENCY,
    wall_ms_total: wallAll,
    aggregate_tok_s: Number((tokOut / genSec).toFixed(0)),
    note: 'aggregate throughput at concurrency 10 - the parallel multi-analysis use case',
  },
  C_depth_one_go: {
    ok: depth.ok, wall_ms: depth.wall_ms, usage: depth.usage,
    lenses_with_substantive_reading: `${depthCovered}/12`,
    completeness: depthCovered / 12,
    note: 'single diffusion pass asked for a per-lens prose reading of every lens on one full circuit',
  },
  honest_limits: [
    'no truth exists for MEF readings by design; the standing-pair mass is agreement between readings, not accuracy',
    'Jev reference 0.870 is from the same corpus but a different question shape (choice vs mass distribution)',
    'one depth subject only; C is a feasibility reading, not a benchmark',
    'rates are vendor claims; usage counts are ours',
  ],
  provenance: { promotion: 'none', reading_class: 'semantic-stochastic' },
};
const outPath = resolve(import.meta.dirname, 'evidence', `mercury-mef-${Date.now()}.json`);
writeFileSync(outPath, JSON.stringify(evidence, null, 2) + '\n');
console.log(JSON.stringify({ A: evidence.A_width, B: evidence.B_parallel, C: evidence.C_depth_one_go }, null, 2));
