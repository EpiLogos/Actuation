import { HOSTS, SERIES1_CAPABILITY_CONTRACT, stableDigest } from './contract.mjs';
import { buildBenchmarkFreeze, verifyFreezeReproducibility } from './freeze.mjs';
import { Series1Workspace } from './host.mjs';
import { providerForHost, SERIES1_PROVIDER, series1ModelId } from './providers.mjs';
import { SERIES1_TASKS, getTask } from './tasks.mjs';

function cli() {
  const values = process.argv.slice(2);
  const live = values.includes('--live');
  const hostIndex = values.indexOf('--host');
  const taskIndex = values.indexOf('--task');
  const host = hostIndex >= 0 ? values[hostIndex + 1] : null;
  const task = taskIndex >= 0 ? values[taskIndex + 1] : null;
  return { live, hosts: host ? [host] : [...HOSTS], task };
}

function selectedTasks(task) {
  if (!task || task === 'all') return SERIES1_TASKS;
  return [getTask(task)];
}

function checkConfiguration() {
  const provider = process.env.QL_SERIES1_PROVIDER ?? SERIES1_PROVIDER;
  const model = series1ModelId();
  const errors = [];
  if (provider !== SERIES1_PROVIDER) {
    errors.push(`Series 1 is currently stipulated to provider '${SERIES1_PROVIDER}', not '${provider}'`);
  }
  if (!model) errors.push('A concrete Series 1 candidate model is required');
  if (!process.env.DEEPSEEK_API_KEY) errors.push('DEEPSEEK_API_KEY is missing');
  return { provider, model, errors };
}

async function main() {
  const { live, hosts, task } = cli();
  for (const host of hosts) {
    if (!HOSTS.includes(host)) throw new Error(`Unknown host '${host}'.`);
  }
  const tasks = selectedTasks(task);
  const config = checkConfiguration();
  const freeze = await buildBenchmarkFreeze();
  const reproducibility = await verifyFreezeReproducibility();
  const workspaceCapabilityDigest = stableDigest(new Series1Workspace(process.cwd()).describe());
  const declaredCapabilityDigest = stableDigest(SERIES1_CAPABILITY_CONTRACT);
  const capabilityParity = workspaceCapabilityDigest === declaredCapabilityDigest && declaredCapabilityDigest === freeze.capability_contract_digest;

  const result = {
    schema: 'ql-series1-preflight/0.5',
    benchmark: freeze.benchmark_id,
    benchmark_revision: freeze.benchmark_revision,
    benchmark_spec_revision: freeze.benchmark_spec_revision,
    task_corpus_revision: freeze.task_corpus_revision,
    runner_revision: freeze.runner_revision,
    review_contract_revision: freeze.review_contract_revision,
    live_requested: live,
    hosts,
    selected_task: task ?? 'all',
    provider: config.provider,
    model: config.model,
    credential_contract: 'DEEPSEEK_API_KEY',
    determination_protocol: 'human-review-first; automated/scalar evals deferred',
    freeze: {
      reproducible: reproducibility.valid,
      capability_contract_parity: capabilityParity,
      capability_contract_digest: freeze.capability_contract_digest,
      tasks: Object.fromEntries(Object.entries(freeze.tasks).map(([id, entry]) => [id, {
        task_revision: entry.task_revision,
        prompt_digest: entry.prompt_digest,
        success_constraints_digest: entry.success_constraints_digest,
        starting_workspace_digest: entry.starting_workspace_digest,
        verification_protocol_digest: entry.verification_protocol_digest,
        starting_workspace_files: entry.starting_workspace_files
      }]))
    },
    structural: {
      task_count: SERIES1_TASKS.length,
      selected_task_count: tasks.length,
      task_categories: Object.fromEntries(SERIES1_TASKS.map((entry) => [entry.id, entry.category])),
      conditions: ['classic', 'ql-direct', 'ql-deep'],
      required_held_constants: ['prompt', 'success/constraint text', 'starting workspace', 'model/parameters', 'capabilities', 'verification protocol', 'execution budget', 'host revision/composition', 'network policy', 'benchmark/task/runner/review revisions'],
      fixture_fallback: false,
      full_model_io_retained: true,
      full_capability_io_retained: true,
      full_before_after_workspace_retained: true
    },
    configuration_errors: config.errors,
    host_checks: []
  };

  if (live) {
    for (const host of hosts) {
      try {
        const provider = providerForHost(host);
        await provider.assertReady();
        result.host_checks.push({ host, ready: true, provider_path: provider.id });
      } catch (error) {
        result.host_checks.push({ host, ready: false, error: error.message });
      }
    }
  }

  const hostFailures = result.host_checks.filter((entry) => !entry.ready);
  result.structural_ready = reproducibility.valid && capabilityParity;
  result.evidence_ready = result.structural_ready && live && config.errors.length === 0 && hostFailures.length === 0;
  process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
  if (!result.structural_ready || (live && !result.evidence_ready)) process.exitCode = 1;
}

main().catch((error) => {
  console.error(error.stack ?? error.message ?? String(error));
  process.exitCode = 1;
});
