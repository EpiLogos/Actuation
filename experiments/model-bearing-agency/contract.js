export const MODEL_CONDITION_SCHEMA = 'actuation.model-condition/v0';
export const MODEL_EXPERIMENT_SCHEMA = 'actuation.model-condition-experiment/v0';

export const MATERIALISATION_MODES = Object.freeze([
  'embedded',
  'process',
  'service',
  'distributed-service',
  'opaque-service'
]);

export const WORLD_RELATIONS = Object.freeze([
  'inside-world',
  'outside-world',
  'spans-worlds',
  'opaque'
]);

export const MODEL_SURFACE_KINDS = Object.freeze([
  'in-process',
  'cli',
  'stdio',
  'http',
  'provider-api',
  'opaque'
]);

export const HARNESS_MODEL_RELATIONS = Object.freeze([
  'embedded',
  'external-surface',
  'none'
]);

export const INTERIOR_ACCESS_LEVELS = Object.freeze([
  'behavioural',
  'output-state',
  'internal-read',
  'internal-write',
  'causal',
  'learning'
]);

export const CONDITION_AXES = Object.freeze([
  'model',
  'engine',
  'materialisation',
  'surface',
  'harness',
  'session',
  'inference-access',
  'control-access',
  'interior-access',
  'policy'
]);

const isObject = (value) => value !== null && typeof value === 'object' && !Array.isArray(value);

function requireRef(value, field) {
  if (typeof value !== 'string' || value.trim() === '') {
    throw new TypeError(`${field} must be a non-empty opaque ref`);
  }
  return value;
}

function optionalRef(value, field) {
  if (value === undefined || value === null) return null;
  return requireRef(value, field);
}

function uniqueStrings(value, field, { min = 0 } = {}) {
  if (!Array.isArray(value)) throw new TypeError(`${field} must be an array`);
  const normalized = value.map((entry, index) => {
    if (typeof entry !== 'string' || entry.trim() === '') {
      throw new TypeError(`${field}[${index}] must be a non-empty string`);
    }
    return entry;
  });
  if (normalized.length < min) throw new TypeError(`${field} must contain at least ${min} item(s)`);
  if (new Set(normalized).size !== normalized.length) throw new TypeError(`${field} must not contain duplicates`);
  return normalized;
}

function enumValue(value, allowed, field) {
  if (!allowed.includes(value)) {
    throw new TypeError(`${field} must be one of: ${allowed.join(', ')}`);
  }
  return value;
}

function cloneMetadata(value, field) {
  if (value === undefined) return {};
  if (!isObject(value)) throw new TypeError(`${field} must be an object`);
  return structuredClone(value);
}

export function createModelConditionReceipt(input) {
  if (!isObject(input)) throw new TypeError('model condition input must be an object');

  const model = input.model;
  const engine = input.engine;
  const materialisation = input.materialisation;
  const surface = input.surface;
  const harness = input.harness ?? {};
  const access = input.access;

  if (!isObject(model)) throw new TypeError('model must be an object');
  if (!isObject(engine)) throw new TypeError('engine must be an object');
  if (!isObject(materialisation)) throw new TypeError('materialisation must be an object');
  if (!isObject(surface)) throw new TypeError('surface must be an object');
  if (!isObject(harness)) throw new TypeError('harness must be an object');
  if (!isObject(access)) throw new TypeError('access must be an object');

  const receipt = {
    schema: MODEL_CONDITION_SCHEMA,
    receipt_ref: requireRef(input.receipt_ref, 'receipt_ref'),
    actuation_ref: requireRef(input.actuation_ref, 'actuation_ref'),
    world_ref: requireRef(input.world_ref, 'world_ref'),
    agent_ref: requireRef(input.agent_ref, 'agent_ref'),
    agency_ref: requireRef(input.agency_ref, 'agency_ref'),
    model: {
      model_ref: requireRef(model.model_ref, 'model.model_ref'),
      variant_ref: optionalRef(model.variant_ref, 'model.variant_ref'),
      artifact_refs: uniqueStrings(model.artifact_refs ?? [], 'model.artifact_refs')
    },
    engine: {
      engine_ref: requireRef(engine.engine_ref, 'engine.engine_ref'),
      source_provenance_refs: uniqueStrings(engine.source_provenance_refs ?? [], 'engine.source_provenance_refs')
    },
    materialisation: {
      mode: enumValue(materialisation.mode, MATERIALISATION_MODES, 'materialisation.mode'),
      world_relation: enumValue(materialisation.world_relation, WORLD_RELATIONS, 'materialisation.world_relation'),
      material_binding_refs: uniqueStrings(materialisation.material_binding_refs ?? [], 'materialisation.material_binding_refs')
    },
    surface: {
      kind: enumValue(surface.kind, MODEL_SURFACE_KINDS, 'surface.kind'),
      contract_ref: optionalRef(surface.contract_ref, 'surface.contract_ref'),
      binding_ref: optionalRef(surface.binding_ref, 'surface.binding_ref')
    },
    harness: {
      harness_ref: optionalRef(harness.harness_ref, 'harness.harness_ref'),
      harness_composition_ref: optionalRef(harness.harness_composition_ref, 'harness.harness_composition_ref'),
      agent_session_ref: optionalRef(harness.agent_session_ref, 'harness.agent_session_ref'),
      model_relation: enumValue(harness.model_relation ?? 'none', HARNESS_MODEL_RELATIONS, 'harness.model_relation')
    },
    access: {
      inference: uniqueStrings(access.inference ?? [], 'access.inference', { min: 1 }),
      control: uniqueStrings(access.control ?? [], 'access.control'),
      interior: enumValue(access.interior, INTERIOR_ACCESS_LEVELS, 'access.interior')
    },
    policy_boundary_refs: uniqueStrings(input.policy_boundary_refs ?? [], 'policy_boundary_refs'),
    evidence_refs: uniqueStrings(input.evidence_refs ?? [], 'evidence_refs'),
    provenance_refs: uniqueStrings(input.provenance_refs ?? [], 'provenance_refs'),
    provider_metadata: cloneMetadata(input.provider_metadata, 'provider_metadata')
  };

  if (receipt.harness.model_relation !== 'none' && receipt.harness.harness_ref === null) {
    throw new TypeError('harness.harness_ref is required when harness.model_relation is not none');
  }

  if (receipt.harness.model_relation === 'none' && (
    receipt.harness.harness_ref !== null ||
    receipt.harness.harness_composition_ref !== null ||
    receipt.harness.agent_session_ref !== null
  )) {
    throw new TypeError('harness refs require an explicit embedded or external-surface model relation');
  }

  return deepFreeze(receipt);
}

export function modelConditionSemanticIdentity(receipt) {
  const condition = createModelConditionReceipt(receipt);
  return deepFreeze({
    actuation_ref: condition.actuation_ref,
    world_ref: condition.world_ref,
    agent_ref: condition.agent_ref,
    agency_ref: condition.agency_ref,
    model_ref: condition.model.model_ref,
    variant_ref: condition.model.variant_ref,
    harness_ref: condition.harness.harness_ref
  });
}

function comparableValue(receipt, axis) {
  switch (axis) {
    case 'model':
      return [receipt.model.model_ref, receipt.model.variant_ref, ...receipt.model.artifact_refs];
    case 'engine':
      return [receipt.engine.engine_ref, ...receipt.engine.source_provenance_refs];
    case 'materialisation':
      return [receipt.materialisation.mode, receipt.materialisation.world_relation, ...receipt.materialisation.material_binding_refs, receipt.surface.binding_ref];
    case 'surface':
      return [receipt.surface.kind, receipt.surface.contract_ref];
    case 'harness':
      return [receipt.harness.harness_ref, receipt.harness.harness_composition_ref, receipt.harness.model_relation];
    case 'session':
      return receipt.harness.agent_session_ref;
    case 'inference-access':
      return receipt.access.inference;
    case 'control-access':
      return receipt.access.control;
    case 'interior-access':
      return receipt.access.interior;
    case 'policy':
      return receipt.policy_boundary_refs;
    default:
      throw new TypeError(`unknown condition axis: ${axis}`);
  }
}

function sameValue(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

export function compareModelConditions(leftInput, rightInput) {
  const left = createModelConditionReceipt(leftInput);
  const right = createModelConditionReceipt(rightInput);

  const changed_axes = CONDITION_AXES.filter((axis) => !sameValue(
    comparableValue(left, axis),
    comparableValue(right, axis)
  ));

  const identity_fields = ['actuation_ref', 'world_ref', 'agent_ref', 'agency_ref'];
  const identity_drift = identity_fields.filter((field) => left[field] !== right[field]);

  return deepFreeze({
    left_receipt_ref: left.receipt_ref,
    right_receipt_ref: right.receipt_ref,
    changed_axes,
    identity_drift,
    same_agent: left.agent_ref === right.agent_ref,
    same_agency: left.agency_ref === right.agency_ref,
    same_world: left.world_ref === right.world_ref,
    same_harness: left.harness.harness_ref === right.harness.harness_ref,
    same_model: left.model.model_ref === right.model.model_ref && left.model.variant_ref === right.model.variant_ref
  });
}

export function createMatchedConditionExperiment(input) {
  if (!isObject(input)) throw new TypeError('experiment input must be an object');
  const baseline = createModelConditionReceipt(input.baseline);
  const candidate = createModelConditionReceipt(input.candidate);
  const comparison = compareModelConditions(baseline, candidate);

  if (comparison.identity_drift.length > 0) {
    throw new TypeError(`matched condition experiment cannot drift situated identity: ${comparison.identity_drift.join(', ')}`);
  }

  const held_constant_axes = uniqueStrings(input.held_constant_axes ?? [], 'held_constant_axes');
  const intended_varied_axes = uniqueStrings(input.intended_varied_axes ?? [], 'intended_varied_axes', { min: 1 });
  for (const axis of [...held_constant_axes, ...intended_varied_axes]) {
    enumValue(axis, CONDITION_AXES, 'experiment axis');
  }

  const overlap = held_constant_axes.filter((axis) => intended_varied_axes.includes(axis));
  if (overlap.length > 0) {
    throw new TypeError(`experiment axes cannot be both held constant and varied: ${overlap.join(', ')}`);
  }

  const broken_constants = held_constant_axes.filter((axis) => comparison.changed_axes.includes(axis));
  if (broken_constants.length > 0) {
    throw new TypeError(`held-constant axes changed: ${broken_constants.join(', ')}`);
  }

  const missing_variations = intended_varied_axes.filter((axis) => !comparison.changed_axes.includes(axis));
  if (missing_variations.length > 0) {
    throw new TypeError(`intended varied axes did not change: ${missing_variations.join(', ')}`);
  }

  return deepFreeze({
    schema: MODEL_EXPERIMENT_SCHEMA,
    experiment_ref: requireRef(input.experiment_ref, 'experiment_ref'),
    baseline_receipt_ref: baseline.receipt_ref,
    candidate_receipt_ref: candidate.receipt_ref,
    held_constant_axes,
    intended_varied_axes,
    observed_changed_axes: comparison.changed_axes,
    evidence_refs: uniqueStrings(input.evidence_refs ?? [], 'evidence_refs'),
    return_refs: uniqueStrings(input.return_refs ?? [], 'return_refs'),
    provenance_refs: uniqueStrings(input.provenance_refs ?? [], 'provenance_refs')
  });
}

function deepFreeze(value) {
  if (value && typeof value === 'object' && !Object.isFrozen(value)) {
    Object.freeze(value);
    for (const nested of Object.values(value)) deepFreeze(nested);
  }
  return value;
}
