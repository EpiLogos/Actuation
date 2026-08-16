import test from 'node:test';
import assert from 'node:assert/strict';
import {
  PAIRING_FAMILIES,
  PAIRING_GRAMMAR,
  CANONICAL_CROSS_PASS,
  buildDModulationFrame,
  buildSquare,
  inspectCanonicalCrossPass,
  listSquares,
  selectPair
} from '../formal/pairing-grammar.js';

test('A/B/C remain three distinct specified pairing families', () => {
  assert.deepEqual(PAIRING_FAMILIES.A, [['P0','P1'], ['P2','P3'], ['P4','P5']]);
  assert.deepEqual(PAIRING_FAMILIES.B, [['P1','P2'], ['P3','P4'], ['P5','P0']]);
  assert.deepEqual(PAIRING_FAMILIES.C, [['P0','P5'], ['P1','P4'], ['P2','P3']]);
  assert.equal(PAIRING_GRAMMAR.status, 'specified-formal-structure');
  assert.equal(PAIRING_GRAMMAR.automatic_traversal, false);
});

test('the square apparatus preserves nine entries, eight oriented structures and seven unordered tetrad sets', () => {
  const squares = listSquares();
  assert.equal(squares.length, 9);
  assert.ok(squares.every((square) => square.cardinality === 4));

  const oriented = new Set(squares.map((square) => `${square.direct_pair.join('>')}|${square.conjugate_pair.join('>')}`));
  const unordered = new Set(squares.map((square) => [...square.elements].sort().join('|')));
  assert.equal(oriented.size, 8);
  assert.equal(unordered.size, 7);

  assert.deepEqual(buildSquare('A', 2).direct_pair, buildSquare('C', 3).direct_pair);
  assert.deepEqual(buildSquare('A', 2).elements, buildSquare('C', 3).elements);

  assert.notDeepEqual(buildSquare('B', 3).direct_pair, buildSquare('C', 1).direct_pair);
  assert.deepEqual([...buildSquare('B', 3).elements].sort(), [...buildSquare('C', 1).elements].sort());
});

test('D1/D2/D3 modulation has exact 2/3/4 element cardinality', () => {
  const d1 = buildDModulationFrame({ family: 'B', pair: 2, level: 'D1' });
  const d2Left = buildDModulationFrame({ family: 'B', pair: 2, level: 'D2', projectionSide: 'left' });
  const d2Right = buildDModulationFrame({ family: 'B', pair: 2, level: 'D2', projectionSide: 'right' });
  const d3 = buildDModulationFrame({ family: 'B', pair: 2, level: 'D3' });

  assert.equal(d1.cardinality, 2);
  assert.deepEqual(d1.elements, ['P3', 'P4']);
  assert.equal(d2Left.cardinality, 3);
  assert.deepEqual(d2Left.elements, ['P3', 'P4', "P3'"]);
  assert.equal(d2Right.cardinality, 3);
  assert.deepEqual(d2Right.elements, ['P3', 'P4', "P4'"]);
  assert.equal(d3.cardinality, 4);
  assert.deepEqual(d3.elements, buildSquare('B', 2).elements);
  assert.equal(d3.full_square, true);
});

test('canonical cross-pass topology remains separately queryable', () => {
  assert.deepEqual(inspectCanonicalCrossPass({ family: 'D1', position: 4 }), ['P4', "P4'"]);
  assert.deepEqual(inspectCanonicalCrossPass({ family: 'D2', subtype: 'transform', position: 5 }), ['P5', "P0'"]);
  assert.deepEqual(inspectCanonicalCrossPass({ family: 'D2', subtype: 'require', position: 0 }), ['P0', "P5'"]);
  assert.deepEqual(inspectCanonicalCrossPass({ family: 'D2', subtype: 'complete', position: 2 }), ['P2', "P3'"]);
  assert.deepEqual(inspectCanonicalCrossPass({ family: 'D3', subtype: 'A' }), CANONICAL_CROSS_PASS.D3.A);
});

test('invalid square/modulation requests fail rather than invent semantics', () => {
  assert.throws(() => selectPair('ABC', 1), /Expected A, B, or C/);
  assert.throws(() => selectPair('A', 4), /1, 2, or 3/);
  assert.throws(() => buildDModulationFrame({ family: 'A', pair: 1, level: 'D2' }), /projectionSide/);
  assert.throws(() => buildDModulationFrame({ family: 'A', pair: 1, level: 'D3', projectionSide: 'left' }), /does not accept/);
  assert.throws(() => inspectCanonicalCrossPass({ family: 'D2', subtype: 'guess', position: 2 }), /requires subtype/);
});
