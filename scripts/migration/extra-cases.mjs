// Explicit adversarial additions to the observed test-call corpus. Expected
// acceptance is authored here, so capture cannot silently bless a regression.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
const rows = JSON.parse(readFileSync(process.argv[2], 'utf8'));
const root = new URL('../../', import.meta.url);
const op = (module, name) => `contracts/${module}.mjs#${name}`;
const seed = operation => {
  const found = rows.filter(row => row.operation === operation && row.expected.ok).sort((a, b) => JSON.stringify(a.args).length - JSON.stringify(b.args).length)[0];
  assert.ok(found, `no inspected seed for ${operation}`);
  return structuredClone(found.args);
};
let count = 0;
async function check(label, operation, args, accepted) {
  const [path, name] = operation.split('#');
  const module = await import(new URL(path, root));
  process.env.ACTUATION_ORACLE_CASE_LABEL = label;
  let result, error;
  try { result = module[name](...args); } catch (caught) { error = caught; }
  delete process.env.ACTUATION_ORACLE_CASE_LABEL;
  assert.equal(!error, accepted, `${label}: ${error?.message ?? 'unexpected acceptance'}`);
  count++;
  return result;
}
const mutate = (args, change) => { const copy = structuredClone(args); change(copy); return copy; };
const bindingOp = op('agency', 'validateWorldBinding');
const binding = seed(bindingOp)[0];
await check('world-binding/direct-without-Factory', bindingOp, [binding], true);
await check('world-binding/optional-null-is-not-inferred', bindingOp, [{ ...binding, purpose_ref: null, continuity_ref: null, bounds_refs: null, authority_refs: null }], true);
await check('world-binding/opaque-extension-preserved', bindingOp, [{ ...binding, external_reading: { owner: 'caller', unknown: null } }], true);
for (const field of ['binding_ref', 'agent_ref', 'agency_ref', 'world_ref', 'scope_ref']) await check(`world-binding/empty-${field}`, bindingOp, [{ ...binding, [field]: ' ' }], false);
await check('world-binding/wrong-schema', bindingOp, [{ ...binding, schema: 'not-actuation/v1' }], false);
const scope = { schema: binding.schema, scope_ref: binding.scope_ref, enclosing_world_ref: binding.world_ref };
await check('root-scope/positional', op('agency', 'validateRootScope'), [scope], true);
await check('root-scope/invalid', op('agency', 'validateRootScope'), [{ ...scope, enclosing_world_ref: null }], false);
await check('root-scope/not-an-Agent-species', op('agency', 'isRootAgency'), [binding, scope], true);
const grantOp = op('agency', 'validateMetagencyGrant');
const grant = seed(grantOp)[0];
await check('metagency/no-implicit-operation', grantOp, [{ ...grant, operations: [] }], false);
await check('metagency/unknown-operation-refused', grantOp, [{ ...grant, operations: ['be-root-and-do-anything'] }], false);
const determinationOp = op('agency', 'validateDetermination');
const determination = seed(determinationOp)[0];
for (const kind of ['self-differentiation', 'delegation', 'derivation', 'federation']) {
  const value = { ...determination, kind };
  if (kind === 'federation') delete value.authority_refs;
  await check(`determination/${kind}`, determinationOp, [value], true);
}
await check('federation/not-authority', determinationOp, [{ ...determination, kind: 'federation', authority_refs: ['authority:smuggled'] }], false);
await check('determination/must-have-bounds', determinationOp, [{ ...determination, bounds_refs: [] }], false);
await check('determination/autonomous-termination-without-fabricated-return', determinationOp, [{ ...determination, return_policy: { mode: 'autonomous-termination' } }], true);
await check('determination/required-return-relation-absent', determinationOp, [{ ...determination, return_policy: { mode: 'required' } }], false);
const parent = { ...determination, determination_ref: 'determination:parent', determining_agency_ref: 'agency:parent', differentiated_agency_ref: 'agency:child', delegated_autonomy: { may_determine_within_bounds: true } };
delete parent.parent_determination_ref;
const child = { ...determination, determination_ref: 'determination:child', parent_determination_ref: parent.determination_ref, determining_agency_ref: parent.differentiated_agency_ref, differentiated_agency_ref: 'agency:grandchild' };
await check('lineage/same-recursive-grammar', op('agency', 'validateDeterminationLineage'), [[parent, child]], true);
await check('lineage/missing-parent', op('agency', 'validateDeterminationLineage'), [[child]], false);
await check('lineage/wrong-Agency-continuation', op('agency', 'validateDeterminationLineage'), [[parent, { ...child, determining_agency_ref: 'agency:unrelated' }]], false);
await check('lineage/no-downward-autonomy', op('agency', 'validateDeterminationLineage'), [[{ ...parent, delegated_autonomy: { may_determine_within_bounds: false } }, child]], false);
const returnOp = op('agency', 'validateReturn');
const returned = seed(returnOp)[0];
await check('return/not-recognition', returnOp, [{ ...returned, received: false, recognition_state: 'pending', world_mutation_state: 'not-applied' }], true);
await check('return/recognition-before-reception-refused', returnOp, [{ ...returned, received: false, recognition_state: 'recognised', world_mutation_state: 'not-applied' }], false);
await check('return/world-mutation-without-recognition-refused', returnOp, [{ ...returned, received: true, recognition_state: 'pending', world_mutation_state: 'applied' }], false);
await check('return/missing-lineage-refused', returnOp, [{ ...returned, provenance: { agency_lineage_refs: [] } }], false);
const actualiseOp = op('agency-actualisation', 'actualiseAgency');
const actualise = seed(actualiseOp);
for (const [label, change] of [
  ['no-grant', x => { delete x[0].metagency_grant; }],
  ['context-cannot-authorise', x => { x[0].metagency_grant.operations = []; x[0].provenance.context_refs = ['context:all-power']; }],
  ['grant-must-bind-exact-Agency', x => { x[0].metagency_grant.agency_ref = 'agency:other'; }],
  ['grant-cannot-exceed-world-bounds', x => { x[0].metagency_grant.bounds_refs.push('bounds:ungranted'); }],
  ['world-binding-cannot-widen-determination', x => { x[0].differentiated_binding.bounds_refs.push('bounds:ungranted'); }],
  ['external-owner-fields-not-accepted-as-authority', x => { x[0].factory_run = 'run:cannot-authorise'; }],
]) await check(`actualisation/${label}`, actualiseOp, mutate(actualise, change), false);
const realisedOp = op('realised-actuation', 'validateRealisedActuation');
const realised = seed(realisedOp)[0];
await check('realised/inert-model-is-not-acting', realisedOp, [{ ...realised, loop: { recurrence: 'turn-based', acting: false } }], false);
await check('realised/observed-requires-evidence', realisedOp, [{ ...realised, observation: { state: 'observed' } }], false);
await check('realised/unsupported-remains-unavailable', realisedOp, [{ ...realised, observation: { state: 'unavailable', unsupported_faculties: ['native-cancel'] } }], true);
for (const field of ['harness_ref', 'session_ref', 'process_ref', 'model_condition_ref', 'material_binding_ref']) await check(`continuity/${field}-does-not-mint-identity`, op('realised-actuation', 'continuityDelta'), [realised, { ...realised, body: { ...(realised.body ?? {}), [field]: `external:${field}:changed` } }], true);
const streamOp = op('actuation-stream', 'validateActuationStream');
const stream = seed(streamOp)[0];
await check('stream/identity-collapse-refused', streamOp, [{ ...stream, stream_ref: stream.agency_ref }], false);
await check('stream/incorrect-cursor-refused', streamOp, [{ ...stream, cursor: { last_sequence: 400, next_sequence: 401 } }], false);
await check('stream/terminal-needs-ended-at', streamOp, [{ ...stream, lifecycle: { state: 'closed' } }], false);
await check('stream/open-cannot-have-ended-at', streamOp, [{ ...stream, lifecycle: { state: 'open', ended_at: '2026-09-10T00:00:00Z' } }], false);
const eventOp = op('actuation-stream', 'validateActuationStreamEvent');
const event = { event_ref: 'event:adversarial', sequence: 1, kind: 'world-observation' };
await check('stream-event/unknown-actor-not-inferred', eventOp, [event], true);
await check('stream-event/multi-locus-attribution', eventOp, [{ ...event, actor: { locus_ref: 'locus:child', agency_ref: 'agency:child' }, native_trace_ref: 'native:opaque:trace' }], true);
await check('stream-event/empty-actor-refused', eventOp, [{ ...event, actor: {} }], false);
await check('stream-event/reference-only-cannot-inline-content', eventOp, [{ ...event, disclosure: 'reference-only', content: 'forbidden' }], false);
await check('stream-event/reference-only-is-sufficient', eventOp, [{ ...event, disclosure: 'reference-only', native_trace_ref: 'native:opaque' }], true);
await check('stream-event/unsafe-sequence-refused', eventOp, [{ ...event, sequence: 9007199254740992 }], false);
await check('stream-event/return-needs-Return', eventOp, [{ ...event, kind: 'return' }], false);
await check('stream-event/custom-must-be-named', eventOp, [{ ...event, kind: 'custom' }], false);
await check('stream-event/date-only-is-ISO-compatible', eventOp, [{ ...event, observed_at: '2026-09-10' }], true);
await check('stream-event/invalid-time-refused', eventOp, [{ ...event, observed_at: 'not-a-timestamp' }], false);
for (const options of [{ afterSequence: -1 }, { afterSequence: 0.5 }, { limit: -1 }, { limit: 0.5 }]) await check(`replay/invalid-${JSON.stringify(options)}`, op('actuation-stream', 'actuationStreamReadModel'), [stream, options], false);
await check('replay/zero-limit-is-empty-not-all', op('actuation-stream', 'actuationStreamReadModel'), [stream, { afterSequence: 0, limit: 0 }], true);
const activityOp = op('activity', 'validateActivity');
const activity = seed(activityOp)[0];
for (const phase of ['queued', 'running', 'waiting', 'completed', 'failed', 'interrupted', 'cancelled']) {
  const value = { ...activity, phase };
  if (['queued', 'running', 'waiting'].includes(phase)) delete value.completed_at;
  else value.completed_at = '2026-09-10T00:01:00Z';
  await check(`activity/${phase}`, activityOp, [value], true);
}
const directActivity = { ...activity };
for (const field of ['plan_ref', 'journey_ref', 'run_ref', 'action_ref', 'invocation_ref']) delete directActivity[field];
await check('activity/Direct-needs-no-Factory', activityOp, [directActivity], true);
await check('activity/distinct-owner-correlations', activityOp, [{ ...directActivity, run_ref: 'factory:run:1', journey_ref: 'factory:journey:1', plan_ref: 'factory:plan:1', agent_session_ref: 'aikit:session:1', invocation_ref: 'oi:invocation:1' }], true);
await check('activity/attention-is-not-Authority', op('activity', 'activityNeedsAttention'), [{ ...activity, needs_attention: true }], true);
for (const [module, validator, factory] of [
  ['harness-detection', 'validateHarnessDescriptor', 'harnessDescriptor'],
  ['harness-detection', 'validateHarnessCatalog', 'harnessCatalog'],
  ['harness-detection', 'validateHarnessDetection', 'harnessDetection'],
  ['harness-detection', 'validateHarnessSelf', 'harnessSelf'],
  ['harness-capability', 'validateHarnessCapability', 'harnessCapability'],
  ['request-correlation', 'validateAuthorityDecision', 'authorityDecision'],
  ['secret-detection', 'validateSecretSourceDescriptor', 'secretSourceDescriptor'],
  ['secret-detection', 'validateSecretSourceCatalog', 'secretSourceCatalog'],
  ['secret-detection', 'validateSecretScan', 'secretScan'],
]) {
  await check(`${module}/${validator}/positive`, op(module, validator), seed(op(module, factory)), true);
  await check(`${module}/${validator}/null`, op(module, validator), [null], false);
}
for (const operation of [bindingOp, grantOp, determinationOp, returnOp, actualiseOp, realisedOp, streamOp, eventOp, activityOp,
  op('instantiation', 'validateModelRelation'), op('instantiation', 'validateModelAccessProfile'), op('instantiation', 'validateActuationReceipt'), op('model-usage', 'validateModelUsageObservation')]) {
  await check(`${operation}/not-an-object`, operation, [[]], false);
}
console.log(`Explicit adversarial cases accepted their authored assertions: ${count}`);
