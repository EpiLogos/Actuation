export const EPISTEMIC_RECORD_SCHEMA = 'actuation.epistemic-record/v0';

export const EPISTEMIC_RECORD_KINDS = Object.freeze([
  'EpistemicCorpus',
  'EpistemicAnnotation',
  'DisclosureTrace',
  'InteriorObservation',
  'InteriorIntervention',
  'CultivationRun',
  'EpistemicEvaluation',
  'StructuralFinding',
]);

export const EPISTEMIC_ACCESS_KINDS = Object.freeze([
  'behavioural',
  'output_state',
  'internal_read',
  'internal_write',
  'causal',
  'learning',
]);

const PROVENANCE_KINDS = new Set(['source', 'human', 'agent', 'transformed', 'synthetic']);
const ACCESS_STATES = new Set(['available', 'unavailable', 'not-assessed']);
const RECORD_FIELDS = new Set([
  'schema', 'record_kind', 'record_ref', 'recorded_at', 'provenance', 'access',
  'subject_refs', 'model_ref', 'checkpoint_ref', 'method_refs', 'coordinate_refs',
  'derivation_refs', 'evidence_refs', 'payload',
]);

function object(value, name) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new TypeError(`${name} must be an object`);
  return value;
}

function exactFields(value, allowed, name) {
  for (const key of Object.keys(value)) {
    if (!allowed.has(key)) throw new TypeError(`${name}.${key} is not declared`);
  }
}

function text(value, name) {
  if (typeof value !== 'string' || value.trim() === '') throw new TypeError(`${name} must be a non-empty string`);
  return value;
}

function refs(value, name, { required = false } = {}) {
  if (value == null) {
    if (required) throw new TypeError(`${name} is required`);
    return [];
  }
  if (!Array.isArray(value) || value.some((entry) => typeof entry !== 'string' || entry.trim() === '')) {
    throw new TypeError(`${name} must be an array of non-empty references`);
  }
  if (required && value.length === 0) throw new TypeError(`${name} must not be empty`);
  return [...value];
}

function provenance(value) {
  const input = object(value, 'provenance');
  exactFields(input, new Set(['kind', 'actor_ref', 'generator_ref', 'source_refs', 'derivation_refs']), 'provenance');
  if (!PROVENANCE_KINDS.has(input.kind)) throw new TypeError(`unknown provenance.kind ${input.kind}`);
  const result = {
    kind: input.kind,
    source_refs: refs(input.source_refs, 'provenance.source_refs'),
    derivation_refs: refs(input.derivation_refs, 'provenance.derivation_refs'),
  };
  if (input.actor_ref != null) result.actor_ref = text(input.actor_ref, 'provenance.actor_ref');
  if (input.generator_ref != null) result.generator_ref = text(input.generator_ref, 'provenance.generator_ref');
  if (input.kind === 'source' && result.source_refs.length === 0) throw new TypeError('source provenance requires source_refs');
  if (['human', 'agent'].includes(input.kind) && !result.actor_ref) throw new TypeError(`${input.kind} provenance requires actor_ref`);
  if (input.kind === 'transformed' && (result.source_refs.length === 0 || result.derivation_refs.length === 0)) {
    throw new TypeError('transformed provenance requires source_refs and derivation_refs');
  }
  if (input.kind === 'synthetic' && !result.generator_ref) throw new TypeError('synthetic provenance requires generator_ref');
  return result;
}

function accessDeclaration(value, name) {
  const input = object(value, name);
  exactFields(input, new Set(['state', 'reason', 'method_refs', 'evidence_refs']), name);
  if (!ACCESS_STATES.has(input.state)) throw new TypeError(`${name}.state is invalid`);
  const result = {
    state: input.state,
    method_refs: refs(input.method_refs, `${name}.method_refs`),
    evidence_refs: refs(input.evidence_refs, `${name}.evidence_refs`),
  };
  if (input.reason != null) result.reason = text(input.reason, `${name}.reason`);
  if (input.state === 'available' && (result.method_refs.length === 0 || result.evidence_refs.length === 0)) {
    throw new TypeError(`${name} available requires method_refs and evidence_refs`);
  }
  if (input.state !== 'available' && !result.reason) throw new TypeError(`${name} ${input.state} requires a reason`);
  if (input.state === 'not-assessed' && (result.method_refs.length > 0 || result.evidence_refs.length > 0)) {
    throw new TypeError(`${name} not-assessed cannot carry method or evidence claims`);
  }
  return result;
}

function accessField(value) {
  const input = object(value, 'access');
  exactFields(input, new Set(EPISTEMIC_ACCESS_KINDS), 'access');
  const result = {};
  for (const kind of EPISTEMIC_ACCESS_KINDS) {
    result[kind] = accessDeclaration(input[kind], `access.${kind}`);
  }
  return result;
}

const PAYLOAD_FIELDS = Object.freeze({
  EpistemicCorpus: ['corpus_ref', 'revision_ref', 'item_refs'],
  EpistemicAnnotation: ['annotation_ref', 'annotation_kind', 'content_ref', 'source_span_refs'],
  DisclosureTrace: ['trace_ref', 'condition_ref', 'step_refs'],
  InteriorObservation: ['observation_ref', 'observed_quantity', 'coordinate_ref'],
  InteriorIntervention: ['intervention_ref', 'operation', 'before_ref', 'after_ref'],
  CultivationRun: ['run_ref', 'input_corpus_ref', 'condition_ref', 'result_state_ref'],
  EpistemicEvaluation: ['evaluation_ref', 'criteria_refs', 'evaluation_state'],
  StructuralFinding: ['finding_ref', 'statement', 'finding_state'],
});

function payload(kind, value) {
  const input = object(value, 'payload');
  const allowed = new Set(PAYLOAD_FIELDS[kind]);
  exactFields(input, allowed, 'payload');
  for (const field of allowed) {
    if (!(field in input)) throw new TypeError(`payload.${field} is required for ${kind}`);
  }
  const result = {};
  for (const [field, entry] of Object.entries(input)) {
    result[field] = field.endsWith('_refs') ? refs(entry, `payload.${field}`, { required: true }) : text(entry, `payload.${field}`);
  }
  if (kind === 'EpistemicEvaluation' && !['declared', 'completed', 'inconclusive'].includes(result.evaluation_state)) {
    throw new TypeError('payload.evaluation_state is invalid');
  }
  if (kind === 'StructuralFinding' && !['hypothesis', 'supported', 'refuted', 'inconclusive'].includes(result.finding_state)) {
    throw new TypeError('payload.finding_state is invalid');
  }
  return result;
}

export function validateEpistemicRecord(value) {
  const input = object(value, 'record');
  exactFields(input, RECORD_FIELDS, 'record');
  if (input.schema !== EPISTEMIC_RECORD_SCHEMA) throw new TypeError(`record.schema must be ${EPISTEMIC_RECORD_SCHEMA}`);
  if (!EPISTEMIC_RECORD_KINDS.includes(input.record_kind)) throw new TypeError(`unknown record_kind ${input.record_kind}`);
  const recordedAt = text(input.recorded_at, 'recorded_at');
  if (!Number.isFinite(Date.parse(recordedAt)) || !recordedAt.endsWith('Z')) throw new TypeError('recorded_at must be an ISO-8601 UTC timestamp');
  const result = {
    schema: EPISTEMIC_RECORD_SCHEMA,
    record_kind: input.record_kind,
    record_ref: text(input.record_ref, 'record_ref'),
    recorded_at: recordedAt,
    provenance: provenance(input.provenance),
    access: accessField(input.access),
    subject_refs: refs(input.subject_refs, 'subject_refs', { required: true }),
    method_refs: refs(input.method_refs, 'method_refs'),
    coordinate_refs: refs(input.coordinate_refs, 'coordinate_refs'),
    derivation_refs: refs(input.derivation_refs, 'derivation_refs'),
    evidence_refs: refs(input.evidence_refs, 'evidence_refs'),
    payload: payload(input.record_kind, input.payload),
  };
  if (input.model_ref != null) result.model_ref = text(input.model_ref, 'model_ref');
  if (input.checkpoint_ref != null) result.checkpoint_ref = text(input.checkpoint_ref, 'checkpoint_ref');
  if (input.checkpoint_ref != null && input.model_ref == null) throw new TypeError('checkpoint_ref requires model_ref');
  if (['InteriorObservation', 'InteriorIntervention', 'CultivationRun'].includes(input.record_kind)) {
    if (!result.model_ref || !result.checkpoint_ref || result.method_refs.length === 0 || result.coordinate_refs.length === 0) {
      throw new TypeError(`${input.record_kind} requires model_ref, checkpoint_ref, method_refs and coordinate_refs`);
    }
  }
  if (input.record_kind === 'InteriorObservation' && result.access.internal_read.state !== 'available') {
    throw new TypeError('InteriorObservation requires evidenced internal_read access');
  }
  if (input.record_kind === 'InteriorIntervention' && !['available'].includes(result.access.internal_write.state)
      && !['available'].includes(result.access.causal.state)) {
    throw new TypeError('InteriorIntervention requires evidenced internal_write or causal access');
  }
  if (input.record_kind === 'StructuralFinding' && ['supported', 'refuted'].includes(result.payload.finding_state)
      && result.evidence_refs.length === 0) {
    throw new TypeError('supported or refuted StructuralFinding requires evidence_refs');
  }
  return result;
}
