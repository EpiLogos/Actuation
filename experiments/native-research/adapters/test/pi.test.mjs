import assert from 'node:assert/strict';
import test from 'node:test';
import { spawnSync } from 'node:child_process';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';
import { PiBody, PACKAGE, VERSION } from '../pi.mjs';

// Real pinned package loading/catalog/auth; only the provider completion below
// is controlled. This is SDK ABI evidence, not a live DeepSeek experiment.
test('Pi native catalogue and ordered completion/error ABI', async () => {
  const previous = process.env.DEEPSEEK_API_KEY;
  process.env.DEEPSEEK_API_KEY = 'deterministic-not-sent';
  try {
    const body = new PiBody();
    const preflight = await body.preflight({ provider: 'deepseek', model: 'deepseek-v4-flash' });
    assert.equal(preflight.package_version, VERSION);
    assert.equal(preflight.provider_request_executed, false);
    assert.equal(preflight.credential_available, true);
    const models = body.models;
    const seen = [];
    body.models = {
      getModel: (...args) => models.getModel(...args),
      complete: async (model, context, options) => {
        seen.push({ model, context, options });
        if (seen.length === 1) throw new Error('deliberate fixture failure');
        return { content: [{ type: 'text', text: '{"content":"fixture","capabilityCalls":[]}' }],
          model: model.id, provider: 'deepseek', stopReason: 'stop' };
      }
    };
    await assert.rejects(body.complete({ system: 'authored system', prompt: 'first', temperature: 0 }));
    const result = await body.complete({ system: 'authored system', prompt: 'second', temperature: 0 });
    assert.equal(result.usage, null);
    assert.equal(seen[1].context.systemPrompt, 'authored system');
    assert.equal(seen[1].context.messages[0].content, 'second');
    assert.ok(seen[1].options.signal instanceof AbortSignal);
    assert.equal(seen[1].options.temperature, 0);
    assert.deepEqual((await body.finalize()).model_calls.map(c => [c.ordinal, c.status]),
      [[0, 'failed'], [1, 'returned']]);
    await assert.rejects(body.preflight(preflight), /rebound/);
    await assert.rejects(new PiBody().preflight({ provider: 'other', model: 'x' }));
    await assert.rejects(new PiBody().preflight({ provider: 'deepseek', model: 'nonexistent-fixture' }));
  } finally {
    if (previous === undefined) delete process.env.DEEPSEEK_API_KEY;
    else process.env.DEEPSEEK_API_KEY = previous;
  }
});

test('Pi import-only package supports standalone JSONL preflight without credentials', () => {
  // This is the exact conditional-export mismatch reproduced in CI before the fix.
  assert.throws(() => createRequire(import.meta.url).resolve(`${PACKAGE}/providers/all`),
    { code: 'ERR_PACKAGE_PATH_NOT_EXPORTED' });
  assert.match(import.meta.resolve(`${PACKAGE}/providers/all`), /^file:/);
  const env = { ...process.env };
  delete env.DEEPSEEK_API_KEY;
  const frames = [
    { type: 'preflight', id: 'pi-preflight', configuration: {
      provider: 'deepseek', model: 'deepseek-v4-flash'
    } },
    { type: 'finalize', id: 'pi-finalize' }
  ];
  const child = spawnSync(process.execPath,
    [fileURLToPath(new URL('../pi.mjs', import.meta.url))], {
      input: frames.map(v => JSON.stringify(v)).join('\n') + '\n',
      encoding: 'utf8', env, timeout: 30_000, maxBuffer: 1024 * 1024
    });
  assert.ifError(child.error);
  assert.equal(child.status, 0, child.stderr);
  const responses = child.stdout.trim().split('\n').map(line => JSON.parse(line));
  assert.equal(responses.length, frames.length);
  for (let i = 0; i < frames.length; i++) {
    assert.equal(responses[i].type, 'response');
    assert.equal(responses[i].id, frames[i].id);
    assert.equal(responses[i].command, frames[i].type);
    assert.equal(responses[i].success, true, responses[i].error);
  }
  assert.equal(responses[0].data.package_version, VERSION);
  assert.equal(responses[0].data.credential_available, false);
  assert.equal(responses[0].data.provider_request_executed, false);
  assert.deepEqual(responses[1].data.model_calls, []);
  assert.equal(responses[1].data.provider_evidence, 'not-assessed');
});
