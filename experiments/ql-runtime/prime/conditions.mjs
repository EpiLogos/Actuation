export const PRIME_CONDITIONS = Object.freeze({
  'prime-native': Object.freeze({
    code: 'P0',
    relational: false,
    maxDepth: 1,
    returnContract: false,
    recursive: false,
    continual: false,
    description: 'Prime RLM control: native recursive harness without QL/MEF faculty.'
  }),
  'prime-relational': Object.freeze({
    code: 'P2',
    relational: true,
    maxDepth: 1,
    returnContract: false,
    recursive: true,
    continual: false,
    description: 'Root and inherited child Agencies can use the QL/MEF/Wiki relational faculty.'
  }),
  'prime-relational-return': Object.freeze({
    code: 'P3',
    relational: true,
    maxDepth: 1,
    returnContract: true,
    recursive: true,
    continual: false,
    description: 'Relational faculty plus explicit structured returned-difference/reconstitution evidence.'
  }),
  'prime-recursive-field': Object.freeze({
    code: 'P4',
    relational: true,
    maxDepth: 2,
    returnContract: true,
    recursive: true,
    continual: false,
    description: 'Recursive relational Agency: child loci may differentiate further when the live Prime runtime admits it.'
  }),
  'prime-continual': Object.freeze({
    code: 'P5',
    relational: true,
    maxDepth: 2,
    returnContract: true,
    recursive: true,
    continual: true,
    description: 'Recursive relational Agency followed by one explicit evidence-backed Continual Harness refinement.'
  })
});

export function getPrimeCondition(id) {
  const value = PRIME_CONDITIONS[id];
  if (!value) throw new Error(`Unknown Prime condition '${id}'.`);
  return value;
}

export function conditionPrompt(condition, task) {
  const lines = [
    `You are the root Agency for Actuation experiment ${condition.code}.`,
    'Work directly on the supplied task and leave the workspace in the requested state.',
    'Prime child agents are differentiated acting loci, not role-play labels. Create them only when the task genuinely benefits from differentiated work.',
  ];

  if (condition.relational) {
    lines.push(
      'A Python-backed ql_relational faculty is available to you and is inherited by Prime child agents.',
      'Use QL/MEF/Wiki operations where they disclose an operationally useful relation. Do not decorate prose with QL names in place of actual operations.',
      'When you create a child, tell it to situate its bounded task in relation to the parent task, use the same relational faculty where useful, preserve source/evidence provenance, and return material difference rather than a generic summary.'
    );
  }

  if (condition.returnContract) {
    lines.push(
      'When a child return materially affects the parent determination, preserve it using the Actuation Prime Return envelope. The ql_relational.return_envelope(...) helper can construct it.',
      'Reconstitution must retain returned difference and unresolved relations before synthesis; do not flatten conflicting child returns merely to reach agreement.'
    );
  }

  if (condition.code === 'P4') {
    lines.push(
      'Descendant recursion is part of this condition: if a child discovers a genuinely subordinate unresolved relation and the current Prime runtime admits another depth, it may create a child of its own. Do not manufacture grandchildren to satisfy the experiment.'
    );
  }

  if (condition.continual) {
    lines.push(
      'This run is authorised for one explicit Continual Harness refinement after task execution. Do not call /refine yourself; the experiment driver performs and records the single refinement from the completed trajectory.'
    );
  }

  lines.push(
    '',
    'TASK',
    task.prompt,
    '',
    'SUCCESS CONDITIONS',
    ...task.successConditions.map((item) => `- ${item}`)
  );
  return lines.join('\n');
}
