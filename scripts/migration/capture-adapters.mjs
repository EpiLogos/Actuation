// One-time source-locked extraction, never part of a passing verification gate.
import assert from 'node:assert/strict';
import { writeFileSync, existsSync, readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { lockOriginalSources, BASE_REVISION } from './source-lock.mjs';
import { evaluateScenario } from './scenarios.mjs';
import { harnessDescriptorBySlug } from '../../detection/catalog.mjs';
const root=fileURLToPath(new URL('../../',import.meta.url));
const out=new URL('../../fixtures/migration/r5/observations.json',import.meta.url);
assert.ok(!existsSync(out),'extraction is frozen once; refuse to overwrite');
const sources=lockOriginalSources(root);
const now='2026-09-10T00:00:00.000Z';
const clone=structuredClone;
const cases=[]; const defects=[];
function probe(id,input){ const s={id,kind:'probe',input:{now,...input}};s.expected=evaluateScenario(s);cases.push(s); }
function defect(id,input,law){ const s={id,kind:'probe',law,input:{now,...input}};s.expected=evaluateScenario(s);defects.push(s); }
function bodyEffects(slug){return {resolveExecutable:{value:{found:true,path:`/specimen/${slug}`}},hashProbe:{value:{ok:true,sha256:'a'.repeat(64)}},statProbe:{by_argument:{[JSON.stringify(`/specimen/${slug}`)]:{exists:true,isDir:false,size:42,mtimeMs:120.4}}},versionProbe:{value:{ok:true,version:`${slug} controlled-1`}}};}
for(const slug of ['claude-code','codex','pi']) {
  probe(`${slug}-observed-not-executed`,{slugs:[slug],effects:bodyEffects(slug)});
  probe(`${slug}-explicit-version`,{slugs:[slug],effects:bodyEffects(slug),versions:true});
  const failed=bodyEffects(slug);failed.versionProbe={value:{ok:false,reason:'controlled version refusal'}};
  probe(`${slug}-version-refusal-keeps-presence`,{slugs:[slug],effects:failed,versions:true});
}
const partial={schema:'actuation.harness-detection/v1',document:'descriptor',slug:'partial',native_kind:'experimental-body',probe:{executable:{names:['partial']}},provenance:{authored_by:'R5 explicit partial specimen',catalog_revision:1}};
probe('partial-body-no-invented-faculties',{descriptors:[partial],effects:bodyEffects('partial')});
probe('partial-body-unavailable',{descriptors:[partial],effects:{resolveExecutable:{value:{error:'controlled permission denial'}}}});
const ollama=bodyEffects('ollama');
ollama.statProbe.by_argument[JSON.stringify('/home/oracle/.ollama')]={exists:true,isDir:true};
ollama.statProbe.by_argument[JSON.stringify('/home/oracle/.ollama/models')]={exists:true,isDir:true};
ollama.dirCountProbe={value:{exists:true,count:9}};
ollama.serviceProbe={value:{ok:true,detail:'http 200 from http://127.0.0.1:11434'}};
ollama.httpJsonProbe={value:{ok:true,body:{models:[{model:'m:1',name:'m:alias',digest:'digest',size:31,private:'do-not-retain'}, {model:'m:1',name:'duplicate'},null,2,{name:'no model'},{model:'m:2',name:'m:2',size:22,details:{not:'scalar'}}]}}};
probe('ollama-named-inventory-is-not-directory-count',{slugs:['ollama'],effects:ollama});
for(const [label,change] of Object.entries({
  'unavailable':{httpJsonProbe:{value:{ok:false,reason:'controlled inventory failure'}}},
  'missing-collection':{httpJsonProbe:{value:{ok:true,body:{not_models:[]}}}},
  'empty-observed':{httpJsonProbe:{value:{ok:true,body:{models:[]}}}},
  'unsupported-read':{omit:['httpJsonProbe']},
  'unverified-service':{serviceProbe:{value:{ok:true,detail:'declared, not verified'}}},
  'failed-service':{serviceProbe:{value:{ok:false,reason:'controlled service mechanism failure'}}},
  'absent-service':{serviceProbe:{value:{ok:true,detail:'no listener at http://127.0.0.1:11434'}}},
  'unsupported-service':{omit:['serviceProbe']},
})) probe(`ollama-inventory-${label}`,{slugs:['ollama'],effects:{...clone(ollama),...change}});
const custom=clone(partial);custom.slug='service-only';custom.probe={service:{kind:'http',default_url:'http://127.0.0.1:11434'}};
probe('service-only-v1-receipt-limit',{descriptors:[custom],effects:{serviceProbe:{value:{ok:true,detail:'http 200 from http://127.0.0.1:11434'}}}});
const config=clone(partial);config.probe['config-dir']={path:'~/.partial'};
defect('absence-detail-is-not-an-executable',{descriptors:[config],effects:{statProbe:{value:{exists:true,isDir:true}},versionProbe:{value:{ok:true,version:'bogus'}},hashProbe:{value:{ok:true,sha256:'b'.repeat(64)}}},versions:true},'Use the actual resolved path or a config-dir receipt; never hash/stat/version the passing absence diagnostic.');
const misleading=bodyEffects('partial');misleading.resolveExecutable.value.path='/specimen/not found but present';
defect('resolved-path-is-not-a-diagnostic',{descriptors:[partial],effects:misleading},'Typed executable presence cannot be overturned by words in its actual path.');
const envMask=clone(partial);envMask.probe.env={any_of:['PRESENT_MARKER']};
defect('marker-does-not-mask-failed-presence-probe',{descriptors:[envMask],effects:{resolveExecutable:{value:{error:'presence mechanism denied'}},envProbe:{value:{ok:true,matched:{PRESENT_MARKER:'not-a-secret'}}}}},'A successful identity marker is not a successful presence measurement; incomplete presence remains unavailable.');
const payload={schema:'actuation.adapter-extraction/v1',source_revision:BASE_REVISION,source_blobs:sources,evidence_class:'D',cases,defects};
writeFileSync(out,JSON.stringify(payload,null,2)+'\n');
const digest=createHash('sha256').update(readFileSync(out)).digest('hex');
writeFileSync(new URL('../../fixtures/migration/r5/SHA256SUMS',import.meta.url),`${digest}  observations.json\n`);
console.log(JSON.stringify({cases:cases.length,defects:defects.length,source_revision:BASE_REVISION,sha256:digest}));
