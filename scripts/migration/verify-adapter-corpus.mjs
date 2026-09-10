// Read-only migration evidence. Expected original behaviour is never recaptured
// by this gate. Target data is compared semantically, not by JSON key order.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {CATALOG_REVISION,harnessDescriptors,capabilityDescriptors} from '../../detection/catalog.mjs';
import {secretSourceCatalog} from '../../detection/secret-sources/catalog.mjs';
import {evaluateScenario,compareScenario} from './scenarios.mjs';
const catalog=JSON.parse(readFileSync('catalog/targets.json','utf8'));
assert.equal(catalog.schema,'actuation.native-catalog/v1');
assert.equal(catalog.catalog_revision,CATALOG_REVISION);
assert.deepEqual(catalog.descriptors,harnessDescriptors());
assert.deepEqual(catalog.capabilities,capabilityDescriptors());
assert.deepEqual(catalog.secret_sources,secretSourceCatalog());
const corrections=JSON.parse(readFileSync('fixtures/migration/r5/corrections.json','utf8'));
assert.equal(corrections.schema,'actuation.r5-explicit-corrections/v1');
assert.equal(corrections.cases.length,3);
for(const [path,expected] of Object.entries(corrections.source_blobs)){
 const bytes=readFileSync(path);assert.equal(createHash('sha1').update(`blob ${bytes.length}\0`).update(bytes).digest('hex'),expected,`regression source changed: ${path}`);
}
for(const row of corrections.cases){assert.ok(row.correction);compareScenario(evaluateScenario(row),row.original);}
for(const line of readFileSync('fixtures/migration/r5/SHA256SUMS','utf8').trim().split('\n')){
 const [expected,path]=line.split(/\s+/);assert.equal(createHash('sha256').update(readFileSync(path)).digest('hex'),expected);
}
console.log(JSON.stringify({schema:'actuation.adapter-corpus-integrity/v1',catalog_revision:CATALOG_REVISION,targets:catalog.descriptors.length,capabilities:catalog.capabilities.length,explicit_corrections:corrections.cases.length,status:'ok',evidence_class:'D'}));
