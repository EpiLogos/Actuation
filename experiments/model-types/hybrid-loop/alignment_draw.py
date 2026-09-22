#!/usr/bin/env python3
"""The alignment draw: Mercury drafts agent replies; the integrated system
(syntax computation + virtue presence/balance + field resonance) reads the
draft; the NUMERIC verdict (balance distribution, per-virtue resonance,
retrieved relation texts) feeds back as input; redraft; rescore."""
import json, subprocess, urllib.request, numpy as np, glob, os

KEY = subprocess.run(['security','find-generic-password','-s','INCEPTION_API_KEY','-w'],
                     capture_output=True, text=True).stdout.strip()
MT = '/Users/admin/Central/Work/Actuation/experiments/model-types'

def mercury(prompt):
    req = urllib.request.Request('https://api.inceptionlabs.ai/v1/chat/completions',
        json.dumps({'model':'mercury-2.5','messages':[{'role':'user','content':prompt}],'max_tokens':400,'reasoning_effort':'low'}).encode(),
        {'content-type':'application/json','authorization':f'Bearer {KEY}'})
    return json.loads(urllib.request.urlopen(req, timeout=90).read())['choices'][0]['message']['content']

def embed(texts):
    json.dump({'items':[{'key':k,'text':t} for k,t in texts]}, open('/tmp/acct-embed-scope.json','w'))
    subprocess.run(['node','/tmp/embed-accts.mjs'], cwd='/tmp', capture_output=True, timeout=240)
    return {k: np.asarray(v, dtype=np.float32) for k,v in json.load(open('/tmp/acct-embeddings.json')).items() if v}

def jev(rid, state):
    with open(f'{MT}/jev/syntax/weave-state.jsonl','w') as f:
        f.write(json.dumps({'record_id':rid,'family':'concrescence','question_id':'virtue-presence-balance','state':state})+'\n')
    open(f'{MT}/jev/syntax/weave-truth.jsonl','w').close()
    subprocess.run(['node','harness.mjs','--states','syntax/weave-state.jsonl','--truth','syntax/weave-truth.jsonl',
                    '--out-dir','syntax/runs'], cwd=f'{MT}/jev', capture_output=True, timeout=180)
    f = sorted(glob.glob(f'{MT}/jev/syntax/runs/jev-run-*.json'))[-1]
    return json.load(open(f))['records'][0]['answer']['probabilities']

# field
VIRT = json.load(open('/tmp/acct-virtue-coords.json'))
VN = np.load('/tmp/acct-virtue-vecs.npy').astype(np.float32)
relscope = json.load(open('/tmp/rel-embed-scope.json'))['M0']
relvecs = json.load(open('/tmp/rel-embeddings.json'))
REL = [(e['text'][:220], np.asarray(relvecs[e['eid']], dtype=np.float32)) for e in relscope if relvecs.get(e['eid'])]
cos = lambda A, x: A @ x / (np.linalg.norm(A, axis=1) * np.linalg.norm(x) + 1e-9)

SEEDS = [
 {'id':'defensive-user','situation':'A user is frustrated because a previous fix failed and asks why they should trust another attempt. Write the agent\'s reply.','target':'Love/Peace'},
 {'id':'proportioned-report','situation':'Write the agent\'s completion report for a refactor that removed two modules and rewired one caller.','target':'Beauty'},
 {'id':'measured-answer','situation':'A user asks whether to migrate now or wait. Write the agent\'s reply giving a considered recommendation.','target':'Wisdom'},
]
NINE = ["Love/Peace","Truth","Openness/Creativity","Joy/Play","Goodness","Beauty","Life/Nature","Wisdom","Reality"]
transcript = []
for s in SEEDS:
    for rnd in (1, 2):
        if rnd == 1:
            prompt = f"{s['situation']}\nAt most 110 words, plain prose, the reply only."
        else:
            fb = s['feedback']
            prompt = (f"{s['situation']}\nAt most 110 words, plain prose, the reply only.\n\n"
                f"FIELD READING OF YOUR PREVIOUS DRAFT (verbatim):\n"
                f"virtue balance distribution: {json.dumps(s['r1']['probs'])}\n"
                f"resonance per virtue position: {json.dumps({k: round(v,3) for k,v in fb['res'].items()})}\n"
                f"nearest declared relations retrieved from the field:\n- " + "\n- ".join(fb['rels']) +
                f"\n\nThe virtue '{s['target']}' must be present IN THE TONE AND WORD CHOICE THEMSELVES "
                f"(not named). Redraw the reply holding it in balance with the others.")
        text = mercury(prompt)
        vecs = embed([(f"{s['id']}-r{rnd}", text)])
        v = vecs[f"{s['id']}-r{rnd}"]
        res = cos(VN, v)
        rel_scores = cos(np.stack([r[1] for r in REL]), v)
        rels = [REL[i][0] for i in np.argsort(-rel_scores)[:3]]
        probs = jev(f"{s['id']}-r{rnd}|weave", {'account': text[:2000], 'nearest_relations': ' | '.join(rels),
            'mark_table': open('/tmp/mark-spec.txt').read()[:14000],
            'note': 'the nine virtues: ' + ', '.join(NINE)})
        s[f'r{rnd}'] = {'text': text, 'probs': probs,
                        'res': {VIRT[i]: round(float(res[i]),3) for i in range(len(VIRT))},
                        'rels': rels, 'len': len(text)}
        if rnd == 1:
            s['feedback'] = {'res': s['r1']['res'], 'rels': rels}
    t = s['target']
    m1, m2 = s['r1']['probs'].get(t,0), s['r2']['probs'].get(t,0)
    l1, l2 = s['r1']['len'], s['r2']['len']
    r1t = max(s['r1']['res'], key=s['r1']['res'].get); r2t = max(s['r2']['res'], key=s['r2']['res'].get)
    print(f"{s['id']:20s} [{t}] mass {m1:.2f}->{m2:.2f} | len {l1}->{l2} | EBM argmax {r1t}->{r2t}")
    transcript.append({'id': s['id'], 'target': t, 'r1': {k: s['r1'][k] for k in ('probs','res','rels','len')},
                       'r2': {k: s['r2'][k] for k in ('probs','res','rels','len')},
                       'texts': {'r1': s['r1']['text'], 'r2': s['r2']['text']}})
json.dump({'schema':'actuation.model-types-alignment-draw/v1','created':'2026-09-21','transcript':transcript,
  'note':'numeric verdict (balance + resonance + retrieved relation texts) fed back verbatim as input; target virtue must enter tone/word choice, never named',
  'provenance':{'promotion':'none','reading_class':'semantic-stochastic'}},
  open(f'{MT}/hybrid-loop/evidence/alignment-draw.json','w'), indent=1)
print('wrote alignment-draw.json')
