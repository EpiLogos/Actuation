const POSITION_COUNTS = Object.freeze({ P0: 14, P1: 18, P2: 18, P3: 18, P4: 18, P5: 14 });
const POSITIONS = Object.freeze(Object.keys(POSITION_COUNTS));

const SCENARIOS = Object.freeze({
  P0: [
    ['Clarify the operative scope before implementation', 'human'],
    ['Revise success conditions after a newly discovered constraint', 'artifact'],
    ['Separate the initiating intent from an assumed implementation choice', 'model'],
    ['Re-establish the frame after evidence changes the problem definition', 'environment']
  ],
  P1: [
    ['Read the failing parser implementation to acquire relevant evidence', 'tool'],
    ['Inspect current test output to establish observed behaviour', 'tool'],
    ['Compare API documentation with the response actually returned', 'artifact'],
    ['Ask for the missing configuration value required to understand the state', 'human']
  ],
  P2: [
    ['Apply the parser fix to the working implementation', 'tool'],
    ['Run the migration against a disposable fixture', 'tool'],
    ['Execute the test suite after changing the implementation', 'environment'],
    ['Transform the input into the canonical schema required downstream', 'model']
  ],
  P3: [
    ['Model the state transition that explains the observed failure', 'model'],
    ['Refactor the interface around the invariant disclosed by the evidence', 'tool'],
    ['Infer a schema that captures the returned data without losing distinctions', 'model'],
    ['Revise the implementation form after a contradiction is found', 'artifact']
  ],
  P4: [
    ['Verify the completion claim against current evidence', 'tool'],
    ['Evaluate compatibility against the whole migration goal', 'artifact'],
    ['Check whether the implementation satisfies the stated acceptance criteria', 'environment'],
    ['Compare the realised behaviour with the initiating success conditions', 'human']
  ],
  P5: [
    ['Determine whether the work is sufficiently complete to close the circuit', 'model'],
    ['Reject the candidate determination because required evidence is missing', 'artifact'],
    ['Accept the realised result while retaining unresolved follow-up questions', 'human'],
    ['Reconsider the candidate after whole-relative evaluation exposes a defect', 'environment']
  ]
});

const DESTINATIONS = Object.freeze({
  P0: ['P0', 'P1', 'P3', 'P4'],
  P1: ['P1', 'P3', 'P4', 'P2'],
  P2: ['P2', 'P3', 'P1', 'P4'],
  P3: ['P3', 'P1', 'P2', 'P4'],
  P4: ['P4', 'P1', 'P3', 'P5'],
  P5: ['P1', 'P2', 'P3', 'P4', 'P0', 'P5']
});

function relation(from, to) {
  return `R${from.slice(1)}${to.slice(1)}`;
}

function makeReference(position, indexWithinPosition, globalIndex) {
  const p5Reopen = position === 'P5' && indexWithinPosition < 10;
  const destinations = DESTINATIONS[position];
  const destination = p5Reopen
    ? ['P1', 'P2', 'P3', 'P4', 'P0'][indexWithinPosition % 5]
    : destinations[indexWithinPosition % destinations.length];
  const ambiguous = globalIndex < 12;
  return {
    primary_position: position,
    secondary_positions: ambiguous ? [POSITIONS[(POSITIONS.indexOf(position) + 1) % POSITIONS.length]] : [],
    relation: relation(position, destination),
    face: 'direct',
    closure: p5Reopen ? 'reopen' : position === 'P5' && destination === 'P5' ? 'candidate' : 'not_applicable',
    reopening_destination: p5Reopen ? destination : null
  };
}

function perturb(reference, globalIndex, mode) {
  const shifted = POSITIONS[(POSITIONS.indexOf(reference.primary_position) + 1) % POSITIONS.length];
  const primary = mode === 'claimed' && globalIndex % 9 === 0 ? shifted : reference.primary_position;
  const relationValue = mode === 'claimed' && globalIndex % 11 === 0
    ? relation(reference.primary_position, reference.primary_position)
    : reference.relation;
  return {
    witness_kind: 'synthetic-fixture',
    primary_position: primary,
    secondary_positions: mode === 'retrospective' && globalIndex < 6 ? [shifted] : [],
    relation: relationValue,
    face: reference.face,
    closure: reference.closure,
    reopening_destination: reference.reopening_destination
  };
}

const records = [];
let globalIndex = 0;
for (const position of POSITIONS) {
  for (let indexWithinPosition = 0; indexWithinPosition < POSITION_COUNTS[position]; indexWithinPosition += 1) {
    const scenario = SCENARIOS[position][indexWithinPosition % SCENARIOS[position].length];
    const reference = makeReference(position, indexWithinPosition, globalIndex);
    const categories = [];
    if (globalIndex < 12) categories.push('ambiguous');
    if (position === 'P5' && indexWithinPosition < 10) categories.push('p5-reopening');
    if (globalIndex >= 70 && globalIndex < 80) categories.push('cross-carrier-same-function');
    if (globalIndex >= 80 && globalIndex < 90) categories.push('same-carrier-cross-function');

    records.push({
      id: `QLT-${String(globalIndex + 1).padStart(3, '0')}`,
      title: scenario[0],
      act: {
        intent: scenario[0],
        carrier: { kind: scenario[1], name: `${scenario[1]}-${indexWithinPosition % 4}` }
      },
      benchmark_reference: {
        ...reference,
        provenance: {
          kind: 'deterministic-benchmark',
          source: 'TYPING-CORPUS-REVIEW-CLARIFICATION-08-14-2026.md',
          generator_version: 'ql-typing-benchmark/0.2'
        }
      },
      model_claimed: perturb(reference, globalIndex, 'claimed'),
      retrospective: perturb(reference, globalIndex, 'retrospective'),
      human_witness: null,
      structural_facts: {
        carrier: scenario[1],
        source_position: position,
        destination_position: reference.reopening_destination ?? reference.relation.slice(-1).replace(/^/, 'P'),
        side_effect: ['tool', 'environment'].includes(scenario[1]) && position === 'P2',
        fixture_index: globalIndex + 1
      },
      ambiguity_notes: globalIndex < 12
        ? 'Intentionally multi-functional case; the benchmark keeps a secondary admissible position visible.'
        : '',
      categories
    });
    globalIndex += 1;
  }
}

export const corpus = Object.freeze(records);

export function corpusStats(recordsInput = corpus) {
  const byPosition = Object.fromEntries(
    POSITIONS.map((position) => [position, recordsInput.filter((record) => record.benchmark_reference.primary_position === position).length])
  );
  const category = (name) => recordsInput.filter((record) => record.categories.includes(name)).length;
  return {
    count: recordsInput.length,
    by_position: byPosition,
    ambiguous: category('ambiguous'),
    p5_reopening: category('p5-reopening'),
    cross_carrier_same_function: category('cross-carrier-same-function'),
    same_carrier_cross_function: category('same-carrier-cross-function'),
    human_witnesses: recordsInput.filter((record) => record.human_witness).length,
    stable_ids: new Set(recordsInput.map((record) => record.id)).size === recordsInput.length,
    benchmark_provenance: recordsInput.every((record) => record.benchmark_reference.provenance?.kind === 'deterministic-benchmark')
  };
}

const exact = (a, b) => a === b ? 1 : 0;
const avg = (values) => values.length ? values.reduce((sum, value) => sum + value, 0) / values.length : null;

function compare(recordsInput, leftKey, rightKey) {
  return {
    position_exact_agreement: avg(recordsInput.map((record) => exact(record[leftKey].primary_position, record[rightKey].primary_position))),
    relation_exact_agreement: avg(recordsInput.map((record) => exact(record[leftKey].relation, record[rightKey].relation))),
    face_agreement: avg(recordsInput.map((record) => exact(record[leftKey].face, record[rightKey].face))),
    primary_vs_secondary_position_agreement: avg(recordsInput.map((record) =>
      record[leftKey].primary_position === record[rightKey].primary_position ||
      (record[rightKey].secondary_positions ?? []).includes(record[leftKey].primary_position) ? 1 : 0
    )),
    closure_reopen_agreement: avg(recordsInput.map((record) => exact(record[leftKey].closure ?? 'not_applicable', record[rightKey].closure ?? 'not_applicable'))),
    reopening_destination_agreement: avg(
      recordsInput
        .filter((record) => record[leftKey].reopening_destination || record[rightKey].reopening_destination)
        .map((record) => exact(record[leftKey].reopening_destination ?? null, record[rightKey].reopening_destination ?? null))
    )
  };
}

export function agreementMetrics(recordsInput = corpus) {
  const humanRecords = recordsInput.filter((record) => record.human_witness);
  return {
    claimed_benchmark: compare(recordsInput, 'model_claimed', 'benchmark_reference'),
    retrospective_benchmark: compare(recordsInput, 'retrospective', 'benchmark_reference'),
    claimed_retrospective: compare(recordsInput, 'model_claimed', 'retrospective'),
    claimed_human: humanRecords.length ? compare(humanRecords, 'model_claimed', 'human_witness') : null,
    retrospective_human: humanRecords.length ? compare(humanRecords, 'retrospective', 'human_witness') : null,
    human_witness_count: humanRecords.length
  };
}
