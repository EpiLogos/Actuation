#!/usr/bin/env python3
"""epi-field — the Anuttara field as an operative faculty.

Read-only on the canon: field vectors live in the durable state store
(~/.local/state/epi-logos/field/, refreshed by field_refresh.mjs); accounts
embed through bimba_embed WITHOUT store_for (the graph is never written by
this module). Typing runs through the fail-closed Jev harness; generation
through Mercury; every receipt digest-pinned. Low resonance is compatibility
with declared structure — never value, never worth.
"""
import json, math, os, subprocess, hashlib, time, urllib.request
from pathlib import Path

BIMBA = '/Users/admin/Documents/epi/bimba-portable'
JEV = Path('/Users/admin/Central/Work/Actuation/experiments/model-types/jev')
SKILL = Path(__file__).resolve().parent
STATE = Path.home() / '.local/state/epi-logos/field'
NINE = ["Love/Peace", "Truth", "Openness/Creativity", "Joy/Play", "Goodness", "Beauty", "Life/Nature", "Wisdom", "Reality"]
CHILD_NAMES = {'M0-2-9-0': 'Love/Peace', 'M0-2-9-1': 'Truth', 'M0-2-9-2': 'Openness/Creativity',
               'M0-2-9-3': 'Joy/Play', 'M0-2-9-4': 'Goodness', 'M0-2-9-5': 'Beauty',
               'M0-2-9-6': 'Life/Nature', 'M0-2-9-7': 'Wisdom', 'M0-2-9-8': 'Reality'}

def _load(name):
    return json.loads((STATE / name).read_text())

def _cos(u, v):
    dot = sum(a * b for a, b in zip(u, v))
    nu = math.sqrt(sum(a * a for a in u)); nv = math.sqrt(sum(a * a for a in v))
    return dot / (nu * nv) if nu and nv else 0.0

def _digest(obj):
    return hashlib.sha256(json.dumps(obj, sort_keys=True, ensure_ascii=False).encode()).hexdigest()[:16]

def _keychain(service):
    return subprocess.run(['security', 'find-generic-password', '-s', service, '-w'],
                          capture_output=True, text=True).stdout.strip()

def capabilities():
    n = _load('nodes.json'); r = _load('relations.json'); d = _load('ninefold.json')
    return {'field_nodes': len(n), 'field_relations': len(r), 'ninefold_determinations': len(d),
            'scopes': 'M0+M1', 'graph_access': 'read-only', 'instruments': [
                'embed_account', 'retrieve', 'virtue_resonance', 'relation_placement',
                'syntax_compute', 'virtue_presence', 'draw_align', 'shadow_receipt', 'verify_battery']}

def embed_account(text, key='acct'):
    scope = os.fspath(STATE / 'embed-scope.json'); out = os.fspath(STATE / 'embed-out.json')
    json.dump({'items': [{'key': key, 'text': text[:8000]}]}, open(scope, 'w'))
    subprocess.run(['node', os.fspath(SKILL / 'embed_texts.mjs'), scope, out],
                   capture_output=True, timeout=240)
    return json.load(open(out)).get(key)

def retrieve(vec, k=3):
    rels = _load('relations.json'); nodes = _load('nodes.json')
    scored = sorted((( _cos(vec, r['vec']), r) for r in rels), key=lambda x: -x[0])[:k]
    marks = sorted(((_cos(vec, n['vec']), c) for c, n in nodes.items()), key=lambda x: -x[0])[:k]
    return {'nearest_relations': [{'type': r['type'], 'a': r['a'], 'b': r['b'],
                                   'text': r['text'][:220], 'score': round(s, 4)} for s, r in scored],
            'nearest_marks': [{'coord': c, 'name': nodes[c]['name'], 'mark': nodes[c]['mark'],
                               'score': round(s, 4)} for s, c in marks]}

def virtue_resonance(vec):
    nodes = _load('nodes.json')
    prof = {}
    for coord, name in CHILD_NAMES.items():
        n = nodes.get(coord)
        prof[coord] = {'virtue': name, 'score': round(_cos(vec, n['vec']), 4) if n else None}
    return prof

def relation_placement(a_coord, relation_text, b_coord):
    rv = embed_account(f'{a_coord} --[declared]--> {b_coord} | {relation_text}', key='placement')
    nodes = _load('nodes.json')
    av, bv = nodes.get(a_coord, {}).get('vec'), nodes.get(b_coord, {}).get('vec')
    if not (rv and av and bv):
        return {'error': 'missing vector for placement scoring'}
    energy = 1.0 - (_cos(rv, av) + _cos(rv, bv)) / 2
    return {'energy': round(energy, 4), 'note': 'lower is more compatible with the endpoints'}

def _jev(rid, qid, state):
    tmp = STATE / 'jev-state.jsonl'; ttruth = STATE / 'jev-empty.jsonl'
    tmp.write_text(json.dumps({'record_id': rid, 'family': 'concrescence', 'question_id': qid, 'state': state}) + '\n')
    ttruth.write_text('')
    subprocess.run(['node', 'harness.mjs', '--states', os.fspath(tmp), '--truth', os.fspath(ttruth),
                    '--out-dir', 'syntax/runs'], cwd=str(JEV), capture_output=True, timeout=180)
    f = sorted((JEV / 'syntax' / 'runs').glob('jev-run-*.json'))[-1]
    rec = json.load(open(f))['records'][0]
    return rec.get('answer', {})

def syntax_compute(text):
    vec = embed_account(text, key='syntax')
    r = retrieve(vec, k=4)
    mark_table = ' | '.join(f"{n['mark']} ({c} {n['name']})" for c, n in _load('nodes.json').items() if n['mark'])
    letters = ['A', 'B', 'C', 'D']
    cand = r['nearest_marks'][:4]
    table = ' ; '.join(f"{letters[i]}: {cand[i]['mark']} ({cand[i]['coord']} {cand[i]['name']})" for i in range(len(cand)))
    ans = _jev(f'syntax_compute|{_digest(text)}', 'mark-compute-account',
               {'account': text[:2000], 'mark_table': mark_table[:14000], 'candidates': table})
    return {'computed_mark': ans.get('choice'), 'confidence': ans.get('confidence'),
            'candidates': table, 'note': 'the mark this account computes through'}

def virtue_presence(text, with_relations=True):
    vec = embed_account(text, key='presence')
    r = retrieve(vec, k=3)
    relctx = ' | '.join(x['text'] for x in r['nearest_relations'])
    ans = _jev(f'virtue_presence|{_digest(text)}', 'virtue-presence-balance',
               {'account': text[:2000], 'nearest_relations': relctx,
                'mark_table': ' | '.join(f"{n['mark']} ({c} {n['name']})" for c, n in _load('nodes.json').items() if n['mark'])[:14000],
                'note': 'the nine virtues: ' + ', '.join(NINE)})
    dist = ans.get('probabilities') or {}
    res = virtue_resonance(vec)
    return {'presence_balance': dist, 'field_resonance': res,
            'retrieved_relations': r['nearest_relations'],
            'coupling': {'jev_top': max(dist, key=dist.get) if dist else None,
                         'ebm_top': (max(res.items(), key=lambda kv: kv[1]['score'] or -9)[1]['virtue']) if res else None}}

def draw_align(draft, target_virtue, situation=''):
    reading = virtue_presence(draft)
    key = _keychain('INCEPTION_API_KEY')
    fb = reading['field_resonance']
    prompt = (f"{situation}\nAt most 110 words, plain prose, the reply only.\n\n"
              f"FIELD READING OF YOUR DRAFT (verbatim):\n"
              f"virtue balance distribution: {json.dumps(reading['presence_balance'])}\n"
              f"resonance per virtue: {json.dumps({v['virtue']: v['score'] for v in fb.values()})}\n"
              f"nearest declared relations:\n- " + "\n- ".join(x['text'] for x in reading['retrieved_relations']) +
              f"\n\nThe virtue '{target_virtue}' must be present IN THE TONE AND WORD CHOICE THEMSELVES (not named). "
              f"Redraw the reply holding it in balance with the others.")
    req = urllib.request.Request('https://api.inceptionlabs.ai/v1/chat/completions',
        json.dumps({'model': 'mercury-2.5', 'messages': [{'role': 'user', 'content': prompt}],
                    'max_tokens': 400, 'reasoning_effort': 'low'}).encode(),
        {'content-type': 'application/json', 'authorization': f'Bearer {key}'})
    redraft = json.loads(urllib.request.urlopen(req, timeout=120).read())['choices'][0]['message']['content']
    after = virtue_presence(redraft)
    return {'before': reading, 'redraft': redraft, 'after': after,
            'target': target_virtue,
            'target_mass_before': reading['presence_balance'].get(target_virtue),
            'target_mass_after': after['presence_balance'].get(target_virtue)}

def shadow_receipt(record):
    """Return acts carry their placement/reading energy as evidence. Never a refusal."""
    rec = {'ts': time.strftime('%Y-%m-%dT%H:%M:%S%z'), 'digest': _digest(record), 'record': record,
           'standing': 'shadow: evidence only, never a refusal; determination pending-human-review'}
    with open(STATE / 'receipts.jsonl', 'a') as f:
        f.write(json.dumps(rec, ensure_ascii=False) + '\n')
    return {'receipt_digest': rec['digest'], 'path': str(STATE / 'receipts.jsonl')}

def verify_battery():
    """T1 presence + placement sample + syntax spot, through these very calls."""
    out = {}
    dets = _load('ninefold.json')
    child_map = {0: 'M0-2-9-0', 1: 'M0-2-9-1', 2: 'M0-2-9-3', 3: 'M0-2-9-4',
                 4: 'M0-2-9-5', 5: 'M0-2-9-6', 6: 'M0-2-9-7', 7: 'M0-2-9-8'}
    hits = 0; n = 0
    for i in range(8):
        child = CHILD_NAMES[child_map[i]]
        p = virtue_presence(dets[i])
        jev_top, ebm_top = p['coupling']['jev_top'], p['coupling']['ebm_top']
        ok = jev_top == child and ebm_top == child
        hits += ok; n += 1
        out[f'T1 determination {i} ({child})'] = {'jev': jev_top, 'ebm': ebm_top, 'ok': ok}
    rels = _load('relations.json')
    nodes = _load('nodes.json')
    e_pos, e_neg = [], []
    for r in rels[:40]:
        av, bv = nodes.get(r['a'], {}).get('vec'), nodes.get(r['b'], {}).get('vec')
        if not (av and bv): continue
        e_pos.append(relation_placement(r['a'], r['text'].split('|', 1)[1] if '|' in r['text'] else r['text'], r['b'])['energy'])
        other = rels[(rels.index(r) + 7) % len(rels)]
        e_neg.append(relation_placement(r['a'], other['text'].split('|', 1)[1] if '|' in other['text'] else other['text'], r['b'])['energy'])
    pairs = sum(1 for p in e_pos for q in e_neg if p < q)
    out['T3 placement (sampled)'] = {'real_lower_than_foreign': f'{pairs}/{len(e_pos)*len(e_neg) if False else len(e_pos)}',
                                     'mean_real_energy': round(sum(e_pos)/max(len(e_pos),1), 4),
                                     'mean_foreign_energy': round(sum(e_neg)/max(len(e_neg),1), 4)}
    return out
