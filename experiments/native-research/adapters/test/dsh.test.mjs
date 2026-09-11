import assert from 'node:assert/strict';
import test from 'node:test';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { DshBody, DSH_UPSTREAM_REVISION, DSH_PACKAGE_VERSION, DSH_INSPECTION_SCHEMA } from '../dsh.mjs';

// Real Cordis/Agent/Session/BlockAssembler/persistence. Test-only chunks replace
// the provider stream and cannot be cited as live provider or owner-machine use.
test('DSH native stream, failure attribution and separate inspection session', async () => {
  const world = await fs.mkdtemp(path.join(os.tmpdir(), 'actuation-dsh-test-'));
  const old = process.env.DEEPSEEK_API_KEY;
  process.env.DEEPSEEK_API_KEY = 'deterministic-not-sent';
  const body = new DshBody();
  try {
    const preflight = await body.preflight({ provider: 'deepseek', model: 'deepseek-v4-flash',
      trace_ref: 'sdk-conformance', world, composition_fingerprint: 'fixture-parent-fingerprint',
      dsh_composition: { upstream_revision: DSH_UPSTREAM_REVISION, package_version: DSH_PACKAGE_VERSION } });
    assert.equal(preflight.provider_request_executed, false);
    assert.equal(preflight.package_version, DSH_PACKAGE_VERSION);
    assert.throws(() => body.agent.send({}), /observational/);
    let called = 0;
    const seen = [];
    body.context.llm.stream = async function* (request) {
      seen.push(request);
      if (called++ === 0) throw new Error('deliberate fixture stream failure');
      const text = '{"content":"fixture","capabilityCalls":[]}';
      yield { type: 'block-start', index: 0, blockType: 'text' };
      yield { type: 'text-delta', index: 0, text };
      yield { type: 'block-end', index: 0, block: { type: 'text', text } };
      yield { type: 'usage', usage: { inputTokens: 3, outputTokens: 1, cacheReadTokens: 2, cacheWriteTokens: 0 } };
      yield { type: 'finish', reason: { kind: 'stop' } };
    };
    await assert.rejects(body.complete({ system: 'system', prompt: 'first' }));
    const result = await body.complete({ system: 'system', prompt: 'second' });
    assert.equal(result.output, '{"content":"fixture","capabilityCalls":[]}');
    assert.equal(result.usage.input_tokens, 5);
    assert.equal(result.usage.total_tokens, 6);
    assert.equal(seen[1].messages.length, 1);
    assert.equal(seen[1].messages[0].content[0].text, 'second');
    assert.equal(body.modelCalls[0].error, true);
    assert.deepEqual(body.modelCalls.map(c => c.ordinal), [0, 1]);
    const inspection = { projection: { schema: DSH_INSPECTION_SCHEMA, read_only: true,
      candidate_context_authority: false, portable_events: [] }, seed: [
      { type: 'series1/portable-event', seq: 0, time: 0, ignorable: true,
        data: { event_type: 'closure-refused', evidence_standing: 'D' } }
    ] };
    const evidence = await body.finalize(inspection);
    assert.equal(evidence.inspection_session_error, null);
    assert.notEqual(evidence.candidate_session.id, evidence.inspection_session.id);
    assert.ok(evidence.candidate_session.raw_artifact.content.includes('assistant/message'));
    assert.ok(evidence.inspection_session.raw_artifact.content.includes('series1/portable-event'));
    assert.equal(evidence.candidate_session.events.filter(e => e.type === 'series1/portable-event').length, 0);
    assert.equal(evidence.model_call_alignment[0].error, true);
  } finally {
    await body.dispose();
    await fs.rm(world, { recursive: true, force: true });
    if (old === undefined) delete process.env.DEEPSEEK_API_KEY;
    else process.env.DEEPSEEK_API_KEY = old;
  }
});
