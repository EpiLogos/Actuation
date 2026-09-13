import { DeepQLOperatorSession } from '../../deep-ql/operator-session.js';
import { buildDModulationFrame } from '../../deep-ql/formal/pairing-grammar.js';
import { QL_RELATIONAL_SYSTEM } from './ql-relational-system-prompt.mjs';

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

// Conjugacy law (owner, 2026-09-13): P and P' are not distinct circuits but
// directional views on the same psychoid #0-#5 — the outward walk and the
// return reading of one field. A model looking back from P5 toward ground is
// therefore already operating in the conjugate direction; in deep mode that
// backward reading is part of determination, not a ceremonial extra operator.
const CONJUGATE_DIRECTION_LAW = 'Positions P0..P5 are directional views on one field. The outward reading (P) runs ground toward determination; the return reading (P-prime) is the same positions seen looking back from a later position toward ground. Looking back from a determination toward the frame is already conjugate operation, not a different circuit.';

// Typed stipulations: task conditions become frame-carried constraint
// bindings (the FullVakBinding pattern — a readable scope does not authorise
// an action). Exclusion conditions forbid the entire class of action,
// including creating new artifacts; this is stated to the model and enforced
// at the closure gate.
export function classifyStipulations(conditions) {
  return (conditions ?? []).map((text, index) => ({
    id: `S${index + 1}`,
    text: String(text),
    kind: /^(do not|never|don't|avoid|without)\b/i.test(String(text).trim()) ? 'exclusion' : 'goal'
  }));
}

// Per-position allowance schedule (owner direction, 2026-09-13): measure is
// staged per position on the kernel's shape — declared with the frame,
// consumed by acts, restated at every control turn, and refused with a typed
// event on overrun (never a silent stop). Acts are the loop-native currency;
// token consumption per position is metered and reported in the manifest.
export const DEFAULT_ALLOWANCE_SCHEDULE = Object.freeze({
  P0: 2, P1: 6, P2: 5, P3: 4, P4: 4, P5: 3
});
const GRACE_ALLOWANCE = 2;

function asPosition(value, fallback) {
  return POSITIONS.includes(value) ? value : fallback;
}

// Accept both the documented object form ({kind, name, args}) and the flat
// form controller models naturally return ({carrier: 'capability',
// capability: 'read_file', args}). Shape leniency only: an unknown carrier
// kind still fails closed rather than degrading to an ordinary tool loop.
// internal_control carriers keep their name: it carries the controller's
// closure-request semantics instead of being discarded.
function asCarrier(value, capabilities, decision = {}) {
  const raw = value ?? { kind: 'model' };
  const carrier = typeof raw === 'string'
    ? { kind: raw, name: decision.capability ?? decision.name ?? decision.tool, input: decision.input, args: decision.args }
    : { kind: raw.kind, name: raw.name ?? raw.capability ?? raw.tool, input: raw.input, args: raw.args };
  if (carrier.kind === 'model') return { kind: 'model' };
  if (carrier.kind === 'internal_control') {
    return { kind: 'internal_control', name: carrier.name ?? null, input: clone(carrier.input ?? carrier.args ?? null) };
  }
  if (carrier.kind === 'capability' || carrier.kind === 'tool') {
    if (!capabilities.includes(carrier.name)) throw new Error(`QL controller selected unavailable capability '${carrier.name}'.`);
    return { kind: 'capability', name: carrier.name, args: clone(carrier.args ?? {}) };
  }
  throw new Error(`QL controller selected unsupported carrier '${carrier.kind}'.`);
}

const CLOSURE_CONTROL_NAMES = new Set(['close', 'stop', 'finalize', 'finish', 'complete', 'end', 'propose_closure']);

export function isClosureControlCarrier(carrier) {
  return carrier?.kind === 'internal_control'
    && typeof carrier.name === 'string'
    && CLOSURE_CONTROL_NAMES.has(carrier.name.trim().toLowerCase());
}

async function control(host, purpose, system, payload) {
  const response = await host.callModel({
    series1Control: {
      purpose,
      // Parity: every QL control turn runs under the QL agent's standing
      // relational protocol, with the turn-specific instruction composed on top.
      system: `${QL_RELATIONAL_SYSTEM}\n\n---\n\n${system}\nReturn exactly one JSON object and no prose outside it.`,
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

export function createModelDrivenQLPolicy({
  mode = 'direct',
  operatorRunId = 'series1:operators',
  allowanceSchedule = DEFAULT_ALLOWANCE_SCHEDULE
} = {}) {
  if (!['direct', 'deep'].includes(mode)) throw new TypeError(`Unknown QL policy mode '${mode}'.`);
  const session = new DeepQLOperatorSession({ runId: operatorRunId });
  const state = {
    depthUsedFor: new Set(),
    operatorEvents: session,
    actsByPosition: new Map(),
    graceUsed: new Set()
  };

  // Allowance is per-position and frame-carried: consumed by acts at the
  // active position, restated in every control payload, refused with a typed
  // event on overrun (one recorded grace extension per position), and closed
  // by routing to determination rather than by a silent stop.
  const allowanceFor = (position) => {
    const scheduled = allowanceSchedule[position] ?? 8;
    const consumed = state.actsByPosition.get(position) ?? 0;
    const graceUsed = state.graceUsed.has(position);
    const limit = graceUsed ? scheduled + GRACE_ALLOWANCE : scheduled;
    return { scheduled, consumed, limit, exhausted: consumed >= limit, grace_extension: GRACE_ALLOWANCE, grace_pending: consumed >= limit && !graceUsed };
  };

  return {
    mode,

    async nextAct({ circuit, request, host }) {
      const capabilities = (request.capabilities ?? []).map((entry) => typeof entry === 'string' ? entry : entry.id).filter(Boolean);
      const active = circuit.activePosition.id;
      const stepsUsed = (circuit.trajectory ?? []).length;
      const stipulations = classifyStipulations(request.successConditions);
      const allowance = allowanceFor(active);

      if (allowance.exhausted) {
        // Typed refusal: no model call, a recorded overrun event, and the
        // loop moves to determination through the closure-request path. The
        // first refusal at a position grants a one-time recorded grace
        // extension before the post-grace limit binds.
        if (allowance.grace_pending) state.graceUsed.add(active);
        return {
          intent: `Allowance at ${active} exhausted (${allowance.consumed}/${allowance.limit}); routed to determination by the allowance schedule.`,
          carrier: { kind: 'internal_control', name: 'close', input: null },
          inputResidueRefs: [],
          claimedPosition: active,
          claimedRelation: null,
          metadata: {
            controller_rationale: 'Typed allowance refusal: the position budget is spent; determination must decide whether the realisable intent is achieved.',
            closure_request: true,
            allowance_refusal: { position: active, consumed: allowance.consumed, scheduled: allowance.scheduled, grace_extension: allowance.grace_pending ? GRACE_ALLOWANCE : 0 }
          }
        };
      }

      state.actsByPosition.set(active, allowance.consumed + 1);
      const budget = {
        max_steps: request.maxSteps ?? null,
        steps_used: stepsUsed,
        allowance: {
          schedule: allowanceSchedule,
          consumed: Object.fromEntries(state.actsByPosition),
          active_position: active,
          position_limit: allowance.limit
        }
      };
      const decision = await control(
        host,
        'ql-next-act',
        `You are controlling a QL-native agent recurrence. Positions are responsibilities, not chronological stages: ${JSON.stringify(POSITION_GUIDE)}. ${CONJUGATE_DIRECTION_LAW} Choose the next exterior act appropriate to the currently active position. Return exactly one JSON object of the form {"intent": string, "carrier": {"kind": "model"|"capability"|"internal_control", "name": <capability id, required when kind is "capability">, "args": object}, "claimed_relation": string|null, "rationale": string}. The "internal_control" kind is only a closure request: use {"kind": "internal_control", "name": "close", "args": {"reason": string}} when the realisable intent is already achieved and no exterior act remains — do not repeat equivalent acts. Stipulations of kind "exclusion" forbid the entire class of action including creating new artifacts: check the carrier choice against every exclusion stipulation before returning. In deep mode, only at P4, you may add "deep_operator": "depth" when a genuinely local whole — a sub-question whose independent resolution would materially change the evaluation, resolvable without the parent's transcript — warrants independent treatment at the lemniscate point; depth at #4 is the nesting entry, not a ceremony. Do not force a six-step path.`,
        { mode, task: request.input, stipulations, success_conditions: request.successConditions, capabilities, circuit: compactCircuit(circuit), budget }
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

      const carrier = asCarrier(decision.carrier, capabilities, decision);
      const closureRequest = isClosureControlCarrier(carrier);
      return {
        intent: decision.intent ?? `Advance the ${active} responsibility for the initiating intent.`,
        carrier,
        inputResidueRefs: Array.isArray(decision.input_residue_refs) ? decision.input_residue_refs : [],
        claimedPosition: active,
        claimedRelation: decision.claimed_relation ?? null,
        metadata: {
          controller_rationale: decision.rationale ?? null,
          budget,
          ...(closureRequest ? { closure_request: true } : {})
        }
      };
    },

    establishDifference({ returned }) {
      return {
        operation_success: returned.operation_success,
        raw_result: clone(returned.raw_result)
      };
    },

    async interpret({ circuit, difference, act, request }) {
      if (act.metadata?.closure_request) {
        // The controller already stated the realisable intent is achieved
        // (or the allowance schedule refused further acts). Route straight to
        // determination: only the P5 propose/evaluate path may establish
        // positive closure.
        return {
          destination: 'P5',
          rationale: act.metadata.controller_rationale ?? 'Controller requested closure; routing to determination.',
          residueDelta: {},
          witness: {
            claimed_position: 'P5',
            observed_position: 'P5',
            ambiguity: null,
            structural_facts: {
              closure_request: true,
              ...(act.metadata.allowance_refusal ? { allowance_refusal: act.metadata.allowance_refusal } : {}),
              carrier: clone(act.carrier),
              operation_success: difference.operation_success
            }
          }
        };
      }

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
        `Interpret the returned difference for the current QL whole. The carrier does NOT determine semantic destination. Worked examples: a successful read of unprocessed evidence belongs at P1 even if the act claimed otherwise; a delivered realisation of the intent belongs at P5; a partial tool result still in use belongs at P2; a model or pattern worth keeping belongs at P3; a whole-relative check belongs at P4. Return exactly one JSON object of the form {"destination": "P0"|"P1"|"P2"|"P3"|"P4"|"P5", "semantic_summary": string, "claimed_position": "P0".."P5"|null, "ambiguity": string|null, "rationale": string}. Choose exactly one destination and explain why. Preserve genuine failure or ambiguity rather than pretending success.`,
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
      const systemBase = `The active responsibility is P5: candidate determination. ${CONJUGATE_DIRECTION_LAW} Synthesize what is actually realised relative to the initiating intent and success conditions; reading back from this determination toward the frame is the return direction of the same field. Return exactly one JSON object of the form {"synthesis": string (the realised outcome in plain text; never empty), "requested_outcome": "close"|"reopen"${mode === 'deep' ? '| "conjugate"' : ''}, "claimed_adequacy": "adequate"|"partial"|"inadequate"|"unknown", "claimed_subject": string, "evidence_refs": string[], "unresolved_refs": string[]}. Use conjugate only when the backward reading genuinely warrants an independent fresh-context check of the determination; it is not mandatory.`;
      const payload = { mode, task: request.input, stipulations: classifyStipulations(request.successConditions), success_conditions: request.successConditions, circuit: compactCircuit(circuit) };

      let decision = null;
      let synthesis = '';
      for (let attempt = 0; attempt < 2; attempt += 1) {
        decision = await control(
          request.__series1Host,
          'ql-propose-determination',
          attempt === 0
            ? systemBase
            : `${systemBase} Your previous response carried no synthesis text; "synthesis" is required.`,
          payload
        );
        synthesis = decision.synthesis ?? decision.answer ?? decision.content ?? '';
        if (String(synthesis).trim() || decision.requested_outcome === 'reopen') break;
      }

      const allowed = mode === 'deep' ? ['close', 'reopen', 'conjugate'] : ['close', 'reopen'];
      const requested = allowed.includes(decision.requested_outcome) ? decision.requested_outcome : 'reopen';
      const emptySynthesis = !String(synthesis).trim();
      // Closure is a positive determination: it may not be requested on an
      // empty synthesis. The model-requested reopen path stays intact.
      const gated = emptySynthesis && requested !== 'reopen' ? 'reopen' : requested;
      return {
        synthesis,
        claimed_adequacy: decision.claimed_adequacy ?? 'unknown',
        claimed_subject: decision.claimed_subject ?? request.taskId,
        claimed_state: decision.claimed_state ?? null,
        evidence_refs: Array.isArray(decision.evidence_refs) ? decision.evidence_refs : [],
        evaluation_refs: (circuit.residues ?? []).filter((entry) => entry.kind === 'evaluation' && !entry.invalidated).map((entry) => entry.id),
        unresolved_refs: emptySynthesis && requested !== 'reopen'
          ? ['determination-synthesis-empty']
          : (Array.isArray(decision.unresolved_refs) ? decision.unresolved_refs : []),
        requested_outcome: gated
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
        `Evaluate positive QL closure. Do not equate no pending tool call with task completion. Compare initiating Frame/P0, whole-relative Evaluation/P4, and candidate Determination/P5 — the return direction reads these back from the determination toward the ground${mode === 'deep' ? '; in deep mode this backward reading is conjugate operation' : ''}. Every stipulation must receive an explicit verdict. Return exactly one JSON object: {"status": "close"|"reopen", "destination": "P0".."P4" (on reopen), "task_success": true|false|unknown, "stipulation_verdicts": [{"id": string, "verdict": "met"|"violated"|"untestable", "evidence": string}], "rationale": string}. Exclusion stipulations forbid the entire class of action including creating new artifacts.`,
        { task: request.input, stipulations: classifyStipulations(request.successConditions), success_conditions: request.successConditions, frame, evaluations, determination, circuit: compactCircuit(circuit) }
      );
      const status = verdict.status === 'close' ? 'close' : 'reopen';
      const stipulationVerdicts = Array.isArray(verdict.stipulation_verdicts) ? verdict.stipulation_verdicts : [];
      const violatedExclusions = classifyStipulations(request.successConditions)
        .filter((binding) => binding.kind === 'exclusion')
        .filter((binding) => stipulationVerdicts.some((entry) => entry.id === binding.id && entry.verdict === 'violated'));
      if (status === 'close') {
        // A violated exclusion forbids positive success: the determination may
        // close, but it closes as failed and the violation stays on the record.
        const violatedIds = violatedExclusions.map((binding) => binding.id).join(', ');
        return {
          status: 'close',
          task_success: violatedExclusions.length ? 'false' : String(verdict.task_success ?? 'unknown'),
          rationale: violatedExclusions.length
            ? `${verdict.rationale ? `${verdict.rationale} ` : ''}Exclusion stipulations violated: ${violatedIds}.`
            : (verdict.rationale ?? null),
          stipulation_verdicts: stipulationVerdicts
        };
      }
      const destination = asPosition(verdict.destination, 'P4');
      return {
        status: 'reopen',
        destination: destination === 'P5' ? 'P4' : destination,
        task_success: violatedExclusions.length ? 'false' : String(verdict.task_success ?? 'false'),
        rationale: verdict.rationale ?? null,
        stipulation_verdicts: stipulationVerdicts
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
