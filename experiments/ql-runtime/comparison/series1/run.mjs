import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { ClassicRuntime } from '../../foundation/classic-runtime/index.js';
import { QLDirectCoreRuntime } from '../../foundation/ql-core-runtime/index.js';
import { createRunManifest, executeRun, runIdForManifest } from '../../foundation/optics/index.js';
import { createDeepQLRuntimeClass } from '../../deep-ql/index.js';
import { createModelDrivenQLPolicy, bindSeries1Host } from './policy.mjs';
import { providerForHost, SERIES1_PROVIDER, series1ModelId } from './providers.mjs';
import { Series1Workspace, createLiveHost } from './host.mjs';
import {
  CONDITIONS,
  DETERMINATION,
  SERIES1_CAPABILITY_CONTRACT,
  SERIES1_SCHEMA,
  assertLiveManifest,
  compareHeldConstant,
  stableDigest
} from './contract.mjs';
import { buildCandidateRequest } from './candidate-boundary.mjs';
import { assertNoConfiguredSecrets, sanitizeEvidence } from './evidence.mjs';
import { buildBenchmarkFreeze, fingerprintWorkspace } from './freeze.mjs';
import { getTask, setupTask, verifyTask } from './tasks.mjs';

const SPEC_REVISION = 'f9d056c54caf094eb672f005ce3c8cbde4de0a5b+QL-PAIRING-SQUARES-CLARIFICATION-08-14-2026+SERIES1-BENCHMARK-V0.1';
const DEEP_CLASS = createDeepQLRuntimeClass(QLDirectCoreRuntime);
const CAPABILITIES = Object.freeze(SERIES1_CAPABILITY_CONTRACT.map(({ name, args }) => ({ id: name, args })));

function args() {
  const values = process.argv.slice(2);
  const read = (name, fallback = null) => {
    const index = values.indexOf(`--${name}`);
    return index >= 0 ? values[index + 1] : fallback;
  };
  return {
    host: read('host', process.env.QL_SERIES1_HOST ?? 'native'),
    task: read('task', process.env.QL_SERIES1_TASK ?? 'S1-CODE-001'),
    repetitions: Number(read('repetitions', process.env.QL_SERIES1_REPETITIONS ?? '1')),
    maxSteps: Number(read('max-steps', process.env.QL_SERIES1_MAX_STEPS ?? '16'))
  };
}

async function snapshot(root) {
  const out = {};
  async function walk(relative = '.') {
    const absolute = path.join(root, relative);
    const entries = await fs.readdir(absolute, { withFileTypes: true });
    for (const entry of entries.sort((a, b) => a.name.localeCompare(b.name))) {
      const child = relative === '.' ? entry.name : path.join(relative, entry.name);
      if (entry.isDirectory()) await walk(child);
      else out[child.split(path.sep).join('/')] = await fs.readFile(path.join(root, child), 'utf8');
    }
  }
  await walk();
  return out;
}

function canonicalModel() {
  const provider = process.env.QL_SERIES1_PROVIDER ?? SERIES1_PROVIDER;
  if (provider !== SERIES1_PROVIDER) {
    throw new Error(`Series 1 is currently stipulated to provider '${SERIES1_PROVIDER}', not '${provider}'.`);
  }
  return { provider, id: series1ModelId(), parameters: { temperature: 0 } };
}

function runtimeDescriptor(condition) {
  if (condition === 'classic') return { id: 'classic', version: '0.1.0-foundation' };
  if (condition === 'ql-direct') return { id: 'ql-core', version: '0.1.0-foundation' };
  if (condition === 'ql-deep') return { id: 'ql-deep', version: '0.1.0-deep-candidate' };
  throw new Error(`Unknown condition '${condition}'.`);
}

function createRuntime(condition, runId) {
  if (condition === 'classic') return { runtime: new ClassicRuntime(), policy: null };
  if (condition === 'ql-direct') {
    const policy = createModelDrivenQLPolicy({ mode: 'direct', operatorRunId: `${runId}:operators` });
    return { runtime: new QLDirectCoreRuntime({ policy }), policy };
  }
  const policy = createModelDrivenQLPolicy({ mode: 'deep', operatorRunId: `${runId}:operators` });
  return { runtime: new DEEP_CLASS({ policy }), policy };
}

async function runCondition({ hostId, task, condition, repetition, model, maxSteps, freeze }) {
  const temp = await fs.mkdtemp(path.join(os.tmpdir(), `ql-series1-${task.id}-${condition}-`));
  let provider = null;
  try {
    await setupTask(task, temp);
    const before = await snapshot(temp);
    const byteFingerprint = await fingerprintWorkspace(temp);
    const frozenTask = freeze.tasks[task.id];
    if (!frozenTask || byteFingerprint.digest !== frozenTask.starting_workspace_digest) {
      const error = new Error(`Series 1 frozen workspace mismatch for ${task.id}: expected ${frozenTask?.starting_workspace_digest ?? 'missing'}, got ${byteFingerprint.digest}`);
      error.code = 'SERIES1_FREEZE_MISMATCH';
      throw error;
    }

    const startStateDigest = byteFingerprint.digest;
    const workspace = new Series1Workspace(temp);
    const actualCapabilityContract = workspace.describe();
    const capabilityContractDigest = stableDigest(actualCapabilityContract);
    if (capabilityContractDigest !== freeze.capability_contract_digest) {
      const error = new Error(`Series 1 capability contract mismatch: expected ${freeze.capability_contract_digest}, got ${capabilityContractDigest}`);
      error.code = 'SERIES1_FREEZE_MISMATCH';
      throw error;
    }

    provider = providerForHost(hostId);
    await provider.assertReady();
    const host = createLiveHost({ hostId, provider, workspace });
    const descriptor = runtimeDescriptor(condition);
    const logicalEnvironment = {
      kind: 'isolated-series1-workspace',
      task_id: task.id,
      network: 'model-provider-only',
      workspace_network_tools: false
    };
    const networkPolicyDigest = stableDigest({
      network: logicalEnvironment.network,
      workspace_network_tools: logicalEnvironment.workspace_network_tools
    });
    const baseRequest = buildCandidateRequest({
      task,
      capabilities: CAPABILITIES,
      maxSteps,
      provenance: { series: 1, benchmark: 'v0.1', repetition, condition }
    });
    const manifest = createRunManifest({
      taskId: task.id,
      fixtureId: `series1:v0.1:${task.id}:r${repetition}`,
      host,
      runtime: descriptor,
      specRevision: SPEC_REVISION,
      model,
      capabilities: CAPABILITIES,
      environment: logicalEnvironment,
      successConditions: task.successConditions,
      startState: { digest: startStateDigest }
    });
    const expectedRunId = runIdForManifest(manifest);
    const { runtime, policy } = createRuntime(condition, expectedRunId);
    const request = bindSeries1Host(baseRequest, host);
    const started = performance.now();
    const record = await executeRun({ runtime, host, request, manifest });
    const elapsedMs = performance.now() - started;

    if (policy?.getOperatorEvents) {
      for (const event of policy.getOperatorEvents()) {
        record.events.push({ ...event, record_index: record.events.length, channel: 'runtime-semantic' });
      }
    }

    await host.capturePortableTrace(record.events, { condition });
    const after = await snapshot(temp);
    const output = typeof record.result?.outcome === 'string' ? record.result.outcome : JSON.stringify(record.result?.outcome ?? '');
    const verification = await verifyTask(task, temp, { before, after, output, record });
    const usage = host.snapshotUsage();
    const executionBudget = { max_steps: maxSteps };
    const hostNativeEvidence = await host.snapshotNativeEvidence();

    return {
      schema: SERIES1_SCHEMA,
      provider_mode: 'live',
      fixture_provider: false,
      source_repository_revision: process.env.GITHUB_SHA ?? process.env.QL_SERIES1_SOURCE_REVISION ?? null,
      benchmark_revision: freeze.benchmark_revision,
      benchmark_spec_revision: freeze.benchmark_spec_revision,
      task_corpus_revision: freeze.task_corpus_revision,
      task_revision: frozenTask.task_revision,
      runner_revision: freeze.runner_revision,
      review_contract_revision: freeze.review_contract_revision,
      host: { id: host.id, revision: host.revision, real_framework_path: host.realFrameworkPath },
      host_revision: host.revision,
      host_composition_fingerprint: host.compositionFingerprint ?? null,
      host_native_evidence: hostNativeEvidence,
      condition,
      repetition,
      runtime: { id: runtime.id, version: runtime.version },
      model,
      task_id: task.id,
      task_digest: stableDigest({ prompt: task.prompt, success_conditions: task.successConditions }),
      prompt_digest: frozenTask.prompt_digest,
      success_constraints_digest: frozenTask.success_constraints_digest,
      start_state_digest: startStateDigest,
      end_state_digest: stableDigest(after),
      capability_digest: capabilityContractDigest,
      capability_contract_digest: capabilityContractDigest,
      capability_contract: actualCapabilityContract,
      verification_protocol_digest: frozenTask.verification_protocol_digest,
      execution_budget_digest: stableDigest(executionBudget),
      network_policy_digest: networkPolicyDigest,
      execution_budget: executionBudget,
      prompt: task.prompt,
      success_conditions: task.successConditions,
      starting_workspace: before,
      starting_workspace_fingerprint: byteFingerprint,
      final_workspace: after,
      outcome: output,
      verification,
      execution_status: record.status.execution,
      semantic_status: record.status.semantic,
      elapsed_ms: elapsedMs,
      model_calls: usage.model_calls,
      total_tokens: usage.total_tokens,
      input_tokens: usage.input_tokens,
      output_tokens: usage.output_tokens,
      model_cost: usage.model_cost,
      capability_calls: record.events.filter((event) => event.channel === 'host' && event.event_type === 'capability_requested').length,
      ql_semantic_events: record.events.filter((event) => event.channel === 'runtime-semantic').length,
      operator_events: record.events.filter((event) => ['conjugate_started','conjugate_completed','child_started','child_completed','child_reintegrated'].includes(event.event_type)).length,
      record
    };
  } finally {
    if (typeof provider?.dispose === 'function') await provider.dispose();
    await fs.rm(temp, { recursive: true, force: true });
  }
}

function rotation(repetition) {
  const n = repetition % CONDITIONS.length;
  return [...CONDITIONS.slice(n), ...CONDITIONS.slice(0, n)];
}

function processSummary(records) {
  const byRepetition = new Map();
  for (const record of records) {
    if (!byRepetition.has(record.repetition)) byRepetition.set(record.repetition, {});
    byRepetition.get(record.repetition)[record.condition] = {
      verification_pass: record.verification?.objective_checks_pass ?? null,
      model_calls: record.model_calls,
      capability_calls: record.capability_calls,
      total_tokens: record.total_tokens,
      elapsed_ms: record.elapsed_ms,
      ql_semantic_events: record.ql_semantic_events,
      operator_events: record.operator_events
    };
  }
  return [...byRepetition.entries()].map(([repetition, conditions]) => ({ repetition, conditions }));
}

async function main() {
  const config = args();
  if (!Number.isInteger(config.repetitions) || config.repetitions < 1) throw new Error('--repetitions must be a positive integer.');
  if (!Number.isInteger(config.maxSteps) || config.maxSteps < 1) throw new Error('--max-steps must be a positive integer.');
  if (!process.env.DEEPSEEK_API_KEY) throw new Error('DEEPSEEK_API_KEY is required for live Series 1 runs.');
  const task = getTask(config.task);
  const model = canonicalModel();
  const freeze = await buildBenchmarkFreeze();
  const records = [];

  for (let repetition = 0; repetition < config.repetitions; repetition += 1) {
    for (const condition of rotation(repetition)) {
      records.push(await runCondition({ hostId: config.host, task, condition, repetition, model, maxSteps: config.maxSteps, freeze }));
    }
  }

  const held = compareHeldConstant(records);
  const manifest = {
    schema: SERIES1_SCHEMA,
    benchmark: freeze.benchmark_id,
    benchmark_revision: freeze.benchmark_revision,
    benchmark_spec_revision: freeze.benchmark_spec_revision,
    task_corpus_revision: freeze.task_corpus_revision,
    runner_revision: freeze.runner_revision,
    review_contract_revision: freeze.review_contract_revision,
    provider_mode: 'live',
    fixture_provider: false,
    credential_contract: 'DEEPSEEK_API_KEY',
    host: records[0].host,
    model,
    conditions: CONDITIONS,
    held_constant: held,
    determination: DETERMINATION,
    task: { id: task.id, category: task.category, revision: freeze.tasks[task.id].task_revision },
    repetitions: config.repetitions,
    review: {
      prompt: task.prompt,
      success_conditions: task.successConditions,
      verification_protocol: task.verificationProtocol,
      focus: task.reviewFocus,
      human_reference: task.reviewReference,
      instruction: 'Inspect prompt apprehension, epistemic conduct, action/tool selection, recovery, closure, output/artifact fulfilment, friction and QL-specific behaviour. Objective verification supports but does not determine the judgement.'
    },
    process_summary: processSummary(records),
    records
  };
  assertLiveManifest(manifest);
  const safeManifest = sanitizeEvidence(manifest);
  assertNoConfiguredSecrets(safeManifest);
  process.stdout.write(`${JSON.stringify(safeManifest, null, 2)}\n`);
}

main().catch((error) => {
  console.error(sanitizeEvidence(error.stack ?? error.message ?? String(error)));
  process.exitCode = 1;
});
