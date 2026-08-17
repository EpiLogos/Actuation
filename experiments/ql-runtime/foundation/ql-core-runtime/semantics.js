export const QL_SPEC = 'ql-agent/0.1';
export const QL_SCHEMA_VERSION = '0.1.0-foundation';
export const DEFAULT_LENSES = Object.freeze(['L1', 'L4′']);
export const FACES = Object.freeze(['direct', 'conjugate']);
export const POSITIONS = Object.freeze(['P0', 'P1', 'P2', 'P3', 'P4', 'P5']);
export const RESIDUE_KIND_BY_POSITION = Object.freeze({
  P0: 'frame',
  P1: 'material',
  P2: 'effect',
  P3: 'form',
  P4: 'evaluation',
  P5: 'determination'
});

const SUCCESS_VALUES = new Set(['true', 'false', 'unknown', 'not_applicable']);

export class QLSemanticError extends Error {
  constructor(message, details = {}) {
    super(message);
    this.name = 'QLSemanticError';
    this.details = details;
  }
}

export function qlPosition(id) {
  if (!POSITIONS.includes(id)) {
    throw new QLSemanticError(`Invalid QL position '${id}'.`);
  }
  return Object.freeze({
    id,
    structuralKind: id === 'P0' || id === 'P5' ? 'implicate' : 'explicate'
  });
}

export function qlFace(id) {
  if (!FACES.includes(id)) {
    throw new QLSemanticError(`Invalid QL face '${id}'.`);
  }
  return id;
}

export function qlRelation(from, to) {
  const source = qlPosition(typeof from === 'string' ? from : from.id);
  const destination = qlPosition(typeof to === 'string' ? to : to.id);
  return Object.freeze({
    id: `R${source.id.slice(1)}${destination.id.slice(1)}`,
    from: source.id,
    to: destination.id
  });
}

export function createSuccessState(overrides = {}) {
  const state = {
    operational: 'unknown',
    artifact: 'unknown',
    task: 'unknown',
    circuit: 'false',
    conjugate_stability: 'not_applicable',
    ...overrides
  };
  for (const [key, value] of Object.entries(state)) {
    if (!SUCCESS_VALUES.has(String(value))) {
      throw new QLSemanticError(`Invalid success value '${value}' for '${key}'.`);
    }
    state[key] = String(value);
  }
  return state;
}
