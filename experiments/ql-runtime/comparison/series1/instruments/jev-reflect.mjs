#!/usr/bin/env node
// Jev reflect instrument: the cognitive engine behind the Night (P') tools.
// stdin:  {"mode": "full_text"|"right_frame"|"being"|"becoming"|"knowing"|"resonant",
//          "subject": string, "own_disclosure": optional string}
// stdout: {"mode", "reading": {...full giving...}, "own_disclosure": echoed, "latency_ms"}
// Ground: the canonical MEF lens register (mef-giving.json, copied from the
// personal context surface to ql-mef docs 2026-09-21). Machinery-first: every
// question carries the lens's defined sub-slots WITH their authored meanings —
// a lens named must run its machinery. The subject is pre-lens data; the
// lenses are refraction media of the P/P' positions, never the thing itself.
// Square modes are register rule 4: lens-pairs over position-pairs.
// Fail-closed: any error exits non-zero and the caller returns it to the
// model as a tool result; nothing here fakes a reading.
import { execFileSync } from 'node:child_process';
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from 'node:url';

const API_URL = 'https://api.typesafe.ai/v1/systemone';
const MODEL_REF = 'jev-latest';
const GIVING = JSON.parse(readFileSync(join(dirname(fileURLToPath(import.meta.url)), 'mef-giving.json'), 'utf8'));

function loadKey() {
  if (process.env.TYPESAFE_API_KEY) return process.env.TYPESAFE_API_KEY;
  return execFileSync('security', ['find-generic-password', '-s', 'TYPESAFE_API_KEY', '-w'], {
    encoding: 'utf8',
  }).trim();
}

function fail(message) {
  process.stderr.write(`jev-reflect: ${String(message).slice(0, 400)}\n`);
  process.exit(2);
}

const req = (() => {
  try { return JSON.parse(readFileSync(0, 'utf8')); } catch (e) { fail(`request parse: ${e.message}`); }
})();
const mode = req.mode ?? '';
const subject = typeof req.subject === 'string' ? req.subject.trim() : '';
const own = typeof req.own_disclosure === 'string' ? req.own_disclosure : '';
if (!subject) fail('request needs a subject');
if (!['full_text', 'right_frame', 'being', 'becoming', 'knowing', 'resonant'].includes(mode)) fail(`unknown mode: ${mode}`);

const slotLine = (s) => `${s.slot} ${s.label} — ${s.meaning}`;
const lensLine = (l) => `${l.id} ${l.name} (${l.ground}): ${l.sublens.map((s) => s.label).join(' · ')}`;
const lensCriteria = () => Object.fromEntries(GIVING.lenses.map((l) => [l.id, lensLine(l)]));
const slotCriteria = (l) => Object.fromEntries(l.sublens.map((s) => [`${s.slot} ${s.label}`, slotLine(s)]));
const fullGiving = (lensId, probs, strongest) => {
  const l = GIVING.lenses.find((x) => x.id === lensId);
  return { lens: lensId, name: l.name, ground: l.ground, strongest, distribution: probs ?? null, machinery: l.sublens };
};

let questions, shape;
if (mode === 'full_text') {
  // Every lens run at its machinery: one question per lens, criteria are the
  // defined sub-slots with their authored meanings.
  shape = 'machinery-first: each of the twelve lenses run across its defined sub-slots (12 x 6, meanings carried)';
  questions = Object.fromEntries(GIVING.lenses.map((l) => [
    l.id,
    {
      type: 'choice',
      instructions: `Run ${l.name} (${l.ground}) over the subject as a presented whole. Its machinery: ${l.sublens.map(slotLine).join('; ')}. At which sub-slot does the subject's weight most strongly sit under this lens? The subject is pre-lens data; the sub-slot is where the lens refracts it.`,
      criteria: slotCriteria(l),
    },
  ]));
} else if (mode === 'right_frame') {
  shape = 'single-lens selection (12-lens distribution, machinery summarised)';
  questions = {
    right_frame: {
      type: 'choice',
      instructions: 'Which single lens is most useful for reading this subject now — the frame that, taken up, does the most work? The lens is a reading taken now, never an identity assigned to the subject.' + (own ? ` The worker's own candidate reading rides with this request; weigh it but decide independently.` : ''),
      criteria: lensCriteria(),
    },
  };
} else if (mode === 'resonant') {
  shape = 'ranked frame relation (full ordering, topk=12)';
  questions = {
    resonant_frames: {
      type: 'choice',
      instructions: 'Which lens is most strongly resonant with this subject as a whole — the first among the frames in relation? The full ranking is taken from the returned distribution. ' + (own ? `The worker's own candidate frames ride with this request; weigh them but decide independently.` : ''),
      criteria: lensCriteria(),
    },
  };
} else {
  // Register rule 4: squares are lens-pairs over position-pairs. Each square
  // reading runs its linked lens at its linked position, machinery carried.
  const squareKey = mode === 'being' ? 'A' : mode === 'becoming' ? 'B' : 'C';
  const square = GIVING.squares[squareKey];
  const dayLensIds = square.lens_pairs.flat().filter((id) => !id.includes("'"));
  const dayLenses = GIVING.lenses.filter((l) => dayLensIds.includes(l.id));
  const posDay = square.positions.filter((p) => !p.includes("'"));
  shape = `square ${squareKey} (${square.titles.filter(Boolean).join(' / ')}): ${square.lens_pairs.map(([a, b]) => `${a}↔${b}`).join(', ')} over ${square.positions.join('/')}`;
  questions = {};
  for (const l of dayLenses) {
    for (const p of posDay) {
      questions[`${l.id}@${p}`] = {
        type: 'choice',
        instructions: `Reading the subject's ${GIVING.positions[p]} through ${l.id} ${l.name} (${l.ground}). Its machinery: ${l.sublens.map(slotLine).join('; ')}. At which sub-slot does it refract the subject's ${GIVING.positions[p]}?`,
        criteria: slotCriteria(l),
      };
    }
  }
}

const state = { kind: `reflect-${mode}`, subject, ...(own ? { own_disclosure: own } : {}) };
const t0 = Date.now();
let res;
try {
  res = await fetch(API_URL, {
    method: 'POST',
    headers: { Authorization: `Bearer ${loadKey()}`, 'Content-Type': 'application/json' },
    body: JSON.stringify({ state, model: MODEL_REF, questions }),
  });
} catch (e) {
  fail(`transport: ${e.message}`);
}
if (!res.ok) fail(`typesafe-api-${res.status}: ${(await res.text()).slice(0, 300)}`);
const result = await res.json();

let reading;
if (mode === 'full_text') {
  reading = {
    shape,
    cells: GIVING.lenses.map((l) => {
      const a = result.answers?.[l.id] ?? {};
      const strongest = a.choice ?? Object.entries(a.probabilities ?? {}).sort((x, y) => y[1] - x[1])[0]?.[0] ?? null;
      return fullGiving(l.id, a.probabilities ?? null, strongest);
    }),
  };
} else if (mode === 'right_frame' || mode === 'resonant') {
  const key = mode === 'right_frame' ? 'right_frame' : 'resonant_frames';
  const a = result.answers?.[key] ?? {};
  const sorted = Object.entries(a.probabilities ?? {}).sort((x, y) => y[1] - x[1]);
  reading = {
    shape,
    selected: a.choice ?? sorted[0]?.[0] ?? null,
    ranking: sorted.map(([id, mass]) => {
      const l = GIVING.lenses.find((x) => x.id === id);
      return { lens: id, name: l?.name, mass };
    }),
    rationale: a.rationale ?? null,
  };
} else {
  const squareKey = mode === 'being' ? 'A' : mode === 'becoming' ? 'B' : 'C';
  const square = GIVING.squares[squareKey];
  reading = {
    shape,
    square: { key: squareKey, titles: square.titles, lens_pairs: square.lens_pairs, positions: square.positions },
    readings: Object.keys(questions).map((q) => {
      const a = result.answers?.[q] ?? {};
      const [lensId, pos] = q.split('@');
      const l = GIVING.lenses.find((x) => x.id === lensId);
      return {
        at: pos,
        lens: lensId,
        name: l.name,
        strongest: a.choice ?? null,
        distribution: a.probabilities ?? null,
        machinery: l.sublens,
      };
    }),
  };
}

process.stdout.write(JSON.stringify({
  mode,
  subject_chars: subject.length,
  reading,
  own_disclosure: own || null,
  latency_ms: Date.now() - t0,
}) + '\n');
