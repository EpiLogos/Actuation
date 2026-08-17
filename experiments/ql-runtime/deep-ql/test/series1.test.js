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
  DEEPSEEK_BASE_URL
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

test('Series 1 uses provider-native DeepSeek candidate defaults', () => {
  assert.equal(SERIES1_PROVIDER, 'deepseek');
  assert.equal(SERIES1_DEFAULT_MODEL, 'deepseek-v4-flash');
  assert.equal(DEEPSEEK_BASE_URL, 'https://api.deepseek.com');
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
