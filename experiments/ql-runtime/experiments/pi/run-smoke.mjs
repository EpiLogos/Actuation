import assert from 'node:assert/strict';
import { ClassicRuntime } from '../../foundation/classic-runtime/index.js';
import { QLDirectCoreRuntime } from '../../foundation/ql-core-runtime/index.js';
import { replayRun, runAB } from '../../foundation/optics/index.js';
import { createPiHost, PI_UPSTREAM } from './shared/host.js';
import { createPiBaselinePolicy } from './ql/policy.js';

const SPEC_REVISION = 'f9d056c54caf094eb672f005ce3c8cbde4de0a5b';
const MODEL = Object.freeze({ id: 'pi-fixture-model', provider: 'fixture', parameters: { temperature: 0 } });
const CAPABILITIES = Object.freeze([{ id: 'inspect', kind: 'pi-agent-tool' }]);
const ENVIRONMENT = Object.freeze({ id: 'pi-fixture-env', network: false });

const request = {
  taskId: 'pi-matched-runtime-smoke',
  input: { subject: 'matched-runtime-baseline', goal: 'verify the host/runtime seam' },
  successConditions: ['matched-runtime-baseline is currently verified'],
  capabilities: CAPABILITIES,
  maxSteps: 8
};

const result = await runAB({
  request,
  hostFactory: createPiHost,
  classicRuntime: new ClassicRuntime(),
  qlRuntime: new QLDirectCoreRuntime({ policy: createPiBaselinePolicy() }),
  specRevision: SPEC_REVISION,
  fixtureId: 'pi-baseline-v1',
  model: MODEL,
  capabilities: CAPABILITIES,
  environment: ENVIRONMENT,
  startState: { pi_revision: PI_UPSTREAM.revision }
});

assert.ok(Object.values(result.comparison.held_constant).every(Boolean));
assert.equal(result.classic.status.execution, 'completed');
assert.equal(result.classic.status.semantic, 'not_applicable');
assert.equal(result.ql.status.execution, 'completed');
assert.equal(result.ql.status.semantic, 'closed');
assert.equal(result.classic.events.some((event) => event.channel === 'runtime-semantic'), false);
assert.ok(result.classic.events.some((event) => event.channel === 'host'));
assert.ok(result.ql.events.some((event) => event.channel === 'host'));
assert.ok(result.ql.events.some((event) => event.channel === 'runtime-semantic'));

const interpreted = result.ql.events.findIndex((event) => event.event_type === 'return_interpreted');
const transition = result.ql.events.findIndex((event) => event.event_type === 'transition');
const closed = result.ql.events.findIndex((event) => event.event_type === 'circuit_closed');
const reentry = result.ql.events.findIndex((event) => event.event_type === 'reentry_created');
assert.ok(interpreted >= 0 && interpreted < transition);
assert.ok(closed >= 0 && closed < reentry);
assert.equal(result.ql.result.circuit.closureState, 'closed');
assert.equal(result.ql.result.reentry.renewed_frame.inherited_delta, result.ql.result.reentryDelta.id);

const storedClassic = JSON.parse(JSON.stringify(result.classic));
const storedQl = JSON.parse(JSON.stringify(result.ql));
assert.equal(replayRun(storedClassic).event_count, result.classic.events.length);
assert.equal(replayRun(storedQl).event_count, result.ql.events.length);

console.log(JSON.stringify({
  host: 'pi',
  upstream_revision: PI_UPSTREAM.revision,
  classic: result.classic,
  ql: result.ql,
  comparison: result.comparison,
  replay: {
    classic: replayRun(storedClassic),
    ql: replayRun(storedQl)
  }
}));
