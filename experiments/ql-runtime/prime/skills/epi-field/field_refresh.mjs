// epi-field refresh — read the field from the canonical bimba map into the
// durable state store. READ-ONLY on the graph (MATCH/RETURN only).
import { createRequire } from 'node:module';
import { writeFileSync, mkdirSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';
const BIMBA = '/Users/admin/Documents/epi/bimba-portable';
const require2 = createRequire(BIMBA + '/package.json');
const neo4j = require2('neo4j-driver');
const driver = neo4j.driver('bolt://100.92.62.101:7687', neo4j.auth.basic('neo4j', 'bimba'));
const RICH = ['c_1_relation_description','c_5_correspondence','l_5_realization_level','l_5_mystical_identity',
  'p_3_pattern_name','p_3_pattern_structure','t_5_insight','s_4_function_role','t_3_developmental_function'];
const V = (v) => { if (!v) return null; const a = typeof v === 'string' ? JSON.parse(v) : v; return Array.isArray(a) && a.length >= 256 ? a : null; };
const STATE = join(homedir(), '.local/state/epi-logos/field');
mkdirSync(STATE, { recursive: true });
const session = driver.session({ database: 'neo4j' });
const nodes = {};
for (const r of (await session.run("MATCH (n:Bimba) WHERE n.coordinate STARTS WITH 'M0' RETURN n.coordinate AS c, n.c_1_name AS nm, n.c_1_symbol AS sym, n.embedding AS e")).records) {
  const vec = V(r.get('e'));
  if (vec) nodes[r.get('c')] = { name: r.get('nm') || '', mark: r.get('sym') || null, vec };
}
let ninefold = [];
for (const r of (await session.run("MATCH (n:Bimba) WHERE n.coordinate = 'M0-2-9' RETURN n.q_5_ninefold_determinations AS d")).records) {
  const d = r.get('d'); ninefold = (typeof d === 'string' ? JSON.parse(d) : d).map(String);
}
const relations = [];
for (const fam of ['M0', 'M1']) {
  const res = await session.run(
    `MATCH (a:Bimba)-[r]->(b:Bimba) WHERE a.coordinate STARTS WITH '${fam}' AND b.coordinate STARTS WITH '${fam}' AND r.embedding IS NOT NULL ` +
    "RETURN elementId(r) AS eid, type(r) AS t, properties(r) AS rp, a.coordinate AS ac, coalesce(a.c_1_name,'') AS an, b.coordinate AS bc, coalesce(b.c_1_name,'') AS bn, r.embedding AS e");
  for (const r of res.records) {
    const vec = V(r.get('e')); if (!vec) continue;
    const rp = r.get('rp') || {};
    const parts = [`${r.get('ac')} ${r.get('an')} --[${r.get('t')}| family ${rp.c_1_relation_family || ''} ]--> ${r.get('bc')} ${r.get('bn')}`];
    for (const k of RICH) if (rp[k]) parts.push(`${k.split('_').slice(2).join('_')}: ${rp[k]}`);
    relations.push({ eid: r.get('eid'), type: r.get('t'), a: r.get('ac'), b: r.get('bc'), text: parts.join(' ').slice(0, 3000), vec });
  }
}
await session.close(); await driver.close();
writeFileSync(join(STATE, 'nodes.json'), JSON.stringify(nodes));
writeFileSync(join(STATE, 'relations.json'), JSON.stringify(relations));
writeFileSync(join(STATE, 'ninefold.json'), JSON.stringify(ninefold));
console.log(JSON.stringify({ nodes: Object.keys(nodes).length, relations: relations.length, determinations: ninefold.length }));
