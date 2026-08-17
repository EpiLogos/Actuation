import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { DSH_INSPECTION_SCHEMA, DSH_PACKAGE_VERSION, DSH_PROVIDER_ROUTE, DSH_UPSTREAM_REVISION, DshSeries1Provider, dshCompositionFingerprint } from './dsh.mjs';

async function main() {
  const workspace = await fs.mkdtemp(path.join(os.tmpdir(), 'ql-series1-dsh-structural-'));
  const previous = process.env.DEEPSEEK_API_KEY;
  process.env.DEEPSEEK_API_KEY = previous || 'structural-only-not-sent';
  const provider = new DshSeries1Provider();
  try {
    await fs.writeFile(path.join(workspace, 'fact.txt'), 'structural only\n');
    await provider.assertReady();
    await provider.attachRun('structural-run', workspace);

    // Exercise real rc.5 SessionStore/Session append + JSONL persistence without
    // making a model request. These are native DSH events, not fixture evidence.
    provider.session.append('turn/start', { turn: 0 });
    provider.session.append('turn/end', { turn: 0, reason: { kind: 'completed' } });

    const portable = [
      { record_index: 0, channel: 'host', event_type: 'model_requested', payload: { purpose: 'structural-only' } },
      { record_index: 1, channel: 'runtime-semantic', event_type: 'ql_position_entered', ql: { position: 'P5' }, payload: { position: 'P5' } },
      { record_index: 2, channel: 'runtime-semantic', event_type: 'ql_closure_recorded', payload: { positive: true } },
      { record_index: 3, channel: 'host', event_type: 'model_returned', payload: { purpose: 'structural-only' } }
    ];
    await provider.capturePortableTrace(portable, { condition: 'ql-direct' });
    const evidence = await provider.snapshotNativeEvidence();

    const candidateEvents = evidence?.candidate_session?.events ?? [];
    const inspectionEvents = evidence?.inspection_session?.events ?? [];
    const inspectionPortable = inspectionEvents.filter((event) => event.type === 'series1/portable-event');
    const checks = {
      provider_route: provider.context?.llm?.listProviders?.().some((entry) => (typeof entry === 'string' ? entry : entry.provider ?? entry.id) === DSH_PROVIDER_ROUTE) ?? false,
      composition_fingerprint: provider.compositionFingerprint === dshCompositionFingerprint(),
      candidate_native_events: candidateEvents.some((event) => event.type === 'turn/start') && candidateEvents.some((event) => event.type === 'turn/end'),
      candidate_raw_jsonl: typeof evidence?.candidate_session?.raw_artifact?.content === 'string' && evidence.candidate_session.raw_artifact.content.includes('turn/start'),
      inspection_schema: evidence?.ql_inspection?.schema === DSH_INSPECTION_SCHEMA,
      inspection_read_only: evidence?.ql_inspection?.read_only === true && evidence?.ql_inspection?.candidate_context_authority === false,
      inspection_seed_accepted: inspectionPortable.length === portable.length,
      inspection_events_ignorable: inspectionPortable.every((event) => event.ignorable === true),
      inspection_raw_jsonl: typeof evidence?.inspection_session?.raw_artifact?.content === 'string' && evidence.inspection_session.raw_artifact.content.includes('series1/portable-event'),
      no_inspection_error: evidence?.inspection_session_error === null,
      package_version: evidence?.package_version === DSH_PACKAGE_VERSION,
      upstream_revision: evidence?.upstream_revision === DSH_UPSTREAM_REVISION
    };
    const valid = Object.values(checks).every(Boolean);
    process.stdout.write(`${JSON.stringify({ schema: 'ql-series1-dsh-structural/0.1', valid, checks, evidence }, null, 2)}\n`);
    if (!valid) process.exitCode = 1;
  } finally {
    await provider.dispose();
    await fs.rm(workspace, { recursive: true, force: true });
    if (previous === undefined) delete process.env.DEEPSEEK_API_KEY;
    else process.env.DEEPSEEK_API_KEY = previous;
  }
}

main().catch((error) => {
  console.error(error.stack ?? error.message ?? String(error));
  process.exitCode = 1;
});
