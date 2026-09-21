#!/usr/bin/env node
// E-MT-AMPLIFY — the jev <-> Mercury classification/amplification loop.
//
// The pairing under test: Mercury writes (diffusion generation — amplification
// prose), jev judges (typed classification: lens, adequacy, direction). No
// autoregressive model in the loop. Each round: Mercury amplifies the subject
// -> jev classifies the amplification (mef-lens-refraction choice, act-adequacy
// score, amplify-direction choice) -> the verdict steers the next Mercury
// round. After K rounds jev answers a closure noul (is the amplification
// mature enough to conclude — the concrescence shape at loop scale).
//
// Every jev call goes through the standing fail-closed harness (states file ->
// harness.mjs -> digest-pinned run record); Mercury calls are direct
// OpenAI-compatible fetches. Subjects: one M0 symbolic formulation (the
// compressed VAK syntax) and one whole text (a full document) — per the
// owner's requirement for longer-form content. Standing: instrument readings,
// semantic-stochastic, promotion none.

import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync, readdirSync } from 'node:fs';
import { resolve } from 'node:path';

const JEV_DIR = resolve(import.meta.dirname, '../jev');
const OUT_DIR = resolve(import.meta.dirname, 'evidence');
const ROUNDS = 3;

let inceptionKey = process.env.INCEPTION_API_KEY;
try { inceptionKey ??= execFileSync('security', ['find-generic-password', '-s', 'INCEPTION_API_KEY', '-w'], { stdio: ['ignore', 'pipe', 'ignore'] }).toString().trim(); } catch {}
if (!inceptionKey) { console.error(JSON.stringify({ refusal: 'missing INCEPTION_API_KEY' })); process.exit(3); }

const symScope = JSON.parse(readFileSync('/tmp/embed-symbols-scope.json', 'utf8')).items;
const m0 = symScope.find((x) => x.coord === 'M0-0-0');
const wholeText = readFileSync(resolve(import.meta.dirname, '../ql-form/README.md'), 'utf8').slice(0, 4000);
const subjects = [
  { id: 'm0-formulation', kind: 'symbolic notation', text: `coordinate M0-0-0 (Anuttara, Transcendent Pole)\n${m0.text}` },
  { id: 'ql-form-readme', kind: 'whole text', text: wholeText },
];

async function mercury(prompt) {
  const t0 = Date.now();
  const res = await fetch('https://api.inceptionlabs.ai/v1/chat/completions', {
    method: 'POST',
    headers: { 'content-type': 'application/json', authorization: `Bearer ${inceptionKey}` },
    body: JSON.stringify({ model: 'mercury-2.5', messages: [{ role: 'user', content: prompt }], max_tokens: 800, reasoning_effort: 'low' }),
  });
  if (!res.ok) throw new Error(`mercury ${res.status}: ${(await res.text()).slice(0, 150)}`);
  const doc = await res.json();
  return { text: doc.choices?.[0]?.message?.content ?? '', usage: doc.usage ?? {}, wall_ms: Date.now() - t0 };
}

function jevAsk(rid, qid, stateText) {
  const tmp = resolve(OUT_DIR, 'tmp-loop-state.jsonl');
  writeFileSync(tmp, JSON.stringify({ record_id: rid, family: 'concrescence', question_id: qid, state: stateText }) + '\n');
  const emptyTruth = resolve(OUT_DIR, 'tmp-empty-truth.jsonl');
  writeFileSync(emptyTruth, '');
  execFileSync('node', ['harness.mjs', '--states', tmp, '--truth', emptyTruth, '--out-dir', resolve(OUT_DIR, 'loop-jev-runs')], { cwd: JEV_DIR, stdio: 'pipe' });
  const runs = readdirSync(resolve(OUT_DIR, 'loop-jev-runs')).filter((f) => f.startsWith('jev-run-')).sort();
  const rec = JSON.parse(readFileSync(resolve(OUT_DIR, 'loop-jev-runs', runs[runs.length - 1]), 'utf8'));
  return rec.records[0].answer;
}

const LENSES = ['L0', "L0'", 'L1', "L1'", 'L2', "L2'", 'L3', "L3'", 'L4', "L4'", 'L5', "L5'"];
const transcript = [];

for (const subject of subjects) {
  const rounds = [];
  let steering = 'First amplification: read the subject through the 12 MEF lenses and write a concise amplification (~200 words): what the lenses reveal, where the reading is productive, where it breaks down. Plain prose.';
  for (let r = 1; r <= ROUNDS; r++) {
    const m = await mercury(`Subject (${subject.kind}):\n${subject.text}\n\n${steering}`);
    const aid = `${subject.id}-r${r}`;
    const lens = jevAsk(`lens|${aid}`, 'mef-lens-refraction', { subject: m.text.slice(0, 1500), task_success_conditions: ['amplification of the subject through the lens manifold'] });
    const adeq = jevAsk(`adequacy|${aid}`, 'act-adequacy', { task_id: subject.id, act_intent: m.text.slice(0, 1200), mode: 'amplify', success_conditions: ['advances the reading of the subject', 'stays with the subject', 'produces prose a person can judge'] });
    const dir = jevAsk(`direction|${aid}`, 'amplify-direction', { subject_id: subject.id, round: r, amplification_excerpt: m.text.slice(0, 1200), lens_reading: lens.choice, adequacy: adeq.score });
    rounds.push({ round: r, mercury_chars: m.text.length, mercury_usage: m.usage, wall_ms: m.wall_ms,
                  amplification: m.text.slice(0, 1200), lens: lens.choice, lens_confidence: lens.confidence,
                  adequacy: adeq.score, adequacy_confidence: adeq.confidence, direction: dir.choice, direction_confidence: dir.confidence });
    console.error(`${subject.id} r${r}: lens=${lens.choice} adeq=${adeq.score} dir=${dir.choice} (${m.text.length} chars)`);
    steering = `Your previous amplification was read through lens ${lens.choice}, adequacy ${adeq.score}/3. Verdict: ${dir.choice}. ` +
      (dir.choice === 'deepen' ? 'Deepen that reading: go further into what it opens, sharpen the formulation.'
        : dir.choice === 'shift-lens' ? `Take a different lens than ${lens.choice} and amplify from there.`
        : 'Conclude: write the final condensed statement of the reading.');
  }
  const fin = jevAsk(`closure|${subject.id}`, 'determination-warranted', {
    initiating_intent: `Amplify the reading of this ${subject.kind} subject through the MEF lens manifold`,
    middle_residues: rounds.map((rr) => ({ position: 'P2', kind: 'amplification', summary: rr.amplification.slice(0, 150) })),
    determination: { synthesis: rounds[rounds.length - 1].amplification.slice(0, 600), claimed_adequacy: 'unknown', evidence_ref_count: 0 },
  });
  transcript.push({ subject: subject.id, kind: subject.kind, rounds, closure_noul: fin.noul });
}

writeFileSync(resolve(OUT_DIR, `amplify-loop-${Date.now()}.json`),
  JSON.stringify({ schema: 'actuation.model-types-amplify-loop/v1', created: new Date().toISOString(),
    loop: 'mercury writes -> jev classifies (lens/adequacy/direction) -> verdict steers next round; closure noul at end',
    rounds_per_subject: ROUNDS, transcript,
    honest_limits: ['instrument readings only, promotion none', 'adequacy/lens/direction are jev readings of Mercury text, not ground truth',
      'the loop is ungraded: this run establishes that the pairing runs and what its trajectory looks like, not that it improves anything'],
    provenance: { promotion: 'none', reading_class: 'semantic-stochastic' } }, null, 2) + '\n');
console.log(JSON.stringify(transcript.map((t) => ({
  subject: t.subject, closure_noul: t.closure_noul,
  trajectory: t.rounds.map((r) => `${r.round}:${r.lens}/a${r.adequacy}/${r.direction}`),
})), null, 1));
