import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { createRequire } from 'node:module';
import { sanitizeEvidence } from './evidence.mjs';
import { SERIES1_CAPABILITY_CONTRACT, stableDigest } from './contract.mjs';

const require = createRequire(import.meta.url);

export const DSH_UPSTREAM_REVISION = '47f943859bef60e4160492346772ded9b24f765a';
export const DSH_PACKAGE_VERSION = '0.1.0-rc.5';
export const DSH_PROVIDER_ROUTE = 'deepseek-official';
export const DSH_INSPECTION_SCHEMA = 'ql-series1-dsh-inspection/0.1';
export const DSH_AGENT_PRESET = 'series1-loopruntime';
export const DSH_PLUGIN_TREE = Object.freeze([
  '@deepseek-ai/cordis',
  '@deepseek-ai/dsh-llm',
  '@deepseek-ai/dsh-llm-deepseek',
  '@deepseek-ai/dsh-session',
  '@deepseek-ai/dsh-session-persistence',
  '@deepseek-ai/dsh-session-persistence-jsonl',
  '@deepseek-ai/dsh-agent',
  'series1-loopruntime-adapter'
]);

function parseJsonObject(text) {
  const value = String(text ?? '').trim();
  try { return JSON.parse(value); } catch {}
  const start = value.indexOf('{');
  const end = value.lastIndexOf('}');
  if (start >= 0 && end > start) return JSON.parse(value.slice(start, end + 1));
  throw new Error(`DSH model did not return a JSON object: ${value.slice(0, 300)}`);
}

function normalizeTurn(text, mode) {
  if (mode === 'control') {
    return { content: '', capabilityCalls: [], control: parseJsonObject(text), usage: null, raw: null };
  }
  const parsed = parseJsonObject(text);
  return {
    content: parsed?.content ?? '',
    capabilityCalls: Array.isArray(parsed?.capabilityCalls) ? parsed.capabilityCalls.map((call, index) => ({
      id: call.id ?? `call-${index + 1}`,
      name: call.name,
      args: call.args ?? {}
    })) : [],
    usage: null,
    raw: null
  };
}

export function dshCompositionFingerprint() {
  return stableDigest({
    upstream_revision: DSH_UPSTREAM_REVISION,
    package_version: DSH_PACKAGE_VERSION,
    provider_route: DSH_PROVIDER_ROUTE,
    plugin_tree: DSH_PLUGIN_TREE,
    candidate_capability_contract_digest: stableDigest(SERIES1_CAPABILITY_CONTRACT),
    inspection_schema: DSH_INSPECTION_SCHEMA,
    persistence: { backend: 'jsonl', compression: 'none', pack_chunks: false },
    model_history_source: 'portable-series1-request-not-session-surface'
  });
}

function qlSummary(events) {
  const semantic = events.filter((event) => event.channel === 'runtime-semantic');
  const relations = semantic.filter((event) => /^R\d\d$/.test(event?.payload?.relation ?? '') || /^R\d\d$/.test(event?.relation ?? ''));
  const positions = semantic.filter((event) => event.ql?.position || event.payload?.position).map((event) => event.ql?.position ?? event.payload?.position);
  const closure = semantic.filter((event) => /closure/i.test(event.event_type ?? ''));
  const reentry = semantic.filter((event) => /reentry|re-entry|reopen/i.test(event.event_type ?? ''));
  const operators = semantic.filter((event) => /conjugate|child_|depth|square|modulation/i.test(event.event_type ?? ''));
  const piRhoDifference = semantic.filter((event) => {
    const text = JSON.stringify(event);
    return /"pi"|"rho"|π|ρ|difference/i.test(text);
  });
  return {
    positions,
    relations: relations.map((event) => ({ record_index: event.record_index, event_type: event.event_type, relation: event.payload?.relation ?? event.relation })),
    closure: closure.map((event) => ({ record_index: event.record_index, event_type: event.event_type, payload: event.payload })),
    reentry: reentry.map((event) => ({ record_index: event.record_index, event_type: event.event_type, payload: event.payload })),
    operators: operators.map((event) => ({ record_index: event.record_index, event_type: event.event_type, payload: event.payload })),
    pi_rho_difference: piRhoDifference.map((event) => ({ record_index: event.record_index, event_type: event.event_type, payload: event.payload }))
  };
}

export function buildDshInspectionProjection(events, { runId, condition, modelCalls = [] } = {}) {
  const safeEvents = sanitizeEvidence(events ?? []);
  return sanitizeEvidence({
    schema: DSH_INSPECTION_SCHEMA,
    run_id: runId,
    condition,
    read_only: true,
    candidate_context_authority: false,
    model_calls: modelCalls,
    ql: qlSummary(safeEvents),
    portable_events: safeEvents.map((event) => ({
      portable_record_index: event.record_index,
      portable_channel: event.channel,
      portable_event_type: event.event_type,
      event
    }))
  });
}

export function buildIgnorableInspectionSeed(projection, { time = 0 } = {}) {
  return projection.portable_events.map((item, index) => ({
    type: 'series1/portable-event',
    seq: index,
    time,
    data: {
      schema: DSH_INSPECTION_SCHEMA,
      run_id: projection.run_id,
      condition: projection.condition,
      ...item
    },
    ignorable: true
  }));
}

function packageVersion(packageName) {
  const pkg = require(`${packageName}/package.json`);
  return pkg.version;
}

function usageFromDsh(usage) {
  if (!usage) return { input_tokens: 0, output_tokens: 0, total_tokens: 0 };
  const input = Number(usage.inputTokens ?? 0) + Number(usage.cacheReadTokens ?? 0) + Number(usage.cacheWriteTokens ?? 0);
  const output = Number(usage.outputTokens ?? 0);
  return { input_tokens: input, output_tokens: output, total_tokens: input + output };
}

function visibleText(message) {
  return (message?.content ?? []).filter((block) => block.type === 'text').map((block) => block.text).join('');
}

function failureForTurn(error) {
  const message = error instanceof Error ? error.message : String(error);
  return { kind: 'error', error: { message, code: error?.code ?? 'UNKNOWN' } };
}

/** Real DSH transport/provider and native-session evidence owner. */
export class DshSeries1Provider {
  constructor({ model = 'deepseek-v4-flash', apiKey = process.env.DEEPSEEK_API_KEY } = {}) {
    this.id = 'dsh-deepseek-official';
    this.model = model;
    this.apiKey = apiKey;
    this.compositionFingerprint = dshCompositionFingerprint();
    this.context = null;
    this.modules = null;
    this.persistenceRoot = null;
    this.session = null;
    this.agent = null;
    this.runId = null;
    this.modelCalls = [];
    this.inspectionProjection = null;
    this.inspectionSession = null;
    this.inspectionSessionError = null;
    this.turn = 0;
  }

  async #loadModules() {
    if (this.modules) return;
    try {
      const [cordis, llm, deepseek, session, persistence, jsonl, agent] = await Promise.all([
        import('@deepseek-ai/cordis'),
        import('@deepseek-ai/dsh-llm'),
        import('@deepseek-ai/dsh-llm-deepseek'),
        import('@deepseek-ai/dsh-session'),
        import('@deepseek-ai/dsh-session-persistence'),
        import('@deepseek-ai/dsh-session-persistence-jsonl'),
        import('@deepseek-ai/dsh-agent')
      ]);
      this.modules = { cordis, llm, deepseek, session, persistence, jsonl, agent };
    } catch (error) {
      throw new Error(`Real DeepSeek Harness packages unavailable. Install the pinned rc.5 DSH Series 1 dependency set. ${error.message}`);
    }
  }

  async #compose() {
    if (this.context) return;
    await this.#loadModules();
    const { Context } = this.modules.cordis;
    const LlmRuntime = this.modules.llm.default ?? this.modules.llm.LlmRuntime;
    const { SessionStore } = this.modules.session;
    const { JsonlSessionPersistence } = this.modules.jsonl;
    const { AgentRegistry } = this.modules.agent;
    if (!Context || !LlmRuntime || !SessionStore || !JsonlSessionPersistence || !AgentRegistry) {
      throw new Error('Pinned DeepSeek Harness packages do not expose the expected public composition services.');
    }
    this.persistenceRoot = await fs.mkdtemp(path.join(os.tmpdir(), 'ql-series1-dsh-session-'));
    const ctx = new Context();
    await ctx.plugin(LlmRuntime);
    await ctx.plugin(SessionStore);
    await ctx.plugin(JsonlSessionPersistence, { root: this.persistenceRoot, compression: 'none', packChunks: false });
    await ctx.plugin(AgentRegistry);
    await ctx.plugin(this.modules.deepseek, {});
    this.context = ctx;
  }

  async assertReady() {
    if (!this.apiKey) throw new Error('DEEPSEEK_API_KEY is required for live DeepSeek Harness runs.');
    await this.#compose();
    const versions = [
      '@deepseek-ai/dsh-llm',
      '@deepseek-ai/dsh-llm-deepseek',
      '@deepseek-ai/dsh-session',
      '@deepseek-ai/dsh-session-persistence',
      '@deepseek-ai/dsh-session-persistence-jsonl',
      '@deepseek-ai/dsh-agent'
    ].map((name) => [name, packageVersion(name)]);
    const mismatch = versions.find(([, version]) => version !== DSH_PACKAGE_VERSION);
    if (mismatch) throw new Error(`DeepSeek Harness package version mismatch: ${mismatch[0]}=${mismatch[1]}, expected ${DSH_PACKAGE_VERSION}.`);
    const providers = this.context.llm.listProviders?.() ?? [];
    const ids = providers.map((entry) => typeof entry === 'string' ? entry : entry.provider ?? entry.id);
    if (!ids.includes(DSH_PROVIDER_ROUTE)) {
      throw new Error(`DeepSeek Harness did not register required provider route '${DSH_PROVIDER_ROUTE}'.`);
    }
  }

  async attachRun(runId, workspaceRoot) {
    await this.#compose();
    if (this.runId && this.runId !== runId) throw new Error('DSH Series 1 provider cannot be rebound to another run.');
    if (this.runId) return;
    const { SessionId } = this.modules.session;
    const id = SessionId(`series1-${String(runId).replace(/[^a-zA-Z0-9._:-]/g, '-')}`);
    const session = this.context.sessions.create(id, { meta: { cwd: path.resolve(workspaceRoot), agentPreset: DSH_AGENT_PRESET } });
    const rejectDrive = () => { throw new Error('Series 1 DSH Agent is observational; frozen LoopRuntime owns execution.'); };
    const agent = {
      id,
      options: { provider: DSH_PROVIDER_ROUTE, model: this.model },
      session,
      inbox: Object.freeze({}),
      status: 'idle',
      ctx: this.context,
      cancel: () => {},
      whenIdle: async () => {},
      runMaintenance: async (task) => task(new AbortController().signal),
      send: rejectDrive,
      followup: rejectDrive,
      steer: rejectDrive,
      inject: rejectDrive
    };
    this.context.agents.register(agent);
    this.runId = runId;
    this.session = session;
    this.agent = agent;
  }

  async complete({ system, prompt, temperature = 0, signal, mode = 'turn' } = {}) {
    if (!this.runId || !this.session) throw new Error('DSH provider is not attached to a Series 1 run.');
    const { BlockAssembler, createUserMessage } = this.modules.llm;
    const turn = this.turn++;
    const step = 0;
    const session = this.session;
    session.append('turn/start', { turn });
    session.append('step/start', { turn, step });
    const user = createUserMessage({ content: [{ type: 'text', text: prompt }], source: { kind: 'plugin', plugin: 'ql-series1' } });
    session.append('user/message', user, { surfaceOp: 'append' });
    session.append('request/header', {
      header: { config: { provider: DSH_PROVIDER_ROUTE, model: this.model, temperature }, system },
      reason: turn === 0 ? 'initial' : 'change'
    });
    session.append('request/context', { provider: DSH_PROVIDER_ROUTE, model: this.model });

    const assembler = new BlockAssembler();
    const chunkSeqs = [];
    const ordinal = this.modelCalls.length;
    const seqStart = session.seq - 1;
    try {
      const request = {
        provider: DSH_PROVIDER_ROUTE,
        model: this.model,
        messages: [user],
        system,
        temperature,
        signal,
        sessionId: session.id
      };
      for await (const chunk of this.context.llm.stream(request)) {
        const event = session.append('assistant/chunk', { turn, step, chunk });
        chunkSeqs.push(event.seq);
        assembler.push(chunk);
      }
      const message = assembler.message({
        kind: 'model',
        provider: DSH_PROVIDER_ROUTE,
        model: this.model,
        ...(assembler.replayState === undefined ? {} : { replayState: assembler.replayState })
      });
      session.append('assistant/message', {
        turn,
        step,
        message,
        ...(assembler.usage === undefined ? {} : { usage: assembler.usage })
      }, { surfaceOp: 'append', sourceEventSeqs: chunkSeqs });
      session.append('step/end', { turn, step });
      const finish = assembler.finish;
      const endReason = finish?.kind === 'max-tokens' ? { kind: 'max-tokens' }
        : finish?.kind === 'aborted' ? { kind: 'aborted', reason: { kind: 'parent' } }
        : finish?.kind === 'error' ? { kind: 'error', error: finish.error ?? { message: 'DSH request failed', code: 'UNKNOWN' } }
        : { kind: 'completed' };
      session.append('turn/end', { turn, reason: endReason });
      const text = visibleText(message);
      const result = normalizeTurn(text, mode);
      result.usage = usageFromDsh(assembler.usage);
      result.raw = sanitizeEvidence({ provider: DSH_PROVIDER_ROUTE, model: this.model, finish, dsh_session_id: session.id });
      this.modelCalls.push({ ordinal, turn, step, dsh_seq_start: seqStart, dsh_seq_end: session.seq - 1 });
      return result;
    } catch (error) {
      try { session.append('step/end', { turn, step }); } catch {}
      try { session.append('turn/end', { turn, reason: failureForTurn(error) }); } catch {}
      this.modelCalls.push({ ordinal, turn, step, dsh_seq_start: seqStart, dsh_seq_end: session.seq - 1, error: true });
      throw error;
    }
  }

  async capturePortableTrace(events, { condition } = {}) {
    if (!this.runId || !this.session) return;
    const hostModelRequests = events.filter((event) => event.channel === 'host' && event.event_type === 'model_requested');
    const hostModelReturns = events.filter((event) => event.channel === 'host' && event.event_type === 'model_returned');
    const alignedCalls = this.modelCalls.map((call, ordinal) => ({
      ...call,
      portable_model_requested_record_index: hostModelRequests[ordinal]?.record_index ?? null,
      portable_model_returned_record_index: hostModelReturns[ordinal]?.record_index ?? null
    }));
    this.inspectionProjection = buildDshInspectionProjection(events, { runId: this.runId, condition, modelCalls: alignedCalls });

    // rc.5 permits unknown seed events only when explicitly ignorable. Keep this
    // in a separate evidence-only Session so it can never become candidate history.
    try {
      const { SessionId } = this.modules.session;
      const seed = buildIgnorableInspectionSeed(this.inspectionProjection);
      const inspectionId = SessionId(`${this.session.id}:inspection`);
      this.inspectionSession = this.context.sessions.create(inspectionId, {
        seed,
        meta: { cwd: this.session.header.cwd, agentPreset: 'series1-inspection-readonly', seedLength: seed.length }
      });
      await this.context.sessions.flush(this.inspectionSession);
    } catch (error) {
      this.inspectionSessionError = error instanceof Error ? error.message : String(error);
      this.inspectionSession = null;
    }
    await this.context.sessions.flush(this.session);
  }

  async snapshotNativeEvidence() {
    if (!this.session) return null;
    let rawArtifact = null;
    try {
      rawArtifact = await this.context.sessionPersistence.readRaw(this.session.id);
    } catch {}
    let inspectionRawArtifact = null;
    if (this.inspectionSession) {
      try { inspectionRawArtifact = await this.context.sessionPersistence.readRaw(this.inspectionSession.id); } catch {}
    }
    return sanitizeEvidence({
      kind: 'deepseek-harness-native-evidence',
      upstream_revision: DSH_UPSTREAM_REVISION,
      package_version: DSH_PACKAGE_VERSION,
      provider_route: DSH_PROVIDER_ROUTE,
      composition_fingerprint: this.compositionFingerprint,
      plugin_tree: DSH_PLUGIN_TREE,
      candidate_session: {
        id: this.session.id,
        header: this.session.header,
        events: this.session.events,
        raw_artifact: rawArtifact
      },
      model_call_alignment: this.inspectionProjection?.model_calls ?? this.modelCalls,
      ql_inspection: this.inspectionProjection,
      inspection_session: this.inspectionSession ? {
        id: this.inspectionSession.id,
        header: this.inspectionSession.header,
        events: this.inspectionSession.events,
        raw_artifact: inspectionRawArtifact
      } : null,
      inspection_session_error: this.inspectionSessionError,
      closure_note: 'DSH turn/session completion is host evidence and is not QLClosure.'
    });
  }

  async dispose() {
    if (this.context?.fiber?.dispose) {
      try { await this.context.fiber.dispose(); } catch {}
    }
    if (this.persistenceRoot) await fs.rm(this.persistenceRoot, { recursive: true, force: true });
    this.context = null;
    this.persistenceRoot = null;
  }
}
