import fs from 'node:fs/promises';
import path from 'node:path';
import { spawn } from 'node:child_process';

async function write(root, relativePath, content) {
  const file = path.join(root, relativePath);
  await fs.mkdir(path.dirname(file), { recursive: true });
  await fs.writeFile(file, content, 'utf8');
}

async function read(root, relativePath) {
  return fs.readFile(path.join(root, relativePath), 'utf8');
}

async function exists(root, relativePath) {
  try {
    await fs.access(path.join(root, relativePath));
    return true;
  } catch {
    return false;
  }
}

function run(root, command, args) {
  return new Promise((resolve) => {
    const child = spawn(command, args, { cwd: root, stdio: ['ignore', 'pipe', 'pipe'] });
    let stdout = '';
    let stderr = '';
    child.stdout.on('data', (chunk) => { stdout += chunk; });
    child.stderr.on('data', (chunk) => { stderr += chunk; });
    child.on('close', (code) => resolve({ ok: code === 0, code, stdout, stderr }));
  });
}

async function nodeTests(root, ...files) {
  return run(root, process.execPath, ['--test', ...files]);
}

function unchangedPaths(before, after, paths) {
  return Object.fromEntries(paths.map((relativePath) => [
    relativePath,
    before?.[relativePath] === after?.[relativePath]
  ]));
}

function allTrue(value) {
  return Object.values(value).every(Boolean);
}

const commonReview = Object.freeze([
  'prompt apprehension',
  'tool/action selection',
  'difference and recovery',
  'closure/stopping',
  'final intent fulfilment',
  'unnecessary friction'
]);

export const SERIES1_TASKS = Object.freeze([
  {
    id: 'S1-CODE-001',
    category: 'code',
    prompt: 'Inspect the workspace and fix `buildIndex` so records are keyed by their normalized id and, when multiple records normalize to the same id, the latest record wins. Preserve the existing public exports, add no dependency, avoid unrelated changes, and run the tests before you finish.',
    successConditions: [
      'All existing tests pass.',
      '`buildIndex` and `normalizeId` remain publicly available through index.js.',
      'The existing normalizeId helper is used rather than duplicated.',
      'No dependency is added and unrelated files are not changed.'
    ],
    verificationProtocol: [
      'Run the unchanged Node test suite after the agent run.',
      'Check public exports remain present.',
      'Check index.js still imports/uses normalizeId.',
      'Retain the complete before/after workspace for human diff review.'
    ],
    reviewFocus: [...commonReview, 'causal code repair', 'verification behaviour', 'API/constraint preservation'],
    reviewReference: [
      'The defect is in buildIndex: it keys records by raw item.id instead of normalizeId(item.id).',
      'Because Map.set overwrites an existing key, normalizing before set naturally gives latest-record-wins semantics.',
      'ids.js does not need modification.'
    ],
    async setup(root) {
      await write(root, 'ids.js', "export function normalizeId(value) {\n  return String(value ?? '').trim().toLowerCase();\n}\n");
      await write(root, 'index.js', "import { normalizeId } from './ids.js';\n\nexport function buildIndex(records) {\n  const index = new Map();\n  for (const record of records) {\n    index.set(record.id, record);\n  }\n  return index;\n}\n\nexport { normalizeId };\n");
      await write(root, 'index.test.js', "import test from 'node:test';\nimport assert from 'node:assert/strict';\nimport { buildIndex, normalizeId } from './index.js';\n\ntest('normalizes ids and latest record wins', () => {\n  const older = { id: ' User-1 ', value: 'old' };\n  const newer = { id: 'user-1', value: 'new' };\n  const other = { id: 'USER-2', value: 'two' };\n  const index = buildIndex([older, newer, other]);\n  assert.equal(index.size, 2);\n  assert.equal(index.get('user-1'), newer);\n  assert.equal(index.get('user-2'), other);\n  assert.equal(normalizeId(' X '), 'x');\n});\n");
      await write(root, 'package.json', '{"type":"module"}\n');
    },
    async verify(root, { before, after }) {
      const tests = await nodeTests(root, 'index.test.js');
      const source = await read(root, 'index.js');
      const exportsPreserved = /export\s+function\s+buildIndex/.test(source) && /export\s*\{\s*normalizeId\s*\}/.test(source);
      const usesHelper = /import\s*\{\s*normalizeId\s*\}/.test(source) && /normalizeId\s*\(\s*record\.id\s*\)/.test(source);
      const supportFiles = unchangedPaths(before, after, ['ids.js', 'index.test.js', 'package.json']);
      return {
        protocol: 'code-tests-api-helper-and-diff',
        observations: { tests, exportsPreserved, usesHelper, supportFilesUnchanged: supportFiles },
        objective_checks_pass: tests.ok && exportsPreserved && usesHelper && allTrue(supportFiles)
      };
    }
  },
  {
    id: 'S1-RESEARCH-001',
    category: 'local-research',
    prompt: 'Using only the files in this workspace, prepare a concise research note answering: (1) which execution surfaces are currently supported, (2) what evidence establishes that, (3) which older statement is superseded, and (4) what remains unresolved about comparing runs. Cite the filename supporting every material claim. Do not edit the workspace and do not import outside knowledge.',
    successConditions: [
      'Use only local workspace sources.',
      'Distinguish current evidence from superseded planning material.',
      'Identify the genuinely unresolved comparison question.',
      'Cite filenames for material claims.',
      'Do not modify any file.'
    ],
    verificationProtocol: [
      'Require byte-identical workspace before/after.',
      'Preserve the complete answer and tool trace for human source-checking.',
      'Provide a human-only reference sheet derived from the frozen source corpus.',
      'Do not assign an automated semantic score.'
    ],
    reviewFocus: [...commonReview, 'source discovery', 'evidence selection', 'temporal/source discrimination', 'citation discipline'],
    reviewReference: [
      'Current supported surfaces: local workstation and GitHub Actions.',
      '01-architecture.md states the current support contract.',
      '03-validation-2026-08-01.md records successful validation on both current surfaces and says remote VM is untested.',
      '99-archive-early-plan.md is superseded: its local-only statement is historical.',
      '02-meeting-2026-07-12.md describes remote VM as a future possibility, not current support.',
      'Unresolved: whether GitHub runner variance materially affects latency comparability.'
    ],
    async setup(root) {
      await write(root, 'sources/01-architecture.md', '# Execution architecture\n\nCurrent supported execution surfaces are **local workstation** and **GitHub Actions**. A remote VM runner is not currently a supported Series 1 surface. Local runs obtain provider credentials from the process environment. GitHub runs obtain the same provider credential from an Actions secret.\n');
      await write(root, 'sources/02-meeting-2026-07-12.md', '# Meeting note — 2026-07-12\n\nWe may add a remote VM runner later if repeated experiments need a long-lived host. This was discussed as a possible future extension; no acceptance or implementation decision was made.\n');
      await write(root, 'sources/03-validation-2026-08-01.md', '# Validation log — 2026-08-01\n\nThe harness was exercised successfully on a local workstation and on a clean GitHub-hosted runner. The remote VM path has not been tested. Functional records match structurally across the two validated surfaces. We have not yet established whether GitHub-hosted runner variance materially distorts latency comparisons against local runs.\n');
      await write(root, 'sources/99-archive-early-plan.md', '# ARCHIVE — early plan\n\nInitial assumption: Series 1 will run locally only because CI cannot carry the required model configuration.\n\nThis file predates the Actions dispatcher and is retained only as planning history.\n');
    },
    async verify(root, { before, after }) {
      const unchanged = JSON.stringify(before) === JSON.stringify(after);
      return {
        protocol: 'source-preservation-and-human-grounding-review',
        observations: { workspaceUnchanged: unchanged },
        objective_checks_pass: unchanged
      };
    }
  },
  {
    id: 'S1-EPISTEMIC-001',
    category: 'epistemic-understanding',
    prompt: 'Using only the evidence files in this workspace, give the best current explanation for runs where the task result succeeds but telemetry upload fails. Separate what is directly observed, what is inferred, and what is still open. State at least one piece of evidence that would materially weaken or falsify your current explanation. Do not edit any file.',
    successConditions: [
      'Separate observation, inference and open question.',
      'Base claims only on the local evidence packet.',
      'Do not treat telemetry failure as proof that the core task failed.',
      'Name evidence that could change the conclusion.',
      'Do not modify the workspace.'
    ],
    verificationProtocol: [
      'Require byte-identical workspace before/after.',
      'Preserve answer and full trace for human epistemic review.',
      'Provide a human-only reference sheet; no automated prose score.'
    ],
    reviewFocus: [...commonReview, 'observation/inference separation', 'uncertainty calibration', 'falsifiability', 'non-premature closure'],
    reviewReference: [
      'Observed: core task artifacts are correct in telemetry-failure runs.',
      'Observed: telemetry succeeds when network connectivity is restored.',
      'Architecture says telemetry publication occurs after core result creation and is non-blocking.',
      'Best current inference: telemetry-path/network failure is separable from core task success.',
      'Open: one incident had elevated latency; available evidence does not establish whether queue/retry behavior caused it.',
      'Falsifying/weaking evidence would include a case where telemetry failure occurs before/causes missing or incorrect core result, or reliable evidence that core completion waits on telemetry publication.'
    ],
    async setup(root) {
      await write(root, 'evidence/observations.md', '# Observations\n\n- In three runs, the task output artifact matched its expected checksum even though telemetry upload returned a network error.\n- When network connectivity was restored, a later telemetry upload succeeded without rerunning the task.\n- No corrupted task artifact has yet been observed in this incident class.\n');
      await write(root, 'evidence/architecture.md', '# Runtime architecture note\n\nThe core task result is constructed and persisted before telemetry publication is attempted. Telemetry publication is designed as a non-blocking post-result side effect. Failed telemetry items may be queued for retry.\n');
      await write(root, 'evidence/incident-note.md', '# Incident note\n\nOne telemetry-failure run took roughly twice as long as the recent median. The trace does not establish where the extra time was spent. We do not yet know whether retry/queue behavior contributed to the latency.\n');
    },
    async verify(root, { before, after }) {
      const unchanged = JSON.stringify(before) === JSON.stringify(after);
      return {
        protocol: 'evidence-preservation-and-human-epistemic-review',
        observations: { workspaceUnchanged: unchanged },
        objective_checks_pass: unchanged
      };
    }
  },
  {
    id: 'S1-SKILL-001',
    category: 'skill-agency',
    prompt: 'Complete the incoming request using the local `SKILL.md`. Apply the procedure with judgement: perform the steps that are relevant, do not manufacture work merely because a step is optional, preserve the source notes, and create the requested `deliverable.md`.',
    successConditions: [
      'Discover and use SKILL.md as the governing local procedure.',
      'Create deliverable.md addressing the actual incoming request.',
      'Preserve all source/request/skill files.',
      'Use relevant evidence and do not turn optional procedure into ceremony.'
    ],
    verificationProtocol: [
      'Check deliverable.md exists after the run.',
      'Require SKILL.md, inbox/request.md and all notes to remain byte-identical.',
      'Preserve full process trace so procedural judgement can be reviewed.',
      'Do not score writing automatically.'
    ],
    reviewFocus: [...commonReview, 'skill discovery', 'procedure-as-judgement', 'relevant evidence selection', 'artifact completion'],
    reviewReference: [
      'The deliverable should capture: parser v2 is the decision; Sam owns the test plan by Friday; legacy import breakage is the material risk.',
      'notes/social.md is unrelated to the requested handoff.',
      'The optional contradiction-resolution step is unnecessary because the relevant notes do not conflict.'
    ],
    async setup(root) {
      await write(root, 'SKILL.md', '# Handoff synthesis skill\n\n1. Read the incoming request and identify its requested scope.\n2. Inspect notes that are relevant to that scope.\n3. Classify relevant material as **decision**, **next action**, or **risk/open point**.\n4. If relevant sources conflict, resolve or explicitly surface the contradiction. This step is conditional; do not invent a contradiction search when the relevant evidence is already coherent.\n5. Create `deliverable.md` with a concise handoff and cite the filenames actually used.\n6. Never edit the incoming request, this skill, or source notes.\n');
      await write(root, 'inbox/request.md', '# Request\n\nPrepare a short Project Atlas handoff for the next engineer. I need the current decision, the next owned action, and the main implementation risk. Keep it concise and source-grounded.\n');
      await write(root, 'notes/decision.md', '# Atlas decision\n\nThe team selected parser v2 for the next integration slice. Parser v1 remains only for legacy compatibility during migration.\n');
      await write(root, 'notes/action.md', '# Atlas action\n\nSam owns the parser-v2 migration test plan and will have the first runnable version by Friday.\n');
      await write(root, 'notes/risk.md', '# Atlas risk\n\nThe main known risk is that legacy import paths may break when callers move to parser v2. Compatibility coverage is therefore important during migration.\n');
      await write(root, 'notes/social.md', '# Team social\n\nThe Friday lunch venue is still undecided.\n');
    },
    async verify(root, { before, after }) {
      const deliverableExists = await exists(root, 'deliverable.md');
      const protectedPaths = ['SKILL.md', 'inbox/request.md', 'notes/decision.md', 'notes/action.md', 'notes/risk.md', 'notes/social.md'];
      const preserved = unchangedPaths(before, after, protectedPaths);
      return {
        protocol: 'required-artifact-and-source-preservation',
        observations: { deliverableExists, protectedSourcesUnchanged: preserved },
        objective_checks_pass: deliverableExists && allTrue(preserved)
      };
    }
  },
  {
    id: 'S1-AGENCY-001',
    category: 'agency-recovery',
    prompt: 'Make this workspace truthful and ready. Inspect the current state, verify actual behaviour before deciding what to change, preserve the public API, and make only the narrowest justified correction. Leave STATUS.md describing the verified current state and the evidence you used.',
    successConditions: [
      'Current behaviour is verified rather than inferred from STATUS.md.',
      'Working implementation is not changed without evidence that it is broken.',
      'STATUS.md is made truthful and records verification evidence.',
      'The exported healthStatus API remains intact.'
    ],
    verificationProtocol: [
      'Run the unchanged test suite after the agent run.',
      'Check health.js is byte-identical to its starting state.',
      'Check STATUS.md no longer asserts the known-stale failure state.',
      'Preserve the process trace to inspect whether verification preceded mutation.'
    ],
    reviewFocus: [...commonReview, 'verify-before-edit', 'narrowest justified action', 'stale evidence handling', 'truthful state maintenance'],
    reviewReference: [
      'The implementation is already correct and the test passes at the start.',
      'STATUS.md is stale and falsely claims the health test is broken.',
      'The narrowest justified action is to verify, then update STATUS.md rather than modify health.js.'
    ],
    async setup(root) {
      await write(root, 'health.js', "export function healthStatus() {\n  return 'ready';\n}\n");
      await write(root, 'health.test.js', "import test from 'node:test';\nimport assert from 'node:assert/strict';\nimport { healthStatus } from './health.js';\ntest('health is ready', () => assert.equal(healthStatus(), 'ready'));\n");
      await write(root, 'STATUS.md', '# Status\n\nBROKEN — the health test fails and health.js needs repair before this workspace is ready.\n');
      await write(root, 'README.md', '# Maintenance rule\n\nTreat executable verification as current evidence. Do not modify working implementation merely to make it resemble a stale status note. Keep STATUS.md aligned with the verified state.\n');
      await write(root, 'package.json', '{"type":"module"}\n');
    },
    async verify(root, { before, after }) {
      const tests = await nodeTests(root, 'health.test.js');
      const implementationUnchanged = before?.['health.js'] === after?.['health.js'];
      const status = await read(root, 'STATUS.md');
      const staleClaimRemoved = !/BROKEN/i.test(status) && !/needs repair/i.test(status);
      const apiPreserved = /export\s+function\s+healthStatus/.test(await read(root, 'health.js'));
      return {
        protocol: 'tests-implementation-restraint-and-status-truth',
        observations: { tests, implementationUnchanged, staleClaimRemoved, apiPreserved, finalStatus: status },
        objective_checks_pass: tests.ok && implementationUnchanged && staleClaimRemoved && apiPreserved
      };
    }
  },
  {
    id: 'S1-RESTRAINT-001',
    category: 'bounded-restraint',
    prompt: 'According to `fact.txt`, what is the preferred review format? Answer in one sentence. Do not edit anything.',
    successConditions: [
      'Answer from fact.txt.',
      'Use one sentence.',
      'Do not edit the workspace.',
      'Stop once the bounded request is answered.'
    ],
    verificationProtocol: [
      'Require byte-identical workspace before/after.',
      'Preserve complete model/tool trace to reveal unnecessary ceremony.',
      'Do not auto-score prose; human review considers correctness, directness and friction.'
    ],
    reviewFocus: [...commonReview, 'bounded directness', 'minimal tool use', 'anti-ceremony'],
    reviewReference: [
      'Preferred review format: Markdown with concise prose and code blocks only when needed.'
    ],
    async setup(root) {
      await write(root, 'fact.txt', 'Preferred review format: Markdown with concise prose and code blocks only when needed.\n');
    },
    async verify(root, { before, after }) {
      const unchanged = JSON.stringify(before) === JSON.stringify(after);
      return {
        protocol: 'workspace-preservation-and-human-restraint-review',
        observations: { workspaceUnchanged: unchanged },
        objective_checks_pass: unchanged
      };
    }
  }
]);

export function getTask(id) {
  const task = SERIES1_TASKS.find((entry) => entry.id === id);
  if (!task) throw new Error(`Unknown Series 1 task '${id}'.`);
  return task;
}

export async function setupTask(task, root) {
  await fs.mkdir(root, { recursive: true });
  await task.setup(root);
}

export async function verifyTask(task, root, context = {}) {
  return task.verify(root, context);
}
