// R3 one-time extraction oracle, never a passing-gate recapture. Generic
// runtime mechanics only: QL formal intelligence remains the QL owner's job.
import assert from 'node:assert/strict';
import { writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { lockOriginalSources, BASE_REVISION } from './source-lock.mjs';
import { ClassicRuntime } from '../../experiments/ql-runtime/foundation/classic-runtime/index.js';
import { dispatchHostCarrier, normalizeRunRequest, RuntimeRegistry } from '../../experiments/ql-runtime/foundation/runtime-contract/index.js';
const destination = process.argv[2];
if (!destination) throw new Error('usage: capture-runtime.mjs <new-corpus.json>');
lockOriginalSources(fileURLToPath(new URL('../../', import.meta.url)));
const inputs = [
  ['zero-tool', { models: [{ content: 'done', capabilityCalls: [] }] }, 'completed'],
  ['two-tools', { models: [{ content: 'tools', capabilityCalls: [{ id:'c1', name:'read', args:{path:'a'} }, { name:'write', args:{path:'b'} }] }, {content:'done'}], capabilities:{read:{content:'a'},write:{ok:true}} }, 'completed'],
  ['failed-tool-recovery', {models:[{capabilityCalls:[{name:'fail'}]},{content:'recovered'}],capabilities:{fail:{throw:'capability failed'}}}, 'completed'],
  ['tool-returns-failure', {models:[{capabilityCalls:[{name:'fail'}]},{content:'recovered'}],capabilities:{fail:{ok:false,error:'truthful failure'}}}, 'completed'],
  ['follow-up', {models:[{content:'first'},{content:'second'}],human:['steer']}, 'completed'],
  ['false-is-follow-up', {models:[{content:'first'},{content:'second'}],human:[false]}, 'completed'],
  ['empty-string-is-follow-up', {models:[{content:'first'},{content:'second'}],human:['']}, 'completed'],
  ['max-step-exhaustion', {models:[{capabilityCalls:[{name:'again'}]}],capabilities:{again:{ok:true}},request:{maxSteps:1}}, 'exhausted'],
  ['cancel-before-start', {aborted:true}, 'cancelled'],
  ['cancel-after-model-before-tool', {models:[{capabilityCalls:[{name:'never'}]}],abort_after_model:true}, 'cancelled'],
  ['host-abort', {models:[{throw:'provider interrupted',error_name:'AbortError'}]}, 'cancelled'],
  ['model-failure', {models:[{throw:'provider unavailable'}]}, 'failed'],
  ['human-failure', {models:[{content:'done'}],human:[{throw:'input failed'}]}, 'failed'],
  ['human-abort', {models:[{content:'done'}],human:[{throw:'input cancelled',error_name:'AbortError'}]}, 'cancelled'],
  ['null-model', {models:[null]}, 'completed'],
  ['null-content-keeps-result', {models:[{content:null,opaque:{native:'trace'}}]}, 'completed'],
  ['caller-role-preserved', {models:[{role:'native-assistant',content:'done'}]}, 'completed'],
];
function hostFor(input, calls, controller) {
  const models = [...(input.models ?? [])], human = [...(input.human ?? [])];
  function resolve(value) {
    if (value && typeof value === 'object' && value.throw) { const error = new Error(value.throw); error.name = value.error_name ?? 'Error'; throw error; }
    return structuredClone(value);
  }
  const record = (method, call) => { const {signal, ...wire} = call; calls.push({method, call: structuredClone(wire)}); };
  return {
    async callModel(call) { record('model', call); if (input.abort_after_model) controller.abort(); return resolve(models.shift() ?? null); },
    async executeCapability(call) { record('capability', call); return resolve(input.capabilities?.[call.name] ?? null); },
    async receiveExternalInput(call) { record('human', call); return resolve(human.shift() ?? null); },
    async readContext(call) { record('context', call); return {context:call.kind}; },
  };
}
const cases=[];
for (const [id,input,status] of inputs) {
  const calls=[],events=[],controller=new AbortController(); if(input.aborted)controller.abort();
  const request={taskId:id,input:'task input',runId:`trace:${id}`,...(input.request??{})};
  const result=await new ClassicRuntime().run(request,hostFor(input,calls,controller),{emit:event=>events.push(structuredClone(event))},controller.signal);
  assert.equal(result.status,status,id);
  cases.push({id,kind:'loop',input:{...input,request},expected:{result,events,calls}});
}
for (const carrier of [
  {},{kind:'model'},{kind:'tool',name:'read',args:{path:'a'}},{kind:'capability',name:'write'},
  {kind:'human'},{kind:'human',inputKind:'follow_up'},
  {kind:'environment',input:{path:'a'}},{kind:'artifact',input:'artifact:1'},
  {kind:'external_evaluator',input:{claim:'x'}},{kind:'internal_control',input:{control:true}},
  {kind:'unimplemented'},
]) {
  const request=normalizeRunRequest({taskId:'carrier',input:'x'}), payload={kind:'inherited-kind',opaque:{ref:'caller-owned'},request:'cannot-shadow',signal:'cannot-shadow'};
  const calls=[],controller=new AbortController(); let result;
  try { result={ok:true,value:await dispatchHostCarrier({host:hostFor({models:[{content:'model'}],human:['human'],capabilities:{read:{ok:true},write:{ok:true}}},calls,controller),carrier,request,signal:controller.signal,payload})}; }
  catch(error){result={ok:false,error:error.message};}
  cases.push({id:`carrier-${JSON.stringify(carrier)}`,kind:'carrier',input:{carrier,request,payload},expected:{result,calls}});
}
const registry = new RuntimeRegistry().register(new ClassicRuntime());
assert.throws(()=>registry.register(new ClassicRuntime()),/already registered/);
assert.throws(()=>registry.get('missing'),/Unknown runtime/);
cases.push({id:'runtime-registry',kind:'registry',input:{},expected:registry.list()});
writeFileSync(destination,JSON.stringify({schema:'actuation.runtime-extraction/v1',source_revision:BASE_REVISION,evidence_class:'D',cases},null,2)+'\n',{flag:'wx'});
console.log(`Frozen ${cases.length} generic runtime cases with authored status assertions.`);
