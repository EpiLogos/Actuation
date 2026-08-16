import { DeepQLOperatorSession } from '../../deep-ql/operator-session.js';
import { buildDModulationFrame } from '../../deep-ql/formal/pairing-grammar.js';

const POSITIONS = ['P0', 'P1', 'P2', 'P3', 'P4', 'P5'];
const RESIDUE_KIND = { P0: 'frame', P1: 'material', P2: 'effect', P3: 'form', P4: 'evaluation', P5: 'determination' };
const clone = (value) => value === undefined ? undefined : structuredClone(value);

const POSITION_GUIDE = Object.freeze({
  P0: 'ground / initiating intent / operative frame',
  P1: 'material / evidence / givens',
  P2: 'effect / operation / transformation',
  P3: 'form / pattern / model / implementation',
  P4: 'whole-relative evaluation / context / adequacy',
  P5: 'candidate determination / synthesis / realised intent'
});

function compactCircuit(circuit) {
  return {
    id: circuit.id,
    depth: circuit.depth,
    face: circuit.face,
    active_position: circuit.activePosition?.id ?? circuit.active_position?.id,
    frame: circuit.frame,
    residues: (circuit.residues ?? []).filter((entry) => !entry.invalidated).map((entry) => ({
      id: entry.id,
      kind: entry.kind,
      position: entry.position,
      value: entry.value
    })),
    trajectory: (circuit.trajectory ?? []).map((entry) => ({ from: entry.from, to: entry.to, relation: entry.relation }))
  };
}

function asPosition(value, fallback) {
  return POSITIONS.includes(value) ? value : fallback;
}

function asCarrier(value, capabilities) {
  const carrier = value ?? { kind: 'model' };
  if (carrier.kind === 'model') return { kind: 'model' };
  if (carrier.kind === 'internal_control') return { kind: 'internal_control', input: clone(carrier.input ?? null) };
  if (carrier.kind === 'capability' || carrier.kind === 'tool') {
    if (!capabilities.includes(carrier.name)) throw new Error(`QL controller selected unavailable capability '${carrier.name}'.`);
    return { kind: 'capability', name: carrier.name, args: clone(carrier.args ?? {}) };
  }
  throw new Error(`QL controller selected unsupported carrier '${carrier.kind}'.`);
}

async function control(host, purpose, system, payload) {
  const response = await host.callModel({
    series1Control: {
      purpose,
      system: `${system}\nReturn exactly one JSON object and no prose outside it.`,
      prompt: JSON.stringify(payload, null, 2)
    }
  });
  if (!response?.control || typeof response.control !== 'object') {
    throw new Error(`QL control turn '${purpose}' did not return a structured object.`);
  }
  return response.control;
}

function operatorCircuit(snapshot) {
  return {
    ...clone(snapshot),
    active_position: clone(snapshot.activePosition ?? snapshot.active_position),
    closure_state: snapshot.closureState ?? snapshot.closure_state,
    success_state: clone(snapshot.successState ?? snapshot.success_state),
    parent_id: snapshot.parentId ?? snapshot.parent_id ?? null
  };
}

async function runDepth({ host, circuit, request, session, capabilities }) {
  const parent = operatorCircuit(circuit);
  const aperture = await control(
    host,
    'ql-depth-aperture',
    'You are deciding a bounded recursive QL child task. The parent is at P4. State one local whole whose independent resolution would materially improve the parent evaluation.',
    { task: request.input, success_conditions: request.successConditions, circuit: compactCircuit(circuit) }
  );
  const localIntent = aperture.local_whole_intent ?? aperture.intent;
  if (!localIntent) throw new Error('Depth request requires local_whole_intent.');

  const child = session.openChild({
    parentCircuit: parent,
    parentPosition: 'P4',
    localWholeIntent: localIntent,
    selectedResidueRefs: (parent.residues ?? []).filter((entry) => !entry.invalidated).map((entry) => entry.id),
    successConditions: aperture.success_conditions ?? request.successConditions
  });

  const childResult = await host.callModel({
    series1Control: {
      purpose: 'ql-child-execution',
      system: 'You are resolving a fresh, independent child whole for a parent agent. Use only the supplied child frame and selected parent evidence. Return JSON with synthesis, evidence, unresolved, and success (true|false|unknown).',
      prompt: JSON.stringify({
        child_frame: child.frame,
        capabilities,
        selected_parent_residues: (parent.residues ?? []).filter((entry) => child.depth_request.selected_residue_refs.includes(entry.id))
      }, null, 2)
    }
  });

  child.active_position = { id: 'P5', structural_class: 'implicate' };
  child.residues.push({
    id: `${child.id}:res:child-result`,
    kind: 'determination',
    position: 'P5',
    value: clone(childResult.control),
    provenance: { live_model_child: true }
  });

  const completed = session.completeChild({
    parentCircuit: parent,
    childCircuit: child,
    determinationRef: `${child.id}:determination:live`,
    returnedDelta: {
      synthesis: childResult.control?.synthesis ?? null,
      evidence: childResult.control?.evidence ?? [],
      unresolved: childResult.control?.unresolved ?? [],
      success: childResult.control?.success ?? 'unknown'
    },
    destination: 'P4'
  });

  return completed.summary;
}

async function runConjugate({ host, circuit, determination, request, session }) {
  const direct = operatorCircuit(circuit);
  const selection = await control(
    host,
    'ql-conjugate-scope',
    `You are choosing how to inspect a candidate determination through a fresh conjugate view. Choose scope whole or current_position. Optionally choose one pairing modulation only when it would sharpen the review: family A|B|C, pair 1|2|3, level D1|D2|D3; D2 additionally requires projection_side left|right. Do not invoke a modulation merely because it exists.`,
    { task: request.input, determination, circuit: compactCircuit(circuit) }
  );

  const scope = selection.scope === 'current_position' ? 'current_position' : 'whole';
  let modulation = null;
  if (selection.pairing_modulation) {
    const requested = selection.pairing_modulation;
    modulation = buildDModulationFrame({
      family: requested.family,
      pair: Number(requested.pair),
      level: requested.level,
      projectionSide: requested.projection_side ?? undefined
    });
  }

  const conjugate = session.openConjugate({
    directCircuit: direct,
    scope,
    intentPacket: direct.frame?.initiating_intent,
    outcomePacket: determination.synthesis,
    selectedResidueRefs: (direct.residues ?? []).filter((entry) => !entry.invalidated).map((entry) => entry.id),
    successConditions: direct.frame?.success_conditions ?? request.successConditions
  });

  const review = await control(
    host,
    'ql-conjugate-review',
    `You are the fresh conjugate review of a candidate agent outcome. You did not receive the persuasive direct transcript. Assess the supplied intent, outcome, selected residues and optional pairing frame. Return status confirm|qualify|reopen|invalidate. If status is reopen or invalidate, choose target_position P0..P4 according to the discrepancy: P1 evidence/material, P2 effect/action, P3 form/implementation, P4 evaluation/context, P0 initiating frame.`,
    {
      conjugate_packet: conjugate.packet,
      pairing_modulation: modulation,
      candidate_determination: determination
    }
  );

  let status = ['confirm', 'qualify', 'reopen', 'invalidate'].includes(review.status) ? review.status : 'qualify';
  if (status === 'invalidate') status = 'reopen';
  const target = status === 'reopen' ? asPosition(review.target_position, 'P4') : null;
  if (target === 'P5') throw new Error('Conjugate reopening cannot target P5.');
  const delta = {
    status,
    discrepancy_type: review.discrepancy_type ?? null,
    target_position: target,
    target_relation: null,
    evidence_refs: review.evidence_refs ?? [],
    analysis_ref: review.analysis ?? review.rationale ?? null,
    recommended_reopening_relation: status === 'reopen' ? `R5${target.slice(1)}` : null,
    pairing_modulation: modulation
  };

  session.completeConjugate({ directCircuit: direct, conjugateCircuit: conjugate, delta });
  return delta;
}

export function createModelDrivenQLPolicy({ mode = 'direct', operatorRunId = 'series1:operators' } = {}) {
  if (!['direct', 'deep'].includes(mode)) throw new TypeError(`Unknown QL policy mode '${mode}'.`);
  const session = new DeepQLOperatorSession({ runId: operatorRunId });
  const state = { depthUsedFor: new Set(), operatorEvents: session };

  return {
    mode,

    async nextAct({ circuit, request, host }) {
      const capabilities = (request.capabilities ?? []).map((entry) => typeof entry === 'string' ? entry : entry.id).filter(Boolean);
      const active = circuit.activePosition.id;
      const decision = await control(
        host,
        'ql-next-act',
        `You are controlling a QL-native agent recurrence. Positions are responsibilities, not chronological stages: ${JSON.stringify(POSITION_GUIDE)}. Choose the next exterior act appropriate to the currently active position. Available carriers: model, capability, internal_control. In deep mode, only at P4, you may request deep_operator='depth' when a genuinely local whole warrants independent treatment. Do not force a six-step path and do not use depth ceremonially.`,
        { mode, task: request.input, success_conditions: request.successConditions, capabilities, circuit: compactCircuit(circuit) }
      );

      if (mode === 'deep' && active === 'P4' && decision.deep_operator === 'depth' && !state.depthUsedFor.has(circuit.id)) {
        state.depthUsedFor.add(circuit.id);
        const summary = await runDepth({ host, circuit, request, session, capabilities });
        return {
          intent: `Reintegrate independently resolved child whole: ${summary.child_intent}`,
          carrier: { kind: 'internal_control', input: { child_summary: summary } },
          metadata: { deep_operator: 'depth', child_summary: summary },
          claimedPosition: active
        };
      }

      return {
        intent: decision.intent ?? `Advance the ${active} responsibility for the initiating intent.`,
        carrier: asCarrier(decision.carrier, capabilities),
        inputResidueRefs: Array.isArray(decision.input_residue_refs) ? decision.input_residue_refs : [],
        claimedPosition: active,
        claimedRelation: decision.claimed_relation ?? null,
        metadata: { controller_rationale: decision.rationale ?? null }
      };
    },

    establishDifference({ returned }) {
      return {
        operation_success: returned.operation_success,
        raw_result: clone(returned.raw_result)
      };
    },

    async interpret({ circuit, difference, act, request }) {
      if (act.metadata?.deep_operator === 'depth') {
        return {
          destination: 'P4',
          rationale: 'A typed child summary returns to whole-relative evaluation without importing the child transcript.',
          residueDelta: {
            create: [{
              kind: 'evaluation',
              position: 'P4',
              value: clone(act.metadata.child_summary),
              provenance: { child_summary: true, typed_summary_only: true }
            }]
          },
          witness: { structural_facts: { deep_operator: 'depth', typed_summary_only: true } }
        };
      }

      const decision = await control(
        request.__series1Host,
        'ql-interpret-return',
        `Interpret the returned difference for the current QL whole. The carrier does NOT determine semantic destination. Choose exactly one destination P0..P5 and explain why. P0=ground/frame, P1=material/evidence, P2=effect/transformation, P3=form/implementation, P4=evaluation/context, P5=candidate determination. Preserve genuine failure or ambiguity rather than pretending success.`,
        { task: request.input, success_conditions: request.successConditions, circuit: compactCircuit(circuit), act, difference }
      );
      const destination = asPosition(decision.destination, circuit.activePosition.id);
      return {
        destination,
        rationale: decision.rationale ?? null,
        residueDelta: {
          create: [{
            kind: RESIDUE_KIND[destination],
            position: destination,
            value: {
              difference: clone(difference),
              semantic_summary: decision.semantic_summary ?? null
            },
            provenance: { live_model_interpretation: true, act_id: act.id }
          }]
        },
        witness: {
          claimed_position: decision.claimed_position ?? null,
          observed_position: destination,
          ambiguity: decision.ambiguity ?? null,
          structural_facts: { carrier: clone(act.carrier), operation_success: difference.operation_success }
        }
      };
    },

    async proposeDetermination({ circuit, request }) {
      const decision = await control(
        request.__series1Host,
        'ql-propose-determination',
        `The active responsibility is P5: candidate determination. Synthesize what is actually realised relative to the initiating intent and success conditions. requested_outcome must be close or reopen${mode === 'deep' ? ' or conjugate' : ''}. Use conjugate only when an independent inverse/critical fresh-context review is warranted; it is not mandatory.`,
        { mode, task: request.input, success_conditions: request.successConditions, circuit: compactCircuit(circuit) }
      );
      const allowed = mode === 'deep' ? ['close', 'reopen', 'conjugate'] : ['close', 'reopen'];
      const requested = allowed.includes(decision.requested_outcome) ? decision.requested_outcome : 'reopen';
      return {
        synthesis: decision.synthesis ?? '',
        claimed_adequacy: decision.claimed_adequacy ?? 'unknown',
        claimed_subject: decision.claimed_subject ?? request.taskId,
        claimed_state: decision.claimed_state ?? null,
        evidence_refs: Array.isArray(decision.evidence_refs) ? decision.evidence_refs : [],
        evaluation_refs: (circuit.residues ?? []).filter((entry) => entry.kind === 'evaluation' && !entry.invalidated).map((entry) => entry.id),
        unresolved_refs: Array.isArray(decision.unresolved_refs) ? decision.unresolved_refs : [],
        requested_outcome: requested
      };
    },

    async evaluateClosure({ circuit, determination, frame, evaluations, request }) {
      if (mode === 'deep' && determination.requested_outcome === 'conjugate') {
        const delta = await runConjugate({ host: request.__series1Host, circuit, determination, request, session });
        if (delta.status === 'reopen') {
          return {
            status: 'reopen',
            destination: delta.target_position,
            task_success: 'false',
            rationale: `Fresh conjugate review reopened the direct determination: ${delta.analysis_ref ?? delta.discrepancy_type ?? 'discrepancy'}`,
            retained_delta_preview: delta
          };
        }
      }

      const verdict = await control(
        request.__series1Host,
        'ql-evaluate-closure',
        `Evaluate positive QL closure. Do not equate no pending tool call with task completion. Compare initiating Frame/P0, whole-relative Evaluation/P4, and candidate Determination/P5. Return status close|reopen. On reopen choose destination P0..P4 and preserve the semantic reason.`,
        { task: request.input, success_conditions: request.successConditions, frame, evaluations, determination, circuit: compactCircuit(circuit) }
      );
      const status = verdict.status === 'close' ? 'close' : 'reopen';
      if (status === 'close') {
        return { status: 'close', task_success: String(verdict.task_success ?? 'true'), rationale: verdict.rationale ?? null };
      }
      const destination = asPosition(verdict.destination, 'P4');
      return {
        status: 'reopen',
        destination: destination === 'P5' ? 'P4' : destination,
        task_success: String(verdict.task_success ?? 'false'),
        rationale: verdict.rationale ?? null
      };
    },

    createReentryDelta({ determination, request }) {
      return {
        achieved_artifact_refs: [],
        changed_assumptions: [],
        unresolved_refs: determination.unresolved_refs ?? [],
        revised_success_conditions: request.successConditions,
        opened_questions: determination.unresolved_refs ?? [],
        provenance: { series1: true, policy_mode: mode }
      };
    },

    getOperatorEvents() {
      return session.snapshot();
    }
  };
}

export function bindSeries1Host(request, host) {
  return { ...request, __series1Host: host };
}
