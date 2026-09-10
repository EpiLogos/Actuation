import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { createHash } from 'node:crypto';
const root = new URL('../../fixtures/migration/r4/',import.meta.url);
const manifest = JSON.parse(readFileSync(new URL('manifest.json',root),'utf8'));
const corpus = JSON.parse(readFileSync(new URL('../../fixtures/migration/scenarios.json',import.meta.url),'utf8'));
assert.equal(manifest.schema,'actuation.r4-frozen-stores/v1');
assert.equal(manifest.source_revision,corpus.source_revision);
assert.equal(manifest.stores.length,8);
assert.deepEqual(readdirSync(new URL('stores/',root)).sort(),manifest.stores.map(s=>s.path.slice('stores/'.length)).sort());
for(const entry of manifest.stores){
  assert.match(entry.path,/^stores\/\d{2}\.jsonl$/);
  const raw=readFileSync(new URL(entry.path,root),'utf8');
  const original=corpus.cases.find(c=>c.id===entry.source_case).expected[entry.source_step];
  assert.equal(original.label,entry.source_label);
  assert.equal(original.files[entry.original_name],raw,'extracted snapshot must be exact R1 bytes');
  assert.equal(createHash('sha256').update(raw).digest('hex'),entry.sha256);
  assert.equal(JSON.parse(raw.split('\n')[0]).stream_ref,entry.stream_ref);
}
console.log(JSON.stringify({schema:'actuation.stream-corpus-integrity/v1',snapshots:manifest.stores.length,status:'ok',evidence_class:'D'}));
