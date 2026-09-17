// Bounded extraction for the ql-deep lane retest (2026-09-13 glm runs).
// Read-only over the run JSONs; prints a compact report.
import { readFileSync } from 'node:fs';

const files = process.argv.slice(2);
for (const f of files) {
  const d = JSON.parse(readFileSync(f, 'utf8'));
  const rec = d.records[0];
  const raw = rec.record;
  const evs = raw.events || [];
  const hostReq = evs.filter(e => e.channel === 'host' && e.event_type === 'model_requested');
  const purposeCounts = {};
  hostReq.forEach(e => {
    const p = e.payload?.purpose || '(none)';
    purposeCounts[p] = (purposeCounts[p] || 0) + 1;
  });

  // conjugate marker in closure/verdict data
  const closure = raw.closure;
  const conjInClosure = closure?.success_state?.conjugate_stability;
  // search whole record for conjugate verdicts of any shape
  const s = JSON.stringify(rec);
  const conjMatches = [...s.matchAll(/"conjugate(?:_stability)?"\s*:\s*"([^"]+)"/g)].map(m => m[1]);
  const conjCounts = conjMatches.reduce((a, v) => (a[v] = (a[v] || 0) + 1, a), {});

  // empty content retry
  const emptyRetry = /empty_content_retry/.test(s);
  // count empty content outputs (model_returned with empty content)
  const hostRet = evs.filter(e => e.channel === 'host' && e.event_type === 'model_returned');
  const emptyReturns = hostRet.filter(e => {
    const c = e.payload?.output?.content;
    return c === '' || c === undefined || c === null;
  }).length;

  // per-position acts: look at ql semantic events and model_requested purposes in sequence
  const purposeSeq = hostReq.map(e => e.payload?.purpose || '(none)');

  // allowance refusals
  const allowanceRefusals = (s.match(/allowance_refusal/g) || []).length;
  const allowanceHits = [...s.matchAll(/"allowance[^"]*"\s*:\s*\{[^}]*\}/g)].map(m => m[0]).slice(0, 6);

  const oc = rec.outcome?.content || rec.outcome?.text || (typeof rec.outcome === 'string' ? rec.outcome : JSON.stringify(rec.outcome));

  console.log('='.repeat(72));
  console.log('FILE:', f.split('/').pop());
  console.log('task:', rec.task_id, '| condition:', rec.condition, '| provider_mode:', rec.provider_mode, '| model:', rec.model);
  console.log('model_calls:', rec.model_calls, '| total_tokens:', rec.total_tokens, `(in ${rec.input_tokens} / out ${rec.output_tokens})`, '| elapsed_s:', (rec.elapsed_ms / 1000).toFixed(1));
  console.log('execution_status:', rec.execution_status, '| semantic_status:', rec.semantic_status);
  const vp = rec.verification?.pass ?? rec.verification?.passed ?? JSON.stringify(rec.verification).slice(0, 200);
  console.log('verification:', JSON.stringify(rec.verification));
  console.log('ql_semantic_events:', rec.ql_semantic_events, '| operator_events:', rec.operator_events);
  console.log('model_requested purposes:', JSON.stringify(purposeCounts));
  console.log('purpose sequence:', JSON.stringify(purposeSeq));
  console.log('conjugate in closure.success_state:', conjInClosure, '| all conjugate markers:', JSON.stringify(conjCounts));
  console.log('empty_content_retry present:', emptyRetry, '| empty model_returned payloads:', emptyReturns);
  console.log('allowance_refusal occurrences:', allowanceRefusals);
  if (allowanceHits.length) console.log('allowance samples:', allowanceHits.slice(0, 3).join(' ; ').slice(0, 400));
  console.log('outcome (first 250 chars):', String(oc).slice(0, 250));

  // closure detail
  if (closure) {
    console.log('closure.success_state:', JSON.stringify(closure.success_state));
    if (closure.verdict) console.log('closure.verdict:', JSON.stringify(closure.verdict).slice(0, 500));
    if (closure.evaluation_refs) console.log('closure.evaluation_refs:', JSON.stringify(closure.evaluation_refs));
  }
  // any ql semantic events listing
  const qse = rec.ql_semantic_events;
  if (Array.isArray(qse)) {
    const kc = {};
    qse.forEach(e => { const k = e.kind || e.type || '?'; kc[k] = (kc[k] || 0) + 1; });
    console.log('ql_semantic_event kinds:', JSON.stringify(kc));
  }
}
