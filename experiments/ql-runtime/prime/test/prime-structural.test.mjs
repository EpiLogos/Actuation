import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import { PRIME_CONDITIONS, conditionPrompt, getPrimeCondition } from '../conditions.mjs';
import { extractPrimeFamily, stableDigest } from '../evidence.mjs';
import { assertPrimeReturn, createPrimeReturn, PRIME_RETURN_SCHEMA } from '../return-contract.mjs';
import { classifyQlRevision, validateSourceLock } from '../source-lock.mjs';
import { PRIME_TASKS } from '../tasks.mjs';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.dirname(HERE);

test('Prime matrix carries real relational, recursive and continual conditions', () => {
  assert.deepEqual(Object.values(PRIME_CONDITIONS).map((item) => item.code), ['P0', 'P2', 'P3', 'P4', 'P5']);
  assert.equal(getPrimeCondition('prime-native').relational, false);
  assert.equal(getPrimeCondition('prime-relational').recursive, true);
  assert.equal(getPrimeCondition('prime-relational').maxDepth, 1);
  assert.equal(getPrimeCondition('prime-recursive-field').maxDepth, 2);
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
  assert.equal(family.child_nodes.length, 2);
  assert.equal(family.nested_edges.length, 1);
});

test('source lock pins current Prime stable/main and truthful QL harmonic standing separately', () => {
  const lock = JSON.parse(fs.readFileSync(path.join(ROOT, 'source-lock.json'), 'utf8'));
  assert.equal(validateSourceLock(lock), lock);
  assert.equal(lock.prime_agent.release, 'v0.9.4');
  assert.equal(lock.prime_agent.release_revision, 'f771dfcedd684d1afff84ca2c6fa95c7a21efbc2');
  assert.equal(lock.prime_agent.observed_main_revision, '55ade48b73f636d992855b7cab797d71dc1f6f1c');
  assert.equal(lock.ql_mef.accepted_main_revision, '44ed3cd0e7a8bc25508a4e18ad3bb4c730013913');
  assert.equal(lock.ql_mef.harmonic_research.pull_request_state, 'closed-unmerged');
  assert.equal(lock.ql_mef.harmonic_research.standing, 'accepted-main-history-carrier');
  assert.equal(classifyQlRevision(lock, lock.ql_mef.accepted_main_revision, { harmonicEnabled: true }), 'accepted-main-harmonic');
  assert.equal(classifyQlRevision(lock, lock.ql_mef.accepted_main_revision, { sourceDirty: true }), 'explicit-drift');
  assert.equal(classifyQlRevision(lock, lock.ql_mef.harmonic_research.revision, { harmonicEnabled: true }), 'historical-harmonic-head');
});

test('evidence digests include nested values and remain key-order stable', () => {
  const first = { result: { state: 'available', count: 2 }, request: ['a'] };
  const reordered = { request: ['a'], result: { count: 2, state: 'available' } };
  const changed = { request: ['a'], result: { count: 3, state: 'available' } };
  assert.equal(stableDigest(first), stableDigest(reordered));
  assert.notEqual(stableDigest(first), stableDigest(changed));
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

test('Prime-native tasks pressure real child composition and nested recursion', () => {
  const composition = PRIME_TASKS.find((task) => task.id === 'PRIME-COMPOSITION-001');
  const recursive = PRIME_TASKS.find((task) => task.id === 'PRIME-RECURSIVE-001');
  assert.equal(composition.primeAcceptance.minChildLoci, 2);
  assert.equal(composition.primeAcceptance.requireNestedChild, false);
  assert.equal(recursive.primeAcceptance.requireNestedChild, true);
  assert.match(recursive.prompt, /child.*own child/i);
});

test('QL skill exposes executable harmonic snapshot on accepted main', () => {
  const source = fs.readFileSync(path.join(ROOT, 'skills/ql-relational/src/ql_relational/__init__.py'), 'utf8');
  assert.match(source, /async def harmonic_snapshot/);
  assert.match(source, /derive_pre_m_music/);
  assert.match(source, /accepted-main/);
});
