import { spawn } from 'node:child_process';
import { StringDecoder } from 'node:string_decoder';

function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export class PrimeRpcClient {
  constructor({
    cwd,
    binary = process.env.PRIME_AGENT_BIN ?? 'prime-agent',
    provider = process.env.QL_PRIME_PROVIDER ?? null,
    model = process.env.QL_PRIME_MODEL ?? null,
    skillPaths = [],
    sessionDir = null,
    extraArgs = [],
    env = {}
  }) {
    this.cwd = cwd;
    this.binary = binary;
    this.provider = provider;
    this.model = model;
    this.skillPaths = [...skillPaths];
    this.sessionDir = sessionDir;
    this.extraArgs = [...extraArgs];
    this.env = { ...env };
    this.process = null;
    this.decoder = new StringDecoder('utf8');
    this.buffer = '';
    this.pending = new Map();
    this.sequence = 0;
    this.records = [];
    this.stderr = '';
    this.exit = null;
  }

  async start() {
    if (this.process) throw new Error('Prime RPC client already started.');
    const args = ['--mode', 'rpc', '--cwd', this.cwd, '--no-extensions', '--no-prompt-templates', '--no-context-files', '--no-skills'];
    if (process.env.QL_PRIME_OFFLINE !== '0') args.push('--offline');
    if (this.provider) args.push('--provider', this.provider);
    if (this.model) args.push('--model', this.model);
    if (this.sessionDir) args.push('--session-dir', this.sessionDir);
    for (const skill of this.skillPaths) args.push('--skill', skill);
    args.push(...this.extraArgs);

    this.process = spawn(this.binary, args, {
      cwd: this.cwd,
      env: { ...process.env, ...this.env },
      stdio: ['pipe', 'pipe', 'pipe']
    });

    this.process.stdout.on('data', (chunk) => this.#consume(chunk));
    this.process.stderr.on('data', (chunk) => { this.stderr += chunk.toString(); });
    this.exit = new Promise((resolve) => this.process.once('exit', (code, signal) => resolve({ code, signal })));

    await delay(150);
    if (this.process.exitCode !== null) {
      const exit = await this.exit;
      throw new Error(`Prime RPC exited during startup (${exit.code ?? exit.signal}): ${this.stderr}`);
    }
    await this.getState();
  }

  #consume(chunk) {
    this.buffer += this.decoder.write(chunk);
    let index;
    while ((index = this.buffer.indexOf('\n')) >= 0) {
      let line = this.buffer.slice(0, index);
      this.buffer = this.buffer.slice(index + 1);
      if (line.endsWith('\r')) line = line.slice(0, -1);
      if (!line) continue;
      let record;
      try {
        record = JSON.parse(line);
      } catch (error) {
        this.records.push({ type: 'client_parse_error', line, error: error.message });
        continue;
      }
      this.records.push(record);
      if (record.type === 'response' && record.id && this.pending.has(record.id)) {
        const pending = this.pending.get(record.id);
        this.pending.delete(record.id);
        clearTimeout(pending.timer);
        if (record.success === false) pending.reject(new Error(record.error ?? `Prime RPC command ${record.command ?? 'unknown'} failed.`));
        else pending.resolve(record);
      }
    }
  }

  request(command, { timeoutMs = 60_000 } = {}) {
    if (!this.process?.stdin?.writable) throw new Error('Prime RPC is not running.');
    const id = command.id ?? `actuation-prime-${++this.sequence}`;
    const payload = { ...command, id };
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error(`Prime RPC request ${id} timed out after ${timeoutMs}ms.`));
      }, timeoutMs);
      this.pending.set(id, { resolve, reject, timer });
      this.process.stdin.write(`${JSON.stringify(payload)}\n`);
    });
  }

  async getState() {
    const response = await this.request({ type: 'get_state' });
    return response.data;
  }

  async prompt(message) {
    return this.request({ type: 'prompt', message });
  }

  async getMessages() {
    const response = await this.request({ type: 'get_messages' });
    return response.data?.messages ?? [];
  }

  async getLastAssistantText() {
    const response = await this.request({ type: 'get_last_assistant_text' });
    return response.data?.text ?? null;
  }

  async getSessionStats() {
    const response = await this.request({ type: 'get_session_stats' });
    return response.data ?? null;
  }

  async refine(instructions) {
    const response = await this.request({ type: 'refine', instructions, global: false }, { timeoutMs: 10 * 60_000 });
    return response.data ?? null;
  }

  async waitForIdle({ timeoutMs = 20 * 60_000, pollMs = 350 } = {}) {
    const deadline = Date.now() + timeoutMs;
    let observedBusy = false;
    let idleWithAnswer = 0;
    while (Date.now() < deadline) {
      const state = await this.getState();
      if (state?.isStreaming || (state?.unfinishedActionCount ?? 0) > 0) {
        observedBusy = true;
        idleWithAnswer = 0;
      } else {
        if (observedBusy) return state;
        const text = await this.getLastAssistantText();
        idleWithAnswer = text ? idleWithAnswer + 1 : 0;
        if (idleWithAnswer >= 2) return state;
      }
      await delay(pollMs);
    }
    throw new Error(`Prime did not return to idle within ${timeoutMs}ms.`);
  }

  async stop() {
    if (!this.process) return;
    for (const pending of this.pending.values()) {
      clearTimeout(pending.timer);
      pending.reject(new Error('Prime RPC client stopped.'));
    }
    this.pending.clear();
    this.process.stdin.end();
    this.process.kill('SIGTERM');
    const timeout = delay(1500).then(() => ({ timeout: true }));
    const result = await Promise.race([this.exit, timeout]);
    if (result?.timeout && this.process.exitCode === null) this.process.kill('SIGKILL');
    this.process = null;
  }
}

export function parseExtraArgs(value = process.env.QL_PRIME_EXTRA_ARGS ?? '') {
  if (!value.trim()) return [];
  return JSON.parse(value);
}
