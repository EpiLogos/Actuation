import crypto from 'node:crypto';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { SERIES1_CAPABILITY_CONTRACT, stableDigest } from './contract.mjs';
import { SERIES1_TASKS, setupTask } from './tasks.mjs';

const HERE = path.dirname(fileURLToPath(import.meta.url));
export const SERIES1_BENCHMARK_ID = 'series1-v0.1-human-review';

function sha256Bytes(value) {
  return crypto.createHash('sha256').update(value).digest('hex');
}

async function fileDigest(relativePath) {
  return sha256Bytes(await fs.readFile(path.join(HERE, relativePath)));
}

export async function fingerprintWorkspace(root) {
  const files = [];
  async function walk(relative = '.') {
    const absolute = path.join(root, relative);
    const entries = await fs.readdir(absolute, { withFileTypes: true });
    for (const entry of entries.sort((a, b) => a.name.localeCompare(b.name))) {
      const child = relative === '.' ? entry.name : path.join(relative, entry.name);
      if (entry.isDirectory()) {
        await walk(child);
      } else {
        const bytes = await fs.readFile(path.join(root, child));
        files.push({
          path: child.split(path.sep).join('/'),
          bytes: bytes.length,
          sha256: sha256Bytes(bytes)
        });
      }
    }
  }
  await walk();
  return { digest: stableDigest(files), files };
}

export async function materializeTaskFingerprint(task) {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), `ql-series1-freeze-${task.id}-`));
  try {
    await setupTask(task, root);
    const workspace = await fingerprintWorkspace(root);
    const verificationProtocolDigest = stableDigest({
      text: task.verificationProtocol,
      executable_verifier: String(task.verify)
    });
    const taskRevision = stableDigest({
      id: task.id,
      category: task.category,
      prompt: task.prompt,
      success_constraints: task.successConditions,
      verification_protocol: task.verificationProtocol,
      executable_verifier_digest: stableDigest(String(task.verify)),
      review_focus: task.reviewFocus,
      review_reference_digest: stableDigest(task.reviewReference),
      starting_workspace_digest: workspace.digest
    });
    return {
      id: task.id,
      category: task.category,
      task_revision: taskRevision,
      prompt_digest: stableDigest(task.prompt),
      success_constraints_digest: stableDigest(task.successConditions),
      starting_workspace_digest: workspace.digest,
      starting_workspace_files: workspace.files,
      verification_protocol_digest: verificationProtocolDigest,
      review_reference_digest: stableDigest(task.reviewReference)
    };
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
}

export async function sourceRevisions() {
  const benchmarkSpecRevision = await fileDigest('BENCHMARK-V0.1.md');
  const taskCorpusRevision = await fileDigest('tasks.mjs');
  const runnerRevision = stableDigest({
    run: await fileDigest('run.mjs'),
    contract: await fileDigest('contract.mjs'),
    host: await fileDigest('host.mjs'),
    providers: await fileDigest('providers.mjs'),
    policy: await fileDigest('policy.mjs'),
    freeze: await fileDigest('freeze.mjs'),
    candidate_boundary: await fileDigest('candidate-boundary.mjs'),
    evidence_boundary: await fileDigest('evidence.mjs'),
    dsh_host: await fileDigest('dsh.mjs'),
    dsh_ui: await fileDigest('dsh-ui-client.tsx'),
    dsh_profile: await fileDigest('DEEPSEEK-HARNESS-MAXIMAL-REFERENCE.md')
  });
  const reviewContractRevision = stableDigest({
    benchmark: benchmarkSpecRevision,
    renderer_cli: await fileDigest('render-review.mjs'),
    renderer: await fileDigest('review.mjs'),
    evidence_boundary: await fileDigest('evidence.mjs'),
    dsh_ui: await fileDigest('dsh-ui-client.tsx')
  });
  return {
    benchmark_spec_revision: benchmarkSpecRevision,
    task_corpus_revision: taskCorpusRevision,
    runner_revision: runnerRevision,
    review_contract_revision: reviewContractRevision
  };
}

export async function buildBenchmarkFreeze({ tasks = SERIES1_TASKS, capabilityContract = SERIES1_CAPABILITY_CONTRACT } = {}) {
  const revisions = await sourceRevisions();
  const taskEntries = [];
  for (const task of tasks) taskEntries.push(await materializeTaskFingerprint(task));
  const taskFingerprints = Object.fromEntries(taskEntries.map((entry) => [entry.id, entry]));
  const capabilityContractDigest = stableDigest(capabilityContract);
  const benchmarkRevision = stableDigest({
    benchmark_id: SERIES1_BENCHMARK_ID,
    benchmark_spec_revision: revisions.benchmark_spec_revision,
    task_corpus_revision: revisions.task_corpus_revision,
    capability_contract_digest: capabilityContractDigest,
    task_revisions: Object.fromEntries(taskEntries.map((entry) => [entry.id, entry.task_revision]))
  });
  return {
    benchmark_id: SERIES1_BENCHMARK_ID,
    benchmark_revision: benchmarkRevision,
    capability_contract_digest: capabilityContractDigest,
    ...revisions,
    tasks: taskFingerprints
  };
}

export async function verifyFreezeReproducibility({ tasks = SERIES1_TASKS } = {}) {
  const results = [];
  for (const task of tasks) {
    const first = await materializeTaskFingerprint(task);
    const second = await materializeTaskFingerprint(task);
    const fields = ['task_revision', 'prompt_digest', 'success_constraints_digest', 'starting_workspace_digest', 'verification_protocol_digest', 'review_reference_digest'];
    const mismatches = fields.filter((field) => first[field] !== second[field]);
    results.push({
      task_id: task.id,
      reproducible: mismatches.length === 0,
      mismatches,
      fingerprint: first
    });
  }
  return {
    valid: results.every((entry) => entry.reproducible),
    results
  };
}
