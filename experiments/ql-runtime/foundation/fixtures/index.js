import { ClassicRuntime } from '../classic-runtime/index.js';
import { QLDirectCoreRuntime } from '../ql-core-runtime/index.js';
import { runAB } from '../optics/index.js';

import { ScriptedHost } from './scripted-host.js';
export { ScriptedHost } from './scripted-host.js';

/**
 * Deterministic closure-integrity check over verification evidence.
 *
 * Models just enough verification identity for the Foundation fixtures:
 * result identity (id), Subject identity (subject), Subject state/revision
 * identity (state), current/stale applicability (current), and the candidate
 * state being claimed complete (determination.claimed_subject/claimed_state).
 *
 * Invariant: a completion claim that cites verification results may only be
 * warranted while every cited result is current AND its Subject/state matches
 * the candidate state being claimed complete. Stale or mismatched evidence
 * cannot by itself warrant closure; it forces the semantically appropriate
 * P5 continuation instead (re-verify material by default; re-check
 * whole-relative adequacy for subject/state mismatch, overridable per result).
 */
export function evaluateVerificationIntegrity({ determination, verification }) {
  if (!verification || typeof verification !== 'object') {
    return { ok: true, problems: [] };
  }
  const results = verification.results ?? {};
  const cited = Array.isArray(determination?.evidence_refs) ? determination.evidence_refs : [];
  const problems = [];
  let destination = 'P1';

  for (const ref of cited) {
    const result = results[ref];
    if (!result) {
      problems.push(`cited verification '${ref}' does not exist`);
      destination = 'P1';
      continue;
    }
    if (result.current !== true) {
      problems.push(`verification '${ref}' is stale (current=${String(result.current)})`);
      destination = result.continuation ?? 'P1';
      continue;
    }
    const subjectMatches = result.subject === determination.claimed_subject;
    const stateMatches = result.state === determination.claimed_state;
    if (!subjectMatches) {
      problems.push(`verification '${ref}' subject '${result.subject}' does not match claimed subject '${determination.claimed_subject}'`);
    }
    if (!stateMatches) {
      problems.push(`verification '${ref}' state '${result.state}' does not match claimed state '${determination.claimed_state}'`);
    }
    if (!subjectMatches || !stateMatches) {
      destination = result.continuation ?? 'P4';
    }
  }

  if (problems.length > 0) {
    return { ok: false, problems, destination, reason: problems.join('; ') };
  }
  return { ok: true, problems: [] };
}

export class ScriptedQLPolicy {
  constructor(steps, { reentryDelta = {}, verification = null } = {}) {
    this.steps = structuredClone(steps);
    this.cursor = 0;
    this.lastStep = null;
    this.reentryDelta = structuredClone(reentryDelta);
    this.verification = verification;
  }

  async nextAct({ circuit }) {
    const step = this.steps[this.cursor++];
    if (!step) return null;
    if (step.source && step.source !== circuit.activePosition.id) {
      throw new Error(`Fixture expected ${step.source}, runtime active at ${circuit.activePosition.id}.`);
    }
    this.lastStep = step;
    return {
      sourcePosition: step.source ?? circuit.activePosition.id,
      intent: step.intent ?? `fixture step ${this.cursor}`,
      carrier: step.carrier ?? { kind: 'internal_control', input: step.raw ?? null },
      claimedPosition: step.claimedPosition,
      claimedRelation: step.claimedRelation,
      metadata: { fixture_step: this.cursor - 1 }
    };
  }

  async establishDifference() {
    return structuredClone(this.lastStep?.difference ?? null);
  }

  async interpret() {
    const step = this.lastStep;
    return {
      destination: step.destination,
      rationale: step.rationale ?? `scripted interpretation to ${step.destination}`,
      residueDelta: structuredClone(step.residueDelta ?? {}),
      witness: structuredClone(step.witness ?? {
        observed_position: step.destination,
        observed_relation: step.source && step.destination
          ? `R${step.source.slice(1)}${step.destination.slice(1)}`
          : undefined,
        confidence: 1,
        ambiguity: [],
        structural_facts: { deterministic_fixture: true }
      })
    };
  }

  async proposeDetermination() {
    return structuredClone(this.lastStep?.determination ?? null);
  }

  async evaluateClosure({ determination }) {
    const integrity = evaluateVerificationIntegrity({
      determination,
      verification: this.verification
    });
    if (!integrity.ok) {
      // Verification is cited on the closure warrant but is stale or
      // subject/state-mismatched: the candidate cannot close on it. Produce
      // the semantically appropriate P5 continuation instead of QLClosure.
      return {
        status: 'reopen',
        destination: integrity.destination,
        task_success: 'false',
        rationale: integrity.reason,
        retained_delta_preview: null
      };
    }
    const verdict = this.lastStep?.verdict;
    if (!verdict) {
      throw new Error('Fixture reached explicit P5 determination without a closure verdict.');
    }
    return structuredClone(verdict);
  }

  async createReentryDelta() {
    return structuredClone(this.lastStep?.reentryDelta ?? this.reentryDelta);
  }
}

const setup = (destination) => ({
  source: 'P0',
  intent: `Establish fixture responsibility ${destination}`,
  carrier: { kind: 'internal_control', input: { setup: destination } },
  difference: `Fixture ground requires ${destination}`,
  destination,
  residueDelta: {}
});

const determination = (requestedOutcome = 'reopen', overrides = {}) => ({
  synthesis: overrides.synthesis ?? 'fixture candidate determination',
  intent_ref: 'fixture-intent',
  claimed_adequacy: overrides.claimed_adequacy ?? 'unknown',
  claimed_subject: overrides.claimed_subject ?? null,
  claimed_state: overrides.claimed_state ?? null,
  evidence_refs: overrides.evidence_refs ?? [],
  evaluation_refs: overrides.evaluation_refs ?? [],
  unresolved_refs: overrides.unresolved_refs ?? [],
  requested_outcome: requestedOutcome
});

function reopenFixture(id, destination, difference) {
  return {
    id,
    expected: {
      relation: `R5${destination.slice(1)}`,
      destination,
      closure: 'open'
    },
    steps: [
      setup('P5'),
      {
        source: 'P5',
        intent: 'Evaluate candidate determination',
        carrier: { kind: 'internal_control', input: 'candidate' },
        difference,
        destination: 'P5',
        determination: determination('reopen', { unresolved_refs: [`${id}:unresolved`] }),
        verdict: {
          status: 'reopen',
          destination,
          task_success: 'false',
          rationale: difference
        }
      }
    ]
  };
}

export const FOUNDATION_FIXTURES = Object.freeze([
  {
    id: 'QLF-001',
    expected: { relation: 'R11', destination: 'P1', closure: 'open' },
    steps: [setup('P1'), {
      source: 'P1',
      intent: 'Inspect parser source',
      carrier: { kind: 'tool', name: 'read', args: { path: 'parser.js' } },
      difference: 'Relevant parser implementation acquired',
      destination: 'P1',
      residueDelta: { create: [{ kind: 'material', position: 'P1', value: 'parser source', provenance: { fixture: 'QLF-001' } }] }
    }]
  },
  {
    id: 'QLF-002',
    expected: { relation: 'R13', destination: 'P3', closure: 'open' },
    steps: [setup('P1'), {
      source: 'P1',
      carrier: { kind: 'tool', name: 'read', args: { path: 'parser.js' } },
      difference: 'Parser state is a recursive tagged union',
      destination: 'P3',
      residueDelta: {
        create: [
          { kind: 'material', position: 'P1', value: 'source definition', provenance: { fixture: 'QLF-002' } },
          { kind: 'form', position: 'P3', value: 'recursive tagged union', provenance: { fixture: 'QLF-002' } }
        ]
      }
    }]
  },
  {
    id: 'QLF-003',
    expected: { relation: 'R23', destination: 'P3', closure: 'open' },
    steps: [setup('P2'), {
      source: 'P2',
      carrier: { kind: 'tool', name: 'test', args: {} },
      difference: 'Representation cannot express required recursive state',
      destination: 'P3',
      residueDelta: {
        create: [
          { kind: 'effect', position: 'P2', value: 'test executed', provenance: { fixture: 'QLF-003' } },
          { kind: 'form', position: 'P3', value: 'representation defect', provenance: { fixture: 'QLF-003' } }
        ]
      }
    }]
  },
  {
    id: 'QLF-004',
    expected: { relation: 'R31', destination: 'P1', closure: 'open' },
    steps: [setup('P3'), {
      source: 'P3',
      carrier: { kind: 'model' },
      difference: 'Public interface contract has not been inspected',
      destination: 'P1'
    }]
  },
  {
    id: 'QLF-005',
    expected: { relation: 'R43', destination: 'P3', closure: 'open' },
    steps: [setup('P4'), {
      source: 'P4',
      carrier: { kind: 'tool', name: 'read', args: { path: 'requirements.md' } },
      difference: 'Proposed architecture breaks required public API',
      destination: 'P3',
      residueDelta: { create: [{ kind: 'evaluation', position: 'P4', value: 'compatibility failed', provenance: { fixture: 'QLF-005' } }] }
    }]
  },
  reopenFixture('QLF-006', 'P1', 'No evidence exists that Windows behaviour was inspected'),
  reopenFixture('QLF-007', 'P2', 'Correct patch exists only in memory and has not been written'),
  reopenFixture('QLF-008', 'P3', 'Tests pass but implementation violates abstraction boundary'),
  reopenFixture('QLF-009', 'P4', 'Two interpretations of the requirement remain unresolved'),
  reopenFixture('QLF-010', 'P0', 'Reported parser bug is malformed upstream input'),
  {
    id: 'QLF-011',
    expected: { closure: 'closed', reentry: true },
    steps: [
      {
        source: 'P0',
        carrier: { kind: 'internal_control', input: 'candidate' },
        difference: 'Candidate form ready for contextual evaluation',
        destination: 'P4',
        residueDelta: { create: [{ kind: 'evaluation', position: 'P4', value: 'requirements satisfied', provenance: { fixture: 'QLF-011' } }] }
      },
      {
        source: 'P4',
        carrier: { kind: 'internal_control', input: 'verified' },
        difference: 'Required behaviour verified and no requirement remains unresolved',
        destination: 'P5',
        determination: determination('close', { synthesis: 'accepted parser behaviour', evidence_refs: ['e:tests'], evaluation_refs: ['e:requirements'] }),
        verdict: { status: 'close', task_success: 'true', rationale: 'P0/P4/P5 coherent' },
        reentryDelta: {
          achieved_artifact_refs: ['artifact:parser'],
          established_material_refs: ['e:tests'],
          retained_form_refs: ['form:parser'],
          changed_assumptions: [],
          unresolved_refs: [],
          opened_questions: [],
          provenance: { fixture: 'QLF-011' }
        }
      }
    ]
  },
  {
    id: 'QLF-012',
    expected: { same_carrier_different_relations: ['R11', 'R13', 'R34'] },
    steps: [
      setup('P1'),
      { source: 'P1', carrier: { kind: 'tool', name: 'read' }, difference: 'read supplies material', destination: 'P1' },
      { source: 'P1', carrier: { kind: 'tool', name: 'read' }, difference: 'read discloses form', destination: 'P3' },
      { source: 'P3', carrier: { kind: 'tool', name: 'read' }, difference: 'read establishes contextual adequacy', destination: 'P4' }
    ]
  },
  {
    id: 'QLF-013',
    expected: { trajectory: ['R03', 'R34', 'R45'], closure: 'closed', capability_calls: 0 },
    steps: [
      { source: 'P0', carrier: { kind: 'model' }, difference: 'Conceptual form established', destination: 'P3', residueDelta: { create: [{ kind: 'form', position: 'P3', value: 'conceptual form', provenance: { fixture: 'QLF-013' } }] } },
      { source: 'P3', carrier: { kind: 'model' }, difference: 'Form is contextually adequate', destination: 'P4', residueDelta: { create: [{ kind: 'evaluation', position: 'P4', value: 'adequate', provenance: { fixture: 'QLF-013' } }] } },
      { source: 'P4', carrier: { kind: 'model' }, difference: 'Intent is realised', destination: 'P5', determination: determination('close', { synthesis: 'conceptual answer' }), verdict: { status: 'close', task_success: 'true' } }
    ]
  },
  {
    id: 'QLF-014',
    expected: { closure: 'open', minimum_capability_calls: 3 },
    steps: [
      { source: 'P0', carrier: { kind: 'tool', name: 'read' }, difference: 'material gathered', destination: 'P1' },
      { source: 'P1', carrier: { kind: 'tool', name: 'write' }, difference: 'change applied but unverified', destination: 'P2' },
      { source: 'P2', carrier: { kind: 'tool', name: 'test' }, difference: 'tests pass but intent remains unresolved', destination: 'P4' }
    ]
  },
  {
    id: 'QLF-017',
    expected: { closure: 'closed', retained: ['artifact:A', 'evidence:E', 'question:Q'] },
    steps: [
      { source: 'P0', carrier: { kind: 'model' }, difference: 'form', destination: 'P3', residueDelta: { create: [{ kind: 'form', position: 'P3', value: 'form:A', provenance: { fixture: 'QLF-017' } }] } },
      { source: 'P3', carrier: { kind: 'model' }, difference: 'evaluation', destination: 'P4', residueDelta: { create: [{ kind: 'evaluation', position: 'P4', value: 'evidence:E', provenance: { fixture: 'QLF-017' } }] } },
      { source: 'P4', carrier: { kind: 'model' }, difference: 'determined with retained question', destination: 'P5', determination: determination('close', { synthesis: 'artifact:A', unresolved_refs: ['question:Q'] }), verdict: { status: 'close', task_success: 'true' }, reentryDelta: { achieved_artifact_refs: ['artifact:A'], established_material_refs: ['evidence:E'], retained_form_refs: ['form:A'], unresolved_refs: ['question:Q'], provenance: { fixture: 'QLF-017' } } }
    ]
  },
  {
    id: 'QLF-018',
    expected: { closure: 'closed', reentry: true },
    verification: {
      results: {
        'ver:qlf018': {
          id: 'ver:qlf018',
          subject: 'parser-v2',
          state: 'rev-7',
          current: true,
          outcome: 'pass',
          provenance: { fixture: 'QLF-018' }
        }
      }
    },
    steps: [
      {
        source: 'P0',
        carrier: { kind: 'internal_control', input: 'candidate parser-v2 rev-7' },
        difference: 'Candidate form ready for contextual evaluation',
        destination: 'P4',
        residueDelta: { create: [{ kind: 'evaluation', position: 'P4', value: 'requirements satisfied', provenance: { fixture: 'QLF-018' } }] }
      },
      {
        source: 'P4',
        carrier: { kind: 'internal_control', input: 'verified' },
        difference: 'Current subject/state-matched verification supports determination',
        destination: 'P5',
        determination: determination('close', {
          synthesis: 'parser-v2 rev-7 accepted',
          claimed_subject: 'parser-v2',
          claimed_state: 'rev-7',
          evidence_refs: ['ver:qlf018'],
          evaluation_refs: ['e:requirements']
        }),
        verdict: { status: 'close', task_success: 'true', rationale: 'verification current and subject/state-matched' },
        reentryDelta: {
          achieved_artifact_refs: ['artifact:parser-v2'],
          established_material_refs: ['ver:qlf018'],
          retained_form_refs: ['form:parser'],
          changed_assumptions: [],
          unresolved_refs: [],
          opened_questions: [],
          provenance: { fixture: 'QLF-018' }
        }
      }
    ]
  }
]);

export const NEGATIVE_FIXTURES = Object.freeze([
  {
    id: 'QLN-001',
    expected: { no_closure: true },
    steps: [
      { source: 'P0', carrier: { kind: 'model' }, difference: 'candidate answer with no tool request', destination: 'P5' }
    ]
  },
  {
    id: 'QLN-002',
    expected: { relation: 'R43', destination: 'P3' },
    steps: [setup('P4'), {
      source: 'P4', carrier: { kind: 'tool', name: 'read' }, difference: 'whole-relative verification reveals form defect', destination: 'P3'
    }]
  },
  {
    id: 'QLN-003',
    expected: { relation: 'R51', no_closure: true },
    steps: [setup('P5'), {
      source: 'P5', carrier: { kind: 'internal_control', input: 'candidate' }, difference: 'required evidence missing', destination: 'P5', determination: determination('reopen', { unresolved_refs: ['e:missing'] }), verdict: { status: 'reopen', destination: 'P1', task_success: 'false' }
    }]
  },
  {
    id: 'QLN-004',
    expected: { generic_restart_is_not_reentry: true },
    steps: [
      { source: 'P0', carrier: { kind: 'model' }, difference: 'generic summary only', destination: 'P3' }
    ]
  },
  {
    id: 'QLN-006',
    expected: { trajectory: ['R03', 'R34', 'R45'], forbidden_relations: ['R01', 'R12', 'R23'] },
    steps: [
      { source: 'P0', carrier: { kind: 'model' }, difference: 'form established directly', destination: 'P3' },
      { source: 'P3', carrier: { kind: 'model' }, difference: 'contextually adequate', destination: 'P4' },
      { source: 'P4', carrier: { kind: 'model' }, difference: 'determined', destination: 'P5', determination: determination('close', { synthesis: 'direct route' }), verdict: { status: 'close', task_success: 'true' } }
    ]
  },
  {
    id: 'QLN-007',
    expected: { no_closure: true, closure: 'open', reopen_relations: ['R51', 'R54'] },
    verification: {
      results: {
        'ver:stale': {
          id: 'ver:stale',
          subject: 'parser-v2',
          state: 'rev-7',
          current: false,
          outcome: 'pass',
          continuation: 'P1'
        },
        'ver:mismatch': {
          id: 'ver:mismatch',
          subject: 'parser-v1',
          state: 'rev-6',
          current: true,
          outcome: 'pass',
          continuation: 'P4'
        }
      }
    },
    steps: [
      setup('P5'),
      {
        source: 'P5',
        carrier: { kind: 'internal_control', input: 'candidate parser-v2 rev-7' },
        difference: 'Candidate claims completion on stale verification',
        destination: 'P5',
        determination: determination('close', {
          synthesis: 'parser-v2 rev-7 complete',
          claimed_subject: 'parser-v2',
          claimed_state: 'rev-7',
          evidence_refs: ['ver:stale']
        }),
        verdict: { status: 'close', task_success: 'true' }
      },
      {
        source: 'P1',
        carrier: { kind: 'tool', name: 'read', args: { path: 'parser-v2.js' } },
        difference: 'Fresh verification gathered for a different subject/state',
        destination: 'P5',
        determination: determination('close', {
          synthesis: 'parser-v2 rev-7 complete',
          claimed_subject: 'parser-v2',
          claimed_state: 'rev-7',
          evidence_refs: ['ver:mismatch']
        }),
        verdict: { status: 'close', task_success: 'true' }
      }
    ]
  }
]);

export function defaultFixtureHost() {
  return new ScriptedHost({
    modelResponses: Array.from({ length: 16 }, () => ({ content: 'scripted model carrier result', capabilityCalls: [] })),
    capabilities: {
      read: async (args) => ({ ok: true, kind: 'read', args }),
      write: async (args) => ({ ok: true, kind: 'write', args }),
      test: async (args) => ({ ok: true, kind: 'test', args })
    },
    context: { fixture: true }
  });
}

export async function runQLFixture(fixture) {
  const policy = new ScriptedQLPolicy(fixture.steps, {
    verification: fixture.verification ?? null
  });
  const runtime = new QLDirectCoreRuntime({ policy });
  const host = defaultFixtureHost();
  const events = [];
  host.attachObserver({ emit: (event) => events.push(structuredClone(event)) }, `fixture:${fixture.id}`);
  const observer = { emit: (event) => events.push(structuredClone(event)) };
  const result = await runtime.run({
    taskId: fixture.id,
    input: fixture.id,
    successConditions: ['fixture expectation'],
    maxSteps: fixture.steps.length + 1,
    runId: `fixture:${fixture.id}`
  }, host, observer);
  return { fixture, result, events };
}

function semanticTransitions(run) {
  return run.events
    .filter((event) => event.channel === 'runtime-semantic' && event.event_type === 'transition')
    .map((event) => event.ql?.relation)
    .filter(Boolean);
}

function eventTypes(run) {
  return run.events.map((event) => event.event_type);
}

function hostCapabilityCalls(run) {
  return run.events.filter((event) => event.channel === 'host' && event.event_type === 'capability_call').length;
}

export function evaluateFixtureRun(run) {
  const { fixture, result } = run;
  const relations = semanticTransitions(run);
  const types = eventTypes(run);
  const expected = fixture.expected;
  const failures = [];

  if (expected.relation && !relations.includes(expected.relation)) {
    failures.push(`expected relation ${expected.relation}; got ${relations.join(', ')}`);
  }
  if (expected.destination) {
    const matched = run.events.some((event) => event.event_type === 'transition' && event.ql?.to === expected.destination && (!expected.relation || event.ql?.relation === expected.relation));
    if (!matched) failures.push(`expected destination ${expected.destination}`);
  }
  if (expected.closure === 'closed' && !types.includes('circuit_closed')) {
    failures.push('expected positive circuit_closed event');
  }
  if (expected.closure === 'open' && types.includes('circuit_closed')) {
    failures.push('expected circuit to remain open');
  }
  if (expected.no_closure && types.includes('circuit_closed')) {
    failures.push('forbidden automatic circuit_closed event observed');
  }
  if (expected.reentry && !types.includes('reentry_created')) {
    failures.push('expected reentry_created event');
  }
  if (expected.reopen_relations) {
    for (const relation of expected.reopen_relations) {
      if (!relations.includes(relation)) {
        failures.push(`expected P5 continuation relation ${relation}; got ${relations.join(', ')}`);
      }
    }
  }
  if (expected.trajectory) {
    const filtered = relations.filter((relation) => expected.trajectory.includes(relation));
    if (JSON.stringify(filtered) !== JSON.stringify(expected.trajectory)) {
      failures.push(`expected trajectory ${expected.trajectory.join('→')}; got ${relations.join('→')}`);
    }
  }
  if (expected.same_carrier_different_relations) {
    for (const relation of expected.same_carrier_different_relations) {
      if (!relations.includes(relation)) failures.push(`same-carrier test missing ${relation}`);
    }
  }
  if (expected.forbidden_relations) {
    for (const relation of expected.forbidden_relations) {
      if (relations.includes(relation)) failures.push(`forbidden manufactured relation ${relation}`);
    }
  }
  if (expected.capability_calls !== undefined && hostCapabilityCalls(run) !== expected.capability_calls) {
    failures.push(`expected ${expected.capability_calls} capability calls; got ${hostCapabilityCalls(run)}`);
  }
  if (expected.minimum_capability_calls !== undefined && hostCapabilityCalls(run) < expected.minimum_capability_calls) {
    failures.push(`expected at least ${expected.minimum_capability_calls} capability calls; got ${hostCapabilityCalls(run)}`);
  }
  if (expected.retained) {
    const retained = [
      ...(result.reentryDelta?.achieved_artifact_refs ?? []),
      ...(result.reentryDelta?.established_material_refs ?? []),
      ...(result.reentryDelta?.unresolved_refs ?? [])
    ];
    for (const ref of expected.retained) {
      if (!retained.includes(ref)) failures.push(`re-entry missing retained ref ${ref}`);
    }
  }
  if (expected.generic_restart_is_not_reentry && types.includes('reentry_created')) {
    failures.push('generic restart masqueraded as re-entry');
  }

  const returnIndices = run.events
    .map((event, index) => ({ event, index }))
    .filter(({ event }) => event.event_type === 'return_interpreted');
  for (const { event, index } of returnIndices) {
    const transitionIndex = run.events.findIndex((candidate, candidateIndex) =>
      candidateIndex > index &&
      candidate.event_type === 'transition' &&
      candidate.ql?.relation === event.ql?.relation
    );
    if (transitionIndex === -1) {
      failures.push(`interpreted return ${event.event_id} has no later transition`);
    }
  }

  return {
    id: fixture.id,
    passed: failures.length === 0,
    failures,
    result_status: result.status,
    relations,
    capability_calls: hostCapabilityCalls(run),
    positive_closure: types.includes('circuit_closed'),
    reentry: types.includes('reentry_created')
  };
}

export async function runFoundationGate() {
  const fixtures = [...FOUNDATION_FIXTURES, ...NEGATIVE_FIXTURES];
  const evidence = [];
  for (const fixture of fixtures) {
    const run = await runQLFixture(fixture);
    evidence.push(evaluateFixtureRun(run));
  }
  return {
    schema: 'ql-foundation-gate/0.1',
    deterministic: true,
    network_required: false,
    model_api_required: false,
    fixture_count: evidence.length,
    passed: evidence.every((item) => item.passed),
    fixtures: evidence
  };
}

export function createABDemo() {
  const hostFactory = () => new ScriptedHost({
    id: 'reference-host',
    revision: 'foundation-1',
    modelResponses: [
      { content: 'inspect', capabilityCalls: [{ name: 'read', args: { path: 'demo.txt' } }] },
      { content: 'classic complete', capabilityCalls: [] },
      { content: 'ql carrier', capabilityCalls: [] },
      { content: 'ql carrier', capabilityCalls: [] },
      { content: 'ql carrier', capabilityCalls: [] }
    ],
    capabilities: {
      read: async (args) => ({ ok: true, args, content: 'demo' })
    }
  });

  const qlPolicy = new ScriptedQLPolicy([
    { source: 'P0', carrier: { kind: 'model' }, difference: 'form can be established from the prompt', destination: 'P3', residueDelta: { create: [{ kind: 'form', position: 'P3', value: 'demo form', provenance: { demo: true } }] } },
    { source: 'P3', carrier: { kind: 'tool', name: 'read', args: { path: 'demo.txt' } }, difference: 'read result establishes contextual adequacy', destination: 'P4', residueDelta: { create: [{ kind: 'evaluation', position: 'P4', value: 'adequate', provenance: { demo: true } }] } },
    { source: 'P4', carrier: { kind: 'model' }, difference: 'intent is realised', destination: 'P5', determination: determination('close', { synthesis: 'ql complete' }), verdict: { status: 'close', task_success: 'true' } }
  ]);

  return {
    hostFactory,
    classicRuntime: new ClassicRuntime(),
    qlRuntime: new QLDirectCoreRuntime({ policy: qlPolicy }),
    request: {
      taskId: 'AB-DEMO',
      input: 'Run the same reference task',
      successConditions: ['return a determinate outcome'],
      maxSteps: 8
    }
  };
}

export async function runABDemo() {
  const demo = createABDemo();
  return runAB({
    ...demo,
    specRevision: 'ql-agent/0.1@f9d056c54caf094eb672f005ce3c8cbde4de0a5b',
    fixtureId: 'AB-DEMO',
    model: { id: 'scripted-model', parameters: {} },
    capabilities: ['read'],
    environment: { id: 'deterministic-fixture' },
    startState: { id: 'AB-DEMO-start' }
  });
}
