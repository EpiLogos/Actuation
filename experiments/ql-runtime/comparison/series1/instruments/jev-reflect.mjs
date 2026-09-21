#!/usr/bin/env node
// Jev reflect instrument: the cognitive engine behind the Night (P') tools.
// stdin:  {"mode": "full_text"|"right_frame"|"being"|"becoming"|"knowing"|"resonant",
//          "subject": string, "own_disclosure": optional string}
// stdout: {"mode", "reading": {...full giving...}, "own_disclosure": echoed, "latency_ms"}
// The reading carries the full MEF giving — lens names, faces, complete sublens
// chains, the family alignment in effect, full distributions — never bare codes.
// The square modes read the subject through the kernel A/B/C family alignments
// with the owner's title sets carried per square. Jev gives the decision; the
// model's own disclosure rides alongside and is echoed. Fail-closed: any error
// exits non-zero and the caller returns it to the model as a tool result.
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

const lensCriteria = () => Object.fromEntries(GIVING.lenses.map((l) => [l.id, `${l.name} — ${l.sublens.join(' → ')}`]));
const withGiving = (probs) => Object.fromEntries(
  GIVING.lenses.map((l) => [l.id, {
    name: l.name,
    face: l.face,
    mass: probs?.[l.id] ?? null,
    sublens: l.sublens,
  }]),
);

let questions, shape;
if (mode === 'full_text') {
  // The 72: for each lens, the subject's presence across its six articulations.
  shape = 'per-lens six-articulation presence (12 x 6 = 72 cells)';
  questions = Object.fromEntries(GIVING.lenses.map((l) => [
    l.id,
    {
      type: 'choice',
      instructions: `Reading the subject as a presented whole: where does ${l.name} live in it? Distribute the subject's presence across ${l.name}'s six articulations (${l.sublens.join(', ')}). One choice = where the articulation is strongest.`,
      criteria: Object.fromEntries(l.sublens.map((s, i) => [`${i + 1}. ${s}`, `${l.name} at its ${s} articulation`])),
    },
  ]));
} else if (mode === 'right_frame') {
  shape = 'single-lens selection (12-lens distribution, full giving)';
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
  // Square modes: the subject read through the square's family alignment —
  // three paired crossings, one lens reading per crossing.
  const squareKey = mode === 'being' ? 'A' : mode === 'becoming' ? 'B' : 'C';
  const square = GIVING.squares[squareKey];
  const family = GIVING.families[square.family_alignment];
  shape = `square ${squareKey} (${square.titles.filter(Boolean).join(' / ')}): lens reading at each of the three paired crossings of the ${family.alignment} alignment`;
  questions = Object.fromEntries(family.pairs.map(([p, q], i) => [
    `crossing_${i + 1}_P${p}_P${q}`,
    {
      type: 'choice',
      instructions: `Reading the subject through the square (${square.titles.filter(Boolean).join(' / ')}): at the crossing between ${GIVING.positions[p]} and ${GIVING.positions[q]}, which lens reads the subject most coherently there?`,
      criteria: lensCriteria(),
    },
  ]));
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
    cells: Object.fromEntries(GIVING.lenses.map((l) => {
      const a = result.answers?.[l.id] ?? {};
      return [`${l.id} ${l.name}`, {
        distribution: a.probabilities ?? null,
        strongest: a.choice ?? Object.entries(a.probabilities ?? {}).sort((x, y) => y[1] - x[1])[0]?.[0] ?? null,
        sublens: l.sublens,
      }];
    })),
  };
} else if (mode === 'right_frame' || mode === 'resonant') {
  const key = mode === 'right_frame' ? 'right_frame' : 'resonant_frames';
  const a = result.answers?.[key] ?? {};
  const sorted = Object.entries(a.probabilities ?? {}).sort((x, y) => y[1] - x[1]);
  reading = {
    shape,
    selected: a.choice ?? sorted[0]?.[0] ?? null,
    ranking: sorted.map(([id, mass]) => ({ lens: id, name: GIVING.lenses.find((l) => l.id === id)?.name, mass })),
    rationale: a.rationale ?? null,
  };
} else {
  const squareKey = mode === 'being' ? 'A' : mode === 'becoming' ? 'B' : 'C';
  const square = GIVING.squares[squareKey];
  const family = GIVING.families[square.family_alignment];
  reading = {
    shape,
    square: { key: squareKey, titles: square.titles, alignment: family.alignment, pairs: family.pairs },
    crossings: family.pairs.map(([p, q], i) => {
      const a = result.answers?.[`crossing_${i + 1}_P${p}_P${q}`] ?? {};
      return {
        crossing: `P${p}/P${q}`,
        positions: [GIVING.positions[p], GIVING.positions[q]],
        lens: a.choice ?? null,
        distribution: a.probabilities ?? null,
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
