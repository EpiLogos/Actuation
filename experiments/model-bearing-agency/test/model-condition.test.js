import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

import {
  MODEL_CONDITION_SCHEMA,
  MODEL_EXPERIMENT_SCHEMA,
  compareModelConditions,
  createMatchedConditionExperiment,
  createModelConditionReceipt,
  modelConditionSemanticIdentity
} from '../contract.js';
import {
  collapsedDirectProcess,
  localServiceExternalToHarness,
  remoteServiceSameHarness
} from '../fixtures.js';
import { sourceLockedCases } from '../source-cases.js';

const copy = (value) => structuredClone(value);

test('collapsed direct-process Actuation needs no standing service, harness, or AgentSession', () => {
  const receipt = createModelConditionReceipt(copy(collapsedDirectProcess));

  assert.equal(receipt.schema, MODEL_CONDITION_SCHEMA);
  assert.equal(receipt.materialisation.mode, 'process');
  assert.equal(receipt.materialisation.world_relation, 'inside-world');
  assert.deepEqual(receipt.materialisation.material_binding_refs, []);
  assert.equal(receipt.surface.kind, 'cli');
  assert.equal(receipt.harness.model_relation, 'none');
  assert.equal(receipt.harness.harness_ref, null);
  assert.equal(receipt.harness.agent_session_ref, null);
});

test('external-to-harness does not imply external-to-world', () => {
  const receipt = createModelConditionReceipt(copy(localServiceExternalToHarness));

  assert.equal(receipt.harness.model_relation, 'external-surface');
  assert.equal(receipt.materialisation.world_relation, 'inside-world');
  assert.equal(receipt.surface.kind, 'http');
  assert.ok(receipt.materialisation.material_binding_refs.length > 0);
});

test('same harness and model survive local-to-remote rematerialisation without situated identity drift', () => {
  const comparison = compareModelConditions(
    copy(localServiceExternalToHarness),
    copy(remoteServiceSameHarness)
  );

  assert.deepEqual(comparison.identity_drift, []);
  assert.equal(comparison.same_agent, true);
  assert.equal(comparison.same_agency, true);
  assert.equal(comparison.same_world, true);
  assert.equal(comparison.same_harness, true);
  assert.equal(comparison.same_model, true);
  assert.ok(comparison.changed_axes.includes('engine'));
  assert.ok(comparison.changed_axes.includes('materialisation'));
  assert.ok(comparison.changed_axes.includes('session'));
  assert.ok(comparison.changed_axes.includes('control-access'));
  assert.ok(comparison.changed_axes.includes('interior-access'));
  assert.ok(!comparison.changed_axes.includes('surface'));
});

test('material endpoint and process provenance do not enter semantic identity', () => {
  const local = copy(localServiceExternalToHarness);
  const rebound = copy(localServiceExternalToHarness);
  rebound.receipt_ref = 'model-condition:local-service-rebound';
  rebound.materialisation.material_binding_refs = ['workcell-binding:replacement'];
  rebound.surface.binding_ref = 'service-binding:model-local-rebound';

  assert.deepEqual(
    modelConditionSemanticIdentity(local),
    modelConditionSemanticIdentity(rebound)
  );

  const comparison = compareModelConditions(local, rebound);
  assert.deepEqual(comparison.identity_drift, []);
  assert.deepEqual([...comparison.changed_axes].sort(), ['materialisation']);
});

test('inference, material control, and model-interior access remain independent', () => {
  const remote = createModelConditionReceipt(copy(remoteServiceSameHarness));

  assert.ok(remote.access.inference.includes('invoke'));
  assert.deepEqual(remote.access.control, []);
  assert.equal(remote.access.interior, 'behavioural');
});

test('matched-condition experiment enforces held constants and intended variation', () => {
  const baseline = copy(localServiceExternalToHarness);
  const candidate = copy(remoteServiceSameHarness);

  const experiment = createMatchedConditionExperiment({
    experiment_ref: 'experiment:local-remote-materialisation',
    baseline,
    candidate,
    held_constant_axes: ['model', 'harness'],
    intended_varied_axes: ['engine', 'materialisation'],
    evidence_refs: [],
    return_refs: [],
    provenance_refs: ['research-plan:model-materialisation-v0']
  });

  assert.equal(experiment.schema, MODEL_EXPERIMENT_SCHEMA);
  assert.deepEqual(experiment.held_constant_axes, ['model', 'harness']);
  assert.ok(experiment.observed_changed_axes.includes('materialisation'));

  assert.throws(
    () => createMatchedConditionExperiment({
      experiment_ref: 'experiment:invalid-held-session',
      baseline,
      candidate,
      held_constant_axes: ['session'],
      intended_varied_axes: ['materialisation']
    }),
    /held-constant axes changed: session/i
  );
});

test('matched-condition experiment rejects Agent, Agency, world, or Actuation drift', () => {
  const baseline = copy(localServiceExternalToHarness);
  const candidate = copy(remoteServiceSameHarness);
  candidate.agent_ref = 'agent:other';

  assert.throws(
    () => createMatchedConditionExperiment({
      experiment_ref: 'experiment:identity-drift',
      baseline,
      candidate,
      intended_varied_axes: ['materialisation']
    }),
    /cannot drift situated identity: agent_ref/i
  );
});

test('harness refs cannot appear without an explicit harness/model relation', () => {
  const direct = copy(collapsedDirectProcess);
  direct.harness.harness_ref = 'harness:accidental';

  assert.throws(
    () => createModelConditionReceipt(direct),
    /harness refs require an explicit embedded or external-surface model relation/i
  );
});

test('published JSON schema carries both receipt and experiment contracts', async () => {
  const schema = JSON.parse(await readFile(
    new URL('../../../schemas/model-condition.v0.schema.json', import.meta.url),
    'utf8'
  ));

  assert.equal(schema.$id, 'https://epilogos.dev/schemas/model-condition.v0.schema.json');
  assert.equal(schema.oneOf.length, 2);
  assert.equal(schema.$defs.ModelConditionReceipt.properties.schema.const, MODEL_CONDITION_SCHEMA);
  assert.equal(schema.$defs.ModelConditionExperiment.properties.schema.const, MODEL_EXPERIMENT_SCHEMA);
});

test('source-locked Ollama, llama.cpp and vLLM cases fit one receipt contract without provider ontology leakage', () => {
  const receipts = Object.values(sourceLockedCases).map(({ condition }) =>
    createModelConditionReceipt(copy(condition))
  );

  assert.equal(receipts.length, 4);
  assert.ok(receipts.every((receipt) => receipt.schema === MODEL_CONDITION_SCHEMA));
  assert.equal(sourceLockedCases.ollamaLocalService.condition.materialisation.mode, 'service');
  assert.equal(sourceLockedCases.llamaCppDirect.condition.materialisation.mode, 'process');
  assert.equal(sourceLockedCases.llamaCppServer.condition.materialisation.mode, 'service');
  assert.equal(sourceLockedCases.vllmDistributedService.condition.materialisation.mode, 'distributed-service');
});

test('llama.cpp direct and server cases prove engine identity is independent of surface/materialisation form', () => {
  const direct = copy(sourceLockedCases.llamaCppDirect.condition);
  const server = copy(sourceLockedCases.llamaCppServer.condition);
  const comparison = compareModelConditions(direct, server);

  assert.deepEqual(comparison.identity_drift, []);
  assert.equal(comparison.same_model, true);
  assert.ok(!comparison.changed_axes.includes('engine'));
  assert.ok(comparison.changed_axes.includes('materialisation'));
  assert.ok(comparison.changed_axes.includes('surface'));
  assert.ok(comparison.changed_axes.includes('harness'));
});
