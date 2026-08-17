import crypto from 'node:crypto';

export const SERIES1_SCHEMA = 'ql-series1-run/0.3';
export const CONDITIONS = Object.freeze(['classic', 'ql-direct', 'ql-deep']);
export const HOSTS = Object.freeze(['pi', 'pydantic-ai', 'native', 'dsh']);
export const DETERMINATION = 'pending-human-review';

export const SERIES1_CAPABILITY_CONTRACT = Object.freeze([
  Object.freeze({ name: 'list_files', description: 'List one directory inside the task workspace.', args: Object.freeze({ path: 'optional relative directory, default .' }) }),
  Object.freeze({ name: 'read_file', description: 'Read one UTF-8 file inside the task workspace.', args: Object.freeze({ path: 'required relative file path' }) }),
  Object.freeze({ name: 'write_file', description: 'Replace one UTF-8 file inside the task workspace.', args: Object.freeze({ path: 'required relative file path', content: 'complete new file content' }) }),
  Object.freeze({ name: 'run_tests', description: 'Run Node test files inside the task workspace. With no files, run the Node test discovery.', args: Object.freeze({ files: 'optional array of relative test paths' }) })
]);

export const REQUIRED_HELD_CONSTANTS = Object.freeze([
  'prompt',
  'success_constraints',
  'start_state',
  'model',
  'capabilities',
  'verification',
  'budget',
  'host_revision',
  'host_composition',
  'network_policy',
  'benchmark_revision',
  'task_revision',
  'runner_revision',
  'review_contract_revision'
]);

function canonicalize(value) {
  if (Array.isArray(value)) return value.map(canonicalize);
  if (value && typeof value === 'object') {
    return Object.fromEntries(Object.keys(value).sort().map((key) => [key, canonicalize(value[key])]));
  }
  return value;
}

export function stableDigest(value) {
  return crypto.createHash('sha256').update(JSON.stringify(canonicalize(value))).digest('hex');
}

const HELD_SELECTORS = Object.freeze({
  prompt: (record) => record.prompt_digest,
  success_constraints: (record) => record.success_constraints_digest,
  start_state: (record) => record.start_state_digest,
  model: (record) => record.model,
  capabilities: (record) => record.capability_contract_digest ?? record.capability_digest,
  verification: (record) => record.verification_protocol_digest,
  budget: (record) => record.execution_budget_digest,
  host_revision: (record) => record.host_revision ?? record.host?.revision,
  host_composition: (record) => record.host_composition_fingerprint ?? null,
  network_policy: (record) => record.network_policy_digest,
  benchmark_revision: (record) => record.benchmark_revision,
  task_revision: (record) => record.task_revision,
  runner_revision: (record) => record.runner_revision,
  review_contract_revision: (record) => record.review_contract_revision
});

export function compareHeldConstant(records) {
  if (!Array.isArray(records) || records.length === 0) {
    return {
      ...Object.fromEntries(REQUIRED_HELD_CONSTANTS.map((field) => [field, false])),
      valid: false,
      mismatches: REQUIRED_HELD_CONSTANTS.map((field) => ({ field, distinct_values: 0 }))
    };
  }

  const result = {};
  const mismatches = [];
  for (const field of REQUIRED_HELD_CONSTANTS) {
    const selector = HELD_SELECTORS[field];
    const variants = new Set(records.map((record) => stableDigest(selector(record))));
    const held = variants.size === 1;
    result[field] = held;
    if (!held) mismatches.push({ field, distinct_values: variants.size });
  }
  return { ...result, valid: mismatches.length === 0, mismatches };
}

export function assertLiveManifest(manifest) {
  const errors = [];
  if (manifest?.schema !== SERIES1_SCHEMA) errors.push(`schema must be ${SERIES1_SCHEMA}`);
  if (manifest?.provider_mode !== 'live') errors.push('provider_mode must be live');
  if (!HOSTS.includes(manifest?.host?.id)) errors.push(`host.id must name one of: ${HOSTS.join(', ')}`);
  if (!manifest?.host?.real_framework_path) errors.push('host.real_framework_path is required');
  if (!manifest?.host?.revision) errors.push('host.revision is required');
  if (!manifest?.model?.id || !manifest?.model?.provider) errors.push('concrete model provider/id is required');
  if (!Array.isArray(manifest?.conditions) || !CONDITIONS.every((condition) => manifest.conditions.includes(condition))) {
    errors.push('classic, ql-direct, and ql-deep conditions are required');
  }
  for (const field of REQUIRED_HELD_CONSTANTS) {
    if (manifest?.held_constant?.[field] !== true) errors.push(`held constant '${field}' must be evidenced`);
  }
  if (manifest?.held_constant?.valid !== true) {
    const fields = manifest?.held_constant?.mismatches?.map((entry) => entry.field).join(', ') || 'unknown';
    errors.push(`matched set has held-constant mismatch: ${fields}`);
  }
  if (manifest?.fixture_provider === true) errors.push('fixture providers are not evidence eligible');
  if (manifest?.determination !== DETERMINATION) errors.push(`determination must begin as ${DETERMINATION}`);
  if (!manifest?.review?.prompt || !Array.isArray(manifest?.review?.focus)) errors.push('human review prompt/focus must be retained');
  if (!Array.isArray(manifest?.records) || manifest.records.length === 0) errors.push('run records are required');
  for (const [index, record] of (manifest?.records ?? []).entries()) {
    if (!record?.benchmark_revision) errors.push(`records[${index}].benchmark_revision is required`);
    if (!record?.task_revision) errors.push(`records[${index}].task_revision is required`);
    if (!record?.runner_revision) errors.push(`records[${index}].runner_revision is required`);
    if (!record?.review_contract_revision) errors.push(`records[${index}].review_contract_revision is required`);
    if (!record?.host_revision && !record?.host?.revision) errors.push(`records[${index}].host_revision is required`);
    if ((record?.host?.id ?? manifest?.host?.id) === 'dsh' && !record?.host_composition_fingerprint) {
      errors.push(`records[${index}].host_composition_fingerprint is required for dsh`);
    }
  }
  if (errors.length) {
    const error = new Error(`Series 1 manifest is not evidence eligible:\n- ${errors.join('\n- ')}`);
    error.code = 'SERIES1_NOT_ELIGIBLE';
    error.errors = errors;
    throw error;
  }
  return manifest;
}
