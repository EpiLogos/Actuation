import {
  RUN_STATUS,
  isAbortRequested,
  normalizeRunRequest,
  dispatchHostCarrier
} from '../runtime-contract/index.js';

import {
  QL_SPEC,
  QL_SCHEMA_VERSION,
  DEFAULT_LENSES,
  RESIDUE_KIND_BY_POSITION,
  QLSemanticError,
  qlPosition,
  qlFace,
  qlRelation,
  createSuccessState
} from './semantics.js';

export {
  QL_SPEC,
  QL_SCHEMA_VERSION,
  DEFAULT_LENSES,
  FACES,
  POSITIONS,
  RESIDUE_KIND_BY_POSITION,
  QLSemanticError,
  qlPosition,
  qlFace,
  qlRelation,
  createSuccessState
} from './semantics.js';

export class UnsupportedQLOperatorError extends Error {
  constructor(operator) {
    super(`${operator} is an explicit post-Foundation-Freeze operator seam and is not implemented by Direct Core.`);
    this.name = 'UnsupportedQLOperatorError';
    this.operator = operator;
  }
}

function clone(value) {
  return value === undefined ? undefined : structuredClone(value);
}

function safeError(error) {
  return error instanceof Error ? error.message : String(error);
}

function normalizeCarrier(carrier = {}) {
  const kind = carrier.kind ?? 'internal_control';
  return {
    kind,
    name: carrier.name ?? kind,
    args: clone(carrier.args ?? {}),
    input: clone(carrier.input ?? null)
  };
}

export class QLDirectCoreRuntime {
  id = 'ql-core';
  version = '0.1.0-foundation';

  constructor({ policy } = {}) {
    if (!policy) {
      throw new QLSemanticError('QLDirectCoreRuntime requires an act/interpretation policy.');
    }
    this.policy = policy;
  }

  openConjugate() {
    throw new UnsupportedQLOperatorError('conjugation');
  }

  openChild() {
    throw new UnsupportedQLOperatorError('recursive depth');
  }

  startCircuit(frame, { runId, circuitId } = {}) {
    const id = circuitId ?? `${runId}:c0`;
    const circuit = {
      id,
      parentId: null,
      depth: 0,
      face: qlFace('direct'),
      frame: clone(frame),
      activePosition: qlPosition('P0'),
      residues: [],
      trajectory: [],
      closureState: 'open',
      successState: createSuccessState(),
      children: [],
      conjugates: [],
      sequence: 0,
      residueSequence: 0,
      actSequence: 0,
      transitionSequence: 0,
      determinationSequence: 0
    };
    return circuit;
  }

  currentState(circuit) {
    return clone(circuit);
  }

  #event(circuit, runId, eventType, { ql = {}, payload = {}, witness = {} } = {}) {
    return {
      spec: QL_SPEC,
      schema_version: QL_SCHEMA_VERSION,
      event_id: `${circuit.id}:evt:${circuit.sequence}`,
      event_type: eventType,
      run_id: runId,
      circuit_id: circuit.id,
      parent_circuit_id: circuit.parentId,
      sequence: circuit.sequence++,
      face: circuit.face,
      ql: clone(ql),
      payload: clone(payload),
      witness: clone(witness)
    };
  }

  #emit(observer, circuit, runId, eventType, data) {
    const event = this.#event(circuit, runId, eventType, data);
    observer.emit({ channel: 'runtime-semantic', ...event });
    return event;
  }

  #emitControl(observer, runId, sequence, eventType, payload = {}) {
    observer.emit({
      channel: 'runtime',
      event_id: `${runId}:ql-control:${sequence}`,
      event_type: eventType,
      run_id: runId,
      sequence,
      runtime: this.id,
      payload: clone(payload)
    });
  }

  createResidue(circuit, positionId, value, provenance, explicitKind) {
    const position = qlPosition(positionId);
    const kind = explicitKind ?? RESIDUE_KIND_BY_POSITION[position.id];
    const allowed = new Set(Object.values(RESIDUE_KIND_BY_POSITION));
    if (!allowed.has(kind)) {
      throw new QLSemanticError(`Unknown residue kind '${kind}'.`);
    }
    const residue = {
      id: `${circuit.id}:res:${circuit.residueSequence++}`,
      kind,
      position: position.id,
      value: clone(value),
      provenance: clone(provenance ?? {})
    };
    circuit.residues.push(residue);
    return residue;
  }

  project(act) {
    return {
      act_id: act.id,
      phase: '0/1',
      carrier: clone(act.carrier),
      projected_context_refs: clone(act.inputResidueRefs ?? [])
    };
  }

  absorb({ act, rawResult, operationSuccess }) {
    return {
      id: `${act.id}:return`,
      act_id: act.id,
      phase: '1/0',
      raw_result: clone(rawResult),
      operation_success: Boolean(operationSuccess),
      difference: undefined
    };
  }

  async interpret(circuit, returned, act, request) {
    const difference = typeof this.policy.establishDifference === 'function'
      ? await this.policy.establishDifference({
          circuit: this.currentState(circuit),
          returned: clone(returned),
          act: clone(act),
          request
        })
      : clone(returned.raw_result);

    returned.difference = clone(difference);

    const interpretation = await this.policy.interpret({
      circuit: this.currentState(circuit),
      difference: clone(difference),
      returned: clone(returned),
      act: clone(act),
      request
    });

    if (!interpretation?.destination) {
      throw new QLSemanticError('QL interpretation must choose a destination after the return difference is established.');
    }

    const destination = qlPosition(interpretation.destination);
    const relation = qlRelation(circuit.activePosition.id, destination.id);

    return {
      return_id: returned.id,
      from_position: circuit.activePosition.id,
      destination_position: destination.id,
      relation: relation.id,
      rationale: clone(interpretation.rationale ?? null),
      witness: clone(interpretation.witness ?? {}),
      residue_delta: clone(interpretation.residueDelta ?? {})
    };
  }

  applyTransition(circuit, interpretation, observer, runId) {
    const relation = qlRelation(interpretation.from_position, interpretation.destination_position);
    if (relation.id !== interpretation.relation) {
      throw new QLSemanticError('Interpretation relation does not match its source/destination positions.');
    }

    const created = [];
    for (const item of interpretation.residue_delta?.create ?? []) {
      const residue = this.createResidue(
        circuit,
        item.position ?? interpretation.destination_position,
        item.value,
        item.provenance,
        item.kind
      );
      created.push(residue.id);
      this.#emit(observer, circuit, runId, 'residue_created', {
        ql: { to: residue.position, relation: relation.id, lens: [...DEFAULT_LENSES] },
        payload: { residue }
      });
    }

    const revised = [];
    for (const item of interpretation.residue_delta?.revise ?? []) {
      const residue = circuit.residues.find((entry) => entry.id === item.id);
      if (!residue) {
        throw new QLSemanticError(`Cannot revise missing residue '${item.id}'.`);
      }
      residue.value = clone(item.value);
      residue.provenance = clone(item.provenance ?? residue.provenance);
      revised.push(residue.id);
      this.#emit(observer, circuit, runId, 'residue_revised', {
        ql: { from: relation.from, to: relation.to, relation: relation.id, lens: [...DEFAULT_LENSES] },
        payload: { residue: clone(residue) }
      });
    }

    const invalidated = [];
    for (const residueId of interpretation.residue_delta?.invalidate ?? []) {
      const residue = circuit.residues.find((entry) => entry.id === residueId);
      if (!residue) {
        throw new QLSemanticError(`Cannot invalidate missing residue '${residueId}'.`);
      }
      residue.invalidated = true;
      invalidated.push(residue.id);
      this.#emit(observer, circuit, runId, 'residue_invalidated', {
        ql: { from: relation.from, to: relation.to, relation: relation.id, lens: [...DEFAULT_LENSES] },
        payload: { residue_id: residue.id }
      });
    }

    const transition = {
      id: `${circuit.id}:transition:${circuit.transitionSequence++}`,
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
    circuit.activePosition = qlPosition(transition.to);

    this.#emit(observer, circuit, runId, 'transition', {
      ql: {
        from: transition.from,
        to: transition.to,
        relation: transition.relation,
        lens: [...DEFAULT_LENSES]
      },
      payload: { transition },
      witness: transition.witness_state
    });

    return transition;
  }

  async proposeDetermination(circuit, request) {
    if (circuit.activePosition.id !== 'P5') {
      return null;
    }
    if (typeof this.policy.proposeDetermination !== 'function') {
      return null;
    }
    const proposed = await this.policy.proposeDetermination({
      circuit: this.currentState(circuit),
      request
    });
    if (!proposed) {
      return null;
    }
    const requestedOutcome = proposed.requested_outcome ?? proposed.requestedOutcome;
    if (!['close', 'reopen', 'conjugate', 'depth'].includes(requestedOutcome)) {
      throw new QLSemanticError('Determination must request close, reopen, conjugate, or depth.');
    }
    return {
      id: `${circuit.id}:determination:${circuit.determinationSequence++}`,
      circuit_id: circuit.id,
      synthesis: clone(proposed.synthesis),
      intent_ref: proposed.intent_ref ?? circuit.frame.id,
      claimed_adequacy: clone(proposed.claimed_adequacy ?? 'unknown'),
      claimed_subject: clone(proposed.claimed_subject ?? null),
      claimed_state: clone(proposed.claimed_state ?? null),
      evidence_refs: clone(proposed.evidence_refs ?? []),
      evaluation_refs: clone(proposed.evaluation_refs ?? []),
      unresolved_refs: clone(proposed.unresolved_refs ?? []),
      requested_outcome: requestedOutcome
    };
  }

  async evaluateClosure(circuit, determination, request) {
    if (typeof this.policy.evaluateClosure !== 'function') {
      throw new QLSemanticError('Direct Core policy must explicitly evaluate P5 closure.');
    }
    const rawVerdict = await this.policy.evaluateClosure({
      circuit: this.currentState(circuit),
      determination: clone(determination),
      frame: clone(circuit.frame),
      evaluations: circuit.residues.filter((residue) => residue.kind === 'evaluation' && !residue.invalidated),
      request
    });
    if (!rawVerdict || !['close', 'reopen'].includes(rawVerdict.status)) {
      throw new QLSemanticError('Closure verdict must be close or reopen.');
    }

    if (rawVerdict.status === 'reopen') {
      const destination = rawVerdict.destination ?? rawVerdict.reopening_destination;
      if (!['P0', 'P1', 'P2', 'P3', 'P4'].includes(destination)) {
        throw new QLSemanticError('P5 reopening must select one distinct destination P0–P4.');
      }
      return {
        status: 'reopen',
        reopening_relation: qlRelation('P5', destination).id,
        destination,
        task_success: String(rawVerdict.task_success ?? 'false'),
        rationale: clone(rawVerdict.rationale ?? null),
        retained_delta_preview: clone(rawVerdict.retained_delta_preview ?? null)
      };
    }

    return {
      status: 'close',
      reopening_relation: null,
      destination: null,
      task_success: String(rawVerdict.task_success ?? 'true'),
      rationale: clone(rawVerdict.rationale ?? null),
      retained_delta_preview: clone(rawVerdict.retained_delta_preview ?? null)
    };
  }

  close(circuit, determination, verdict) {
    circuit.closureState = 'closed';
    circuit.successState = createSuccessState({
      ...circuit.successState,
      task: verdict.task_success,
      circuit: 'true'
    });
    return {
      id: `${circuit.id}:closure`,
      circuit_id: circuit.id,
      determination_ref: determination.id,
      frame_ref: circuit.frame.id,
      evaluation_refs: circuit.residues
        .filter((residue) => residue.kind === 'evaluation' && !residue.invalidated)
        .map((residue) => residue.id),
      success_state: clone(circuit.successState),
      closed_at_position: 'P5',
      // ReentryDelta is derived only after positive closure. The closure is
      // created before the delta exists; the caller fills this slot after
      // derivation, never before the circuit closed.
      reentry_delta_ref: null
    };
  }

  async reenter(circuit, determination, verdict, request, closure) {
    const policyDelta = typeof this.policy.createReentryDelta === 'function'
      ? await this.policy.createReentryDelta({
          circuit: this.currentState(circuit),
          determination: clone(determination),
          verdict: clone(verdict),
          request
        })
      : {};

    const delta = {
      id: `${circuit.id}:reentry-delta`,
      source_circuit: circuit.id,
      achieved_artifact_refs: clone(policyDelta.achieved_artifact_refs ?? []),
      established_material_refs: clone(policyDelta.established_material_refs ?? circuit.residues
        .filter((residue) => residue.kind === 'material' && !residue.invalidated)
        .map((residue) => residue.id)),
      retained_form_refs: clone(policyDelta.retained_form_refs ?? circuit.residues
        .filter((residue) => residue.kind === 'form' && !residue.invalidated)
        .map((residue) => residue.id)),
      changed_assumptions: clone(policyDelta.changed_assumptions ?? []),
      unresolved_refs: clone(policyDelta.unresolved_refs ?? determination.unresolved_refs ?? []),
      revised_success_conditions: clone(policyDelta.revised_success_conditions ?? request.successConditions),
      new_capabilities: clone(policyDelta.new_capabilities ?? []),
      discovered_risks: clone(policyDelta.discovered_risks ?? []),
      opened_questions: clone(policyDelta.opened_questions ?? []),
      provenance: clone(policyDelta.provenance ?? {
        circuit_id: circuit.id,
        determination_id: determination.id
      })
    };

    const renewedFrame = {
      ...clone(circuit.frame),
      id: `${circuit.frame.id}+`,
      inherited_delta: delta.id,
      success_conditions: clone(delta.revised_success_conditions)
    };

    return {
      delta,
      reentry: {
        prior_circuit: circuit.id,
        closure_ref: closure?.id ?? null,
        delta_ref: delta.id,
        renewed_frame: renewedFrame
      }
    };
  }

  async #executeCarrier(act, host, request, signal) {
    const carrier = act.carrier;
    if (carrier.kind === 'child_circuit') {
      return this.openChild({ act, request });
    }
    // Mechanically shared host/carrier boundary from the QL-free contract.
    // The QL act rides along as an opaque payload; interpretation of the
    // return remains QL loop logic, not carrier mechanics.
    const dispatchCarrier = carrier.kind === 'human'
      ? { ...carrier, inputKind: carrier.inputKind ?? 'ql_act' }
      : carrier;
    return dispatchHostCarrier({
      host,
      carrier: dispatchCarrier,
      request,
      signal,
      payload: { qlAct: clone(act) }
    });
  }

  async run(inputRequest, host, observer, signal) {
    const request = normalizeRunRequest(inputRequest);
    const runId = request.runId ?? `run:${request.taskId}:ql-core`;
    const frame = {
      id: `${runId}:frame:0`,
      initiating_intent: clone(request.input),
      operative_scope: clone(request.operativeScope ?? null),
      constraints: clone(request.constraints ?? []),
      available_capabilities: clone(request.capabilities ?? []),
      success_conditions: clone(request.successConditions),
      inherited_delta: clone(request.inheritedDelta ?? null),
      provenance: clone(request.provenance ?? { task_id: request.taskId })
    };
    const circuit = this.startCircuit(frame, { runId });
    let controlSequence = 0;
    let steps = 0;

    this.#emitControl(observer, runId, controlSequence++, 'run_started', { task_id: request.taskId });
    this.#emit(observer, circuit, runId, 'circuit_started', {
      ql: { to: 'P0', lens: [...DEFAULT_LENSES] },
      payload: { circuit_id: circuit.id, depth: circuit.depth }
    });

    const frameResidue = this.createResidue(
      circuit,
      'P0',
      frame,
      frame.provenance,
      'frame'
    );
    this.#emit(observer, circuit, runId, 'frame_established', {
      ql: { to: 'P0', lens: [...DEFAULT_LENSES] },
      payload: { frame, residue_ref: frameResidue.id }
    });
    this.#emit(observer, circuit, runId, 'residue_created', {
      ql: { to: 'P0', lens: [...DEFAULT_LENSES] },
      payload: { residue: frameResidue }
    });

    try {
      while (circuit.closureState === 'open' && steps < request.maxSteps) {
        if (isAbortRequested(signal)) {
          this.#emitControl(observer, runId, controlSequence++, 'run_cancelled', { steps });
          return {
            status: RUN_STATUS.CANCELLED,
            runtime: this.id,
            runtimeVersion: this.version,
            runId,
            circuit: this.currentState(circuit)
          };
        }

        const policyAct = await this.policy.nextAct({
          circuit: this.currentState(circuit),
          request,
          host
        });

        if (!policyAct) {
          this.#emitControl(observer, runId, controlSequence++, 'run_exhausted', {
            reason: 'policy returned no next act before positive closure',
            steps
          });
          return {
            status: RUN_STATUS.EXHAUSTED,
            runtime: this.id,
            runtimeVersion: this.version,
            runId,
            circuit: this.currentState(circuit)
          };
        }

        const sourcePosition = qlPosition(policyAct.sourcePosition ?? circuit.activePosition.id);
        if (sourcePosition.id !== circuit.activePosition.id) {
          throw new QLSemanticError(
            `Act source ${sourcePosition.id} does not match active circuit position ${circuit.activePosition.id}.`
          );
        }

        const act = {
          id: `${circuit.id}:act:${circuit.actSequence++}`,
          run_id: runId,
          circuit_id: circuit.id,
          face: circuit.face,
          source_position: sourcePosition.id,
          intent: clone(policyAct.intent ?? null),
          carrier: normalizeCarrier(policyAct.carrier),
          inputResidueRefs: clone(policyAct.inputResidueRefs ?? []),
          claimed_position: clone(policyAct.claimedPosition ?? sourcePosition.id),
          claimed_relation: clone(policyAct.claimedRelation ?? null),
          metadata: clone(policyAct.metadata ?? {})
        };

        this.#emit(observer, circuit, runId, 'act_created', {
          ql: { from: sourcePosition.id, lens: [...DEFAULT_LENSES] },
          payload: { act },
          witness: {
            claimed_position: act.claimed_position,
            claimed_relation: act.claimed_relation,
            structural_facts: { carrier: clone(act.carrier) }
          }
        });

        const projection = this.project(act);
        this.#emit(observer, circuit, runId, 'projection', {
          ql: { from: sourcePosition.id, projection: '0/1', lens: [...DEFAULT_LENSES] },
          payload: { projection }
        });

        let rawResult;
        let operationSuccess = true;
        try {
          rawResult = await this.#executeCarrier(act, host, request, signal);
        } catch (error) {
          if (isAbortRequested(signal) || error?.name === 'AbortError') {
            this.#emitControl(observer, runId, controlSequence++, 'run_cancelled', { steps });
            return {
              status: RUN_STATUS.CANCELLED,
              runtime: this.id,
              runtimeVersion: this.version,
              runId,
              circuit: this.currentState(circuit)
            };
          }
          operationSuccess = false;
          rawResult = { error: safeError(error) };
        }

        const returned = this.absorb({ act, rawResult, operationSuccess });
        const interpretation = await this.interpret(circuit, returned, act, request);

        this.#emit(observer, circuit, runId, 'return_received', {
          ql: { from: sourcePosition.id, return: '1/0', lens: [...DEFAULT_LENSES] },
          payload: { returned: clone(returned) }
        });

        this.#emit(observer, circuit, runId, 'return_interpreted', {
          ql: {
            from: interpretation.from_position,
            to: interpretation.destination_position,
            relation: interpretation.relation,
            return: '1/0',
            lens: [...DEFAULT_LENSES]
          },
          payload: {
            difference: clone(returned.difference),
            interpretation: clone(interpretation)
          },
          witness: clone(interpretation.witness ?? {})
        });

        this.applyTransition(circuit, interpretation, observer, runId);
        steps += 1;

        if (circuit.activePosition.id !== 'P5') {
          continue;
        }

        const determination = await this.proposeDetermination(circuit, request);
        if (!determination) {
          continue;
        }

        const determinationResidue = this.createResidue(
          circuit,
          'P5',
          determination,
          { act_id: act.id, return_id: returned.id },
          'determination'
        );
        this.#emit(observer, circuit, runId, 'residue_created', {
          ql: { to: 'P5', lens: [...DEFAULT_LENSES] },
          payload: { residue: determinationResidue }
        });
        this.#emit(observer, circuit, runId, 'determination_proposed', {
          ql: { to: 'P5', lens: [...DEFAULT_LENSES] },
          payload: { determination }
        });

        const verdict = await this.evaluateClosure(circuit, determination, request);
        this.#emit(observer, circuit, runId, 'closure_evaluated', {
          ql: { from: 'P5', to: verdict.destination ?? 'P5', relation: verdict.reopening_relation, lens: [...DEFAULT_LENSES] },
          payload: { verdict }
        });

        if (verdict.status === 'reopen') {
          const reopeningInterpretation = {
            return_id: determination.id,
            from_position: 'P5',
            destination_position: verdict.destination,
            relation: verdict.reopening_relation,
            rationale: clone(verdict.rationale),
            witness: { structural_facts: { closure_status: 'reopen' } },
            residue_delta: {}
          };
          this.#emit(observer, circuit, runId, 'circuit_reopened', {
            ql: {
              from: 'P5',
              to: verdict.destination,
              relation: verdict.reopening_relation,
              lens: [...DEFAULT_LENSES]
            },
            payload: { determination_ref: determination.id, verdict }
          });
          this.applyTransition(circuit, reopeningInterpretation, observer, runId);
          continue;
        }

        // Semantic order: positive ClosureVerdict -> QLClosure closes the
        // circuit -> only then derive ReentryDelta -> only then construct and
        // emit renewed P0+ / QLReentry. No re-entry material exists before the
        // circuit is closed.
        const closure = this.close(circuit, determination, verdict);
        this.#emit(observer, circuit, runId, 'circuit_closed', {
          ql: { from: 'P5', to: 'P5', relation: 'R55', lens: [...DEFAULT_LENSES] },
          payload: { closure }
        });

        const { delta, reentry } = await this.reenter(circuit, determination, verdict, request, closure);
        closure.reentry_delta_ref = delta.id;

        this.#emit(observer, circuit, runId, 'reentry_created', {
          ql: { from: 'P5', to: 'P0', relation: 'R50', lens: [...DEFAULT_LENSES] },
          payload: { delta, reentry }
        });
        this.#emitControl(observer, runId, controlSequence++, 'run_completed', {
          circuit_id: circuit.id,
          closure_id: closure.id,
          reentry_delta_id: delta.id
        });

        return {
          status: RUN_STATUS.COMPLETED,
          runtime: this.id,
          runtimeVersion: this.version,
          runId,
          outcome: clone(determination.synthesis),
          circuit: this.currentState(circuit),
          determination,
          closure,
          reentryDelta: delta,
          reentry
        };
      }

      this.#emitControl(observer, runId, controlSequence++, 'run_exhausted', {
        max_steps: request.maxSteps,
        steps
      });
      return {
        status: RUN_STATUS.EXHAUSTED,
        runtime: this.id,
        runtimeVersion: this.version,
        runId,
        circuit: this.currentState(circuit)
      };
    } catch (error) {
      this.#emitControl(observer, runId, controlSequence++, 'run_failed', {
        steps,
        error: safeError(error)
      });
      return {
        status: RUN_STATUS.FAILED,
        runtime: this.id,
        runtimeVersion: this.version,
        runId,
        error: safeError(error),
        circuit: this.currentState(circuit)
      };
    }
  }
}
