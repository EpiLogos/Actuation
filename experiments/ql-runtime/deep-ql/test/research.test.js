import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';

const registry = JSON.parse(fs.readFileSync(new URL('../extensions/registry.json', import.meta.url), 'utf8'));

test('research registry excludes the specified pairing grammar', () => {
  assert.equal(registry.schema, 'ql-extension-registry/0.2');
  assert.equal(registry.unknown_extensions_ignorable, true);
  assert.ok(registry.specified_formal_modules.some((entry) => entry.namespace === 'ql.pairing' && entry.status === 'specified-formal-structure'));
  assert.equal(registry.extensions.some((entry) => entry.namespace === 'ql.harmonic'), false);

  const open = new Set(registry.extensions.map((entry) => entry.namespace));
  for (const namespace of ['ql.state64', 'ql.mef', 'ql.context', 'ql.epogdoon', 'ql.topology']) {
    assert.ok(open.has(namespace));
  }
  assert.ok(registry.extensions.every((entry) => entry.status === 'research' && entry.core_required === false));
});
