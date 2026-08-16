function residue(kind, position, value) {
  return { kind, position, value, provenance: { fixture: 'pi-baseline' } };
}

export function createPiBaselinePolicy() {
  return {
    nextAct({ circuit }) {
      switch (circuit.activePosition.id) {
        case 'P0':
          return { intent: 'Establish the matched task material.', carrier: { kind: 'model' } };
        case 'P1':
          return {
            intent: 'Inspect the matched subject.',
            carrier: { kind: 'capability', name: 'inspect', args: { subject: 'matched-runtime-baseline' } }
          };
        case 'P4':
          return { intent: 'Evaluate whether the matched subject is complete.', carrier: { kind: 'model' } };
        default:
          return null;
      }
    },

    establishDifference({ returned }) {
      return returned.raw_result;
    },

    interpret({ circuit, difference }) {
      const from = circuit.activePosition.id;
      if (from === 'P0') {
        return {
          destination: 'P1',
          rationale: 'The initiating encounter establishes material to inspect.',
          residueDelta: { create: [residue('material', 'P1', { model: difference?.content ?? null })] }
        };
      }
      if (from === 'P1') {
        return {
          destination: 'P4',
          rationale: 'The host-native return supplies current subject evidence.',
          residueDelta: { create: [residue('material', 'P1', difference)] }
        };
      }
      if (from === 'P4') {
        return {
          destination: 'P5',
          rationale: 'Current evidence and evaluation support candidate determination.',
          residueDelta: {
            create: [residue('evaluation', 'P4', {
              subject: 'matched-runtime-baseline',
              current: true,
              assessment: difference?.content ?? null
            })]
          }
        };
      }
      throw new Error(`Unexpected baseline interpretation source ${from}.`);
    },

    proposeDetermination({ circuit }) {
      const evidence = circuit.residues.filter((entry) => entry.kind === 'evaluation' && !entry.invalidated);
      return {
        synthesis: 'Matched runtime baseline verified.',
        claimed_adequacy: 'sufficient',
        claimed_subject: 'matched-runtime-baseline',
        claimed_state: 'verified',
        evidence_refs: evidence.map((entry) => entry.id),
        unresolved_refs: [],
        requested_outcome: 'close'
      };
    },

    evaluateClosure({ evaluations }) {
      const matched = evaluations.some((entry) =>
        entry.value?.subject === 'matched-runtime-baseline' && entry.value?.current === true
      );
      return matched
        ? { status: 'close', task_success: 'true', rationale: 'Current subject-matched evidence exists.' }
        : { status: 'reopen', destination: 'P4', task_success: 'false', rationale: 'Current evidence is missing.' };
    },

    createReentryDelta() {
      return {
        achieved_artifact_refs: ['artifact:pi:matched-runtime-baseline'],
        changed_assumptions: [],
        opened_questions: [],
        provenance: { fixture: 'pi-baseline', host: 'pi' }
      };
    }
  };
}
