import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import { PRIME_CONDITIONS, conditionPrompt, getPrimeCondition } from '../conditions.mjs';
import { extractPrimeFamily } from '../evidence.mjs';
import { assertPrimeReturn, createPrimeReturn, PRIME_RETURN_SCHEMA } from '../return-contract.mjs';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.dirname(HERE);

test('Prime matrix carries real relational, recursive and continual conditions', () => {
  assert.deepEqual(Object.values(PRIME_CONDITIONS).map((item) => item.code), ['P0', 'P2', 'P3', 'P4', 'P5']);
  assert.equal(getPrimeCondition('prime-native').relational, false);
  assert.equal(getPrimeCondition('prime-relational').recursive, true);
  assert.equal(getPrimeCondition('prime-continual').continual, true);
});

test('relational prompts give the inherited child Agency relational intelligence', () => {
  const task = { prompt: 'Inspect the relation.', successConditions: ['Return evidence.'] };
  const native = conditionPrompt(getPrimeCondition('prime-native'), task);
  const relational = conditionPrompt(getPrimeCondition('prime-relational-return'), task);
  assert.doesNotMatch(native, /ql_relational/);
  assert.match(relational, /ql_relational/);
  assert.match(relational, /inherited by Prime child agents/);
  assert.match(relational, /returned difference/);
});

test('Prime Return preserves result, difference, unresolved and provenance', () => {
  const value = createPrimeReturn({
    subject_ref: 'task:1',
    relation_to_parent: 'supports',
    determination: 'review',
    result: 'A',
    difference: 'B changes parent decision',
    evidence_refs: ['file:a'],
    unresolved: ['counter-evidence'],
    provenance: { child: 'x' }
  });
  assert.equal(value.schema, PRIME_RETURN_SCHEMA);
  assert.equal(assertPrimeReturn(value).difference, 'B changes parent decision');
  assert.deepEqual(value.unresolved, ['counter-evidence']);
});

test('family extractor retains child handles and parent lineage when Prime emits them', () => {
  const family = extractPrimeFamily([
    { type: 'event', data: { rlm_child_id: 'child-a', active_session_id: 'session-a', name: 'research' } },
    { type: 'event', data: { childId: 'child-b', parentSessionId: 'child-a', sessionName: 'nested' } }
  ]);
  assert.equal(family.nodes.length, 2);
  assert.deepEqual(family.edges, [{ parent: 'child-a', child: 'child-b' }]);
});

test('source lock pins Prime stable, QL main and optional harmonic development separately', () => {
  const lock = JSON.parse(fs.readFileSync(path.join(ROOT, 'source-lock.json'), 'utf8'));
  assert.equal(lock.prime_agent.release, 'v0.9.1');
  assert.equal(lock.prime_agent.release_revision, '81ae3cb34d27d38ee37f9e205a1e73694993b344');
  assert.equal(lock.ql_mef.accepted_main_revision, 'cddd97d3e7717954256a46f482bd569fa7448870');
  assert.equal(lock.ql_mef.harmonic_research.standing, 'current-development-not-accepted-main');
});

test('Python-backed QL skill package is complete', () => {
  for (const relative of [
    'skills/ql-relational/SKILL.md',
    'skills/ql-relational/pyproject.toml',
    'skills/ql-relational/src/ql_relational/__init__.py'
  ]) {
    assert.equal(fs.existsSync(path.join(ROOT, relative)), true, relative);
  }
});
