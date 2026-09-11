import assert from 'node:assert/strict';
import test from 'node:test';
import { PiBody, VERSION } from '../pi.mjs';

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
