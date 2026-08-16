function residue(kind, position, value) {
  return { kind, position, value, provenance: { fixture: 'pydantic-baseline' } };
}

export function createPydanticBaselinePolicy() {
  return {
    nextAct({ circuit }) {
      if (circuit.activePosition.id === 'P0') return { intent: 'Establish matched material.', carrier: { kind: 'model' } };
      if (circuit.activePosition.id === 'P1') return { intent: 'Inspect matched subject.', carrier: { kind: 'capability', name: 'inspect', args: { subject: 'matched-runtime-baseline' } } };
      if (circuit.activePosition.id === 'P4') return { intent: 'Evaluate matched subject.', carrier: { kind: 'model' } };
      return null;
    },
    establishDifference({ returned }) { return returned.raw_result; },
    interpret({ circuit, difference }) {
      const from = circuit.activePosition.id;
      if (from === 'P0') return { destination: 'P1', rationale: 'Material established.', residueDelta: { create: [residue('material', 'P1', { model: difference?.content ?? null })] } };
      if (from === 'P1') return { destination: 'P4', rationale: 'Host evidence returned.', residueDelta: { create: [residue('material', 'P1', difference)] } };
      if (from === 'P4') return { destination: 'P5', rationale: 'Evaluation supports determination.', residueDelta: { create: [residue('evaluation', 'P4', { subject: 'matched-runtime-baseline', current: true, assessment: difference?.content ?? null })] } };
      throw new Error(`Unexpected baseline source ${from}.`);
    },
    proposeDetermination({ circuit }) {
      const evidence = circuit.residues.filter((entry) => entry.kind === 'evaluation' && !entry.invalidated);
      return { synthesis: 'Matched runtime baseline verified.', claimed_adequacy: 'sufficient', claimed_subject: 'matched-runtime-baseline', claimed_state: 'verified', evidence_refs: evidence.map((entry) => entry.id), unresolved_refs: [], requested_outcome: 'close' };
    },
    evaluateClosure({ evaluations }) {
      const matched = evaluations.some((entry) => entry.value?.subject === 'matched-runtime-baseline' && entry.value?.current === true);
      return matched ? { status: 'close', task_success: 'true', rationale: 'Current subject-matched evidence exists.' } : { status: 'reopen', destination: 'P4', task_success: 'false', rationale: 'Current evidence is missing.' };
    },
    createReentryDelta() {
      return { achieved_artifact_refs: ['artifact:pydantic:matched-runtime-baseline'], changed_assumptions: [], opened_questions: [], provenance: { fixture: 'pydantic-baseline', host: 'pydantic-ai' } };
    }
  };
}
