import { assessComparisonEvidence, buildMaskMapping, sanitizeEvidence } from './evidence.mjs';

function fence(value, language = 'json') {
  const safe = sanitizeEvidence(value);
  const text = typeof safe === 'string' ? safe : JSON.stringify(safe, null, 2);
  return `~~~${language}\n${text}\n~~~`;
}

function changedFiles(before = {}, after = {}) {
  const names = [...new Set([...Object.keys(before), ...Object.keys(after)])].sort();
  return names.filter((name) => before[name] !== after[name]);
}

function renderWorkspace(workspace = {}, heading) {
  const lines = [`### ${heading}`, ''];
  const names = Object.keys(workspace).sort();
  if (names.length === 0) return [...lines, '_No files._', ''].join('\n');
  for (const name of names) {
    lines.push(`#### \`${name}\``, '', fence(workspace[name], 'text'), '');
  }
  return lines.join('\n');
}

function renderChangedFiles(record) {
  const changed = changedFiles(record.starting_workspace, record.final_workspace);
  const lines = [`### Changed-file view (${changed.length})`, ''];
  if (changed.length === 0) return [...lines, '_No workspace files changed._', ''].join('\n');
  for (const name of changed) {
    lines.push(`#### \`${name}\` — before`, '', fence(record.starting_workspace?.[name] ?? '<absent>', 'text'), '');
    lines.push(`#### \`${name}\` — after`, '', fence(record.final_workspace?.[name] ?? '<absent>', 'text'), '');
  }
  return lines.join('\n');
}

function metricLine(record) {
  const cost = record.model_cost === null || record.model_cost === undefined ? 'unavailable' : String(record.model_cost);
  return `Model calls: **${record.model_calls}** · Capability calls: **${record.capability_calls}** · Tokens: **${record.total_tokens}** · Cost: **${cost}** · Elapsed: **${Math.round(record.elapsed_ms)} ms**`;
}

function maskedHostEvent(event, index) {
  const number = `${index + 1}.`;
  if (event.event_type === 'capability_requested') {
    return `#### ${number} capability requested\n\n${fence({ name: event.payload?.name, args: event.payload?.args ?? {} })}\n`;
  }
  if (event.event_type === 'capability_returned') {
    return `#### ${number} capability returned\n\n${fence({ name: event.payload?.name, ok: event.payload?.ok, result: event.payload?.result, error: event.payload?.error })}\n`;
  }
  if (event.event_type === 'model_requested') {
    return `#### ${number} model requested\n\n_Model input details that would disclose runtime/controller mechanics are reserved for Pass B. The exact shared benchmark prompt is shown above._\n`;
  }
  if (event.event_type === 'model_returned') {
    const usage = event.payload?.output?.usage ?? null;
    return `#### ${number} model returned\n\n${usage ? fence({ usage }) : '_Return envelope details that would disclose runtime/controller mechanics are reserved for Pass B._'}\n`;
  }
  return `#### ${number} host event\n\n_Event details reserved for Pass B._\n`;
}

function unmaskedEvent(event, index) {
  const title = `${index + 1}. ${event.channel ?? 'event'} :: ${event.event_type ?? event.type ?? 'unknown'}`;
  return `#### ${title}\n\n${fence(event)}\n`;
}

function renderPortableRecord(record, heading, { masked }) {
  const parts = [
    `## ${heading} — repetition ${record.repetition}`,
    '',
    `Execution: \`${record.execution_status}\`  `,
    `Objective verification: \`${record.verification?.objective_checks_pass}\`  `,
    metricLine(record),
    '',
    '### Final outcome',
    '',
    fence(record.outcome, 'text'),
    '',
    '### Objective verification evidence',
    '',
    fence(record.verification),
    '',
    renderWorkspace(record.starting_workspace, 'Complete starting workspace'),
    renderChangedFiles(record),
    renderWorkspace(record.final_workspace, 'Complete final workspace')
  ];

  if (!masked && record.host_native_evidence) {
    parts.push('### Host-native evidence links', '', fence(record.host_native_evidence), '');
  }

  if (masked) {
    parts.push('### Ordinary host chronology — condition masked', '');
    const hostEvents = (record.record?.events ?? []).filter((event) => event.channel === 'host');
    for (const [index, event] of hostEvents.entries()) parts.push(maskedHostEvent(event, index));
  } else {
    parts.splice(2, 0,
      `Runtime: \`${record.runtime?.id}@${record.runtime?.version}\`  `,
      `Semantic state: \`${record.semantic_status}\`  `,
      `Host revision: \`${record.host_revision ?? record.host?.revision ?? 'unknown'}\`  `
    );
    parts.push('### Complete chronological trace — host + QL semantics/operators', '');
    for (const [index, event] of (record.record?.events ?? []).entries()) parts.push(unmaskedEvent(event, index));
  }
  return parts.join('\n');
}

function header(manifest, assessment, { masked }) {
  const invalid = assessment.valid
    ? ['> **Comparison evidence status: VALID FOR HUMAN COMPARISON.**', '']
    : ['> **Comparison evidence status: INVALID / INCOMPLETE. Do not compare these conditions as benchmark evidence.**', '', ...assessment.reasons.map((reason) => `> - ${reason}`), ''];
  const lines = [
    masked ? '# Series 1 v0.1 — PASS A: condition-masked human review' : '# Series 1 v0.1 — PASS B: unmasked QL human review',
    '',
    ...invalid,
    `Benchmark: \`${manifest.benchmark}\`  `,
    `Benchmark revision: \`${manifest.benchmark_revision ?? 'unknown'}\`  `,
    `Host: \`${manifest.host.id}\`  `,
    `Model: \`${manifest.model.provider}:${manifest.model.id}\`  `,
    `Determination: **${manifest.determination}**`,
    ''
  ];
  if (!masked) {
    lines.push(
      `Host path: \`${manifest.host.real_framework_path}\`  `,
      `Runner revision: \`${manifest.runner_revision ?? 'unknown'}\`  `,
      `Review contract revision: \`${manifest.review_contract_revision ?? 'unknown'}\`  `,
      ''
    );
  }
  return lines;
}

function sharedReviewMaterial(manifest, { masked }) {
  const lines = [
    '## Exact benchmark prompt', '', fence(manifest.review.prompt, 'text'), '',
    '## Success / constraint text', '', manifest.review.success_conditions.map((item) => `- ${item}`).join('\n'), '',
    '## Verification protocol', '', manifest.review.verification_protocol.map((item) => `- ${item}`).join('\n'), '',
    '## Human review focus', '', manifest.review.focus.map((item) => `- ${item}`).join('\n'), ''
  ];
  if (!masked) {
    lines.push(
      '## Human-only source/reference anchors', '',
      '_Reviewer-only material. It is not part of candidate-visible context._', '',
      manifest.review.human_reference.map((item) => `- ${item}`).join('\n'), '',
      '## Held-constant evidence', '', fence(manifest.held_constant), '',
      '## Process summary — descriptive only', '', fence(manifest.process_summary), ''
    );
  } else {
    lines.push(
      '## Pass A boundary', '',
      'Condition identity, runtime IDs, semantic/closure state, QL event names, controller payloads, QL-only model envelopes, human answer/reference anchors, host-native observer links, and the neutral-label mapping are intentionally withheld from this artifact. Tool calls/results, workspace effects, verifier facts, final outcome and descriptive friction metrics remain visible.', '',
      '_The neutral-label mapping is emitted separately as machine evidence and must not be opened during Pass A._', ''
    );
  }
  return lines;
}

export function renderMaskedReview(manifest) {
  const safe = sanitizeEvidence(manifest);
  const assessment = assessComparisonEvidence(safe);
  const mask = buildMaskMapping(safe);
  const lines = [...header(safe, assessment, { masked: true }), ...sharedReviewMaterial(safe, { masked: true })];
  for (const record of safe.records ?? []) {
    const heading = mask.mapping?.[record.repetition]?.[record.condition] ?? 'Candidate ?';
    lines.push(renderPortableRecord(record, heading, { masked: true }), '');
  }
  lines.push('---', '', 'Pass A records ordinary behaviour only. Do not infer a winner from counts; record concrete observations and proceed to Pass B only after this review is complete.');
  return `${lines.join('\n')}\n`;
}

export function renderUnmaskedReview(manifest) {
  const safe = sanitizeEvidence(manifest);
  const assessment = assessComparisonEvidence(safe);
  const lines = [...header(safe, assessment, { masked: false }), ...sharedReviewMaterial(safe, { masked: false })];
  for (const record of safe.records ?? []) lines.push(renderPortableRecord(record, record.condition, { masked: false }), '');
  lines.push('---', '', 'Pass B reveals QL mechanics after ordinary behaviour review. Additional recurrence/operator activity is evidence to interpret, not a quality score. Human determination remains open.');
  return `${lines.join('\n')}\n`;
}

export function renderMaskMapping(manifest) {
  return `${JSON.stringify(sanitizeEvidence(buildMaskMapping(manifest)), null, 2)}\n`;
}

export function reviewEvidenceAssessment(manifest) {
  return assessComparisonEvidence(sanitizeEvidence(manifest));
}
