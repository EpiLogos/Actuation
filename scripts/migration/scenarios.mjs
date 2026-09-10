// Temporary, effect-injected parity surface. No production API is added.
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, writeFileSync, readdirSync, rmSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import * as store from '../../contracts/actuation-stream-store.mjs';
import { runDetection } from '../../detection/detect.mjs';
import { resolveSelf } from '../../detection/self.mjs';
import { harnessDescriptors, capabilityDescriptors, CATALOG_REVISION } from '../../detection/catalog.mjs';
import { scanSecretSources } from '../../detection/secret-sources/scan.mjs';
import { secretSourceCatalog } from '../../detection/secret-sources/catalog.mjs';
const repo = fileURLToPath(new URL('../../', import.meta.url));
const clone = value => JSON.parse(JSON.stringify(value));
export function caught(call) {
  try { return { ok: true, value: clone(call()) }; }
  catch (error) { return { ok: false, error: { name: error.name, message: error.message } }; }
}
function fakeEffects(spec, calls) {
  const defaults = {
    resolveExecutable: { found: false, path: null }, statProbe: { exists: false }, hashProbe: { ok: false },
    dirCountProbe: { exists: false }, versionProbe: { ok: false, reason: 'not supplied' },
    serviceProbe: { ok: true, detail: 'declared, not verified' }, httpJsonProbe: { ok: false, reason: 'not supplied' },
    envProbe: { ok: true, matched: {} }, env: { ok: true, matched: {} },
    'file-pattern': { ok: true, matched: [] }, 'vault-item': { ok: true, found: false }, 'cli-presence': { ok: true, found: false },
  };
  const effects = {};
  for (const name of ['expandHome', ...Object.keys(defaults)]) {
    effects[name] = (...args) => {
      calls.push({ effect: name, args: clone(args) });
      const configured = spec[name];
      const keyed = configured?.by_argument?.[JSON.stringify(args[0])];
      if (keyed !== undefined) return clone(keyed);
      if (configured?.value !== undefined) return clone(configured.value);
      if (name === 'expandHome') return args[0].replace(/^~/, '/home/oracle');
      return clone(defaults[name]);
    };
  }
  for (const name of spec.omit ?? []) delete effects[name];
  return effects;
}
export function evaluateScenario(scenario, cli = [process.execPath, join(repo, 'bin/actuation')]) {
  if (scenario.kind === 'catalog') return { revision: CATALOG_REVISION, harnesses: harnessDescriptors(), capabilities: capabilityDescriptors(), secret_sources: secretSourceCatalog() };
  if (scenario.kind === 'probe') {
    const { input } = scenario;
    const calls = [];
    const effects = fakeEffects(input.effects ?? {}, calls);
    const descriptors = input.descriptors ?? harnessDescriptors().filter(d => !input.slugs || input.slugs.includes(d.slug));
    const result = caught(() => input.mode === 'self'
      ? resolveSelf({ descriptors, effects, env: {}, now: new Date(input.now) })
      : runDetection({ descriptors, effects, now: new Date(input.now), probeVersions: input.versions ?? false }));
    return { result, calls };
  }
  if (scenario.kind === 'secret') {
    const calls = [];
    const value = scanSecretSources({ roots: ['/oracle/project'], effects: fakeEffects(scenario.input.effects ?? {}, calls) });
    value.scan_ref = 'secret-scan:$CLOCK'; value.observed_at = '$CLOCK';
    return { value, calls };
  }
  if (scenario.kind === 'fold') return caught(() => store.foldStreamFile(scenario.input.raw));
  if (scenario.kind === 'filename') return caught(() => store.streamFileName(scenario.input.ref));
  const directory = mkdtempSync(join(tmpdir(), 'actuation-scenario-'));
  try {
    if (scenario.kind === 'store') {
      const results = [];
      for (const action of scenario.input.actions) {
        const before = Object.fromEntries(readdirSync(directory).map(name => [name, readFileSync(join(directory, name), 'utf8')]));
        if (action.write !== undefined) writeFileSync(join(directory, store.streamFileName(action.stream_ref)), action.write);
        const result = caught(() => {
          assert.equal(typeof store[action.operation], 'function', `missing store operation ${action.operation}`);
          return store[action.operation]({ ...clone(action.args), root: directory });
        });
        if (action.expect_ok !== undefined) assert.equal(result.ok, action.expect_ok, `authored store assertion: ${action.label}`);
        const raw = Object.fromEntries(readdirSync(directory).sort().map(name => [name, readFileSync(join(directory, name), 'utf8')]));
        if (action.unchanged) assert.deepEqual(raw, before, `refusal/dedup must not write: ${action.label}`);
        if (action.append_only) for (const name of Object.keys(before)) assert.ok(raw[name]?.startsWith(before[name]), 'event append rewrote history');
        results.push({ label: action.label, result, files: raw });
      }
      return results;
    }
    if (scenario.kind === 'cli') {
      for (const [name, text] of Object.entries(scenario.input.files ?? {})) {
        assert.ok(!name.includes('/') && name !== '..', 'fixture file must be a basename');
        writeFileSync(join(directory, name), text);
      }
      const args = scenario.input.argv.map(arg => arg.replaceAll('$STORE', directory));
      const run = spawnSync(cli[0], [...cli.slice(1), ...args], { cwd: directory, encoding: 'utf8', input: scenario.input.stdin ?? '', timeout: 30000,
        env: { PATH: '', HOME: directory, ACTUATION_STREAM_STORE: directory, LANG: 'C.UTF-8' } });
      if (run.error) throw run.error;
      let stdout = run.stdout.trimEnd();
      try { stdout = JSON.parse(stdout); if (stdout.revision !== undefined) stdout.revision = '$REVISION'; } catch {}
      return { code: run.status, stdout, stderr: run.stderr.replaceAll(directory, '$STORE') };
    }
    throw new Error(`unknown scenario kind ${scenario.kind}`);
  } finally { rmSync(directory, { recursive: true, force: true }); }
}
export function compareScenario(actual, expected) {
  function normalise(value) {
    if (Array.isArray(value)) return value.map(normalise);
    if (!value || typeof value !== 'object') return value;
    if (value.ok === false && value.error) return { ok: false };
    const out = Object.fromEntries(Object.entries(value).map(([key, item]) => [key, normalise(item)]));
    if (out.code === 2 && typeof out.stderr === 'string') {
      assert.ok(out.stderr.startsWith('actuation: '), 'CLI refusal must be recognisable on stderr'); out.stderr = 'actuation: $DIAGNOSTIC\n';
    }
    if (out.files) {
      // JSON member order is not wire semantics. Append preservation and
      // no-write-on-refusal are separately asserted inside each runtime.
      out.files = Object.fromEntries(Object.entries(out.files).map(([name, raw]) => [name, raw.split('\n').map(line => { try { return JSON.parse(line); } catch { return line; } })]));
    }
    return out;
  }
  assert.deepEqual(normalise(actual), normalise(expected));
}
