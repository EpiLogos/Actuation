import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import {
  DETERMINATION,
  REQUIRED_HELD_CONSTANTS,
  SERIES1_CAPABILITY_CONTRACT,
  SERIES1_SCHEMA,
  assertLiveManifest,
  compareHeldConstant,
  stableDigest
} from '../../comparison/series1/contract.mjs';
import { assertCandidateBoundary, buildCandidateRequest } from '../../comparison/series1/candidate-boundary.mjs';
import { buildBenchmarkFreeze, verifyFreezeReproducibility } from '../../comparison/series1/freeze.mjs';
import { LiveRuntimeHost, Series1Workspace } from '../../comparison/series1/host.mjs';
import {
  normalizeEnvelope,
  parseJsonObject,
  SERIES1_PROVIDER,
  SERIES1_DEFAULT_MODEL,
  SERIES1_BASE_URL,
  SERIES1_CREDENTIAL_ENV
} from '../../comparison/series1/providers.mjs';
import { SERIES1_TASKS, setupTask, verifyTask } from '../../comparison/series1/tasks.mjs';

test('Series 1 refuses fixture-shaped evidence and requires human-review contract', () => {
  assert.throws(() => assertLiveManifest({
    schema: SERIES1_SCHEMA,
    provider_mode: 'fixture',
    fixture_provider: true,
    host: { id: 'pi', revision: 'x', real_framework_path: 'fixture' },
    model: { provider: 'fixture', id: 'fake' },
    conditions: ['classic','ql-direct','ql-deep'],
    held_constant: { valid: false, mismatches: [] },
    determination: DETERMINATION,
    review: { prompt: 'x', focus: ['x'] },
    records: [{}]
  }), /not evidence eligible/);
});

test('Series 1 uses provider-native ZAI candidate defaults (2026-09-13 amendment)', () => {
  assert.equal(SERIES1_PROVIDER, 'zai');
  assert.equal(SERIES1_DEFAULT_MODEL, 'glm-5.3-flash');
  assert.equal(SERIES1_BASE_URL, 'https://api.z.ai/api/coding/paas/v4');
  assert.equal(SERIES1_CREDENTIAL_ENV, 'ZAI_API_KEY');
  assert.equal(DETERMINATION, 'pending-human-review');
});

function heldRecords() {
  return ['classic','ql-direct','ql-deep'].map((condition) => ({
    condition,
    prompt_digest: 'prompt-a',
    success_constraints_digest: 'success-a',
    start_state_digest: 'start-a',
    model: { provider: 'deepseek', id: 'deepseek-v4-flash', parameters: { temperature: 0 } },
    capability_contract_digest: 'caps-a',
    verification_protocol_digest: 'verifier-a',
    execution_budget_digest: 'budget-a',
    host_revision: 'host-a',
    host_composition_fingerprint: null,
    network_policy_digest: 'network-a',
    benchmark_revision: 'benchmark-a',
    task_revision: 'task-a',
    runner_revision: 'runner-a',
    review_contract_revision: 'review-a'
  }));
}

test('held-constant comparison covers every frozen comparison field', () => {
  const comparison = compareHeldConstant(heldRecords());
  for (const field of REQUIRED_HELD_CONSTANTS) assert.equal(comparison[field], true, field);
  assert.equal(comparison.valid, true);
  assert.deepEqual(comparison.mismatches, []);
  assert.equal(stableDigest({ a: { x: 1, y: 2 }, b: 3 }), stableDigest({ b: 3, a: { y: 2, x: 1 } }));
});

test('an intentional mismatch in every required held constant invalidates comparison and identifies the field', () => {
  const mutate = {
    prompt: (record) => { record.prompt_digest = 'prompt-b'; },
    success_constraints: (record) => { record.success_constraints_digest = 'success-b'; },
    start_state: (record) => { record.start_state_digest = 'start-b'; },
    model: (record) => { record.model = { ...record.model, parameters: { temperature: 0.5 } }; },
    capabilities: (record) => { record.capability_contract_digest = 'caps-b'; },
    verification: (record) => { record.verification_protocol_digest = 'verifier-b'; },
    budget: (record) => { record.execution_budget_digest = 'budget-b'; },
    host_revision: (record) => { record.host_revision = 'host-b'; },
    host_composition: (record) => { record.host_composition_fingerprint = 'profile-b'; },
    network_policy: (record) => { record.network_policy_digest = 'network-b'; },
    benchmark_revision: (record) => { record.benchmark_revision = 'benchmark-b'; },
    task_revision: (record) => { record.task_revision = 'task-b'; },
    runner_revision: (record) => { record.runner_revision = 'runner-b'; },
    review_contract_revision: (record) => { record.review_contract_revision = 'review-b'; }
  };

  assert.deepEqual(Object.keys(mutate), [...REQUIRED_HELD_CONSTANTS]);
  for (const field of REQUIRED_HELD_CONSTANTS) {
    const records = heldRecords();
    mutate[field](records[2]);
    const comparison = compareHeldConstant(records);
    assert.equal(comparison[field], false, field);
    assert.equal(comparison.valid, false, field);
    assert.ok(comparison.mismatches.some((entry) => entry.field === field), field);
  }
});

test('workspace capabilities cannot escape the isolated task root', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'series1-boundary-'));
  try {
    const workspace = new Series1Workspace(root);
    await fs.writeFile(path.join(root, 'ok.txt'), 'ok');
    assert.deepEqual(workspace.describe(), SERIES1_CAPABILITY_CONTRACT);
    assert.equal((await workspace.execute('read_file', { path: 'ok.txt' })).content, 'ok');
    await assert.rejects(() => workspace.execute('read_file', { path: '../escape.txt' }), /escapes Series 1 workspace/);
    await assert.rejects(() => workspace.execute('write_file', { path: '../../escape.txt', content: 'bad' }), /escapes Series 1 workspace/);
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
});

test('human-only review material cannot cross the candidate request/model/capability boundary', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'series1-review-boundary-'));
  const sentinel = '__SERIES1_HUMAN_REVIEW_ONLY_SENTINEL__';
  try {
    const sourceTask = SERIES1_TASKS.find((entry) => entry.id === 'S1-RESTRAINT-001');
    const task = { ...sourceTask, reviewReference: [sentinel] };
    await setupTask(task, root);
    const workspace = new Series1Workspace(root);
    const captures = [];
    const provider = {
      id: 'test-live-provider',
      async complete(input) {
        captures.push(structuredClone(input));
        return { content: 'done', capabilityCalls: [], usage: { input_tokens: 1, output_tokens: 1, total_tokens: 2 }, raw: null };
      }
    };
    const host = new LiveRuntimeHost({ id: 'native', revision: 'x', realFrameworkPath: 'test', provider, workspace });
    const events = [];
    host.attachObserver({ emit: (event) => events.push(event) }, 'run-non-leak');
    const request = buildCandidateRequest({
      task,
      capabilities: SERIES1_CAPABILITY_CONTRACT,
      maxSteps: 4,
      provenance: { series: 1, benchmark: 'v0.1', condition: 'classic' }
    });

    assertCandidateBoundary(request);
    assert.throws(() => assertCandidateBoundary({ ...request, human_reference: [sentinel] }), /Human-review-only field/);
    assert.doesNotMatch(JSON.stringify(request), new RegExp(sentinel));
    assert.doesNotMatch(JSON.stringify(await workspace.execute('read_file', { path: 'fact.txt' })), new RegExp(sentinel));

    await host.callModel({ request });
    await host.callModel({ request, history: [{ role: 'assistant', content: 'prior ordinary turn' }] });
    await host.callModel({ request, qlAct: { intent: 'inspect current evidence' } });
    await host.callModel({ series1Control: { system: 'controller', prompt: JSON.stringify(request), purpose: 'test-controller' } });

    assert.doesNotMatch(JSON.stringify(captures), new RegExp(sentinel));
    assert.doesNotMatch(JSON.stringify(events), new RegExp(sentinel));
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
});

test('live host observation retains complete model and capability IO', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'series1-trace-'));
  try {
    await fs.writeFile(path.join(root, 'fact.txt'), 'value');
    const workspace = new Series1Workspace(root);
    const provider = {
      id: 'test-live-provider',
      async complete() {
        return { content: 'done', capabilityCalls: [], usage: { input_tokens: 1, output_tokens: 1, total_tokens: 2 }, raw: { model: 'x' } };
      }
    };
    const host = new LiveRuntimeHost({ id: 'native', revision: 'x', realFrameworkPath: 'test', provider, workspace });
    const events = [];
    host.attachObserver({ emit: (event) => events.push(event) }, 'run-test');
    await host.callModel({ request: { input: 'prompt' } });
    await host.executeCapability({ name: 'read_file', args: { path: 'fact.txt' } });
    assert.equal(events.find((e) => e.event_type === 'model_requested').payload.input.prompt, '"prompt"');
    assert.equal(events.find((e) => e.event_type === 'model_returned').payload.output.content, 'done');
    assert.equal(events.find((e) => e.event_type === 'capability_returned').payload.result.content, 'value');
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
});

test('live response parser enforces the portable turn envelope', () => {
  assert.deepEqual(normalizeEnvelope('{"content":"done","capabilityCalls":[]}').capabilityCalls, []);
  assert.equal(parseJsonObject('prefix {"choice":"P4"} suffix').choice, 'P4');
  assert.throws(() => parseJsonObject('not json'), /did not return a JSON object/);
});

test('benchmark v0.1 executable prompts match the normative six-task corpus exactly', () => {
  const expected = new Map([
    ['S1-CODE-001', 'Inspect the workspace and fix `buildIndex` so records are keyed by their normalized id and, when multiple records normalize to the same id, the latest record wins. Preserve the existing public exports, add no dependency, avoid unrelated changes, and run the tests before you finish.'],
    ['S1-RESEARCH-001', 'Using only the files in this workspace, prepare a concise research note answering: (1) which execution surfaces are currently supported, (2) what evidence establishes that, (3) which older statement is superseded, and (4) what remains unresolved about comparing runs. Cite the filename supporting every material claim. Do not edit the workspace and do not import outside knowledge.'],
    ['S1-EPISTEMIC-001', 'Using only the evidence files in this workspace, give the best current explanation for runs where the task result succeeds but telemetry upload fails. Separate what is directly observed, what is inferred, and what is still open. State at least one piece of evidence that would materially weaken or falsify your current explanation. Do not edit any file.'],
    ['S1-SKILL-001', 'Complete the incoming request using the local `SKILL.md`. Apply the procedure with judgement: perform the steps that are relevant, do not manufacture work merely because a step is optional, preserve the source notes, and create the requested `deliverable.md`.'],
    ['S1-AGENCY-001', 'Make this workspace truthful and ready. Inspect the current state, verify actual behaviour before deciding what to change, preserve the public API, and make only the narrowest justified correction. Leave STATUS.md describing the verified current state and the evidence you used.'],
    ['S1-RESTRAINT-001', 'According to `fact.txt`, what is the preferred review format? Answer in one sentence. Do not edit anything.']
  ]);
  assert.equal(SERIES1_TASKS.length, expected.size);
  for (const task of SERIES1_TASKS) assert.equal(task.prompt, expected.get(task.id), task.id);
});

test('benchmark v0.1 spans six distinct human-review task families with frozen protocols', async () => {
  const expected = new Map([
    ['S1-CODE-001', 'code'],
    ['S1-RESEARCH-001', 'local-research'],
    ['S1-EPISTEMIC-001', 'epistemic-understanding'],
    ['S1-SKILL-001', 'skill-agency'],
    ['S1-AGENCY-001', 'agency-recovery'],
    ['S1-RESTRAINT-001', 'bounded-restraint']
  ]);
  assert.equal(SERIES1_TASKS.length, expected.size);
  for (const task of SERIES1_TASKS) {
    assert.equal(task.category, expected.get(task.id));
    assert.ok(task.prompt.length > 20);
    assert.ok(task.successConditions.length >= 3);
    assert.ok(task.verificationProtocol.length >= 3);
    assert.ok(task.reviewFocus.length >= 6);
    assert.ok(task.reviewReference.length >= 1);
    assert.equal('quality' in task, false);

    const root = await fs.mkdtemp(path.join(os.tmpdir(), `series1-${task.id}-`));
    try {
      await setupTask(task, root);
      const names = await fs.readdir(root);
      assert.ok(names.length > 0);
      if (task.id === 'S1-RESEARCH-001' || task.id === 'S1-EPISTEMIC-001' || task.id === 'S1-RESTRAINT-001') {
        const snapshot = {};
        async function walk(dir = '.') {
          for (const entry of await fs.readdir(path.join(root, dir), { withFileTypes: true })) {
            const child = dir === '.' ? entry.name : `${dir}/${entry.name}`;
            if (entry.isDirectory()) await walk(child);
            else snapshot[child] = await fs.readFile(path.join(root, child), 'utf8');
          }
        }
        await walk();
        const verification = await verifyTask(task, root, { before: snapshot, after: { ...snapshot }, output: '' });
        assert.equal(verification.objective_checks_pass, true);
      }
    } finally {
      await fs.rm(root, { recursive: true, force: true });
    }
  }
});

test('benchmark/task/workspace/verifier fingerprints are reproducible across independent materializations', async () => {
  const first = await buildBenchmarkFreeze();
  const second = await buildBenchmarkFreeze();
  const reproducibility = await verifyFreezeReproducibility();
  assert.equal(reproducibility.valid, true);
  assert.equal(first.benchmark_revision, second.benchmark_revision);
  assert.equal(first.capability_contract_digest, stableDigest(SERIES1_CAPABILITY_CONTRACT));
  assert.equal(Object.keys(first.tasks).length, 6);
  for (const task of SERIES1_TASKS) {
    assert.equal(first.tasks[task.id].task_revision, second.tasks[task.id].task_revision, task.id);
    assert.equal(first.tasks[task.id].starting_workspace_digest, second.tasks[task.id].starting_workspace_digest, task.id);
    assert.ok(first.tasks[task.id].starting_workspace_files.length > 0, task.id);
    assert.ok(first.tasks[task.id].verification_protocol_digest, task.id);
  }
});

test('QL controller carrier decisions are accepted in both documented and flat shapes', async () => {
  const { createModelDrivenQLPolicy } = await import('../../comparison/series1/policy.mjs');

  const circuit = {
    id: 'run_test:c0',
    depth: 0,
    face: 'direct',
    activePosition: { id: 'P0' },
    residues: [],
    trajectory: []
  };
  const request = {
    input: 'According to `fact.txt`, what is the preferred review format?',
    successConditions: ['Answer from fact.txt.'],
    capabilities: [{ id: 'read_file', args: { path: 'required relative file path' } }]
  };
  const scripted = (control) => ({ mode: 'direct', async callModel() { return { control }; } });

  const flat = createModelDrivenQLPolicy({ mode: 'direct' });
  const flatAct = await flat.nextAct({
    circuit,
    request,
    host: scripted({ carrier: 'capability', capability: 'read_file', args: { path: 'fact.txt' } })
  });
  assert.deepEqual(flatAct.carrier, { kind: 'capability', name: 'read_file', args: { path: 'fact.txt' } });

  const documented = createModelDrivenQLPolicy({ mode: 'direct' });
  const documentedAct = await documented.nextAct({
    circuit,
    request,
    host: scripted({ carrier: { kind: 'capability', name: 'read_file', args: { path: 'fact.txt' } } })
  });
  assert.deepEqual(documentedAct.carrier, { kind: 'capability', name: 'read_file', args: { path: 'fact.txt' } });

  const modelCarrier = createModelDrivenQLPolicy({ mode: 'direct' });
  const modelAct = await modelCarrier.nextAct({
    circuit,
    request,
    host: scripted({ carrier: 'model' })
  });
  assert.deepEqual(modelAct.carrier, { kind: 'model' });

  const nullCarrier = createModelDrivenQLPolicy({ mode: 'direct' });
  const nullAct = await nullCarrier.nextAct({
    circuit,
    request,
    host: scripted({ intent: 'answer directly' })
  });
  assert.deepEqual(nullAct.carrier, { kind: 'model' });

  const refusing = createModelDrivenQLPolicy({ mode: 'direct' });
  await assert.rejects(
    refusing.nextAct({ circuit, request, host: scripted({ carrier: 'teleport' }) }),
    /unsupported carrier 'teleport'/
  );
});

test('closure-request carriers are preserved and routed to the determination path', async () => {
  const { createModelDrivenQLPolicy } = await import('../../comparison/series1/policy.mjs');
  const circuit = { id: 'run_t:c0', depth: 0, face: 'direct', activePosition: { id: 'P4' }, residues: [], trajectory: [] };
  const request = {
    input: 'bounded task',
    successConditions: ['done'],
    capabilities: [{ id: 'read_file', args: {} }],
    maxSteps: 16
  };
  const scripted = (control) => ({ mode: 'direct', async callModel() { return { control }; } });

  const policy = createModelDrivenQLPolicy({ mode: 'direct' });
  const act = await policy.nextAct({
    circuit,
    request,
    host: scripted({ intent: 'close the bounded request', carrier: { kind: 'internal_control', name: 'close', args: { reason: 'already achieved' } } })
  });
  assert.equal(act.carrier.name, 'close');
  assert.equal(act.metadata.closure_request, true);

  const interpretation = await policy.interpret({ circuit, difference: { operation_success: true }, act, request });
  assert.equal(interpretation.destination, 'P5');
  assert.equal(interpretation.witness.structural_facts.closure_request, true);
  assert.deepEqual(interpretation.residueDelta, {});
});

test('next-act control payload discloses the execution budget', async () => {
  const { createModelDrivenQLPolicy } = await import('../../comparison/series1/policy.mjs');
  const seen = [];
  const policy = createModelDrivenQLPolicy({ mode: 'direct' });
  await policy.nextAct({
    circuit: { id: 'run_t:c0', depth: 0, face: 'direct', activePosition: { id: 'P0' }, residues: [], trajectory: [] },
    request: { input: 'x', successConditions: [], capabilities: [], maxSteps: 16 },
    host: {
      mode: 'direct',
      async callModel(request2) {
        seen.push(JSON.parse(request2.series1Control.prompt));
        return { control: { carrier: 'model' } };
      }
    }
  });
  assert.equal(seen[0].budget.max_steps, 16);
  assert.equal(seen[0].budget.steps_used, 0);
  assert.equal(seen[0].budget.allowance.active_position, 'P0');
  assert.ok(seen[0].stipulations, 'stipulations must ride the control payload');
});

test('determination synthesis falls back to the returned answer and refuses to close on empty synthesis', async () => {
  const { createModelDrivenQLPolicy } = await import('../../comparison/series1/policy.mjs');
  const circuit = { id: 'run_t:c0', depth: 0, face: 'direct', activePosition: { id: 'P5' }, residues: [], trajectory: [] };
  const scripted = (control) => ({ mode: 'direct', async callModel() { return { control }; } });
  const request = { input: 'x', successConditions: ['done'], taskId: 'T', __series1Host: null };

  const answered = createModelDrivenQLPolicy({ mode: 'direct' });
  request.__series1Host = scripted({ requested_outcome: 'close', answer: 'The preferred review format is Markdown.' });
  const good = await answered.proposeDetermination({ circuit, request });
  assert.equal(good.synthesis, 'The preferred review format is Markdown.');
  assert.equal(good.requested_outcome, 'close');
  assert.deepEqual(good.unresolved_refs, []);

  const empty = createModelDrivenQLPolicy({ mode: 'direct' });
  let calls = 0;
  request.__series1Host = {
    mode: 'direct',
    async callModel() { calls += 1; return { control: { requested_outcome: 'close' } }; }
  };
  const gated = await empty.proposeDetermination({ circuit, request });
  assert.equal(calls, 2, 'empty synthesis should trigger exactly one re-ask');
  assert.equal(gated.requested_outcome, 'reopen');
  assert.ok(gated.unresolved_refs.includes('determination-synthesis-empty'));

  const reopening = createModelDrivenQLPolicy({ mode: 'direct' });
  let reopenCalls = 0;
  request.__series1Host = {
    mode: 'direct',
    async callModel() { reopenCalls += 1; return { control: { requested_outcome: 'reopen', unresolved_refs: ['open'] } }; }
  };
  const reopened = await reopening.proposeDetermination({ circuit, request });
  assert.equal(reopenCalls, 1, 'reopen requests must not be re-asked');
  assert.equal(reopened.requested_outcome, 'reopen');
});

test('native transport repairs malformed turns by re-asking, retaining the failed attempts', async () => {
  const { NativeOpenAICompatibleProvider } = await import('../../comparison/series1/providers.mjs');
  const provider = new NativeOpenAICompatibleProvider({ baseUrl: 'https://stub.invalid', apiKey: 'test-key', model: 'stub-model' });
  const realFetch = globalThis.fetch;
  const bodies = [];
  globalThis.fetch = async (_url, options) => {
    const sent = JSON.parse(options.body);
    bodies.push(sent.messages.map((m) => m.role));
    if (bodies.length === 1) {
      return { ok: true, json: async () => ({ choices: [{ message: { content: 'I will read the governing procedure first, then write the deliverable.' }, finish_reason: 'stop' }], usage: { prompt_tokens: 10, completion_tokens: 5, total_tokens: 15 } }) };
    }
    return { ok: true, json: async () => ({ choices: [{ message: { content: '{"content":"done","capabilityCalls":[]}' }, finish_reason: 'stop' }], usage: { prompt_tokens: 30, completion_tokens: 4, total_tokens: 34 } }) };
  };
  try {
    const result = await provider.complete({ prompt: 'do the task' });
    assert.equal(result.content, 'done');
    assert.equal(result.repairs, 1);
    assert.deepEqual(result.usage, { input_tokens: 40, output_tokens: 9, total_tokens: 49, cached_input_tokens: 0 });
    assert.equal(result.raw.failed_attempts.length, 1);
    assert.deepEqual(bodies[1], ['system', 'user', 'assistant', 'user'], 'repair turn must carry the failed response back');
  } finally {
    globalThis.fetch = realFetch;
  }
});

test('native transport repairs control-mode turns and still fails closed after the repair budget', async () => {
  const { NativeOpenAICompatibleProvider } = await import('../../comparison/series1/providers.mjs');
  const provider = new NativeOpenAICompatibleProvider({ baseUrl: 'https://stub.invalid', apiKey: 'test-key', model: 'stub-model' });
  const realFetch = globalThis.fetch;
  let calls = 0;
  globalThis.fetch = async () => {
    calls += 1;
    return { ok: true, json: async () => ({ choices: [{ message: { content: 'still not json' }, finish_reason: 'stop' }], usage: { prompt_tokens: 1, completion_tokens: 1, total_tokens: 2 } }) };
  };
  try {
    await assert.rejects(
      provider.complete({ mode: 'control', prompt: 'decide' }),
      /did not return a JSON object/
    );
    assert.equal(calls, 3, 'initial turn plus two repairs');
  } finally {
    globalThis.fetch = realFetch;
  }
});

test('QL control turns run under the Relational Logos system prompt', async () => {
  const { createModelDrivenQLPolicy } = await import('../../comparison/series1/policy.mjs');
  const policy = createModelDrivenQLPolicy({ mode: 'direct' });
  let seenSystem = null;
  await policy.nextAct({
    circuit: { id: 'run_t:c0', depth: 0, face: 'direct', activePosition: { id: 'P0' }, residues: [], trajectory: [] },
    request: { input: 'x', successConditions: [], capabilities: [], maxSteps: 16 },
    host: {
      mode: 'direct',
      async callModel(request2) {
        seenSystem = request2.series1Control.system;
        return { control: { carrier: 'model' } };
      }
    }
  });
  assert.ok(seenSystem.startsWith('# Relational Logos'), 'control turns must carry the QL relational protocol as the standing system text');
  assert.ok(seenSystem.includes('Return exactly one JSON object'), 'turn-specific schema instruction composed on top');
});

test('per-position allowance refusal fires as a typed closure request without a model call', async () => {
  const { createModelDrivenQLPolicy } = await import('../../comparison/series1/policy.mjs');
  const circuit = { id: 'run_t:c0', depth: 0, face: 'direct', activePosition: { id: 'P1' }, residues: [], trajectory: [] };
  const request = { input: 'x', successConditions: ['done'], capabilities: [], maxSteps: 16 };
  let calls = 0;
  const host = {
    mode: 'direct',
    async callModel() { calls += 1; return { control: { carrier: 'model' } }; }
  };
  const policy = createModelDrivenQLPolicy({ mode: 'direct', allowanceSchedule: { P1: 2 } });
  const acts = [];
  for (let i = 0; i < 6; i += 1) acts.push(await policy.nextAct({ circuit, request, host }));
  assert.equal(calls, 4, 'two scheduled acts + two grace acts; refusals skip the model');
  assert.equal(acts[0].metadata.allowance_refusal, undefined);
  assert.equal(acts[2].metadata.allowance_refusal.position, 'P1');
  assert.equal(acts[2].metadata.closure_request, true);
  assert.equal(acts[2].metadata.allowance_refusal.consumed, 2);
  assert.equal(acts[2].metadata.allowance_refusal.grace_extension, 2, 'first refusal grants the recorded grace extension');
  assert.equal(acts[5].metadata.allowance_refusal.consumed, 4, 'post-grace refusal reports full consumption');
  assert.equal(acts[5].metadata.allowance_refusal.grace_extension, 0);
  const routed = await policy.interpret({ circuit, difference: { operation_success: true }, act: acts[2], request });
  assert.equal(routed.destination, 'P5');
  assert.equal(routed.witness.structural_facts.allowance_refusal.position, 'P1');
});

test('task conditions classify into typed goal and exclusion stipulations', async () => {
  const { classifyStipulations } = await import('../../comparison/series1/policy.mjs');
  const bindings = classifyStipulations([
    'Deliver a one-page note answering all four questions.',
    'Do not modify any file.',
    'Never import outside knowledge.'
  ]);
  assert.deepEqual(bindings.map((b) => b.kind), ['goal', 'exclusion', 'exclusion']);
  assert.equal(bindings[1].id, 'S2');
});

test('a violated exclusion stipulation forces the closure to record failure', async () => {
  const { createModelDrivenQLPolicy } = await import('../../comparison/series1/policy.mjs');
  const policy = createModelDrivenQLPolicy({ mode: 'direct' });
  const request = {
    input: 'x',
    successConditions: ['Answer the questions.', 'Do not modify any file.'],
    taskId: 'T',
    __series1Host: {
      mode: 'direct',
      async callModel() {
        return { control: { status: 'close', task_success: true, stipulation_verdicts: [{ id: 'S1', verdict: 'met' }, { id: 'S2', verdict: 'violated', evidence: 'wrote research-note.md' }] } };
      }
    }
  };
  const verdict = await policy.evaluateClosure({ circuit: {}, determination: {}, frame: {}, evaluations: [], request });
  assert.equal(verdict.status, 'close');
  assert.equal(verdict.task_success, 'false');
  assert.ok(verdict.rationale.includes('S2'));
  assert.equal(verdict.stipulation_verdicts.length, 2);
});

test('compressed interpret applies the loop law by carrier kind without a model call', async () => {
  const { createModelDrivenQLPolicy } = await import('../../comparison/series1/policy.mjs');
  const policy = createModelDrivenQLPolicy({ mode: 'direct', compressedControl: true });
  let modelCalls = 0;
  const request = {
    input: 'task',
    successConditions: [],
    __series1Host: { mode: 'direct', async callModel() { modelCalls += 1; return { control: {} }; } }
  };
  const actFor = (carrier) => ({ id: 'run_t:c0:act:0', carrier, claimed_position: 'P1', metadata: {} });
  const emptyCircuit = { activePosition: { id: 'P1' }, residues: [], trajectory: [] };

  // Failed operation -> P1 with failure residue.
  const failed = await policy.interpret({
    circuit: emptyCircuit,
    difference: { operation_success: false, raw_result: { error: 'ENOENT: no such file or directory' } },
    act: actFor({ kind: 'capability', name: 'read_file', args: { path: 'missing.txt' } }),
    request
  });
  assert.equal(failed.destination, 'P1');
  assert.equal(failed.residueDelta.create[0].kind, 'material');
  assert.equal(failed.witness.compressed_control.rule, 'interpret:failed-operation');

  // Successful read of unprocessed evidence -> P1.
  const read = await policy.interpret({
    circuit: emptyCircuit,
    difference: { operation_success: true, raw_result: { ok: true, path: 'fact.txt', content: 'markdown' } },
    act: actFor({ kind: 'capability', name: 'read_file', args: { path: 'fact.txt' } }),
    request
  });
  assert.equal(read.destination, 'P1');
  assert.equal(read.witness.compressed_control.rule, 'interpret:successful-read');
  assert.equal(read.witness.compressed_control.basis.unprocessed, true);

  // Re-read of already-recorded content is disclosed in the basis, still P1.
  const alreadyRead = {
    activePosition: { id: 'P1' },
    residues: [{
      id: 'run_t:c0:res:0', kind: 'material', position: 'P1',
      value: { difference: { operation_success: true, raw_result: { ok: true, path: 'fact.txt', content: 'markdown' } } }
    }],
    trajectory: []
  };
  const reread = await policy.interpret({
    circuit: alreadyRead,
    difference: { operation_success: true, raw_result: { ok: true, path: 'fact.txt', content: 'markdown' } },
    act: actFor({ kind: 'capability', name: 'read_file', args: { path: 'fact.txt' } }),
    request
  });
  assert.equal(reread.destination, 'P1');
  assert.equal(reread.witness.compressed_control.basis.unprocessed, false);

  // Successful mutation -> P2 effect.
  const written = await policy.interpret({
    circuit: emptyCircuit,
    difference: { operation_success: true, raw_result: { ok: true, path: 'deliverable.md', bytes: 42 } },
    act: actFor({ kind: 'capability', name: 'write_file', args: { path: 'deliverable.md', content: 'x' } }),
    request
  });
  assert.equal(written.destination, 'P2');
  assert.equal(written.residueDelta.create[0].kind, 'effect');
  assert.equal(written.witness.compressed_control.rule, 'interpret:successful-mutation');

  // Successful operation (passing tests) -> P2; failing run is a failed operation -> P1.
  const testsPassed = await policy.interpret({
    circuit: emptyCircuit,
    difference: { operation_success: true, raw_result: { ok: true, exit_code: 0, stdout: 'pass', stderr: '' } },
    act: actFor({ kind: 'capability', name: 'run_tests', args: {} }),
    request
  });
  assert.equal(testsPassed.destination, 'P2');
  assert.equal(testsPassed.witness.compressed_control.rule, 'interpret:successful-operation');

  const testsFailed = await policy.interpret({
    circuit: emptyCircuit,
    difference: { operation_success: true, raw_result: { ok: false, exit_code: 1, stdout: '', stderr: 'fail' } },
    act: actFor({ kind: 'capability', name: 'run_tests', args: {} }),
    request
  });
  assert.equal(testsFailed.destination, 'P1');
  assert.equal(testsFailed.witness.compressed_control.rule, 'interpret:failed-operation');

  // Delivered model realisation with non-empty content -> P5; empty model return -> P2.
  const realised = await policy.interpret({
    circuit: emptyCircuit,
    difference: { operation_success: true, raw_result: { content: 'The preferred review format is Markdown.', capabilityCalls: [] } },
    act: actFor({ kind: 'model' }),
    request
  });
  assert.equal(realised.destination, 'P5');
  assert.equal(realised.residueDelta.create[0].kind, 'determination');
  assert.equal(realised.witness.compressed_control.rule, 'interpret:delivered-realisation');

  const emptyReturn = await policy.interpret({
    circuit: emptyCircuit,
    difference: { operation_success: true, raw_result: { content: '', capabilityCalls: [] } },
    act: actFor({ kind: 'model' }),
    request
  });
  assert.equal(emptyReturn.destination, 'P2');
  assert.equal(emptyReturn.witness.compressed_control.rule, 'interpret:empty-return');

  assert.equal(modelCalls, 0, 'compressed interpret must decide without any model call');
});

test('compressed next-act planner reads unread task files before any model turn', async () => {
  const { createModelDrivenQLPolicy } = await import('../../comparison/series1/policy.mjs');
  let modelCalls = 0;
  const host = { mode: 'direct', async callModel() { modelCalls += 1; return { control: { carrier: 'model' } }; } };
  const request = {
    input: 'According to `fact.txt`, what is the preferred review format? Answer in one sentence. Do not edit anything.',
    successConditions: ['Answer from fact.txt.', 'Do not edit anything.'],
    capabilities: [{ id: 'read_file', args: { path: 'required relative file path' } }],
    maxSteps: 16
  };
  const emptyCircuit = { id: 'run_t:c0', depth: 0, face: 'direct', activePosition: { id: 'P1' }, residues: [], trajectory: [] };

  const policy = createModelDrivenQLPolicy({ mode: 'direct', compressedControl: true });
  const readAct = await policy.nextAct({ circuit: emptyCircuit, request, host });
  assert.equal(modelCalls, 0, 'unread task file must be compressed into a deterministic read');
  assert.deepEqual(readAct.carrier, { kind: 'capability', name: 'read_file', args: { path: 'fact.txt' } });
  assert.equal(readAct.metadata.compressed_control.rule, 'next-act:read-unread-task-file');
  assert.deepEqual(readAct.metadata.compressed_control.basis.unread_files, ['fact.txt']);

  const readRecorded = {
    ...emptyCircuit,
    residues: [{
      id: 'run_t:c0:res:0', kind: 'material', position: 'P1',
      value: { difference: { operation_success: true, raw_result: { ok: true, path: 'fact.txt', content: 'markdown' } } }
    }]
  };
  const modelAct = await policy.nextAct({ circuit: readRecorded, request, host });
  assert.equal(modelCalls, 1, 'with nothing unread the turn falls through to the model-carried act');
  assert.deepEqual(modelAct.carrier, { kind: 'model' });

  const uncompressed = createModelDrivenQLPolicy({ mode: 'direct' });
  const uncompressedAct = await uncompressed.nextAct({ circuit: emptyCircuit, request, host });
  assert.equal(modelCalls, 2, 'without compressed control the same state asks the model');
  assert.deepEqual(uncompressedAct.carrier, { kind: 'model' });
});

test('compressed planner requests closure only when every goal condition is machine-verified', async () => {
  const { createModelDrivenQLPolicy } = await import('../../comparison/series1/policy.mjs');
  let modelCalls = 0;
  const host = { mode: 'direct', async callModel() { modelCalls += 1; return { control: { carrier: 'model' } }; } };
  const passedTestsResidue = {
    id: 'run_t:c0:res:1', kind: 'effect', position: 'P2',
    value: { difference: { operation_success: true, raw_result: { ok: true, exit_code: 0, stdout: 'pass', stderr: '' } } }
  };
  const request = {
    input: 'fix the code and run the tests',
    successConditions: ['All existing tests pass.'],
    capabilities: [{ id: 'run_tests', args: {} }],
    maxSteps: 16
  };
  const advancedCircuit = {
    id: 'run_t:c0', depth: 0, face: 'direct', activePosition: { id: 'P4' },
    residues: [passedTestsResidue],
    trajectory: [{ from: 'P2', to: 'P4', relation: 'R24' }]
  };

  const policy = createModelDrivenQLPolicy({ mode: 'direct', compressedControl: true });
  const closureAct = await policy.nextAct({ circuit: advancedCircuit, request, host });
  assert.equal(modelCalls, 0, 'machine-verified closure must be claimed without a model turn');
  assert.equal(closureAct.carrier.kind, 'internal_control');
  assert.equal(closureAct.carrier.name, 'close');
  assert.equal(closureAct.metadata.closure_request, true);
  assert.equal(closureAct.metadata.compressed_control.rule, 'next-act:conditions-verified-closure');
  assert.deepEqual(closureAct.metadata.compressed_control.basis.verified_goals, ['S1']);

  // The closure request routes to determination through the interpret gate,
  // and the compressed planner's rule rides the witness.
  const routed = await policy.interpret({
    circuit: advancedCircuit,
    difference: { operation_success: true },
    act: closureAct,
    request
  });
  assert.equal(routed.destination, 'P5');
  assert.equal(routed.witness.compressed_control.rule, 'next-act:conditions-verified-closure');

  // No closure on the first step: nothing is verified before an act has run.
  const freshCircuit = { ...advancedCircuit, trajectory: [] };
  const earlyAct = await policy.nextAct({ circuit: freshCircuit, request, host });
  assert.equal(modelCalls, 1, 'an unverified first turn falls through to the model');
  assert.deepEqual(earlyAct.carrier, { kind: 'model' });

  // Exclusion-only conditions are never machine-verifiable closure grounds.
  const exclusionRequest = { ...request, successConditions: ['Do not modify any file.'] };
  const exclusionAct = await policy.nextAct({ circuit: advancedCircuit, request: exclusionRequest, host });
  assert.equal(modelCalls, 2);
  assert.deepEqual(exclusionAct.carrier, { kind: 'model' });
});
