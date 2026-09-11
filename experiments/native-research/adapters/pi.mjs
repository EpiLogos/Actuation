/** Pi's native package ABI only. Actuation Rust supplies context, normalizes
 * model output and owns the experiment. No QL algebra or comparison is here. */
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import readline from 'node:readline';
export const PACKAGE = '@earendil-works/pi-ai';
export const VERSION = '0.84.1';
function installedVersion() {
  // The pinned package exposes import conditions, not CommonJS require conditions.
  // Resolve exactly the ESM entry that preflight imports; do not bypass exports.
  let dir = path.dirname(fileURLToPath(import.meta.resolve(`${PACKAGE}/providers/all`)));
  for (let n = 0; n < 8; n++, dir = path.dirname(dir)) {
    const p = path.join(dir, 'package.json');
    if (fs.existsSync(p)) {
      const v = JSON.parse(fs.readFileSync(p, 'utf8'));
      if (v.name === PACKAGE) return v.version;
    }
  }
  throw new Error('Pi package manifest unavailable');
}
const observed = (n) => typeof n === 'number' && Number.isFinite(n) && n >= 0 ? n : null;
export class PiBody {
  constructor() { this.config = null; this.models = null; this.calls = []; }
  async preflight(config) {
    if (this.config) throw new Error('Pi body cannot be rebound');
    if (config.provider !== 'deepseek' || !config.model) throw new Error('Explicit DeepSeek model required');
    if (installedVersion() !== VERSION) throw new Error('Pi package is not the pinned version');
    this.models = (await import(`${PACKAGE}/providers/all`)).builtinModels();
    const model = this.models.getModel(config.provider, config.model);
    if (!model) throw new Error('Selected model absent from pinned Pi catalogue');
    const auth = await this.models.getAuth(model);
    this.config = { provider: config.provider, model: config.model };
    return { ready: true, ...this.config, framework: 'pi-ai', package_version: installedVersion(),
      credential_available: Boolean(auth), provider_request_executed: false };
  }
  async complete(request) {
    if (!this.config) throw new Error('Pi body requires preflight');
    const model = this.models.getModel(this.config.provider, this.config.model);
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 300_000);
    const ordinal = this.calls.length;
    try {
      const response = await this.models.complete(model, {
        systemPrompt: request.system,
        messages: [{ role: 'user', content: request.prompt, timestamp: Date.now() }], tools: []
      }, { signal: controller.signal, temperature: request.temperature });
      if (response.stopReason === 'error' || response.stopReason === 'aborted') {
        throw new Error('Pi native completion did not succeed');
      }
      const text = (response.content ?? []).filter(b => b.type === 'text').map(b => b.text).join('\n');
      const u = response.usage;
      const result = { output: text, usage: u ? {
        input_tokens: observed(u.input), output_tokens: observed(u.output),
        total_tokens: observed(u.totalTokens), cost: observed(u.cost?.total)
      } : null, raw: { framework: 'pi-ai', package_version: VERSION, provider: response.provider ?? null,
        model: response.model ?? null, stop_reason: response.stopReason ?? null } };
      this.calls.push({ ordinal, status: 'returned', raw: result.raw });
      return result;
    } catch (e) {
      this.calls.push({ ordinal, status: 'failed', error_kind: e?.name ?? 'Error' });
      throw new Error('Pi SDK completion failed; failure retained in native evidence');
    } finally { clearTimeout(timeout); }
  }
  async finalize() {
    return { kind: 'pi-native-sdk-evidence', framework: 'pi-ai', package_version: VERSION,
      model_calls: this.calls, session_evidence: null, provider_evidence: 'not-assessed' };
  }
}
export async function serve(body = new PiBody()) {
  const input = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
  for await (const line of input) {
    if (Buffer.byteLength(line) > 16 * 1024 * 1024) throw new Error('SDK input frame exceeds bound');
    const v = JSON.parse(line);
    let data = null, error = null;
    try {
      if (v.type === 'preflight') data = await body.preflight(v.configuration);
      else if (v.type === 'complete') data = await body.complete(v.request);
      else if (v.type === 'finalize') data = await body.finalize(v.inspection);
      else throw new Error('Unsupported Pi adapter command');
    } catch (e) { error = e instanceof Error ? e.message : 'Pi adapter failed'; }
    const out = JSON.stringify({ type: 'response', command: v.type, id: v.id,
      success: error === null, data, error });
    if (Buffer.byteLength(out) > 16 * 1024 * 1024) throw new Error('SDK response exceeds bound');
    process.stdout.write(out + '\n');
    if (v.type === 'finalize') break;
  }
}
if (process.argv[1] && fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  serve().catch(() => { process.stderr.write('Pi SDK transport failed\n'); process.exitCode = 1; });
}
