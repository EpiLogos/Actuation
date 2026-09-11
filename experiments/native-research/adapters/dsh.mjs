/** Bounded DSH package/session ABI. The Rust parent supplies normalized
 * context and derives the separate read-only inspection; QL is not implemented
 * in this language adapter. */
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createRequire } from 'node:module';
import readline from 'node:readline';
const require = createRequire(import.meta.url);
export const DSH_UPSTREAM_REVISION = '47f943859bef60e4160492346772ded9b24f765a';
export const DSH_PACKAGE_VERSION = '0.1.0-rc.5';
export const DSH_PROVIDER_ROUTE = 'deepseek-official';
export const DSH_INSPECTION_SCHEMA = 'ql-series1-dsh-inspection/0.1';
export const DSH_AGENT_PRESET = 'series1-loopruntime';
function packageVersion(packageName) {
  const pkg = require(`${packageName}/package.json`);
  return pkg.version;
}

function usageFromDsh(usage) {
  if (!usage) return null;
  const n = value => typeof value === 'number' && Number.isFinite(value) && value >= 0 ? value : null;
  const input = n(usage.inputTokens), output = n(usage.outputTokens);
  const cacheRead = n(usage.cacheReadTokens), cacheWrite = n(usage.cacheWriteTokens);
  // Do not manufacture absent counters. The raw SDK usage remains inspectable.
  const totalInput = input !== null && cacheRead !== null && cacheWrite !== null ? input + cacheRead + cacheWrite : null;
  return { input_tokens: totalInput, output_tokens: output,
    total_tokens: totalInput !== null && output !== null ? totalInput + output : null, native_usage: usage };
}
function visibleText(message) {
  return (message?.content ?? []).filter((block) => block.type === 'text').map((block) => block.text).join('');
}

function failureForTurn(error) {
  return { kind: 'error', error: { message: 'Native SDK stream failed', code: error?.code ?? 'UNKNOWN' } };
}

/** Real DSH transport/provider and native-session evidence owner. */
export class DshBody {
  constructor({ model = null, apiKey = process.env.DEEPSEEK_API_KEY } = {}) {
    this.id = 'dsh-deepseek-official';
    this.model = model;
    this.apiKey = apiKey;
    this.compositionFingerprint = null;
    this.configuration = null;
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

  async preflight(configuration) {
    if (this.configuration) throw new Error('DSH body cannot be rebound');
    if (configuration.provider !== 'deepseek' || !configuration.model || !configuration.trace_ref || !configuration.world) {
      throw new Error('Explicit DeepSeek model, trace and World required');
    }
    const composition = configuration.dsh_composition;
    if (composition?.upstream_revision !== DSH_UPSTREAM_REVISION || composition?.package_version !== DSH_PACKAGE_VERSION ||
        !configuration.composition_fingerprint) throw new Error('Pinned DSH composition descriptor required');
    this.configuration = configuration;
    this.compositionFingerprint = configuration.composition_fingerprint;
    this.model = configuration.model;
    await this.assertReady();
    await this.attachRun(configuration.trace_ref, configuration.world);
    return { ready: true, provider: 'deepseek', model: this.model, framework: 'deepseek-harness',
      package_version: DSH_PACKAGE_VERSION, upstream_revision: DSH_UPSTREAM_REVISION,
      credential_available: Boolean(this.apiKey), provider_request_executed: false,
      native_session_ref: this.session.id, composition_fingerprint: this.compositionFingerprint };
  }

  async assertReady() {
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

  async complete({ system, prompt, temperature = 0, signal } = {}) {
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
    let ended = false;
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
        : finish?.kind === 'error' ? { kind: 'error', error: finish.failure ?? { message: 'DSH request failed', code: 'UNKNOWN' } }
        : { kind: 'completed' };
      session.append('turn/end', { turn, reason: endReason });
      ended = true;
      if (finish?.kind === 'error' || finish?.kind === 'aborted') {
        throw new Error('DSH native stream did not complete successfully');
      }
      const text = visibleText(message);
      const result = { output: text, usage: null, raw: null };
      result.usage = usageFromDsh(assembler.usage);
      result.raw = { provider: DSH_PROVIDER_ROUTE, model: this.model, finish, dsh_session_id: session.id };
      this.modelCalls.push({ ordinal, turn, step, dsh_seq_start: seqStart, dsh_seq_end: session.seq - 1 });
      return result;
    } catch (error) {
      if (!ended) {
        try { session.append('step/end', { turn, step }); } catch {}
        try { session.append('turn/end', { turn, reason: failureForTurn(error) }); } catch {}
      }
      this.modelCalls.push({ ordinal, turn, step, dsh_seq_start: seqStart, dsh_seq_end: session.seq - 1, error: true });
      throw new Error('DSH SDK completion failed; native failure retained');
    }
  }

  async captureInspection(inspection) {
    if (!this.runId || !this.session) return;
    const projection = inspection?.projection;
    if (projection?.schema !== DSH_INSPECTION_SCHEMA || projection.read_only !== true ||
        projection.candidate_context_authority !== false || !Array.isArray(inspection.seed)) {
      throw new Error('DSH inspection must be an explicitly read-only native projection');
    }
    this.inspectionProjection = { ...projection, model_calls: this.modelCalls };
    try {
      const { SessionId } = this.modules.session;
      const inspectionId = SessionId(`${this.session.id}:inspection`);
      this.inspectionSession = this.context.sessions.create(inspectionId, {
        seed: inspection.seed,
        meta: { cwd: this.session.header.cwd, agentPreset: 'series1-inspection-readonly', seedLength: inspection.seed.length }
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
    return ({
      kind: 'deepseek-harness-native-evidence',
      upstream_revision: DSH_UPSTREAM_REVISION,
      package_version: DSH_PACKAGE_VERSION,
      provider_route: DSH_PROVIDER_ROUTE,
      composition_fingerprint: this.compositionFingerprint,
      plugin_tree: this.configuration?.dsh_composition?.plugin_tree ?? null,
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

  async finalize(inspection) {
    try {
      await this.captureInspection(inspection);
      return await this.snapshotNativeEvidence();
    } finally { await this.dispose(); }
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

export async function serve(body = new DshBody()) {
  const input = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
  try {
    for await (const line of input) {
      if (Buffer.byteLength(line) > 16 * 1024 * 1024) throw new Error('DSH input frame exceeds bound');
      const v = JSON.parse(line);
      let data = null, error = null;
      try {
        if (v.type === 'preflight') data = await body.preflight(v.configuration);
        else if (v.type === 'complete') data = await body.complete(v.request);
        else if (v.type === 'finalize') data = await body.finalize(v.inspection);
        else throw new Error('Unsupported DSH adapter command');
      } catch (e) { error = e instanceof Error ? e.message : 'DSH adapter failed'; }
      const out = JSON.stringify({ type: 'response', command: v.type, id: v.id,
        success: error === null, data, error });
      if (Buffer.byteLength(out) > 16 * 1024 * 1024) throw new Error('DSH response exceeds bound');
      process.stdout.write(out + '\n');
      if (v.type === 'finalize') break;
    }
  } finally { await body.dispose(); }
}
if (process.argv[1] && fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  serve().catch(() => { process.stderr.write('DSH SDK transport failed\n'); process.exitCode = 1; });
}
