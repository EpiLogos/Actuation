#!/usr/bin/env node
// Analysis pass over the general-sweep outputs. Prints a full computed report.
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

const DIR = '/Users/admin/.cache/actuation/analysis/general-sweep';
const meta = JSON.parse(readFileSync(join(DIR, 'corpus-meta.json'), 'utf8'));
const LENSES = ['L0', 'L1', 'L2', 'L3', 'L4', 'L5', "L0'", "L1'", "L2'", "L3'", "L4'", "L5'"];
const MODES = ['full_text', 'nameonly', 'resonant', 'being', 'becoming', 'knowing'];

const load = (nn, mode, rep) =>
  JSON.parse(readFileSync(join(DIR, `c${nn}-${mode}-${rep}.json`), 'utf8'));

const short = (s) => (s ?? '—').replace(/\s*\(\.\/\?\)/, '').replace(/\s*\(σοφία\)/, '').replace(/\s*\(ἐπί-λόγος\)/, '');
const nnOf = (id) => meta.find((m) => m.id === id).nn;

// ---------- per-entry data + stability + machinery-vs-nameonly ----------
const perEntry = {};
let stabAgree = 0, stabTot = 0, machAgree = 0, machTot = 0;
const diffExamples = [];
for (const m of meta) {
  const ft1 = load(m.nn, 'full_text', 1).reading;
  const ft2 = load(m.nn, 'full_text', 2).reading;
  const no1 = load(m.nn, 'nameonly', 1).reading;
  const c1 = Object.fromEntries(ft1.cells.map((c) => [c.lens, c]));
  const c2 = Object.fromEntries(ft2.cells.map((c) => [c.lens, c]));
  const cn = Object.fromEntries(no1.cells.map((c) => [c.lens, c]));
  const resonant = load(m.nn, 'resonant', 1).reading;
  const squares = {};
  for (const mode of ['being', 'becoming', 'knowing']) {
    squares[mode] = load(m.nn, mode, 1).reading.readings.map((r) => ({
      q: `${r.lens}@${r.at}`, strongest: short(r.strongest),
      top: r.distribution ? Object.entries(r.distribution).sort((a, b) => b[1] - a[1])[0] : null,
    }));
  }
  let agree = 0;
  const lensRows = [];
  for (const lens of LENSES) {
    const s1 = short(c1[lens].strongest), s2 = short(c2[lens].strongest), sn = short(cn[lens].strongest);
    if (s1 === s2) agree++;
    const same = s1 === sn;
    lensRows.push({ lens, rep1: s1, rep2: s2, nameonly: sn, same, p1: c1[lens].distribution, p2: c2[lens].distribution, pn: cn[lens].distribution });
    stabTot++; if (s1 === s2) stabAgree++;
    machTot++; if (same) machAgree++;
    if (!same) diffExamples.push({ entry: m.id, title: m.title, lens, rep1: s1, nameonly: sn, p1: c1[lens].distribution, pn: cn[lens].distribution });
  }
  perEntry[m.id] = {
    title: m.title, subject: m.subject, lensRows, agree,
    resonantTop3: resonant.ranking.slice(0, 3).map((r) => `${r.lens} ${r.name} ${r.mass.toFixed(2)}`),
    squares,
  };
}

// ---------- differentiation: pairwise signature similarity ----------
const sig = (id) => perEntry[id].lensRows.map((r) => r.rep1);
const ids = meta.map((m) => m.id);
const pairs = [];
for (let i = 0; i < ids.length; i++) for (let j = i + 1; j < ids.length; j++) {
  const a = sig(ids[i]), b = sig(ids[j]);
  const same = a.filter((s, k) => s === b[k]).length;
  pairs.push({ pair: `${ids[i]}-${ids[j]}`, same });
}
pairs.sort((x, y) => y.same - x.same);

// ---------- per-lens slot histograms across entries (anomaly check) ----------
const hist = {};
for (const lens of LENSES) {
  const h = {};
  for (const id of ids) { const s = sig(id)[LENSES.indexOf(lens)]; h[s] = (h[s] || 0) + 1; }
  hist[lens] = Object.entries(h).sort((a, b) => b[1] - a[1]);
}

// ---------- distribution sharpness / degeneracy ----------
let sharpSum = 0, sharpN = 0, degen = 0;
for (const id of ids) for (const r of perEntry[id].lensRows) {
  if (!r.p1) { degen++; continue; }
  const vals = Object.values(r.p1);
  const max = Math.max(...vals);
  const sum = vals.reduce((a, b) => a + b, 0);
  sharpSum += max / sum; sharpN++;
  if (max === 0) degen++;
}

// ---------- print report ----------
console.log('=== PER ENTRY ===');
for (const m of meta) {
  const e = perEntry[m.id];
  console.log(`\n## ${m.id}. ${m.title}`);
  console.log(`Core move (from subject tail): ${m.subject.split(/Core move:/)[1]?.replace(/\s+/g, ' ').trim()}`);
  for (const r of e.lensRows) {
    console.log(`  ${r.lens.padEnd(4)} rep1=${r.rep1.padEnd(26)} rep2=${(r.rep2 === r.rep1 ? '=' : r.rep2).padEnd(26)} nameonly=${r.nameonly}${r.same ? '' : '  <DIFF>'}`);
  }
  console.log(`  stability: ${e.agree}/12 | resonant top3: ${e.resonantTop3.join(' | ')}`);
  for (const [mode, rows] of Object.entries(e.squares)) {
    console.log(`  ${mode}: ${rows.map((r) => `${r.q}=${r.strongest}`).join('; ')}`);
  }
}

console.log('\n=== STABILITY (full_text rep1 vs rep2) ===');
for (const m of meta) console.log(`${m.id}: ${perEntry[m.id].agree}/12`);
console.log(`OVERALL: ${stabAgree}/${stabTot} = ${(100 * stabAgree / stabTot).toFixed(1)}%`);

console.log('\n=== MACHINERY vs NAME-ONLY ===');
for (const m of meta) console.log(`${m.id}: ${perEntry[m.id].lensRows.filter((r) => r.same).length}/12`);
console.log(`OVERALL: ${machAgree}/${machTot} = ${(100 * machAgree / machTot).toFixed(1)}%`);
console.log(`differing lens-cells: ${diffExamples.length}`);
for (const d of diffExamples) {
  const top1 = Object.entries(d.p1 || {}).sort((a, b) => b[1] - a[1]).slice(0, 2).map(([k, v]) => `${short(k)}=${v}`).join(', ');
  const topn = Object.entries(d.pn || {}).sort((a, b) => b[1] - a[1]).slice(0, 2).map(([k, v]) => `${short(k)}=${v}`).join(', ');
  console.log(`  ${d.entry} ${d.lens}: machinery=${d.rep1} (${top1}) vs nameonly=${d.nameonly} (${topn})`);
}

console.log('\n=== DIFFERENTIATION ===');
console.log('most alike pairs (same strongest slot count of 12):');
for (const p of pairs.slice(0, 6)) console.log(`  ${p.pair}: ${p.same}/12`);
console.log('most separated pairs:');
for (const p of pairs.slice(-6).reverse()) console.log(`  ${p.pair}: ${p.same}/12`);
const distinctPerEntry = meta.map((m) => new Set(sig(m.id)).size);
console.log('distinct slots used per entry (of 12):', distinctPerEntry.join(','));

console.log('\n=== PER-LENS HISTOGRAMS (rep1 strongest across 12 entries) ===');
for (const lens of LENSES) {
  console.log(`${lens}: ${hist[lens].map(([s, n]) => `${s}x${n}`).join(', ')}`);
}

console.log('\n=== ANOMALY CHECKS ===');
console.log(`mean top-probability share (rep1): ${(sharpSum / sharpN).toFixed(3)} over ${sharpN} cells; zero-mass cells: ${degen}`);
// identical distribution repetition across entries
const distSig = {};
for (const id of ids) for (const r of perEntry[id].lensRows) {
  const k = r.lens + '|' + JSON.stringify(r.p1);
  distSig[k] = (distSig[k] || 0) + 1;
}
const repeats = Object.entries(distSig).filter(([, n]) => n > 1);
console.log(`identical (lens,distribution) repeats across entries: ${repeats.length}`);
for (const [k, n] of repeats.slice(0, 10)) console.log(`  x${n}: ${k.slice(0, 110)}`);
