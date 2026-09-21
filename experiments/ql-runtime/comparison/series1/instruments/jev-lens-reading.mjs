#!/usr/bin/env node
// Jev lens-reading instrument: the cognitive tool behind `lens_reading`.
// stdin:  {"subject": "..."}
// stdout: {"lens": "L4'", "rationale": string, "probabilities": {...}, "latency_ms": n}
// Model: jev-latest via the TypeSafe System One API. The lens question is
// verbatim the classification thread's LENS_QUESTION (jev_mef_harness.mjs) —
// same instrument wording, so a live in-work reading and a batch reading are
// the same reading. The lens is a reading taken now, never an identity
// assigned to the subject. Key from env TYPESAFE_API_KEY or, on macOS, the
// login keychain. Fail-closed: any error exits non-zero and the caller
// returns the error to the model as a tool result; nothing here fakes a
// reading.
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';

const API_URL = 'https://api.typesafe.ai/v1/systemone';
const MODEL_REF = 'jev-latest';

function loadKey() {
  if (process.env.TYPESAFE_API_KEY) return process.env.TYPESAFE_API_KEY;
  return execFileSync('security', ['find-generic-password', '-s', 'TYPESAFE_API_KEY', '-w'], {
    encoding: 'utf8',
  }).trim();
}

function fail(message) {
  process.stderr.write(`jev-lens-reading: ${String(message).slice(0, 400)}\n`);
  process.exit(2);
}

let req;
try {
  req = JSON.parse(readFileSync(0, 'utf8'));
} catch (e) {
  fail(`request parse: ${e.message}`);
}
const subject = typeof req.subject === 'string' ? req.subject.trim() : '';
if (!subject) fail('request needs a subject');

const state = {
  kind: 'lens-reading',
  subject,
};
const questions = {
  'mef-lens-refraction': {
    type: 'choice',
    instructions:
      'Through which MEF lens is this subject most coherently read? The lens is a reading taken now, never an identity assigned to the subject: subject identity is not lens identity.',
    criteria: {
      L0: 'Quaternal — why/what/how questioning articulation',
      "L0'": 'Archetypal-Numerical — one through six as archetypal number',
      L1: 'Causal — svatantrya, material/efficient/formal/final cause, will',
      "L1'": 'Phenomenal — introversion, sensation, feeling, thinking, intuition, extroversion',
      L2: 'Logical — tetralemmaic ground: IS, IS-NOT, BOTH, NEITHER, SILENCE',
      "L2'": 'Alchemical-Elemental — elemental correspondence articulation',
      L3: 'Processual — concrescent desire, actual occasion (Whitehead)',
      "L3'": 'Chronological — Spirit (Geist), spring, summer, autumn, winter, life (Aufhebung)',
      L4: 'Phenomenological — Sein, Geworfenheit, Dasein, Zeit, Besorge, Gelassenheit',
      "L4'": 'Scientific — prompts, traces, challenges, patterns, discovery, insight',
      L5: 'Para Vāk — anuttara/asambhava, para vāk, paśyantī, madhyamā, vaikharī, mātṛkā',
      "L5'": 'Divine Logos — arche, apokalypsis, dynamis and the logological articulation',
    },
  },
};

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
const answer = result.answers?.['mef-lens-refraction'] ?? {};
const lens =
  answer.choice ?? Object.entries(answer.probabilities ?? {}).sort((a, b) => b[1] - a[1])[0]?.[0];
if (!lens) fail(`no usable lens in answer: ${JSON.stringify(answer).slice(0, 200)}`);
process.stdout.write(
  JSON.stringify({
    lens,
    rationale: answer.rationale ?? `probabilities: ${JSON.stringify(answer.probabilities ?? {})}`,
    probabilities: answer.probabilities ?? null,
    latency_ms: Date.now() - t0,
  }) + '\n',
);
