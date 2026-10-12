#!/usr/bin/env python3
"""Corpus-scale virtue readings through the verified Jev instrument.

Takes real loop determinations (determination-warranted | intact variant) from
the ql-form concrescence corpus, composes the verified virtue-presence-balance
state for each circuit (account + nearest 3 field relations + mark table +
ninefold note), runs the Jev harness once over the batch, and writes the
per-circuit nine-way virtue distributions to hybrid-loop evidence.

stdlib only. Embedding goes through the epi-field embed script (node, MCP).
Fails closed: if embedding fails twice, the run aborts honestly — no
substitution, no synthetic vectors.
"""

import glob
import json
import math
import os
import subprocess
import sys
import time
from datetime import datetime, timezone

ROOT = '/Users/admin/Central/Work/Actuation/experiments/model-types'
JEV_DIR = f'{ROOT}/jev'
CORPUS = f'{ROOT}/ql-form/concrescence/states.jsonl'
FIELD = os.path.expanduser('~/.local/state/epi-logos/field')
EMBED = '/Users/admin/Central/Work/Actuation/experiments/ql-runtime/prime/skills/epi-field/embed_texts.mjs'
SCOPE = '/tmp/corpus-embed-scope.json'
EMBOUT = '/tmp/corpus-embed-out.json'
STATES_OUT = '/tmp/corpus-jev-states.jsonl'
TRUTH_OUT = '/tmp/corpus-jev-empty.jsonl'
EVIDENCE = f'{ROOT}/hybrid-loop/evidence/corpus-virtue-readings.json'
CAP = 13
NOTE = 'the nine virtues: Love/Peace, Truth, Openness/Creativity, Joy/Play, Goodness, Beauty, Life/Nature, Wisdom, Reality'
VIRTUES = ['Love/Peace', 'Truth', 'Openness/Creativity', 'Joy/Play', 'Goodness', 'Beauty', 'Life/Nature', 'Wisdom', 'Reality']


def cos(a, b):
    d = na = nb = 0.0
    for x, y in zip(a, b):
        d += x * y
        na += x * x
        nb += y * y
    return d / math.sqrt(na * nb) if na > 0.0 and nb > 0.0 else 0.0


# ---- a. load corpus: determination-warranted, intact variant, dedupe by circuit
def load_corpus():
    circuits, seen, skipped = [], set(), 0
    with open(CORPUS) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            rec = json.loads(line)
            if not rec['record_id'].startswith('determination-warranted|') or rec.get('variant') != 'intact':
                continue
            st = rec.get('state') or {}
            text = ((st.get('determination') or {}).get('synthesis') or '').strip()
            circuit_id = st.get('circuit_id')
            if not text or not circuit_id or circuit_id in seen:
                skipped += 1
                continue
            seen.add(circuit_id)
            circuits.append({'circuit_id': circuit_id, 'task_id': st.get('task_id'), 'text': text})
            if len(circuits) >= CAP:
                break
    if not circuits:
        sys.exit('FAIL: no determination-warranted intact records found in corpus')
    print(f'corpus: {len(circuits)} unique circuits (skipped {skipped} records)')
    return circuits


# ---- b. embed via epi-field (retry once, then fail closed)
def embed_all(items):
    with open(SCOPE, 'w') as f:
        json.dump({'items': items}, f)
    for attempt in (1, 2):
        print(f'embed attempt {attempt}: {len(items)} texts via epi-field (MCP startup ~60-90s)...')
        try:
            r = subprocess.run(['node', EMBED, SCOPE, EMBOUT], capture_output=True, text=True, timeout=420)
        except subprocess.TimeoutExpired:
            print(f'  attempt {attempt}: timeout after 420s', file=sys.stderr)
            continue
        if r.returncode == 0:
            with open(EMBOUT) as f:
                out = json.load(f)
            if all(out.get(it['key']) for it in items):
                print(f'  ok: {sum(1 for v in out.values() if v)} vectors')
                return out
            missing = [it['key'] for it in items if not out.get(it['key'])]
            print(f'  attempt {attempt}: {len(missing)} vectors missing ({missing[:3]}...)', file=sys.stderr)
        else:
            print(f'  attempt {attempt}: rc={r.returncode} stderr={r.stderr[-300:]}', file=sys.stderr)
    sys.exit('FAIL: embed script failed after one retry; refusing to substitute synthetic vectors')


def compose_state(text, vec, relations, nodes_marked):
    rels = sorted(relations, key=lambda r: -cos(vec, r['vec']))[:3]
    nearest = ' | '.join(r['text'][:220] for r in rels)
    mark_table = ' | '.join(f"{n['mark']} ({cid} {n['name']})" for cid, n in nodes_marked.items())[:14000]
    return {
        'account': text[:2000],
        'nearest_relations': nearest,
        'mark_table': mark_table,
        'note': NOTE,
    }, [r['eid'] for r in rels]


def main():
    t0 = time.time()
    circuits = load_corpus()

    relations = json.load(open(f'{FIELD}/relations.json'))
    nodes = json.load(open(f'{FIELD}/nodes.json'))
    nodes_marked = {c: n for c, n in nodes.items() if n.get('mark')}
    print(f'field store: {len(relations)} relations, {len(nodes_marked)} marked nodes')

    vecs = embed_all([{'key': c['circuit_id'], 'text': c['text']} for c in circuits])

    # ---- c. write one state line per circuit
    with open(STATES_OUT, 'w') as f:
        for c in circuits:
            state, rel_eids = compose_state(c['text'], vecs[c['circuit_id']], relations, nodes_marked)
            c['rel_eids'] = rel_eids
            f.write(json.dumps({
                'record_id': f"virtue-presence-balance|{c['circuit_id']}",
                'family': 'concrescence',
                'question_id': 'virtue-presence-balance',
                'state': state,
            }) + '\n')
    open(TRUTH_OUT, 'w').close()
    print(f'states written: {STATES_OUT} ({os.path.getsize(STATES_OUT)} bytes)')

    # ---- d. run the harness once (cwd = jev dir; keychain credential read fresh)
    before = set(glob.glob(f'{JEV_DIR}/syntax/runs/jev-run-*.json'))
    print('running jev harness over batch...')
    r = subprocess.run(
        ['node', 'harness.mjs', '--states', STATES_OUT, '--truth', TRUTH_OUT,
         '--out-dir', 'syntax/runs', '--limit', '50'],
        cwd=JEV_DIR, capture_output=True, text=True, timeout=900)
    print(r.stdout[-2000:])
    if r.returncode != 0:
        sys.exit(f'FAIL: harness exited {r.returncode}\nstderr: {r.stderr[-2000:]}')

    new_runs = [p for p in glob.glob(f'{JEV_DIR}/syntax/runs/jev-run-*.json') if p not in before]
    if not new_runs:
        sys.exit('FAIL: harness wrote no new run record')
    run_path = max(new_runs, key=os.path.getmtime)
    run = json.load(open(run_path))
    print(f'run record: {run_path} ({len(run["records"])} records)')

    # ---- e. per-circuit extraction
    by_id = {rec['record_id']: rec for rec in run['records']}
    results = []
    for c in circuits:
        rec = by_id.get(f"virtue-presence-balance|{c['circuit_id']}")
        if not rec:
            print(f'WARN: no run record for {c["circuit_id"]}', file=sys.stderr)
            continue
        probs = rec['answer'].get('probabilities') or {}
        ranked = sorted(probs.items(), key=lambda kv: -kv[1])
        argmax = rec['answer'].get('choice') or (ranked[0][0] if ranked else None)
        mass_gap = round(ranked[0][1] - ranked[1][1], 6) if len(ranked) > 1 else None
        results.append({
            'circuit_id': c['circuit_id'],
            'task_id': c['task_id'],
            'argmax_virtue': argmax,
            'probabilities': {v: probs.get(v, 0.0) for v in VIRTUES},
            'top3': [{'virtue': k, 'p': p} for k, p in ranked[:3]],
            'mass_gap': mass_gap,
            'confidence': rec['answer'].get('confidence'),
            'latency_ms': rec.get('latency_ms'),
            'nearest_relation_eids': c['rel_eids'],
        })

    tally = {}
    for res in results:
        tally[res['argmax_virtue']] = tally.get(res['argmax_virtue'], 0) + 1

    # ---- f. evidence record
    evidence = {
        'schema': 'actuation.model-types-corpus-virtue-readings/v1',
        'created': datetime.now(timezone.utc).isoformat(),
        'provenance': {
            'promotion': 'none',
            'reading_class': 'semantic-stochastic',
            'instrument': 'jev virtue-presence-balance (choice over nine virtues)',
            'model_ref': run.get('model_ref'),
            'harness_run': os.path.relpath(run_path, JEV_DIR),
            'corpus': os.path.relpath(CORPUS, ROOT),
            'corpus_filter': 'record_id startswith determination-warranted| AND variant==intact; deduped by circuit_id',
            'field_store': FIELD,
            'state_composition': 'verified pipeline: account[:2000] + nearest 3 relation texts[:220] + full mark table[:14000] + ninefold note',
        },
        'honest_limits': [
            'no ground truth: these are instrument readings of real loop determinations, run against an empty truth file (correct=null, brier uninformative)',
            'ninefold determinations are offset by the instrument: run records carry determination pending-human-review (readings, never the determination)',
            'n<=13: capped sample of unique circuits, semantic-stochastic so single readings are not stable estimates',
        ],
        'circuits': results,
        'corpus_summary': {
            'n_circuits': len(results),
            'argmax_tally': tally,
            'mean_top_mass': round(sum(max(r['probabilities'].values()) for r in results) / len(results), 6) if results else None,
            'mean_mass_gap': round(sum(r['mass_gap'] for r in results) / len(results), 6) if results else None,
        },
        'usage': run.get('usage'),
    }
    os.makedirs(os.path.dirname(EVIDENCE), exist_ok=True)
    with open(EVIDENCE, 'w') as f:
        json.dump(evidence, f, indent=2)
    print(f'evidence written: {EVIDENCE} ({time.time() - t0:.0f}s total)')

    print('\ncircuit | task | argmax | top3 | gap')
    for res in results:
        top3 = ', '.join(f"{t['virtue']} {t['p']:.2f}" for t in res['top3'])
        print(f"{res['circuit_id']} | {res['task_id']} | {res['argmax_virtue']} | {top3} | {res['mass_gap']}")
    print(f"\ntally: {tally}")


if __name__ == '__main__':
    main()
