#!/usr/bin/env node
// Jev close-check instrument for the ql-toolset return condition.
// stdin:  {"success_conditions": [...], "synthesis": "..."}
// stdout: {"verdict": "close"|"reopen", "rationale": string, "probabilities": {...}, "latency_ms": n}
// Model: jev-latest via the TypeSafe System One API (same conventions as
// experiments/model-types/jev/harness.mjs). Key from env TYPESAFE_API_KEY
// or, on macOS, the login keychain. Fail-closed: any error exits non-zero
// and the caller records the refusal; nothing here ever fakes a close.
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
  process.stderr.write(`jev-close-check: ${String(message).slice(0, 400)}\n`);
  process.exit(2);
}

let req;
try {
  req = JSON.parse(readFileSync(0, 'utf8'));
} catch (e) {
  fail(`request parse: ${e.message}`);
}
const conditions = Array.isArray(req.success_conditions) ? req.success_conditions : [];
const synthesis = typeof req.synthesis === 'string' ? req.synthesis : '';
if (!conditions.length || !synthesis.trim()) fail('request needs success_conditions and synthesis');

const state = {
  kind: 'return-condition-check',
  success_conditions: conditions,
  synthesis,
};
const questions = {
  verdict: {
    type: 'choice',
    instructions:
      'Does the synthesis actually realise every success condition? Answer "close" only if all conditions hold in the synthesis as stated; otherwise "reopen".',
    criteria: { close: 'every success condition is realised by the synthesis', reopen: 'at least one condition is not realised or the synthesis is silent on it' },
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
const answer = result.answers?.verdict ?? {};
const choice = answer.choice ?? Object.entries(answer.probabilities ?? {}).sort((a, b) => b[1] - a[1])[0]?.[0];
if (choice !== 'close' && choice !== 'reopen') fail(`no usable verdict in answer: ${JSON.stringify(answer).slice(0, 200)}`);
process.stdout.write(
  JSON.stringify({
    verdict: choice,
    rationale: answer.rationale ?? `probabilities: ${JSON.stringify(answer.probabilities ?? {})}`,
    probabilities: answer.probabilities ?? null,
    latency_ms: Date.now() - t0,
  }) + '\n',
);
