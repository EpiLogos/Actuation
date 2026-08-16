import assert from 'node:assert/strict';
import { ClassicRuntime } from '../../foundation/classic-runtime/index.js';
import { QLDirectCoreRuntime } from '../../foundation/ql-core-runtime/index.js';
import { replayRun, runAB } from '../../foundation/optics/index.js';
import { createPydanticHost, PYDANTIC_UPSTREAM } from './shared/host.js';
import { createPydanticBaselinePolicy } from './ql/policy.js';

const capabilities = [{ id: 'inspect', kind: 'pydantic-tool' }];
const result = await runAB({
  request: {
    taskId: 'pydantic-matched-runtime-smoke',
    input: { subject: 'matched-runtime-baseline' },
    successConditions: ['matched-runtime-baseline is currently verified'],
    capabilities,
    maxSteps: 8
  },
  hostFactory: createPydanticHost,
  classicRuntime: new ClassicRuntime(),
  qlRuntime: new QLDirectCoreRuntime({ policy: createPydanticBaselinePolicy() }),
  specRevision: 'f9d056c54caf094eb672f005ce3c8cbde4de0a5b',
  fixtureId: 'pydantic-baseline-v1',
  model: { id: 'pydantic-fixture-model', provider: 'fixture', parameters: { temperature: 0 } },
  capabilities,
  environment: { id: 'pydantic-fixture-env', network: false },
  startState: { pydantic_revision: PYDANTIC_UPSTREAM.revision }
});

assert.ok(Object.values(result.comparison.held_constant).every(Boolean));
assert.deepEqual(result.classic.status, { execution: 'completed', semantic: 'not_applicable' });
assert.deepEqual(result.ql.status, { execution: 'completed', semantic: 'closed' });
assert.equal(result.classic.events.some((event) => event.channel === 'runtime-semantic'), false);
assert.ok(result.classic.events.some((event) => event.channel === 'host'));
assert.ok(result.ql.events.some((event) => event.channel === 'runtime-semantic'));

const hostNodeNames = result.ql.events
  .filter((event) => event.channel === 'host' && event.event_type === 'pydantic_graph_event')
  .map((event) => event.payload?.node)
  .filter(Boolean);
assert.ok(hostNodeNames.includes('ModelRequestNode'));
assert.ok(hostNodeNames.includes('CallToolsNode'));
assert.equal(hostNodeNames.some((name) => ['P0', 'P1', 'P2', 'P3', 'P4', 'P5'].includes(name)), false);

const semanticTypes = result.ql.events.map((event) => event.event_type);
assert.ok(semanticTypes.indexOf('return_interpreted') < semanticTypes.indexOf('transition'));
assert.ok(semanticTypes.indexOf('circuit_closed') < semanticTypes.indexOf('reentry_created'));

const storedClassic = JSON.parse(JSON.stringify(result.classic));
const storedQl = JSON.parse(JSON.stringify(result.ql));
assert.equal(replayRun(storedClassic).event_count, result.classic.events.length);
assert.equal(replayRun(storedQl).event_count, result.ql.events.length);

console.log(JSON.stringify({ host: 'pydantic-ai', upstream_revision: PYDANTIC_UPSTREAM.revision, classic: result.classic, ql: result.ql, comparison: result.comparison }));
