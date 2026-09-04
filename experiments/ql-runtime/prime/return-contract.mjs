export const PRIME_RETURN_SCHEMA = 'actuation.prime-return/v0';

function nonEmpty(value, name) {
  if (typeof value !== 'string' || value.trim() === '') {
    throw new Error(`${name} must be a non-empty string.`);
  }
  return value;
}

function stringArray(value, name) {
  if (value == null) return [];
  if (!Array.isArray(value) || value.some((entry) => typeof entry !== 'string')) {
    throw new Error(`${name} must be an array of strings.`);
  }
  return [...value];
}

export function createPrimeReturn({
  subject_ref,
  relation_to_parent,
  determination,
  result,
  difference,
  evidence_refs = [],
  ql_reading_refs = [],
  unresolved = [],
  next_relations = [],
  child_ref = null,
  parent_ref = null,
  provenance = {}
}) {
  const value = {
    schema: PRIME_RETURN_SCHEMA,
    subject_ref: nonEmpty(subject_ref, 'subject_ref'),
    relation_to_parent: nonEmpty(relation_to_parent, 'relation_to_parent'),
    determination: nonEmpty(determination, 'determination'),
    result: nonEmpty(result, 'result'),
    difference: nonEmpty(difference, 'difference'),
    evidence_refs: stringArray(evidence_refs, 'evidence_refs'),
    ql_reading_refs: stringArray(ql_reading_refs, 'ql_reading_refs'),
    unresolved: stringArray(unresolved, 'unresolved'),
    next_relations: stringArray(next_relations, 'next_relations'),
    provenance: provenance && typeof provenance === 'object' && !Array.isArray(provenance) ? { ...provenance } : {}
  };
  if (child_ref != null) value.child_ref = nonEmpty(child_ref, 'child_ref');
  if (parent_ref != null) value.parent_ref = nonEmpty(parent_ref, 'parent_ref');
  return value;
}

export function assertPrimeReturn(value) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('Return must be an object.');
  if (value.schema !== PRIME_RETURN_SCHEMA) throw new Error(`Return schema must be ${PRIME_RETURN_SCHEMA}.`);
  return createPrimeReturn(value);
}
