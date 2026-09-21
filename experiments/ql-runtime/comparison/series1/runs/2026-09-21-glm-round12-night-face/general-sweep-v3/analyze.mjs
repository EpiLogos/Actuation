#!/usr/bin/env node
// v3 analyzer: L2 maps, stability across 3 full_text reps, function-type
// concentration/spread, cross-entry L2 distinguishability, v2 comparability.
import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const DIR = '/Users/admin/.cache/actuation/analysis/general-sweep-v3';
const V2DIR = '/Users/admin/.cache/actuation/analysis/general-sweep';
const GIVING = JSON.parse(readFileSync('/Users/admin/.cache/actuation/mef-giving.json', 'utf8'));
const meta = JSON.parse(readFileSync(join(DIR, 'corpus-meta.json'), 'utf8'));

const slotKeys = Object.fromEntries(GIVING.lenses.map((l) => [l.id, l.sublens.map((s) => `${s.slot} ${s.label}`)]));
const lensFn = Object.fromEntries(GIVING.lenses.map((l) => [l.id, l.function ?? 'select']));
const short = (k) => k.replace(/\s*\(.*\)$/, '');
const LENSES = GIVING.lenses.map((l) => l.id);
const ENTRIES = meta.map((m) => m.id);

const load = (dir, file) => JSON.parse(readFileSync(join(dir, file), 'utf8'));
const ft = (id, rep) => load(DIR, `c${id.slice(1).padStart(2, '0')}-full_text-${rep}.json`).reading;
const cell = (reading, lens) => reading.cells.find((c) => c.lens === lens);
const dist = (c) => c.distribution ?? {};
const argmax = (d) => Object.entries(d).sort((a, b) => b[1] - a[1])[0]?.[0] ?? null;

function entropy(d) {
  const vals = Object.values(d);
  const tot = vals.reduce((a, b) => a + b, 0);
  if (tot <= 0) return 0;
  let h = 0;
  for (const v of vals) { if (v > 0) { const p = v / tot; h -= p * Math.log2(p); } }
  return h; // bits; max log2(6) = 2.585
}

// adjacency for progression lenses: mass within +/-1 index of argmax vs farther
function adjacency(d) {
  const keys = slotKeys[Object.keys({})] ; // placeholder, real keys passed below
  return null;
}
function adjSpread(lensId, d) {
  const order = slotKeys[lensId];
  const am = argmax(d);
  const iAm = order.indexOf(am);
  if (iAm < 0) return null;
  let adj = 0, far = 0;
  for (const [k, v] of Object.entries(d)) {
    const i = order.indexOf(k);
    if (i < 0) continue;
    const disti = Math.abs(i - iAm);
    if (disti === 1) adj += v; else if (disti > 1) far += v;
  }
  return { adjacent: adj, farther: far };
}

// ---------- collect per-entry per-rep data ----------
const data = {};
for (const e of ENTRIES) {
  data[e] = {};
  for (const rep of [1, 2, 3]) {
    const r = ft(e, rep);
    data[e][rep] = Object.fromEntries(LENSES.map((l) => [l, cell(r, l)]));
  }
}

// ---------- a) L2 maps ----------
const l2maps = {};
for (const e of ENTRIES) {
  l2maps[e] = [1, 2, 3].map((rep) => {
    const d = dist(data[e][rep].L2);
    return { rep, dist: d, argmax: argmax(d) };
  });
}
// cross-entry distinguishability on rep-averaged L2 distributions
function avgDist(entry, lens) {
  const keys = slotKeys[lens];
  const out = {};
  for (const k of keys) out[k] = 0;
  for (const rep of [1, 2, 3]) { const d = dist(data[entry][rep][lens]); for (const k of keys) out[k] += (d[k] ?? 0) / 3; }
  return out;
}
const l2avg = Object.fromEntries(ENTRIES.map((e) => [e, avgDist(e, 'L2')]));
const l2pairs = [];
for (let i = 0; i < ENTRIES.length; i++) for (let j = i + 1; j < ENTRIES.length; j++) {
  const a = l2avg[ENTRIES[i]], b = l2avg[ENTRIES[j]];
  const l1 = slotKeys.L2.reduce((s, k) => s + Math.abs((a[k] ?? 0) - (b[k] ?? 0)), 0);
  l2pairs.push({ pair: `${ENTRIES[i]}-${ENTRIES[j]}`, l1 });
}
l2pairs.sort((x, y) => x.l1 - y.l1);

// how many slots carry >=10% / >=5% mass per entry (map flatness), rep-averaged
const l2spread = {};
for (const e of ENTRIES) {
  const d = l2avg[e];
  const vals = slotKeys.L2.map((k) => d[k] ?? 0).sort((x, y) => y - x);
  l2spread[e] = {
    ge10: vals.filter((v) => v >= 0.10).length,
    ge05: vals.filter((v) => v >= 0.05).length,
    top1: vals[0], top2: vals[1], top3: vals[2],
    entropy: entropy(d),
  };
}

// ---------- b) stability ----------
const stability = { all3_agree: 0, rep12_agree: 0, total: 0, per_lens: {}, disagreements: [] };
for (const l of LENSES) stability.per_lens[l] = { all3: 0, rep12: 0, n: 0 };
for (const e of ENTRIES) for (const l of LENSES) {
  const a1 = argmax(dist(data[e][1][l]));
  const a2 = argmax(dist(data[e][2][l]));
  const a3 = argmax(dist(data[e][3][l]));
  stability.total++;
  stability.per_lens[l].n++;
  const r12 = a1 === a2, all3 = r12 && a2 === a3;
  if (r12) { stability.rep12_agree++; stability.per_lens[l].rep12++; }
  if (all3) { stability.all3_agree++; stability.per_lens[l].all3++; }
  if (!all3) stability.disagreements.push({ entry: e, lens: l, fn: lensFn[l], rep1: a1, rep2: a2, rep3: a3 });
}
// v2 comparability (rep1 vs rep2, machinery form) from the v2 directory
let v2rep12 = 0, v2total = 0;
const v2dis = [];
try {
  for (const e of ENTRIES) for (const l of LENSES) {
    const r1 = load(V2DIR, `c${e.slice(1).padStart(2, '0')}-full_text-1.json`).reading;
    const r2 = load(V2DIR, `c${e.slice(1).padStart(2, '0')}-full_text-2.json`).reading;
    const a1 = argmax(dist(cell(r1, l))); const a2 = argmax(dist(cell(r2, l)));
    v2total++; if (a1 === a2) v2rep12++; else v2dis.push({ entry: e, lens: l, rep1: a1, rep2: a2 });
  }
} catch (err) { v2dis.push({ note: `v2 comparison incomplete: ${err.message}` }); }

// ---------- c) function-type concentration / spread ----------
const fnTypes = { exhaustive: [], locate: [], select: [] };
for (const l of LENSES) fnTypes[lensFn[l]].push(l);
const fnStats = {};
for (const [fn, lenses] of Object.entries(fnTypes)) {
  const cells = [];
  for (const e of ENTRIES) for (const l of lenses) for (const rep of [1, 2, 3]) {
    const d = dist(data[e][rep][l]);
    const rec = { entry: e, lens: l, rep, top1: Math.max(...Object.values(d)), entropy: entropy(d) };
    const s = adjSpread(l, d);
    if (s) { rec.adjacent = s.adjacent; rec.farther = s.farther; }
    cells.push(rec);
  }
  const mean = (xs) => xs.reduce((a, b) => a + b, 0) / xs.length;
  fnStats[fn] = {
    lenses,
    n_cells: cells.length,
    mean_top1: mean(cells.map((c) => c.top1)),
    mean_entropy_bits: mean(cells.map((c) => c.entropy)),
    mean_adjacent: cells.some((c) => c.adjacent !== undefined) ? mean(cells.filter((c) => c.adjacent !== undefined).map((c) => c.adjacent)) : null,
    mean_farther: cells.some((c) => c.farther !== undefined) ? mean(cells.filter((c) => c.farther !== undefined).map((c) => c.farther)) : null,
  };
}
// per-lens detail (rep-averaged) for the locate lenses and a few select contrasts
const perLens = {};
for (const l of LENSES) {
  const rows = ENTRIES.map((e) => {
    const d = avgDist(e, l);
    const s = adjSpread(l, d);
    return { entry: e, top1: Math.max(...slotKeys[l].map((k) => d[k] ?? 0)), entropy: entropy(d), adjacent: s?.adjacent ?? null, farther: s?.farther ?? null };
  });
  const mean = (xs) => xs.reduce((a, b) => a + b, 0) / xs.length;
  perLens[l] = {
    fn: lensFn[l],
    mean_top1: mean(rows.map((r) => r.top1)),
    mean_entropy: mean(rows.map((r) => r.entropy)),
    mean_adjacent: rows[0].adjacent !== null ? mean(rows.map((r) => r.adjacent)) : null,
    mean_farther: rows[0].farther !== null ? mean(rows.map((r) => r.farther)) : null,
  };
}

// ---------- resonant + squares for the readings section ----------
const resonant = {};
for (const e of ENTRIES) {
  const r = load(DIR, `c${e.slice(1).padStart(2, '0')}-resonant-1.json`).reading;
  resonant[e] = { selected: r.selected, top3: r.ranking.slice(0, 3) };
}
const squares = {};
for (const e of ENTRIES) {
  squares[e] = {};
  for (const mode of ['being', 'becoming', 'knowing']) {
    const r = load(DIR, `c${e.slice(1).padStart(2, '0')}-${mode}-1.json`).reading;
    squares[e][mode] = r.readings.map((x) => `${x.lens}@${x.at}=${short(x.strongest ?? '?')}`).join('; ');
  }
}

// ---------- run log ----------
const runlog = JSON.parse(readFileSync(join(DIR, 'run-log.json'), 'utf8'));
const wall = runlog.log.filter((x) => x.ok).map((x) => x.wall_ms);
const lat = ENTRIES.flatMap((e) => [1, 2, 3].map((r) => load(DIR, `c${e.slice(1).padStart(2, '0')}-full_text-${r}.json`).latency_ms));

const report = {
  l2maps: Object.fromEntries(ENTRIES.map((e) => [e, {
    per_rep: l2maps[e].map((m) => ({ rep: m.rep, argmax: m.argmax, dist: m.dist })),
    avg: l2avg[e], spread: l2spread[e],
  }])),
  l2_closest_pairs: l2pairs.slice(0, 6),
  l2_farthest_pairs: l2pairs.slice(-6),
  stability: {
    total: stability.total,
    all3_agree: stability.all3_agree,
    rep12_agree: stability.rep12_agree,
    rep12_rate: (stability.rep12_agree / stability.total).toFixed(4),
    all3_rate: (stability.all3_agree / stability.total).toFixed(4),
    disagreements: stability.disagreements,
    per_lens: stability.per_lens,
    v2_comparison: { total: v2total, rep12_agree: v2rep12, rate: (v2rep12 / v2total).toFixed(4), disagreements: v2dis },
  },
  fnStats, perLens,
  resonant, squares,
  run: { calls: runlog.total, ok: runlog.done, failed: runlog.failed, wall_min_ms: Math.min(...wall), wall_max_ms: Math.max(...wall), latency_min_ms: Math.min(...lat), latency_max_ms: Math.max(...lat) },
};
writeFileSync(join(DIR, 'computed-report.json'), JSON.stringify(report, null, 2));

// printed digest
console.log('=== a) L2 maps (rep-averaged, slot order = register order) ===');
for (const e of ENTRIES) {
  const d = l2avg[e];
  const map = slotKeys.L2.map((k) => `${short(k).replace('L2-', '')}:${(d[k] ?? 0).toFixed(2)}`).join('  ');
  console.log(`${e}  ${map}`);
}
console.log('\nL2 closest pairs:', report.l2_closest_pairs.map((p) => `${p.pair}(${p.l1.toFixed(2)})`).join(' '));
console.log('L2 farthest pairs:', report.l2_farthest_pairs.map((p) => `${p.pair}(${p.l1.toFixed(2)})`).join(' '));
console.log('\n=== b) stability ===');
console.log(`rep1=rep2: ${stability.rep12_agree}/${stability.total} = ${(stability.rep12_agree / stability.total * 100).toFixed(1)}%  (v2 baseline ${v2rep12}/${v2total} = ${(v2rep12 / v2total * 100).toFixed(1)}%)`);
console.log(`all-3 agree: ${stability.all3_agree}/${stability.total} = ${(stability.all3_agree / stability.total * 100).toFixed(1)}%`);
console.log('disagreements (non-all3):');
for (const d of stability.disagreements) console.log(` ${d.entry} ${d.lens} (${d.fn}): [${short(d.rep1)}] vs [${short(d.rep2)}] vs [${short(d.rep3)}]`);
console.log('\n=== c) function types ===');
for (const [fn, s] of Object.entries(fnStats)) {
  console.log(`${fn} [${s.lenses.join(',')}] n=${s.n_cells} top1=${s.mean_top1.toFixed(3)} entropy=${s.mean_entropy_bits.toFixed(3)}bits` + (s.mean_adjacent !== null ? ` adjacent=${s.mean_adjacent.toFixed(3)} farther=${s.mean_farther.toFixed(3)}` : ''));
}
console.log('\nper-lens (rep-averaged):');
for (const l of LENSES) {
  const p = perLens[l];
  console.log(` ${l} (${p.fn}): top1=${p.mean_top1.toFixed(3)} entropy=${p.mean_entropy.toFixed(3)}` + (p.mean_adjacent !== null ? ` adj=${p.mean_adjacent.toFixed(3)} far=${p.mean_farther.toFixed(3)}` : ''));
}
console.log('\n=== run ===');
console.log(JSON.stringify(report.run));
