const POSITION_IDS = Object.freeze(['P0', 'P1', 'P2', 'P3', 'P4', 'P5']);
const FAMILY_IDS = Object.freeze(['A', 'B', 'C']);
const D_LEVELS = Object.freeze(['D1', 'D2', 'D3']);

const RAW_FAMILIES = Object.freeze({
  A: Object.freeze([[0, 1], [2, 3], [4, 5]]),
  B: Object.freeze([[1, 2], [3, 4], [5, 0]]),
  C: Object.freeze([[0, 5], [1, 4], [2, 3]])
});

function freezePair([left, right]) {
  return Object.freeze([`P${left}`, `P${right}`]);
}

export const PAIRING_FAMILIES = Object.freeze(Object.fromEntries(
  Object.entries(RAW_FAMILIES).map(([family, pairs]) => [family, Object.freeze(pairs.map(freezePair))])
));

export const CANONICAL_CROSS_PASS = Object.freeze({
  D1: Object.freeze(POSITION_IDS.map((position) => Object.freeze([position, `${position}'`]))),
  D2: Object.freeze({
    transform: Object.freeze(POSITION_IDS.map((_, index) => Object.freeze([`P${index}`, `P${(index + 1) % 6}'`]))),
    require: Object.freeze(POSITION_IDS.map((_, index) => Object.freeze([`P${index}`, `P${(index + 5) % 6}'`]))),
    complete: Object.freeze(POSITION_IDS.map((_, index) => Object.freeze([`P${index}`, `P${5 - index}'`])))
  }),
  D3: Object.freeze(Object.fromEntries(
    FAMILY_IDS.map((family) => [family, Object.freeze(PAIRING_FAMILIES[family].map(([left, right]) => Object.freeze([`${left}'`, `${right}'`])))])
  ))
});

function assertFamily(family) {
  if (!FAMILY_IDS.includes(family)) {
    throw new TypeError(`Unknown QL pairing family '${family}'. Expected A, B, or C.`);
  }
  return family;
}

function assertPairIndex(pair) {
  if (!Number.isInteger(pair) || pair < 1 || pair > 3) {
    throw new TypeError(`QL pairing index must be 1, 2, or 3; received '${pair}'.`);
  }
  return pair;
}

function assertDLevel(level) {
  if (!D_LEVELS.includes(level)) {
    throw new TypeError(`Unknown QL D modulation '${level}'. Expected D1, D2, or D3.`);
  }
  return level;
}

export function prime(position) {
  if (!POSITION_IDS.includes(position)) {
    throw new TypeError(`Unknown QL position '${position}'.`);
  }
  return `${position}'`;
}

export function selectPair(family, pair) {
  assertFamily(family);
  assertPairIndex(pair);
  const direct = PAIRING_FAMILIES[family][pair - 1];
  const conjugate = Object.freeze(direct.map(prime));
  return Object.freeze({
    family,
    pair,
    direct,
    conjugate,
    provenance: Object.freeze({
      basis: 'ql-musical-derivation-v3 §§II-2.4..II-3.4',
      relation_kind: 'within-pass-pairing'
    })
  });
}

export function buildSquare(family, pair) {
  const selected = selectPair(family, pair);
  const elements = Object.freeze([...selected.direct, ...selected.conjugate]);
  return Object.freeze({
    id: `${family}-square-${pair}`,
    family,
    pair,
    direct_pair: selected.direct,
    conjugate_pair: selected.conjugate,
    elements,
    cardinality: 4,
    provenance: Object.freeze({
      basis: 'ql-musical-derivation-v3 §II-3.4',
      relation_kind: 'full-square'
    })
  });
}

export function listSquares() {
  return Object.freeze(FAMILY_IDS.flatMap((family) => [1, 2, 3].map((pair) => buildSquare(family, pair))));
}

export function buildDModulationFrame({ family, pair, level, projectionSide } = {}) {
  assertDLevel(level);
  const selected = selectPair(family, pair);
  const [left, right] = selected.direct;
  const [leftPrime, rightPrime] = selected.conjugate;

  let elements;
  let projection = null;

  if (level === 'D1') {
    elements = [left, right];
  } else if (level === 'D2') {
    if (!['left', 'right'].includes(projectionSide)) {
      throw new TypeError("D2 requires projectionSide='left' or projectionSide='right'.");
    }
    projection = projectionSide === 'left' ? leftPrime : rightPrime;
    elements = [left, right, projection];
  } else {
    if (projectionSide !== undefined && projectionSide !== null) {
      throw new TypeError('D3 is the complete square and does not accept projectionSide.');
    }
    elements = [left, right, leftPrime, rightPrime];
  }

  return Object.freeze({
    id: `${family}${pair}:${level}${projectionSide ? `:${projectionSide}` : ''}`,
    family,
    pair,
    level,
    projection_side: projectionSide ?? null,
    direct_pair: selected.direct,
    conjugate_pair: selected.conjugate,
    elements: Object.freeze(elements),
    cardinality: elements.length,
    full_square: level === 'D3',
    provenance: Object.freeze({
      formal_basis: 'A/B/C pairing grammar + conjugate face',
      software_operator_semantics: 'QL-PAIRING-SQUARES-CLARIFICATION-08-14-2026',
      relation_kind: 'conjugate-modulation-degree'
    })
  });
}

export function inspectCanonicalCrossPass({ family = 'D1', subtype, position = 0 } = {}) {
  if (!Number.isInteger(position) || position < 0 || position > 5) {
    throw new TypeError('Canonical cross-pass position must be an integer from 0 through 5.');
  }
  if (family === 'D1') return CANONICAL_CROSS_PASS.D1[position];
  if (family === 'D2') {
    if (!['transform', 'require', 'complete'].includes(subtype)) {
      throw new TypeError("Canonical D2 requires subtype='transform', 'require', or 'complete'.");
    }
    return CANONICAL_CROSS_PASS.D2[subtype][position];
  }
  if (family === 'D3') {
    assertFamily(subtype);
    return CANONICAL_CROSS_PASS.D3[subtype];
  }
  throw new TypeError(`Unknown canonical cross-pass family '${family}'.`);
}

export const PAIRING_GRAMMAR = Object.freeze({
  positions: POSITION_IDS,
  families: PAIRING_FAMILIES,
  d_levels: D_LEVELS,
  automatic_traversal: false,
  status: 'specified-formal-structure'
});
