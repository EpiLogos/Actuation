// Read-only migration evidence. Expected original behaviour is never recaptured
// by this gate. Since the R7 cutover the served harness catalog is
// catalog/targets.json (bundled into the native binary); this script checks the
// frozen R5 explicit-corrections evidence and the published checksums only.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
const catalog=JSON.parse(readFileSync('catalog/targets.json','utf8'));
assert.equal(catalog.schema,'actuation.native-catalog/v1');
assert.ok(catalog.catalog_revision >= 1);
assert.ok(catalog.descriptors.length >= 1);
assert.ok(catalog.capabilities.length >= 1);
const corrections=JSON.parse(readFileSync('fixtures/migration/r5/corrections.json','utf8'));
assert.equal(corrections.schema,'actuation.r5-explicit-corrections/v1');
assert.equal(corrections.cases.length,3);
// corrections.source_blobs pinned the served MJS regression sources by sha1;
// those sources retired at the R7 cutover and their executable law is now
// asserted natively (crates/actuation-adapters tests, r5 explicit
// corrections). The frozen corrections document itself stays sha256-pinned
// via fixtures/migration/r5/SHA256SUMS below.
assert.equal(Object.keys(corrections.source_blobs).length,3);
for(const row of corrections.cases){assert.ok(row.correction);}
for(const line of readFileSync('fixtures/migration/r5/SHA256SUMS','utf8').trim().split('\n')){
 const [expected,path]=line.split(/\s+/);assert.equal(createHash('sha256').update(readFileSync(path)).digest('hex'),expected);
}
console.log(JSON.stringify({schema:'actuation.adapter-corpus-integrity/v1',catalog_revision:catalog.catalog_revision,targets:catalog.descriptors.length,capabilities:catalog.capabilities.length,explicit_corrections:corrections.cases.length,status:'ok',evidence_class:'D'}));
