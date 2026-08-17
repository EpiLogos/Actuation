import assert from 'node:assert/strict';
import { ClassicRuntime } from '../../foundation/classic-runtime/index.js';
import { QLDirectCoreRuntime } from '../../foundation/ql-core-runtime/index.js';
import { replayRun, runAB } from '../../foundation/optics/index.js';
import { createNativeHost } from './shared/host.js';
import { createNativeBaselinePolicy } from './ql/policy.js';

const capabilities = [{ id: 'inspect', kind: 'native-capability' }];
const result = await runAB({
  request: {
    taskId: 'native-matched-runtime-smoke',
    input: { subject: 'matched-runtime-baseline' },
    successConditions: ['matched-runtime-baseline is currently verified'],
    capabilities,
    maxSteps: 8
  },
  hostFactory: createNativeHost,
  classicRuntime: new ClassicRuntime(),
  qlRuntime: new QLDirectCoreRuntime({ policy: createNativeBaselinePolicy() }),
  specRevision: 'f9d056c54caf094eb672f005ce3c8cbde4de0a5b',
  fixtureId: 'native-baseline-v1',
  model: { id: 'native-fixture-model', provider: 'fixture', parameters: { temperature: 0 } },
  capabilities,
  environment: { id: 'native-fixture-env', network: false },
  startState: { native_contract: 'native-host-contract-v1' }
});

assert.ok(Object.values(result.comparison.held_constant).every(Boolean));
assert.deepEqual(result.classic.status, { execution: 'completed', semantic: 'not_applicable' });
assert.deepEqual(result.ql.status, { execution: 'completed', semantic: 'closed' });
assert.equal(result.classic.events.some((event) => event.channel === 'runtime-semantic'), false);
assert.ok(result.classic.events.some((event) => event.channel === 'host'));
assert.ok(result.ql.events.some((event) => event.channel === 'runtime-semantic'));

const nativeTypes = result.ql.events
  .filter((event) => event.channel === 'host' && event.event_type === 'native_event')
  .map((event) => event.payload?.type)
  .filter(Boolean);
assert.ok(nativeTypes.includes('model_request'));
assert.ok(nativeTypes.includes('capability_request'));
assert.equal(nativeTypes.some((name) => ['message_start', 'turn_end', 'ModelRequestNode', 'CallToolsNode'].includes(name)), false);

const semanticTypes = result.ql.events.map((event) => event.event_type);
assert.ok(semanticTypes.indexOf('return_interpreted') < semanticTypes.indexOf('transition'));
assert.ok(semanticTypes.indexOf('circuit_closed') < semanticTypes.indexOf('reentry_created'));

const storedClassic = JSON.parse(JSON.stringify(result.classic));
const storedQl = JSON.parse(JSON.stringify(result.ql));
assert.equal(replayRun(storedClassic).event_count, result.classic.events.length);
assert.equal(replayRun(storedQl).event_count, result.ql.events.length);

console.log(JSON.stringify({ host: 'native', host_revision: 'native-host-contract-v1', classic: result.classic, ql: result.ql, comparison: result.comparison }));
