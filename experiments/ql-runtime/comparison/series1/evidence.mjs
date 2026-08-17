import { REQUIRED_HELD_CONSTANTS, stableDigest } from './contract.mjs';

const SECRET_KEY = /(api[_-]?key|authorization|credential|password|secret|access[_-]?token|refresh[_-]?token|bearer)/i;
const REDACTED = '[REDACTED]';

function configuredSecrets() {
  return [process.env.DEEPSEEK_API_KEY]
    .filter((value) => typeof value === 'string' && value.length > 0);
}

function redactString(value, secrets) {
  let text = value;
  for (const secret of secrets) text = text.split(secret).join(REDACTED);
  return text;
}

export function sanitizeEvidence(value, { secrets = configuredSecrets() } = {}) {
  if (typeof value === 'string') return redactString(value, secrets);
  if (Array.isArray(value)) return value.map((entry) => sanitizeEvidence(entry, { secrets }));
  if (!value || typeof value !== 'object') return value;
  return Object.fromEntries(Object.entries(value).map(([key, child]) => [
    key,
    SECRET_KEY.test(key) && key !== 'credential_contract'
      ? REDACTED
      : sanitizeEvidence(child, { secrets })
  ]));
}

export function assertNoConfiguredSecrets(value, { secrets = configuredSecrets() } = {}) {
  const serialized = JSON.stringify(value);
  for (const secret of secrets) {
    if (serialized.includes(secret)) {
      const error = new Error('Series 1 evidence contains a configured credential value.');
      error.code = 'SERIES1_SECRET_LEAK';
      throw error;
    }
  }
  return value;
}

function recordCompleteness(record) {
  const missing = [];
  const requireValue = (name, value) => {
    if (value === undefined || value === null) missing.push(name);
  };
  requireValue('prompt', record?.prompt);
  requireValue('success_conditions', record?.success_conditions);
  requireValue('starting_workspace', record?.starting_workspace);
  requireValue('final_workspace', record?.final_workspace);
  requireValue('verification', record?.verification);
  requireValue('outcome', record?.outcome);
  requireValue('model', record?.model);
  requireValue('host_revision', record?.host_revision ?? record?.host?.revision);
  requireValue('benchmark_revision', record?.benchmark_revision);
  requireValue('task_revision', record?.task_revision);
  requireValue('runner_revision', record?.runner_revision);
  requireValue('review_contract_revision', record?.review_contract_revision);
  requireValue('elapsed_ms', record?.elapsed_ms);
  requireValue('model_calls', record?.model_calls);
  requireValue('capability_calls', record?.capability_calls);
  requireValue('total_tokens', record?.total_tokens);
  if (!Array.isArray(record?.record?.events)) missing.push('record.events');
  else {
    const hostEvents = record.record.events.filter((event) => event.channel === 'host');
    const modelRequests = hostEvents.filter((event) => event.event_type === 'model_requested');
    const modelReturns = hostEvents.filter((event) => event.event_type === 'model_returned');
    const capabilityRequests = hostEvents.filter((event) => event.event_type === 'capability_requested');
    const capabilityReturns = hostEvents.filter((event) => event.event_type === 'capability_returned');
    if (modelRequests.length !== modelReturns.length) missing.push('balanced model request/return chronology');
    if (capabilityRequests.length !== capabilityReturns.length) missing.push('balanced capability request/return chronology');
    for (const [index, event] of record.record.events.entries()) {
      if (event.record_index !== index) missing.push(`record.events[${index}].record_index`);
    }
  }
  return { complete: missing.length === 0, missing };
}

export function assessComparisonEvidence(manifest) {
  const reasons = [];
  for (const field of REQUIRED_HELD_CONSTANTS) {
    if (manifest?.held_constant?.[field] !== true) reasons.push(`held constant mismatch: ${field}`);
  }
  if (manifest?.held_constant?.valid !== true) reasons.push('held-constant comparison is invalid');
  const records = (manifest?.records ?? []).map((record) => ({
    condition: record.condition,
    repetition: record.repetition,
    ...recordCompleteness(record)
  }));
  for (const record of records) {
    for (const missing of record.missing) reasons.push(`${record.condition ?? 'unknown'} r${record.repetition ?? '?'} missing ${missing}`);
  }
  return {
    valid: reasons.length === 0,
    reasons: [...new Set(reasons)],
    records
  };
}

function labelForIndex(index) {
  return `Candidate ${String.fromCharCode(65 + index)}`;
}

export function buildMaskMapping(manifest) {
  const mapping = {};
  const repetitions = [...new Set((manifest.records ?? []).map((record) => record.repetition))].sort((a, b) => a - b);
  for (const repetition of repetitions) {
    const records = manifest.records.filter((record) => record.repetition === repetition);
    const seed = `${manifest.benchmark_revision ?? manifest.benchmark}:${manifest.host?.id}:${manifest.task?.id}:${repetition}`;
    const ordered = [...records].sort((a, b) => stableDigest(`${seed}:${a.condition}`).localeCompare(stableDigest(`${seed}:${b.condition}`)));
    mapping[repetition] = Object.fromEntries(ordered.map((record, index) => [record.condition, labelForIndex(index)]));
  }
  return {
    schema: 'ql-series1-mask-map/0.1',
    benchmark_revision: manifest.benchmark_revision ?? null,
    host: manifest.host?.id ?? null,
    task: manifest.task?.id ?? null,
    mapping
  };
}

export function attachHostNativeEvidence(record, hostNativeEvidence = null) {
  return {
    ...record,
    host_native_evidence: hostNativeEvidence ? sanitizeEvidence(hostNativeEvidence) : null
  };
}
