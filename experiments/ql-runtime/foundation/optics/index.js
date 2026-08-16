import { createHash } from 'node:crypto';
import { RuntimeRegistry, normalizeRunRequest } from '../runtime-contract/index.js';

function canonicalize(value) {
  if (Array.isArray(value)) {
    return value.map(canonicalize);
  }
  if (value && typeof value === 'object') {
    return Object.fromEntries(
      Object.keys(value)
        .sort()
        .map((key) => [key, canonicalize(value[key])])
    );
  }
  return value;
}

function digest(value) {
  return createHash('sha256')
    .update(JSON.stringify(canonicalize(value)))
    .digest('hex')
    .slice(0, 16);
}

export class MemoryObserver {
  constructor() {
    this.events = [];
  }

  emit(event) {
    this.events.push({
      ...structuredClone(event),
      record_index: this.events.length
    });
  }
}

export function createRunManifest({
  taskId,
  fixtureId = null,
  host,
  runtime,
  specRevision,
  model = null,
  capabilities = [],
  environment = null,
  successConditions = [],
  startState = null
}) {
  return {
    task_id: taskId,
    fixture_id: fixtureId,
    host: {
      id: host.id,
      revision: host.revision ?? 'unknown'
    },
    runtime: {
      id: runtime.id,
      revision: runtime.version
    },
    spec_revision: specRevision,
    model: structuredClone(model),
    capability_surface: structuredClone(capabilities),
    environment: structuredClone(environment),
    start_state: structuredClone(startState),
    success_conditions: structuredClone(successConditions)
  };
}

export function runIdForManifest(manifest) {
  return `run_${digest(manifest)}`;
}

export function deriveRunStatus(result) {
  // QL-free projection: execution/chronology status and semantic closure
  // status are distinct facts about a run. Runtimes that carry no closure
  // semantics (Classic) are reported as not_applicable rather than being
  // forced to pretend to a QLClosure.
  const execution = result?.status ?? 'unknown';
  const semantic = result?.circuit?.closureState
    ? (result.circuit.closureState === 'closed' ? 'closed' : 'open')
    : result?.closure
      ? 'closed'
      : 'not_applicable';
  return { execution, semantic };
}

export function closureSummaryForResult(result, runId) {
  if (!result?.closure) {
    return null;
  }
  return {
    closure_id: result.closure.id,
    circuit_id: result.closure.circuit_id,
    run_refs: [runId],
    evidence_refs: Array.isArray(result.determination?.evidence_refs)
      ? [...result.determination.evidence_refs]
      : [],
    evaluation_refs: Array.isArray(result.closure.evaluation_refs)
      ? [...result.closure.evaluation_refs]
      : [],
    success_state: structuredClone(result.closure.success_state ?? null)
  };
}

export async function executeRun({ runtime, host, request, manifest, signal }) {
  const observer = new MemoryObserver();
  const runId = runIdForManifest(manifest);
  if (typeof host.attachObserver === 'function') {
    host.attachObserver(observer, runId);
  }
  const normalizedRequest = normalizeRunRequest({ ...request, runId });
  const registry = new RuntimeRegistry().register(runtime);
  const result = await registry.run(runtime.id, normalizedRequest, host, observer, signal);
  const record = {
    manifest: structuredClone(manifest),
    run_id: runId,
    status: deriveRunStatus(result),
    closure: closureSummaryForResult(result, runId),
    result: structuredClone(result),
    events: observer.events
  };
  return record;
}

export function replayRun(record) {
  const ordered = [...record.events].sort(
    (a, b) => (a.record_index ?? 0) - (b.record_index ?? 0)
  );
  return {
    run_id: record.run_id,
    manifest: structuredClone(record.manifest),
    status: structuredClone(record.status ?? deriveRunStatus(record.result)),
    closure: structuredClone(record.closure ?? null),
    execution_status: record.status?.execution ?? record.result?.status,
    event_count: ordered.length,
    events: ordered
  };
}

export function formatReplay(record) {
  const replay = replayRun(record);
  const status = replay.status
    ? `execution=${replay.status.execution} semantic=${replay.status.semantic}`
    : `status=${replay.execution_status ?? 'unknown'}`;
  const lines = [
    `${replay.run_id}  runtime=${replay.manifest.runtime.id}@${replay.manifest.runtime.revision}  ${status}`
  ];
  for (const event of replay.events) {
    const relation = event.ql?.relation ? ` ${event.ql.relation}` : '';
    const phase = event.ql?.projection ?? event.ql?.return;
    lines.push(`${event.channel ?? 'event'} ${event.event_type}${relation}${phase ? ` ${phase}` : ''}`);
  }
  return lines.join('\n');
}

export function compareRunRecords(classic, ql) {
  const held = {
    task_id: classic.manifest.task_id === ql.manifest.task_id,
    fixture_id: classic.manifest.fixture_id === ql.manifest.fixture_id,
    host: JSON.stringify(classic.manifest.host) === JSON.stringify(ql.manifest.host),
    model: JSON.stringify(classic.manifest.model) === JSON.stringify(ql.manifest.model),
    capability_surface: JSON.stringify(classic.manifest.capability_surface) === JSON.stringify(ql.manifest.capability_surface),
    environment: JSON.stringify(classic.manifest.environment) === JSON.stringify(ql.manifest.environment),
    start_state: JSON.stringify(classic.manifest.start_state) === JSON.stringify(ql.manifest.start_state),
    success_conditions: JSON.stringify(classic.manifest.success_conditions) === JSON.stringify(ql.manifest.success_conditions)
  };
  return {
    held_constant: held,
    changed: {
      runtime: {
        classic: classic.manifest.runtime,
        ql: ql.manifest.runtime
      }
    },
    results: {
      classic: {
        run_id: classic.run_id,
        execution_status: classic.status.execution,
        semantic_status: classic.status.semantic
      },
      ql: {
        run_id: ql.run_id,
        execution_status: ql.status.execution,
        semantic_status: ql.status.semantic
      }
    },
    event_counts: {
      classic: classic.events.length,
      ql: ql.events.length,
      ql_semantic: ql.events.filter((event) => event.channel === 'runtime-semantic').length,
      ql_host: ql.events.filter((event) => event.channel === 'host').length
    }
  };
}

export async function runAB({
  request,
  hostFactory,
  classicRuntime,
  qlRuntime,
  specRevision,
  fixtureId = null,
  model = null,
  capabilities = [],
  environment = null,
  startState = null
}) {
  const classicHost = hostFactory();
  const qlHost = hostFactory();
  const base = {
    taskId: request.taskId,
    fixtureId,
    specRevision,
    model,
    capabilities,
    environment,
    successConditions: request.successConditions ?? [],
    startState
  };
  const classicManifest = createRunManifest({ ...base, host: classicHost, runtime: classicRuntime });
  const qlManifest = createRunManifest({ ...base, host: qlHost, runtime: qlRuntime });
  const classic = await executeRun({ runtime: classicRuntime, host: classicHost, request, manifest: classicManifest });
  const ql = await executeRun({ runtime: qlRuntime, host: qlHost, request, manifest: qlManifest });
  return {
    classic,
    ql,
    comparison: compareRunRecords(classic, ql)
  };
}