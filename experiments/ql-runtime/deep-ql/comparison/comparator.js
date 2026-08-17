function semanticSignature(event) {
  return {
    event_type: event.event_type,
    face: event.face ?? null,
    parented: Boolean(event.parent_circuit_id),
    ql: {
      from: event.ql?.from ?? null,
      to: event.ql?.to ?? null,
      relation: event.ql?.relation ?? null,
      projection: event.ql?.projection ?? null,
      return: event.ql?.return ?? null,
      lens: event.ql?.lens ?? []
    }
  };
}

function stable(value) {
  return JSON.stringify(value);
}

function relationList(events) {
  return events
    .filter((event) => event.event_type === 'transition')
    .map((event) => event.ql?.relation ?? null);
}

export function compareTraces(left, right) {
  const leftSignatures = left.map(semanticSignature);
  const rightSignatures = right.map(semanticSignature);
  const length = Math.max(leftSignatures.length, rightSignatures.length);
  const mismatches = [];

  for (let index = 0; index < length; index += 1) {
    const a = leftSignatures[index] ?? null;
    const b = rightSignatures[index] ?? null;
    if (stable(a) !== stable(b)) {
      mismatches.push({ index, left: a, right: b });
    }
  }

  return {
    equal: mismatches.length === 0,
    left_event_count: left.length,
    right_event_count: right.length,
    left_relations: relationList(left),
    right_relations: relationList(right),
    mismatches,
    left_signatures: leftSignatures,
    right_signatures: rightSignatures
  };
}
