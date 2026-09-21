#!/usr/bin/env node
// R6 — Mercury 2.5 vendor-claim validation: measured tok/s and cost.
//
// The vendor claims (1107 tok/s; $0.20/$0.75 per M in/out) are recorded as
// unverified in ../specimens.json. This script measures what this programme
// actually gets: N fixed generation requests through the OpenAI-compatible
// API, tok/s computed from the response's usage.completion_tokens over the
// measured wall time of the request, cost computed from usage at the vendor's
// recorded rates (rates stay vendor claims; only usage numbers are ours).
// Credential from INCEPTION_API_KEY env or the macOS login keychain; fails
// closed without one. Output: one evidence JSON, promotion none.

import { execFileSync } from 'node:child_process';
import { writeFileSync } from 'node:fs';

const API = 'https://api.inceptionlabs.ai/v1/chat/completions';
const MODEL = 'mercury-2.5';
const N = Number(process.argv[2] ?? 5);

let key = process.env.INCEPTION_API_KEY;
if (!key) {
  try {
    key = execFileSync('security', ['find-generic-password', '-s', 'INCEPTION_API_KEY', '-w'],
      { stdio: ['ignore', 'pipe', 'ignore'] }).toString().trim();
  } catch { /* fall through */ }
}
if (!key) {
  console.error(JSON.stringify({ refusal: 'missing-credential', required: 'INCEPTION_API_KEY (env or login keychain)' }));
  process.exit(3);
}

const PROMPTS = [
  'Summarise the QL positions P0..P5 in one short paragraph each. Be plain and concrete.',
  'Write a concise design note: what an energy gate on candidate acts must never do. Three bullets.',
  'Draft a two-paragraph incident report: a graph container was lost, the map was restored from a snapshot volume.',
  'Explain in plain language what an AUC of 0.75 means for ranking real relations above corrupted ones.',
  'List five honest limitations of scoring synthetic corruptions instead of real loop transitions.',
];

const runs = [];
for (let i = 0; i < N; i++) {
  const body = {
    model: MODEL,
    messages: [{ role: 'user', content: PROMPTS[i % PROMPTS.length] }],
    max_tokens: 700,
    reasoning_effort: 'low',
  };
  const t0 = Date.now();
  const res = await fetch(API, {
    method: 'POST',
    headers: { 'content-type': 'application/json', authorization: `Bearer ${key}` },
    body: JSON.stringify(body),
  });
  const wall_ms = Date.now() - t0;
  if (!res.ok) {
    console.error(JSON.stringify({ error: res.status, detail: (await res.text()).slice(0, 300) }));
    process.exit(2);
  }
  const doc = await res.json();
  const u = doc.usage ?? {};
  const ct = u.completion_tokens ?? null;
  const content = doc.choices?.[0]?.message?.content;
  runs.push({
    i,
    wall_ms,
    content_nonempty: Boolean(content && content.trim().length > 0),
    content_chars: content ? content.length : 0,
    usage: u,
    tok_s_usage: ct && wall_ms ? (ct / (wall_ms / 1000)) : null,
  });
  console.error(`#${i} ${wall_ms}ms completion_tokens=${ct} tok/s=${runs[i].tok_s_usage?.toFixed(0)}`);
}

const speeds = runs.filter((r) => r.tok_s_usage).map((r) => r.tok_s_usage).sort((a, b) => a - b);
const med = speeds[Math.floor(speeds.length / 2)];
const totIn = runs.reduce((s, r) => s + (r.usage.prompt_tokens ?? 0), 0);
const totOut = runs.reduce((s, r) => s + (r.usage.completion_tokens ?? 0), 0);
const VENDOR_IN = 0.20 / 1e6, VENDOR_OUT = 0.75 / 1e6; // vendor-claimed rates, not verified terms
const evidence = {
  schema: 'actuation.model-types-mercury-measurement/v1',
  created: new Date().toISOString(),
  model: MODEL,
  n_requests: N,
  max_tokens_each: 700,
  results: runs.map(({ usage, ...rest }) => ({ ...rest, usage })),
  measured: {
    tok_s_median: med ? Number(med.toFixed(0)) : null,
    tok_s_min: speeds.length ? Number(speeds[0].toFixed(0)) : null,
    tok_s_max: speeds.length ? Number(speeds[speeds.length - 1].toFixed(0)) : null,
    wall_ms_total: runs.reduce((s, r) => s + r.wall_ms, 0),
    completion_tokens_total: totOut,
    prompt_tokens_total: totIn,
    cost_usd_at_vendor_rates: Number((totIn * VENDOR_IN + totOut * VENDOR_OUT).toFixed(6)),
  },
  vendor_claims_checked: [
    { claim: '1107 tokens/sec', standing: 'vendor-claim', our_measurement: `${med?.toFixed(0)} tok/s median over ${N} short completions (usage.completion_tokens / wall time)` },
    { claim: '$0.20/$0.75 per M tokens', standing: 'vendor-claim', our_measurement: `usage-derived cost at those rates: $${(totIn * VENDOR_IN + totOut * VENDOR_OUT).toFixed(6)} for ${totIn} in / ${totOut} out tokens (rates not independently verified)` },
  ],
  honest_limits: [
    'small-N short-completion measurement; not a streaming benchmark, not a throughput-at-scale claim',
    'wall time includes network round trip; vendor tok/s claims may measure decode only',
    'rates are vendor claims; only the usage token counts are this programme\'s measurements',
  ],
  provenance: { promotion: 'none', reading_class: 'research' },
};
const out = new URL('./evidence/mercury-measurement-' + Date.now() + '.json', 'file://' + process.cwd() + '/');
writeFileSync(out, JSON.stringify(evidence, null, 2) + '\n');
console.log(JSON.stringify({ measured: evidence.measured, vendor_claims_checked: evidence.vendor_claims_checked }, null, 2));
