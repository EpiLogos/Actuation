import { DEFAULT_LENSES, QL_SPEC } from '../foundation/ql-core-runtime/semantics.js';
import {
  openConjugateCircuit,
  reintegrateConjugateDelta,
  openChildCircuit,
  closeChildCircuit,
  reintegrateChildSummary
} from './operators.js';

const clone = (value) => value === undefined ? undefined : structuredClone(value);
const positionId = (circuit) => circuit?.active_position?.id ?? circuit?.activePosition?.id ?? null;

export class DeepQLOperatorSession {
  constructor({ runId, schemaVersion = '0.1.0-deep', seedEvents = [] } = {}) {
    if (!runId) throw new TypeError('DeepQLOperatorSession requires runId.');
    this.runId = runId;
    this.schemaVersion = schemaVersion;
    this.events = seedEvents.map(clone);
    this.nextSequence = new Map();
    for (const event of this.events) {
      const next = Math.max(this.nextSequence.get(event.circuit_id) ?? 0, (event.sequence ?? -1) + 1);
      this.nextSequence.set(event.circuit_id, next);
    }
  }

  #emit({ circuit, eventType, ql = {}, payload = {}, witness = {} }) {
    const circuitId = circuit.id;
    const sequence = this.nextSequence.get(circuitId) ?? 0;
    this.nextSequence.set(circuitId, sequence + 1);
    const event = {
      spec: QL_SPEC,
      schema_version: this.schemaVersion,
      event_id: `${this.runId}:${circuitId}:deep:${sequence}`,
      event_type: eventType,
      run_id: this.runId,
      circuit_id: circuitId,
      parent_circuit_id: circuit.parent_id ?? circuit.parentId ?? null,
      sequence,
      face: circuit.face,
      ql: { ...clone(ql), lens: clone(ql.lens ?? DEFAULT_LENSES) },
      payload: clone(payload),
      witness: clone(witness)
    };
    this.events.push(event);
    return event;
  }

  openConjugate(args) {
    const direct = args.directCircuit;
    const conjugate = openConjugateCircuit(args);
    this.#emit({
      circuit: conjugate,
      eventType: 'conjugate_started',
      ql: { from: positionId(direct), to: 'P0' },
      payload: {
        direct_circuit_ref: direct.id,
        requested_scope: clone(conjugate.packet.requested_scope),
        packet: clone(conjugate.packet),
        depth: conjugate.depth
      },
      witness: {
        structural_facts: {
          reconstructed_context: conjugate.packet.provenance.reconstructed_context,
          complete_direct_transcript_inherited: conjugate.packet.provenance.complete_direct_transcript_inherited
        }
      }
    });
    return conjugate;
  }

  completeConjugate({ directCircuit, conjugateCircuit, delta }) {
    this.#emit({
      circuit: conjugateCircuit,
      eventType: 'conjugate_completed',
      ql: { from: positionId(conjugateCircuit) },
      payload: { direct_circuit_ref: directCircuit.id, delta: clone(delta) },
      witness: { structural_facts: { status: delta.status } }
    });

    if (delta.status !== 'reopen') {
      return { circuit: directCircuit, delta, events: this.snapshot() };
    }

    const before = positionId(directCircuit);
    const result = reintegrateConjugateDelta({ directCircuit, delta });
    const transition = result.circuit.trajectory.at(-1);
    this.#emit({
      circuit: result.circuit,
      eventType: 'circuit_reopened',
      ql: { from: before, to: transition.to, relation: transition.relation },
      payload: {
        source: 'conjugate',
        conjugate_circuit_ref: conjugateCircuit.id,
        delta: clone(delta)
      },
      witness: { structural_facts: { conjugate_reintegration: true } }
    });
    this.#emit({
      circuit: result.circuit,
      eventType: 'transition',
      ql: { from: transition.from, to: transition.to, relation: transition.relation },
      payload: { transition: clone(transition) },
      witness: clone(transition.witness_state ?? {})
    });
    return { ...result, events: this.snapshot() };
  }

  openChild(args) {
    const parent = args.parentCircuit;
    const child = openChildCircuit(args);
    this.#emit({
      circuit: child,
      eventType: 'child_started',
      ql: { from: positionId(parent), to: 'P0' },
      payload: {
        parent_circuit_ref: parent.id,
        parent_position: child.depth_request.parent_position,
        depth: child.depth,
        frame: clone(child.frame)
      },
      witness: {
        structural_facts: {
          independent_frame: true,
          parent_circuit: parent.id,
          transcript_required: false
        }
      }
    });
    return child;
  }

  completeChild({ parentCircuit, childCircuit, determinationRef, returnedDelta = {}, destination = 'P4' }) {
    const summary = closeChildCircuit({ childCircuit, determinationRef, returnedDelta });
    this.#emit({
      circuit: childCircuit,
      eventType: 'child_completed',
      ql: { to: 'P5' },
      payload: { summary: clone(summary) },
      witness: { structural_facts: { transcript_in_summary: 'transcript' in summary } }
    });

    const result = reintegrateChildSummary({ parentCircuit, summary, destination });
    this.#emit({
      circuit: result.circuit,
      eventType: 'child_reintegrated',
      ql: {
        from: result.circuit.trajectory.at(-1).from,
        to: destination,
        relation: result.relation
      },
      payload: {
        child_circuit_ref: childCircuit.id,
        summary: clone(summary),
        residue_ref: result.residue.id
      },
      witness: {
        structural_facts: {
          typed_summary_only: result.residue.provenance.typed_summary_only,
          transcript_required: false
        }
      }
    });
    return { ...result, summary, events: this.snapshot() };
  }

  snapshot() {
    return this.events.map(clone);
  }
}
