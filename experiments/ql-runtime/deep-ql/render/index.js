const clone = (value) => value === undefined ? undefined : structuredClone(value);

function blankState({ runId = null, circuitId = null, parentCircuitId = null, face = null } = {}) {
  return {
    run_id: runId,
    circuit_id: circuitId,
    parent_circuit_id: parentCircuitId,
    depth: 0,
    face,
    active_position: null,
    relation: null,
    exchange: null,
    closure_state: 'open',
    child_count: 0,
    conjugate_state: 'available',
    event_count: 0,
    last_event: null
  };
}

function differenceKind(value) {
  if (value === null || value === undefined) return String(value);
  if (Array.isArray(value)) return 'array';
  return typeof value;
}

export function replayCircuits(events) {
  const states = new Map();
  const order = [];

  const ensure = (circuitId, defaults = {}) => {
    if (!circuitId) return null;
    if (!states.has(circuitId)) {
      states.set(circuitId, blankState({ circuitId, ...defaults }));
      order.push(circuitId);
    }
    return states.get(circuitId);
  };

  for (const event of events) {
    const state = ensure(event.circuit_id, {
      runId: event.run_id ?? null,
      parentCircuitId: event.parent_circuit_id ?? null,
      face: event.face ?? null
    });
    if (!state) continue;

    state.run_id = event.run_id ?? state.run_id;
    state.parent_circuit_id = event.parent_circuit_id ?? state.parent_circuit_id;
    state.face = event.face ?? state.face;
    state.event_count += 1;
    state.last_event = event.event_type;

    if (event.event_type === 'circuit_started' || event.event_type === 'child_started' || event.event_type === 'conjugate_started') {
      state.depth = event.payload?.depth ?? state.depth;
    }
    if (event.ql?.to) state.active_position = event.ql.to;
    if (event.ql?.relation) state.relation = event.ql.relation;

    if (event.event_type === 'projection') {
      const projection = event.payload?.projection ?? {};
      state.exchange = {
        projection: event.ql?.projection ?? projection.phase ?? '0/1',
        carrier: {
          kind: projection.carrier?.kind ?? null,
          name: projection.carrier?.name ?? null
        }
      };
    }
    if (event.event_type === 'return_received') {
      const returned = event.payload?.returned ?? {};
      state.exchange = {
        ...(state.exchange ?? {}),
        return: event.ql?.return ?? returned.phase ?? '1/0',
        operation_success: returned.operation_success ?? null,
        difference_kind: differenceKind(returned.difference)
      };
    }

    if (event.event_type === 'circuit_closed' || event.event_type === 'child_completed') state.closure_state = 'closed';
    if (event.event_type === 'circuit_reopened') state.closure_state = 'open';

    if (event.event_type === 'child_started' && event.parent_circuit_id) {
      const parent = ensure(event.parent_circuit_id, { runId:event.run_id, face:'direct' });
      parent.child_count += 1;
    }
    if (event.event_type === 'conjugate_started' && event.parent_circuit_id) {
      const parent = ensure(event.parent_circuit_id, { runId:event.run_id, face:'direct' });
      parent.conjugate_state = 'active';
    }
    if (event.event_type === 'conjugate_completed' && event.parent_circuit_id) {
      const parent = ensure(event.parent_circuit_id, { runId:event.run_id, face:'direct' });
      parent.conjugate_state = 'completed';
    }
  }

  return order.map((id) => clone(states.get(id)));
}

export function replayState(events, circuitId = null) {
  const states = replayCircuits(events);
  if (circuitId) return states.find((state) => state.circuit_id === circuitId) ?? null;
  return states[0] ?? blankState();
}

export function renderCircuit(state) {
  return [
    `QL  ${state.run_id ?? '-'} / ${state.circuit_id ?? '-'}  depth=${state.depth}`,
    `PARENT  ${state.parent_circuit_id ?? '-'}`,
    `FACE  ${(state.face ?? 'unknown').toUpperCase()}`,
    `ACTIVE  ${state.active_position ?? '-'}`,
    `RELATION  ${state.relation ?? '-'}`,
    `EXCHANGE  ${JSON.stringify(state.exchange ?? {})}`,
    `CLOSURE  ${state.closure_state}`,
    `CHILDREN  ${state.child_count}`,
    `CONJUGATE  ${state.conjugate_state}`,
    `EVENTS  ${state.event_count}`,
    `LAST  ${state.last_event ?? '-'}`
  ].join('\n');
}

export function renderRun(events) {
  const states = replayCircuits(events);
  if (!states.length) return renderCircuit(blankState());
  return states.map(renderCircuit).join('\n\n---\n\n');
}
