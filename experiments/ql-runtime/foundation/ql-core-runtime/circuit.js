import {
  DEFAULT_LENSES,
  RESIDUE_KIND_BY_POSITION,
  QLSemanticError,
  createSuccessState,
  qlFace,
  qlPosition,
  qlRelation
} from './semantics.js';

const clone = (value) => value === undefined ? undefined : structuredClone(value);

export function createCircuit(frame, { runId, circuitId } = {}) {
  if (!frame?.id || !frame?.initiating_intent) {
    throw new QLSemanticError('Direct Core circuit requires a stable Frame id and initiating_intent.');
  }
  return {
    id: circuitId ?? `${runId ?? 'run'}:c0`,
    parent_id: null,
    depth: 0,
    face: qlFace('direct'),
    frame: clone(frame),
    active_position: qlPosition('P0'),
    residues: [],
    trajectory: [],
    closure_state: 'open',
    success_state: createSuccessState(),
    children: [],
    conjugates: [],
    residue_sequence: 0,
    transition_sequence: 0,
    determination_sequence: 0
  };
}

export function currentCircuitState(circuit) {
  return clone(circuit);
}

export function createResidue(circuit, positionId, value, provenance = {}, explicitKind) {
  const position = qlPosition(positionId);
  const kind = explicitKind ?? RESIDUE_KIND_BY_POSITION[position.id];
  if (!Object.values(RESIDUE_KIND_BY_POSITION).includes(kind)) {
    throw new QLSemanticError(`Unknown residue kind '${kind}'.`);
  }
  const residue = {
    id: `${circuit.id}:res:${circuit.residue_sequence++}`,
    kind,
    position: position.id,
    value: clone(value),
    provenance: clone(provenance)
  };
  circuit.residues.push(residue);
  return clone(residue);
}

export function projectAct(act) {
  return {
    act_id: act.id,
    phase: '0/1',
    carrier: clone(act.carrier),
    projected_context_refs: clone(act.input_residue_refs ?? [])
  };
}

export function absorbReturn({ act, rawResult, operationSuccess }) {
  return {
    id: `${act.id}:return`,
    act_id: act.id,
    phase: '1/0',
    raw_result: clone(rawResult),
    operation_success: Boolean(operationSuccess),
    difference: undefined
  };
}

export function establishDifference(returned, difference) {
  returned.difference = clone(difference);
  return clone(returned);
}

export function interpretReturn(circuit, returned, interpretation) {
  if (returned.difference === undefined) {
    throw new QLSemanticError('Returned difference must be established before QL interpretation.');
  }
  const destination = qlPosition(interpretation?.destination);
  const relation = qlRelation(circuit.active_position.id, destination.id);
  return {
    return_id: returned.id,
    from_position: circuit.active_position.id,
    destination_position: destination.id,
    relation: relation.id,
    rationale: clone(interpretation.rationale ?? null),
    witness: clone(interpretation.witness ?? {}),
    residue_delta: clone(interpretation.residue_delta ?? {}),
    lenses: [...DEFAULT_LENSES]
  };
}

export function applyTransition(circuit, interpretation) {
  const relation = qlRelation(interpretation.from_position, interpretation.destination_position);
  if (relation.id !== interpretation.relation) {
    throw new QLSemanticError('Interpretation relation does not match source and destination.');
  }
  if (circuit.active_position.id !== relation.from) {
    throw new QLSemanticError('Transition source does not match the circuit active position.');
  }

  const created = [];
  for (const item of interpretation.residue_delta?.create ?? []) {
    const residue = createResidue(
      circuit,
      item.position ?? relation.to,
      item.value,
      item.provenance,
      item.kind
    );
    created.push(residue.id);
  }

  const revised = [];
  for (const item of interpretation.residue_delta?.revise ?? []) {
    const residue = circuit.residues.find((entry) => entry.id === item.id);
    if (!residue) throw new QLSemanticError(`Cannot revise missing residue '${item.id}'.`);
    residue.value = clone(item.value);
    residue.provenance = clone(item.provenance ?? residue.provenance);
    revised.push(residue.id);
  }

  const invalidated = [];
  for (const residueId of interpretation.residue_delta?.invalidate ?? []) {
    const residue = circuit.residues.find((entry) => entry.id === residueId);
    if (!residue) throw new QLSemanticError(`Cannot invalidate missing residue '${residueId}'.`);
    residue.invalidated = true;
    invalidated.push(residue.id);
  }

  const transition = {
    id: `${circuit.id}:transition:${circuit.transition_sequence++}`,
    interpretation_id: interpretation.return_id,
    from: relation.from,
    to: relation.to,
    relation: relation.id,
    created_residue_refs: created,
    revised_residue_refs: revised,
    invalidated_residue_refs: invalidated,
    witness_state: clone(interpretation.witness ?? {})
  };
  circuit.trajectory.push(transition);
  circuit.active_position = qlPosition(transition.to);
  return clone(transition);
}

export function createDetermination(circuit, proposed = {}) {
  if (circuit.active_position.id !== 'P5') {
    throw new QLSemanticError('A QL determination can only be proposed at P5.');
  }
  const requestedOutcome = proposed.requested_outcome ?? proposed.requestedOutcome;
  if (!['close', 'reopen', 'conjugate', 'depth'].includes(requestedOutcome)) {
    throw new QLSemanticError('Determination must explicitly request close, reopen, conjugate, or depth.');
  }
  return {
    id: `${circuit.id}:determination:${circuit.determination_sequence++}`,
    circuit_id: circuit.id,
    synthesis: clone(proposed.synthesis),
    intent_ref: proposed.intent_ref ?? circuit.frame.id,
    claimed_adequacy: clone(proposed.claimed_adequacy ?? 'unknown'),
    evidence_refs: clone(proposed.evidence_refs ?? []),
    evaluation_refs: clone(proposed.evaluation_refs ?? []),
    unresolved_refs: clone(proposed.unresolved_refs ?? []),
    requested_outcome: requestedOutcome
  };
}

export function createClosureVerdict({ status, destination = null, task_success, rationale = null, retained_delta_preview = null }) {
  if (!['close', 'reopen'].includes(status)) {
    throw new QLSemanticError('Closure verdict must be close or reopen.');
  }
  if (status === 'reopen') {
    if (!['P0', 'P1', 'P2', 'P3', 'P4'].includes(destination)) {
      throw new QLSemanticError('P5 reopening must select one destination P0-P4.');
    }
    return {
      status,
      destination,
      reopening_relation: qlRelation('P5', destination).id,
      task_success: String(task_success ?? 'false'),
      rationale: clone(rationale),
      retained_delta_preview: clone(retained_delta_preview)
    };
  }
  return {
    status,
    destination: null,
    reopening_relation: null,
    task_success: String(task_success ?? 'true'),
    rationale: clone(rationale),
    retained_delta_preview: clone(retained_delta_preview)
  };
}

export function createReentryDelta(circuit, determination, values = {}) {
  if (circuit.closure_state !== 'closed') {
    throw new QLSemanticError('ReentryDelta can only be derived after positive QLClosure.');
  }
  return {
    id: `${circuit.id}:reentry-delta`,
    source_circuit: circuit.id,
    achieved_artifact_refs: clone(values.achieved_artifact_refs ?? []),
    established_material_refs: clone(values.established_material_refs ?? circuit.residues.filter((r) => r.kind === 'material' && !r.invalidated).map((r) => r.id)),
    retained_form_refs: clone(values.retained_form_refs ?? circuit.residues.filter((r) => r.kind === 'form' && !r.invalidated).map((r) => r.id)),
    changed_assumptions: clone(values.changed_assumptions ?? []),
    unresolved_refs: clone(values.unresolved_refs ?? determination.unresolved_refs ?? []),
    revised_success_conditions: clone(values.revised_success_conditions ?? circuit.frame.success_conditions ?? []),
    new_capabilities: clone(values.new_capabilities ?? []),
    discovered_risks: clone(values.discovered_risks ?? []),
    opened_questions: clone(values.opened_questions ?? []),
    provenance: clone(values.provenance ?? { circuit_id: circuit.id, determination_id: determination.id })
  };
}

export function closeCircuit(circuit, determination, verdict) {
  if (circuit.active_position.id !== 'P5' || verdict.status !== 'close') {
    throw new QLSemanticError('Positive QL closure requires a P5 close verdict.');
  }
  circuit.closure_state = 'closed';
  circuit.success_state = createSuccessState({ ...circuit.success_state, task: verdict.task_success, circuit: 'true' });
  return {
    id: `${circuit.id}:closure`,
    circuit_id: circuit.id,
    determination_ref: determination.id,
    frame_ref: circuit.frame.id,
    evaluation_refs: circuit.residues.filter((r) => r.kind === 'evaluation' && !r.invalidated).map((r) => r.id),
    success_state: clone(circuit.success_state),
    closed_at_position: 'P5',
    // ReentryDelta is derived only after positive closure. The closure record
    // is created before the delta exists; the slot is filled by the caller once
    // the delta has been produced, never before the circuit closed.
    reentry_delta_ref: null
  };
}

export function createReentry(circuit, delta, closure) {
  if (circuit.closure_state !== 'closed' || !closure?.id) {
    throw new QLSemanticError('QL re-entry requires a prior positive QLClosure.');
  }
  return {
    prior_circuit: circuit.id,
    closure_ref: closure.id,
    delta_ref: delta.id,
    renewed_frame: {
      ...clone(circuit.frame),
      id: `${circuit.frame.id}+`,
      inherited_delta: delta.id,
      success_conditions: clone(delta.revised_success_conditions)
    }
  };
}