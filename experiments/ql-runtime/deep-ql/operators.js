import { DEFAULT_LENSES, QLSemanticError, createSuccessState, qlFace, qlPosition, qlRelation } from '../foundation/ql-core-runtime/semantics.js';

const clone = (v) => v === undefined ? undefined : structuredClone(v);
const P = ['P0','P1','P2','P3','P4','P5'];

function scopeValue(scope) {
  if (scope === 'whole' || scope === 'current_position') return scope;
  if (scope?.position && P.includes(scope.position)) return { position: scope.position };
  if (scope?.relation && /^R[0-5][0-5]$/.test(scope.relation)) return { relation: scope.relation };
  throw new QLSemanticError(`Unsupported conjugate scope '${JSON.stringify(scope)}'.`);
}

export function createConjugatePacket({ directCircuit, scope='whole', intentPacket, outcomePacket, selectedResidueRefs, successConditions }) {
  if (!directCircuit?.id) throw new QLSemanticError('Conjugation requires a direct circuit.');
  const selected = selectedResidueRefs ?? directCircuit.residues?.filter(r=>!r.invalidated).map(r=>r.id) ?? [];
  const byKind = (kind) => selected.filter(id => directCircuit.residues?.some(r=>r.id===id && r.kind===kind));
  return {
    direct_circuit_ref: directCircuit.id,
    source_position: directCircuit.active_position?.id ?? directCircuit.activePosition?.id ?? 'P5',
    intent_packet: clone(intentPacket ?? directCircuit.frame?.initiating_intent),
    outcome_packet: clone(outcomePacket ?? null),
    material_refs: byKind('material'), effect_refs: byKind('effect'), form_refs: byKind('form'), evaluation_refs: byKind('evaluation'),
    success_conditions: clone(successConditions ?? directCircuit.frame?.success_conditions ?? []),
    requested_scope: scopeValue(scope),
    provenance: { reconstructed_context: true, complete_direct_transcript_inherited: false }
  };
}

export function openConjugateCircuit(args) {
  const packet = createConjugatePacket(args);
  const direct = args.directCircuit;
  return {
    id: `${direct.id}:j:${(direct.conjugates?.length ?? 0)+1}`,
    parent_id: direct.id, depth: direct.depth ?? 0, face: qlFace('conjugate'), packet,
    frame: { id:`${direct.id}:conjugate-frame`, initiating_intent:packet.intent_packet, operative_scope:packet.requested_scope,
      constraints:[], available_capabilities:[], success_conditions:packet.success_conditions, provenance:{direct_circuit_ref:direct.id,reconstructed:true} },
    active_position: qlPosition('P0'), residues:[], trajectory:[], closure_state:'open', success_state:createSuccessState({conjugate_stability:'unknown'}), children:[], conjugates:[]
  };
}

export function createConjugateDelta({ status, targetPosition, targetRelation, evidenceRefs=[], analysisRef=null, discrepancyType=null }) {
  if (!['confirm','qualify','reopen','invalidate'].includes(status)) throw new QLSemanticError(`Invalid ConjugateDelta status '${status}'.`);
  let relation = targetRelation ?? null;
  if (status === 'reopen') {
    if (!['P0','P1','P2','P3','P4'].includes(targetPosition)) throw new QLSemanticError('Conjugate reopening must target P0-P4.');
    relation = qlRelation('P5', targetPosition).id;
  }
  return { status, discrepancy_type:discrepancyType, target_position:targetPosition ?? null, target_relation:targetRelation ?? null,
    evidence_refs:clone(evidenceRefs), analysis_ref:analysisRef, recommended_reopening_relation:relation };
}

export function reintegrateConjugateDelta({ directCircuit, delta }) {
  if (directCircuit.closure_state === 'closed' || directCircuit.closureState === 'closed') throw new QLSemanticError('A conjugate result cannot retroactively reopen a positively closed direct circuit.');
  if (delta.status !== 'reopen') return { circuit:directCircuit, delta };
  const active = directCircuit.active_position?.id ?? directCircuit.activePosition?.id;
  if (active !== 'P5') throw new QLSemanticError('Conjugate reopening reintegrates at an open P5 determination.');
  const to = delta.target_position;
  directCircuit.trajectory ??= [];
  directCircuit.trajectory.push({ id:`${directCircuit.id}:conjugate-reintegration:${directCircuit.trajectory.length}`, from:'P5', to, relation:qlRelation('P5',to).id, witness_state:{conjugate:true} });
  if ('active_position' in directCircuit) directCircuit.active_position = qlPosition(to); else directCircuit.activePosition = qlPosition(to);
  return { circuit:directCircuit, delta };
}

export function createDepthRequest({ parentCircuit, parentPosition, localWholeIntent, selectedResidueRefs=[], successConditions=[], extension=null }) {
  const pos = parentPosition ?? parentCircuit?.active_position?.id ?? parentCircuit?.activePosition?.id;
  if (pos !== 'P4' && !extension?.startsWith('ql.')) throw new QLSemanticError('Core 0.1 recursive depth opens from P4 unless a namespaced extension explicitly declares another aperture.');
  return { parent_circuit:parentCircuit.id, parent_position:pos, local_whole_intent:clone(localWholeIntent), selected_residue_refs:clone(selectedResidueRefs), success_conditions:clone(successConditions), extension };
}

export function openChildCircuit(args) {
  const request = createDepthRequest(args);
  const parent = args.parentCircuit;
  const childIndex = (parent.children?.length ?? 0)+1;
  const child = {
    id:`${parent.id}:child:${childIndex}`, parent_id:parent.id, depth:(parent.depth ?? 0)+1, face:qlFace('direct'), depth_request:request,
    frame:{ id:`${parent.id}:child:${childIndex}:frame:0`, initiating_intent:request.local_whole_intent, operative_scope:{parent_position:request.parent_position}, constraints:[], available_capabilities:[], success_conditions:request.success_conditions, inherited_delta:null, provenance:{parent_circuit:parent.id,parent_position:request.parent_position} },
    active_position:qlPosition('P0'), residues:[], trajectory:[], closure_state:'open', success_state:createSuccessState(), children:[], conjugates:[]
  };
  child.residues.push({ id:`${child.id}:res:0`, kind:'frame', position:'P0', value:clone(child.frame), provenance:clone(child.frame.provenance) });
  parent.children ??= []; parent.children.push(child.id);
  return child;
}

export function createChildCircuitSummary({ childCircuit, determinationRef=null, relevantResidueRefs, returnedDelta={}, status='closed' }) {
  return { child_circuit:childCircuit.id, parent_circuit:childCircuit.parent_id, child_intent:clone(childCircuit.frame.initiating_intent), determination_ref:determinationRef,
    relevant_residue_refs:clone(relevantResidueRefs ?? childCircuit.residues.filter(r=>!r.invalidated).map(r=>r.id)), success_state:clone(childCircuit.success_state), returned_delta:clone(returnedDelta), status };
}

export function closeChildCircuit({ childCircuit, determinationRef='child-determination', returnedDelta={} }) {
  if ((childCircuit.active_position?.id ?? childCircuit.activePosition?.id) !== 'P5') throw new QLSemanticError('Child positive closure requires P5.');
  childCircuit.closure_state='closed'; childCircuit.success_state=createSuccessState({...childCircuit.success_state,task:'true',circuit:'true'});
  return createChildCircuitSummary({childCircuit,determinationRef,returnedDelta,status:'closed'});
}

export function reintegrateChildSummary({ parentCircuit, summary, destination='P4' }) {
  if (summary.parent_circuit !== parentCircuit.id) throw new QLSemanticError('Child summary parent mismatch.');
  const from = parentCircuit.active_position?.id ?? parentCircuit.activePosition?.id;
  const relation = qlRelation(from,destination);
  parentCircuit.residues ??= [];
  const residue={id:`${parentCircuit.id}:child-summary:${summary.child_circuit}`,kind:'evaluation',position:destination,value:clone(summary),provenance:{child_circuit:summary.child_circuit,typed_summary_only:true}};
  parentCircuit.residues.push(residue); parentCircuit.trajectory ??=[];
  parentCircuit.trajectory.push({id:`${parentCircuit.id}:child-reintegration:${parentCircuit.trajectory.length}`,from,to:destination,relation:relation.id,created_residue_refs:[residue.id],witness_state:{child_summary:true,transcript_required:false}});
  if ('active_position' in parentCircuit) parentCircuit.active_position=qlPosition(destination); else parentCircuit.activePosition=qlPosition(destination);
  return { circuit:parentCircuit, residue, relation:relation.id };
}

export const DEEP_OPERATOR_PROFILE = Object.freeze({
  name:'ql-agent/0.1-deep-required', lenses:[...DEFAULT_LENSES], conjugation:['whole','current_position'], depth:{default_aperture:'P4'}
});
