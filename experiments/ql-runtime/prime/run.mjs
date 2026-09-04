#!/usr/bin/env node
import { spawn } from 'node:child_process';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { getTask, setupTask, verifyTask } from '../comparison/series1/tasks.mjs';
import { getPrimeCondition, conditionPrompt } from './conditions.mjs';
import { extractPrimeFamily, readJsonl, sourceSummary } from './evidence.mjs';
import { PrimeRpcClient, parseExtraArgs } from './prime-rpc.mjs';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const LOCK = JSON.parse(await fs.readFile(path.join(HERE, 'source-lock.json'), 'utf8'));

function parseArgs() {
  const args = process.argv.slice(2);
  const read = (name, fallback = null) => {
    const index = args.indexOf(`--${name}`);
    return index >= 0 ? args[index + 1] : fallback;
  };
  return {
    condition: read('condition', process.env.QL_PRIME_CONDITION ?? 'prime-relational-return'),
    task: read('task', process.env.QL_PRIME_TASK ?? 'S1-CODE-001'),
    output: read('output', process.env.QL_PRIME_OUTPUT ?? null),
    timeoutMs: Number(read('timeout-ms', process.env.QL_PRIME_TIMEOUT_MS ?? String(20 * 60_000)))
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

function exec(command, args, options = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, { cwd: options.cwd, env: options.env ?? process.env, stdio: ['ignore', 'pipe', 'pipe'] });
    let stdout = '';
    let stderr = '';
    child.stdout.on('data', (chunk) => { stdout += chunk; });
    child.stderr.on('data', (chunk) => { stderr += chunk; });
    child.once('error', reject);
    child.once('close', (code) => resolve({ code, stdout, stderr }));
  });
}

async function primeVersion() {
  const result = await exec(process.env.PRIME_AGENT_BIN ?? 'prime-agent', ['--version']);
  if (result.code !== 0) throw new Error(`Prime Agent preflight failed: ${result.stderr || result.stdout}`);
  return result.stdout.trim() || result.stderr.trim();
}

async function qlState(root, harmonicEnabled) {
  if (!root) return { root: null, revision: null, harmonic_enabled: false, status: 'unbound' };
  const revisionResult = await exec('git', ['rev-parse', 'HEAD'], { cwd: root });
  if (revisionResult.code !== 0) throw new Error(`QL_MEF_ROOT is not a readable Git checkout: ${revisionResult.stderr}`);
  const revision = revisionResult.stdout.trim();
  const accepted = revision === LOCK.ql_mef.accepted_main_revision;
  const harmonic = revision === LOCK.ql_mef.harmonic_research.revision;
  if (harmonicEnabled && !harmonic && process.env.QL_PRIME_ALLOW_QL_DRIFT !== '1') {
    throw new Error(`Harmonic condition requires the source-locked QL-MEF #81 head ${LOCK.ql_mef.harmonic_research.revision}; observed ${revision}.`);
  }
  if (!harmonicEnabled && !accepted && process.env.QL_PRIME_ALLOW_QL_DRIFT !== '1') {
    throw new Error(`Relational condition requires source-locked QL-MEF main ${LOCK.ql_mef.accepted_main_revision}; observed ${revision}. Set QL_PRIME_ALLOW_QL_DRIFT=1 only for an explicitly recorded development run.`);
  }
  return { root, revision, harmonic_enabled: harmonicEnabled, status: harmonic ? 'harmonic-development-head' : (accepted ? 'accepted-main' : 'explicit-drift') };
}

async function main() {
  const config = parseArgs();
  if (!Number.isFinite(config.timeoutMs) || config.timeoutMs <= 0) throw new Error('--timeout-ms must be positive.');
  const condition = getPrimeCondition(config.condition);
  const task = getTask(config.task);
  const version = await primeVersion();

  if (!version.includes(LOCK.prime_agent.release.replace(/^v/, '')) && process.env.QL_PRIME_ALLOW_VERSION_DRIFT !== '1') {
    throw new Error(`Prime version drift: source lock requires ${LOCK.prime_agent.release}; observed '${version}'.`);
  }

  if (!process.env.QL_PRIME_PROVIDER || !process.env.QL_PRIME_MODEL) {
    throw new Error('QL_PRIME_PROVIDER and QL_PRIME_MODEL are required for a live Prime experiment.');
  }

  const harmonicEnabled = process.env.QL_PRIME_HARMONIC === '1';
  const ql = condition.relational ? await qlState(process.env.QL_MEF_ROOT ?? null, harmonicEnabled) : { status: 'not-used', root: null, revision: null, harmonic_enabled: false };
  if (condition.relational && !ql.root) throw new Error('QL_MEF_ROOT is required for relational Prime conditions.');

  const runRoot = await fs.mkdtemp(path.join(os.tmpdir(), `actuation-prime-${condition.code}-`));
  const workspace = path.join(runRoot, 'workspace');
  const sessionDir = path.join(runRoot, 'prime-sessions');
  const qlEvidence = path.join(runRoot, 'ql-relational-evidence.jsonl');
  await fs.mkdir(workspace, { recursive: true });
  await fs.mkdir(sessionDir, { recursive: true });

  let client = null;
  try {
    await setupTask(task, workspace);
    const before = await snapshot(workspace);
    const skillPaths = condition.relational ? [path.join(HERE, 'skills', 'ql-relational')] : [];
    const env = {
      QL_MEF_ROOT: ql.root ?? '',
      QL_RELATIONAL_EVIDENCE_LOG: qlEvidence,
      QL_PRIME_HARMONIC: harmonicEnabled ? '1' : '0',
      QL_PRIME_SOURCE_LOCK: path.join(HERE, 'source-lock.json')
    };

    client = new PrimeRpcClient({
      cwd: workspace,
      provider: process.env.QL_PRIME_PROVIDER,
      model: process.env.QL_PRIME_MODEL,
      skillPaths,
      sessionDir,
      extraArgs: parseExtraArgs(),
      env
    });
    await client.start();

    const prompt = conditionPrompt(condition, task);
    await client.prompt(prompt);
    const finalState = await client.waitForIdle({ timeoutMs: config.timeoutMs });
    const output = await client.getLastAssistantText() ?? '';
    const messages = await client.getMessages();
    const stats = await client.getSessionStats();
    const after = await snapshot(workspace);
    const verification = await verifyTask(task, workspace, { before, after, output, record: { events: client.records } });
    const qlOperations = await readJsonl(qlEvidence);
    const family = extractPrimeFamily(client.records);

    let refinement = null;
    if (condition.continual) {
      refinement = await client.refine(
        'Using only evidence from the completed trajectory, retain one small reusable improvement that would help a future similar task. Prefer a focused memory, supplemental prompt note, skill description, or subagent specification. Preserve uncertainty and do not generalise beyond the evidence.'
      );
    }

    const manifest = {
      schema: 'actuation.prime-recursive-experiment/v1',
      condition: { id: config.condition, ...condition },
      task: { id: task.id, category: task.category, prompt: task.prompt, success_conditions: task.successConditions },
      source: sourceSummary(LOCK, ql),
      prime: {
        observed_version: version,
        provider: process.env.QL_PRIME_PROVIDER,
        model: process.env.QL_PRIME_MODEL,
        final_state: finalState,
        session_stats: stats,
        family,
        rpc_records: client.records
      },
      ql: {
        state: ql,
        relational_operations: qlOperations
      },
      workspace: { before, after },
      outcome: output,
      verification,
      continual_refinement: refinement,
      claims: {
        live_prime_run: true,
        ql_relational_faculty_exercised: qlOperations.length > 0,
        observed_child_loci: family.nodes.length,
        observed_lineage_edges: family.edges.length,
        continual_refinement_invoked: Boolean(condition.continual)
      }
    };

    const rendered = `${JSON.stringify(manifest, null, 2)}\n`;
    if (config.output) {
      await fs.mkdir(path.dirname(path.resolve(config.output)), { recursive: true });
      await fs.writeFile(config.output, rendered, 'utf8');
    } else {
      process.stdout.write(rendered);
    }
  } finally {
    await client?.stop();
    if (process.env.QL_PRIME_KEEP_RUN_ROOT === '1') {
      process.stderr.write(`Prime run root retained at ${runRoot}\n`);
    } else {
      await fs.rm(runRoot, { recursive: true, force: true });
    }
  }
}

main().catch((error) => {
  console.error(error.stack ?? error.message ?? String(error));
  process.exitCode = 1;
});
