import fs from 'node:fs';
import { QLDirectCoreRuntime } from '../../foundation/ql-core-runtime/index.js';
import { createPiHost, PI_UPSTREAM } from '../../experiments/pi/shared/host.js';
import { createPydanticHost, PYDANTIC_UPSTREAM } from '../../experiments/pydantic/shared/host.js';
import { createNativeHost } from '../../experiments/native/shared/host.js';
import { createDeepQLRuntimeClass } from '../index.js';
import { runProductReviewSet } from './review-runs.js';

const SPEC_REVISION = 'f9d056c54caf094eb672f005ce3c8cbde4de0a5b';
const DeepRuntime = createDeepQLRuntimeClass(QLDirectCoreRuntime);
const scenarios = JSON.parse(fs.readFileSync(new URL('./dry-runs.json', import.meta.url), 'utf8')).cases;

const profiles = [
  {
    id:'pi',
    hostFactory:createPiHost,
    model:{ id:'pi-fixture-model', provider:'fixture', parameters:{ temperature:0 } },
    capabilities:[{ id:'inspect', kind:'pi-agent-tool' }],
    startState:{ pi_revision:PI_UPSTREAM.revision }
  },
  {
    id:'pydantic-ai',
    hostFactory:createPydanticHost,
    model:{ id:'pydantic-fixture-model', provider:'fixture', parameters:{ temperature:0 } },
    capabilities:[{ id:'inspect', kind:'pydantic-tool' }],
    startState:{ pydantic_revision:PYDANTIC_UPSTREAM.revision }
  },
  {
    id:'native',
    hostFactory:createNativeHost,
    model:{ id:'native-fixture-model', provider:'fixture', parameters:{ temperature:0 } },
    capabilities:[{ id:'inspect', kind:'native-capability' }],
    startState:{ native_contract:'native-host-contract-v1' }
  }
];

const records = await runProductReviewSet({ profiles, scenarios, DeepRuntime, specRevision:SPEC_REVISION });
const exactRevision = process.env.DEEP_REVISION ?? process.env.GITHUB_HEAD_SHA ?? process.env.GITHUB_SHA ?? 'working-tree';

console.log(JSON.stringify({
  schema:'ql-product-review-bundle/0.1',
  deep_revision:exactRevision,
  spec_revision:SPEC_REVISION,
  profiles:profiles.map((profile) => profile.id),
  scenario_count:scenarios.length,
  recorded_run_count:records.length,
  all_completed:records.every((record) => record.execution === 'completed' && record.semantic === 'closed'),
  operator_scenarios:records.filter((record) => record.operator_review).map((record) => ({
    profile:record.profile,
    scenario:record.scenario,
    event_count:record.operator_review.event_count,
    event_types:record.operator_review.event_types
  })),
  records
}, null, 2));
