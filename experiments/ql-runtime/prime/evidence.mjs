import crypto from 'node:crypto';
import fs from 'node:fs/promises';

export function stableDigest(value) {
  return crypto.createHash('sha256').update(JSON.stringify(value, Object.keys(value ?? {}).sort())).digest('hex');
}

export async function readJsonl(pathname) {
  try {
    const text = await fs.readFile(pathname, 'utf8');
    return text.split('\n').filter(Boolean).map((line) => JSON.parse(line));
  } catch (error) {
    if (error?.code === 'ENOENT') return [];
    throw error;
  }
}

function walk(value, visitObject, visitString, path = []) {
  if (typeof value === 'string') {
    visitString(value, path);
    return;
  }
  if (Array.isArray(value)) {
    value.forEach((entry, index) => walk(entry, visitObject, visitString, [...path, index]));
    return;
  }
  if (!value || typeof value !== 'object') return;
  visitObject(value, path);
  for (const [key, entry] of Object.entries(value)) walk(entry, visitObject, visitString, [...path, key]);
}

export function extractPrimeFamily(records) {
  const nodes = new Map();
  const edges = new Map();
  const upsertChild = (id, extras = {}) => {
    if (typeof id !== 'string' || !id) return;
    const existing = nodes.get(id) ?? { id };
    Object.assign(existing, extras);
    if (!existing.rlm_child_id && /^sub-/.test(id)) existing.rlm_child_id = id;
    nodes.set(id, existing);
  };

  for (const record of records) {
    walk(record, (object) => {
      const childId = object.rlm_child_id ?? object.rlmChildId ?? object.child_id ?? object.childId ?? object.child_ref ?? null;
      const activeId = object.active_session_id ?? object.activeSessionId ?? object.session_id ?? object.sessionId ?? null;
      const parentId = object.parent_session_id ?? object.parentSessionId ?? object.parent_id ?? object.parentId ?? object.parent_ref ?? null;
      const name = object.session_name ?? object.sessionName ?? object.name ?? null;
      const id = childId ?? activeId;
      if (typeof id === 'string' && id) {
        const extras = {};
        if (typeof name === 'string' && name) extras.name = name;
        if (typeof activeId === 'string' && activeId) extras.active_session_id = activeId;
        if (typeof childId === 'string' && childId) extras.rlm_child_id = childId;
        upsertChild(id, extras);
      }
      if (typeof parentId === 'string' && parentId && typeof id === 'string' && id && parentId !== id) {
        edges.set(`${parentId}->${id}`, { parent: parentId, child: id });
      }
    }, (text) => {
      for (const match of text.matchAll(/rlm_child_id["']?\s*[:=]\s*["']?(sub-[A-Za-z0-9_-]+)/g)) upsertChild(match[1], { rlm_child_id: match[1] });
      for (const match of text.matchAll(/"child_ref"\s*:\s*"([^"]+)"/g)) upsertChild(match[1], { rlm_child_id: match[1] });
      for (const match of text.matchAll(/"parent_ref"\s*:\s*"([^"]+)"[\s\S]{0,500}?"child_ref"\s*:\s*"([^"]+)"/g)) {
        upsertChild(match[2], { rlm_child_id: match[2] });
        edges.set(`${match[1]}->${match[2]}`, { parent: match[1], child: match[2] });
      }
    });
  }

  const nodeValues = [...nodes.values()];
  const edgeValues = [...edges.values()];
  const childNodes = nodeValues.filter((node) => node.rlm_child_id);
  const childIds = new Set(childNodes.map((node) => node.id));
  const nestedEdges = edgeValues.filter((edge) => childIds.has(edge.parent) && childIds.has(edge.child));
  return { nodes: nodeValues, edges: edgeValues, child_nodes: childNodes, nested_edges: nestedEdges };
}

export function sourceSummary(lock, qlState = null) {
  return {
    prime_release: lock.prime_agent.release,
    prime_release_revision: lock.prime_agent.release_revision,
    actuation_base_revision: lock.actuation.base_revision,
    ql_mef_expected_main_revision: lock.ql_mef.accepted_main_revision,
    ql_mef_observed: qlState?.revision ?? null,
    harmonic_research_revision: lock.ql_mef.harmonic_research.revision,
    harmonic_enabled: Boolean(qlState?.harmonic_enabled)
  };
}
