#!/usr/bin/env node
// Jev typed-question harness — fail-closed, digest-pinned run records.
//
// Talks to the TypeSafe System One API directly (POST /v1/systemone,
// Bearer TYPESAFE_API_KEY) — no SDK dependency. The credential is read from
// the environment or, on macOS, from the login keychain. Without a usable
// credential the harness refuses (exit 3) before any network call: no fixture
// fallback, no masked failure. Run records always carry determination
// pending-human-review (instrument readings, never the determination).

import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync, existsSync, mkdirSync } from 'node:fs';
import { resolve } from 'node:path';

const API_URL = 'https://api.typesafe.ai/v1/systemone';
const MODEL_REF = 'jev-latest';

function sha(s) { return createHash('sha256').update(s).digest('hex'); }
function refuse(reason, detail = {}) {
  console.error(JSON.stringify({ refusal: reason, ...detail }, null, 2));
  process.exit(3);
}

const argv = process.argv.slice(2);
function arg(name, dflt) {
  const i = argv.indexOf(name);
  return i >= 0 ? argv[i + 1] : dflt;
}
const statesPath = resolve(arg('--states', 'states.jsonl'));
const truthPath = resolve(arg('--truth', 'truth.jsonl'));
const cataloguePath = resolve(arg('--catalogue', new URL('./question-catalogue.json', import.meta.url).pathname));
const outDir = resolve(arg('--out-dir', 'runs'));
const limit = Number(arg('--limit', String(Infinity)));

// ---- credential: env first, then macOS login keychain
let key = process.env.TYPESAFE_API_KEY;
if (!key) {
  try {
    key = execFileSync('security', ['find-generic-password', '-s', 'TYPESAFE_API_KEY', '-w'], {
      stdio: ['ignore', 'pipe', 'ignore'],
    }).toString().trim();
  } catch {
    // fall through to refusal
  }
}
if (!key) {
  refuse('missing-credential', {
    required: 'TYPESAFE_API_KEY (environment variable or macOS login keychain)',
    law: 'fail-closed: no fixture fallback, no masked failure',
  });
}
for (const [label, p] of [['states', statesPath], ['truth', truthPath], ['catalogue', cataloguePath]]) {
  if (!existsSync(p)) refuse('missing-input', { which: label, path: p });
}

const catalogue = JSON.parse(readFileSync(cataloguePath, 'utf8'));
const questionsById = new Map();
for (const family of Object.values(catalogue.families)) {
  for (const q of family.questions ?? []) questionsById.set(q.id, q);
  for (const dp of family.decision_points ?? []) {
    for (const q of dp.questions ?? []) questionsById.set(q.id, { ...q, rust_point: dp.rust_point });
  }
}

const states = readFileSync(statesPath, 'utf8').trim().split('\n').map((l) => JSON.parse(l)).slice(0, limit);
const truth = new Map(
  readFileSync(truthPath, 'utf8').trim().split('\n').filter(Boolean).map((l) => {
    const r = JSON.parse(l);
    return [r.record_id, r];
  }),
);

function questionBody(q) {
  const body = { type: q.type, instructions: q.instructions };
  if (Array.isArray(q.criteria)) body.criteria = q.criteria;
  else if (q.criteria && !Array.isArray(q.options)) body.criteria = q.criteria;
  else if (q.options) body.criteria = Object.fromEntries(q.options.map((o) => [o, o]));
  return body;
}

function scoreAnswer(q, answer, truthAnswer) {
  if (q.type === 'choice') {
    const probs = answer.probabilities ?? {};
    const pred = answer.choice ?? Object.entries(probs).sort((a, b) => b[1] - a[1])[0]?.[0] ?? null;
    const brier = Object.keys(probs).length
      ? Object.entries(probs).reduce((acc, [o, p]) => acc + (p - (o === truthAnswer ? 1 : 0)) ** 2, 0)
      : null;
    return { correct: truthAnswer === undefined ? null : pred === truthAnswer, brier, predicted: pred, confidence: answer.confidence };
  }
  if (q.type === 'noul') {
    const p = answer.noul ?? null;
    return { correct: p === null || truthAnswer === undefined ? null : (p >= 0.5) === truthAnswer, brier: p === null || truthAnswer === undefined ? null : (p - (truthAnswer ? 1 : 0)) ** 2, probability: p };
  }
  return { correct: null, brier: null, score: answer.score, legend: answer.legend, confidence: answer.confidence };
}

async function ask(state, questions) {
  const t0 = Date.now();
  const res = await fetch(API_URL, {
    method: 'POST',
    headers: { Authorization: `Bearer ${key}`, 'Content-Type': 'application/json' },
    body: JSON.stringify({ state, model: MODEL_REF, questions }),
  });
  const latency_ms = Date.now() - t0;
  if (!res.ok) {
    throw new Error(`typesafe-api-${res.status}: ${(await res.text()).slice(0, 300)}`);
  }
  return { result: await res.json(), latency_ms };
}

const perQuestion = new Map();
const records = [];
let usage = { input_tokens: 0, output_tokens: 0 };
let standingPairMass = { sum: 0, n: 0 };

for (const rec of states) {
  const q = questionsById.get(rec.question_id);
  if (!q) continue;
  const t = truth.get(rec.record_id);
  const { result, latency_ms } = await ask(rec.state, { [rec.question_id]: questionBody(q) });
  const answer = result.answers?.[rec.question_id] ?? {};
  const scored = scoreAnswer(q, answer, t?.answer);
  usage.input_tokens += result.usage?.input_tokens ?? 0;
  usage.output_tokens += result.usage?.output_tokens ?? 0;

  const agg = perQuestion.get(rec.question_id) ?? { n: 0, correct: 0, scored: 0, brierSum: 0, confSum: 0 };
  agg.n += 1;
  if (scored.correct !== null) { agg.correct += scored.correct ? 1 : 0; agg.scored += 1; }
  if (scored.brier !== null) agg.brierSum += scored.brier;
  if (typeof scored.confidence === 'number') agg.confSum += scored.confidence;
  perQuestion.set(rec.question_id, agg);

  if (rec.question_id === 'mef-lens-refraction' && answer.probabilities) {
    const pair = rec.expected_standing_pair ?? ['L1', "L4'"];
    const mass = pair.reduce((acc, lens) => acc + (answer.probabilities[lens] ?? 0), 0);
    standingPairMass.sum += mass;
    standingPairMass.n += 1;
    records.push({
      record_id: rec.record_id, question_id: rec.question_id,
      answer, standing_pair_mass: mass, latency_ms, truth: null,
      scored: { correct: null, brier: null },
    });
  } else {
    records.push({
      record_id: rec.record_id, question_id: rec.question_id,
      answer, truth: t?.answer ?? null, scored, latency_ms,
    });
  }
}

const per_question = Object.fromEntries(
  [...perQuestion.entries()].map(([id, a]) => [id, {
    n: a.n,
    accuracy: a.scored ? a.correct / a.scored : null,
    brier_mean: a.scored ? a.brierSum / a.scored : null,
    confidence_mean: a.n ? a.confSum / a.n : null,
  }]),
);

const run = {
  schema: 'actuation.model-types-jev-run/v1',
  created: new Date().toISOString(),
  model_ref: MODEL_REF,
  api: API_URL,
  instrument_standing: catalogue.instrument_standing,
  inputs: {
    catalogue: { path: cataloguePath, sha256: sha(readFileSync(cataloguePath)) },
    states: { path: statesPath, sha256: sha(readFileSync(statesPath)), count: states.length },
    truth: { path: truthPath, sha256: sha(readFileSync(truthPath)) },
  },
  per_question,
  standing_refraction_pair_mass: standingPairMass.n
    ? { pair: ['L1', "L4'"], mean_mass: standingPairMass.sum / standingPairMass.n, n: standingPairMass.n,
        note: 'agreement-of-readings with the runtime standing pair, not accuracy' }
    : null,
  usage,
  determination: 'pending-human-review',
  provenance: { promotion: 'none', reading_class: 'semantic-stochastic' },
  records,
};

mkdirSync(outDir, { recursive: true });
const outPath = `${outDir}/jev-run-${new Date().toISOString().replace(/[:.]/g, '-')}.json`;
writeFileSync(outPath, JSON.stringify(run, null, 2) + '\n');
console.log(JSON.stringify({
  wrote: outPath,
  per_question,
  standing_refraction_pair_mass: run.standing_refraction_pair_mass,
  usage,
}, null, 2));
