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

function walk(value, visit, path = []) {
  if (Array.isArray(value)) {
    value.forEach((entry, index) => walk(entry, visit, [...path, index]));
    return;
  }
  if (!value || typeof value !== 'object') return;
  visit(value, path);
  for (const [key, entry] of Object.entries(value)) walk(entry, visit, [...path, key]);
}

export function extractPrimeFamily(records) {
  const nodes = new Map();
  const edges = new Map();

  for (const record of records) {
    walk(record, (object) => {
      const childId = object.rlm_child_id ?? object.rlmChildId ?? object.child_id ?? object.childId ?? null;
      const activeId = object.active_session_id ?? object.activeSessionId ?? object.session_id ?? object.sessionId ?? null;
      const parentId = object.parent_session_id ?? object.parentSessionId ?? object.parent_id ?? object.parentId ?? null;
      const name = object.session_name ?? object.sessionName ?? object.name ?? null;
      const id = childId ?? activeId;
      if (typeof id === 'string' && id) {
        const existing = nodes.get(id) ?? { id };
        if (typeof name === 'string' && name) existing.name = name;
        if (typeof activeId === 'string' && activeId) existing.active_session_id = activeId;
        if (typeof childId === 'string' && childId) existing.rlm_child_id = childId;
        nodes.set(id, existing);
      }
      if (typeof parentId === 'string' && parentId && typeof id === 'string' && id && parentId !== id) {
        edges.set(`${parentId}->${id}`, { parent: parentId, child: id });
      }
    });
  }

  return { nodes: [...nodes.values()], edges: [...edges.values()] };
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
