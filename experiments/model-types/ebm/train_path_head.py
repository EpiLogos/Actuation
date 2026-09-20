#!/usr/bin/env python3
"""E-MT EBM Phase 3 — joint traversal-level energy head (the real path test).

R3 previewed path scoring by SUMMING pairwise hop energies. This is the actual
test: one energy head over the whole traversal a -> b -> c as a single object,
with features that only exist at path level — the chord (a,c) similarity
alongside the two hops, common prefix across all three coordinates, per-hop
directed structure, and both hop types jointly. The question a loop gate asks
is exactly this shape: is THIS transition coherent with the field?

Corruption ladder (seeded 20260917; head seed 20260918; split by traversal
digest mod 5 — no traversal or corruption of it is both trained and evaluated):
  endpoint-rewire    c replaced by a different-root node (coarse)
  hard-rewire        c replaced by a same-root node
  sibling-rewire     c replaced by a sibling of c (the fine-grained frontier)
  mid-rewire         b replaced — the traversal no longer exists (continuity)
  direction-reversal c -> b -> a (direction)

Reported per operator, held-out: joint head vs the summed pairwise v2b head
(retrained inline, identical protocol) vs summed cosine vs summed combined —
so the actual question "does joint scoring beat summed scoring" is answered
directly, with calibration for the joint head.

Standing law: semantic-stochastic, promotion none; low energy is compatibility
with learned structure, never value or correctness.
"""

import json
import sys
import time
from collections import defaultdict
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent))
from field_probe import lineage_energy, parse_coord, rank_auc, root_of  # noqa: E402
from train_energy_head import h, edge_id, seg_prefix  # noqa: E402
from train_energy_head_v2 import (  # noqa: E402
    FeatureSpace,
    MLP2,
    PCA_DIM,
    calib_readout,
    fit_pca,
    load_graph,
    type_hash,
    profile_hash,
    PROFILE_HASH_DIMS,
    TYPE_HASH_DIMS,
    FAMILIES,
)
from field_probe import parent_key  # noqa: E402

OPS = ["endpoint-rewire", "hard-rewire", "sibling-rewire", "mid-rewire", "direction-reversal"]


def segs_of(n):
    return n["parsed"]["segments"]


def structure_block(a, b, c):
    la, lb, lc = lineage_energy(a["parsed"], b["parsed"]), lineage_energy(b["parsed"], c["parsed"]), lineage_energy(a["parsed"], c["parsed"])
    pa, pb, pc = a["parsed"], b["parsed"], c["parsed"]
    sa, sb, sc = segs_of(a), segs_of(b), segs_of(c)
    lcp_ab = seg_prefix(sa, sb)
    lcp_bc = seg_prefix(sb, sc)
    lcp_ac = seg_prefix(sa, sc)
    lcp_all = 0
    for x, y, z in zip(sa, sb, sc):
        if x != y or y != z:
            break
        lcp_all += 1
    a_anc_b = pa["type"] == pb["type"] and lcp_ab == len(sa) < len(sb)
    b_anc_c = pb["type"] == pc["type"] and lcp_bc == len(sb) < len(sc)
    a_anc_c = pa["type"] == pc["type"] and lcp_ac == len(sa) < len(sc)
    b_anc_a = pa["type"] == pb["type"] and lcp_ab == len(sb) < len(sa)
    c_anc_b = pb["type"] == pc["type"] and lcp_bc == len(sc) < len(sb)
    dense = [
        la, lb, lc,
        float(lcp_ab), float(lcp_bc), float(lcp_ac), float(lcp_all),
        float(a_anc_b), float(b_anc_c), float(a_anc_c), float(b_anc_a), float(c_anc_b),
        float(parent_key(pa) == parent_key(pb)) if parent_key(pa) else 0.0,
        float(parent_key(pb) == parent_key(pc)) if parent_key(pb) else 0.0,
        float(pa["type"] == pb["type"]), float(pb["type"] == pc["type"]),
        float(len(sa)), float(len(sb)), float(len(sc)),
        float(pa["is_prime"]), float(pb["is_prime"]), float(pc["is_prime"]),
    ]
    fams = []
    for p in (pa, pb, pc):
        f = [0.0] * len(FAMILIES)
        f[FAMILIES.index(p["type"])] = 1.0
        fams += f
    roots = []
    for s in (sa, sb, sc):
        r = [0.0] * 6
        if s:
            r[s[0] % 6] = 1.0
        roots += r
    return np.array(dense + fams + roots, dtype=np.float32)


class PathFeatureSpace:
    """Joint features for traversal (a, b, c, hop types)."""

    def __init__(self, nodes, emb, profiles, pcab, pcap):
        self.nodes, self.emb, self.profiles = nodes, emb, profiles
        self.pcab, self.pcap = pcab, pcap
        self.dim = 22 + 3 * len(FAMILIES) + 18 + 4 * PCA_DIM + 3 + 2 * TYPE_HASH_DIMS + 2 * PROFILE_HASH_DIMS

    def _p(self, x):
        return ((self.emb[x["emb_idx"]][None, :] - self.pcab[0]) @ self.pcab[1]).ravel()

    def features(self, a, b, c, t1, t2):
        ea, eb, ec = (self.emb[n["emb_idx"]] for n in (a, b, c))

        def cos(u, v):
            return float(u @ v / ((np.linalg.norm(u) * np.linalg.norm(v)) or 1.0))

        prod = (ea * ec)[None, :]
        sem = np.concatenate([
            self._p(a), self._p(b), self._p(c),
            ((prod - self.pcap[0]) @ self.pcap[1]).ravel(),
            [cos(ea, eb), cos(eb, ec), cos(ea, ec)],
        ]).astype(np.float32)
        th = np.zeros(2 * TYPE_HASH_DIMS, dtype=np.float32)
        type_hash(th[:TYPE_HASH_DIMS], t1, TYPE_HASH_DIMS)
        type_hash(th[TYPE_HASH_DIMS:], t2, TYPE_HASH_DIMS)
        prof = np.zeros(2 * PROFILE_HASH_DIMS, dtype=np.float32)
        for side, n in ((0, a), (1, c)):
            p = self.profiles.get(n["nid"])
            if p:
                sub = prof[side * PROFILE_HASH_DIMS:(side + 1) * PROFILE_HASH_DIMS]
                profile_hash(sub, p["types"], PROFILE_HASH_DIMS)
        return np.concatenate([structure_block(a, b, c), sem, th, prof])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--seed", type=int, default=20260917)
    ap.add_argument("--head-seed", type=int, default=20260918)
    ap.add_argument("--neg-per-pos", type=int, default=2)
    ap.add_argument("--out", type=Path, default=Path(__file__).resolve().parent / "evidence")
    args = ap.parse_args()

    import random as pyrandom

    pyrng = pyrandom.Random(args.seed)
    t0 = time.time()

    meta, emb, edges, profiles = load_graph("bolt://100.92.62.101:7687", "neo4j")
    nodes = {}
    for nid, m in meta["nodes"].items():
        parsed = parse_coord(m["coord"])
        if parsed is None:
            continue
        nodes[nid] = {"nid": nid, "coord": m["coord"], "parsed": parsed,
                      "emb_idx": m["emb_idx"], "emb_vec": emb[m["emb_idx"]]}
    valid = [(ia, ib, t) for ia, ib, t in edges
             if ia in nodes and ib in nodes
             and nodes[ia]["emb_vec"].any() and nodes[ib]["emb_vec"].any()]
    out_edges = defaultdict(list)
    for ia, ib, t in valid:
        out_edges[ia].append((ib, t))

    root_pool, parent_pool = defaultdict(list), defaultdict(list)
    for n in nodes.values():
        root_pool[root_of(n["parsed"])].append(n)
        pk = parent_key(n["parsed"])
        if pk is not None:
            parent_pool[pk].append(n)
    all_positioned = list(nodes.values())

    # PCA bases: node vectors + endpoint product (a⊙c); difference basis reused
    idx = [n["emb_idx"] for n in nodes.values() if n["emb_vec"].any()]
    mat = emb[idx].astype(np.float64)
    n_pairs = min(len(mat) // 2, 2000)
    pcab = fit_pca(mat, PCA_DIM)
    pcap = fit_pca(mat[0:2 * n_pairs:2] * mat[1:2 * n_pairs:2], PCA_DIM)
    pfs = PathFeatureSpace(nodes, emb, profiles, pcab, pcap)
    print(json.dumps({"path_feature_dim": pfs.dim}))

    # sample real traversals
    def sample_real():
        ia, ib, t1 = pyrng.choice(valid)
        cands = out_edges.get(ib)
        if not cands:
            return None
        ic, t2 = pyrng.choice(cands)
        if ic == ia:
            return None
        return (ia, ib, ic, t1, t2)

    def corrupt_traversal(op, tr):
        ia, ib, ic, t1, t2 = tr
        a, b, c = nodes[ia], nodes[ib], nodes[ic]
        if op == "direction-reversal":
            return (ic, ib, ia, t2, t1)
        if op == "mid-rewire":
            pool = [n for n in all_positioned if n["nid"] not in (ia, ib, ic)]
        elif op == "endpoint-rewire":
            pool = [n for n in all_positioned
                    if root_of(n["parsed"]) != root_of(c["parsed"]) and n["nid"] not in (ia, ib)]
        elif op == "hard-rewire":
            pool = [n for n in root_pool.get(root_of(c["parsed"]), [])
                    if n["nid"] not in (ia, ib, ic)]
        else:  # sibling
            pk = parent_key(c["parsed"])
            pool = [n for n in parent_pool.get(pk, []) if n["nid"] not in (ia, ib, ic)]
        if not pool:
            return None
        nc = pyrng.choice(pool)
        if op == "mid-rewire":
            return (ia, nc["nid"], ic, t1, t2)
        return (ia, ib, nc["nid"], t1, t2)

    traversals = []
    seen = set()
    while len(traversals) < len(valid) and len(traversals) < 12000:
        tr = sample_real()
        if tr is None:
            continue
        key = (tr[0], tr[1], tr[2])
        if key in seen:
            continue
        seen.add(key)
        traversals.append(tr)
    train = [t for t in traversals if int.from_bytes(h("|".join(t))[:4], "little") % 5 != 0]
    test = [t for t in traversals if t not in train]
    print(json.dumps({"traversals": len(traversals), "train": len(train), "test": len(test)}))

    # ---- summed pairwise baseline head (v2b protocol: typed 1x32 + mining)
    # trained on EDGES with the edge-digest split, exactly as v2b did
    from train_energy_head import corrupt as pair_corrupt, edge_id as _eid
    edge_train = [e for e in valid
                  if int.from_bytes(h(_eid(*e))[:4], "little") % 5 != 0]
    fs2 = FeatureSpace(nodes, emb, profiles, 2, pcas=(pcab, pcab, pcap))
    PAIR_OPS = ["endpoint-rewire", "hard-rewire", "sibling-rewire", "direction-reversal"]
    Xs, ys = [], []
    for op in PAIR_OPS:
        for ia, ib, t in edge_train:
            a, b = nodes[ia], nodes[ib]
            if not (a["emb_vec"].any() and b["emb_vec"].any()):
                continue
            Xs.append(fs2.features(a, b, t))
            ys.append(1.0)
            for _ in range(2):
                cc = pair_corrupt(op, ia, ib, t, nodes, pyrng, root_pool, parent_pool, all_positioned)
                if cc is None:
                    continue
                Xs.append(fs2.features(cc[0], cc[1], cc[2]))
                ys.append(0.0)
    X2 = np.stack(Xs)
    y2 = np.array(ys, dtype=np.float32)
    head2 = MLP2(fs2.dim, hidden=32, layers=1, seed=args.head_seed % 100000)
    head2.fit(X2, y2, epochs=120, lr=1e-2, bs=256, seed=args.head_seed % 100000)
    al_p, al_n = [], []
    for ia, ib, t in edge_train:
        a, b = nodes[ia], nodes[ib]
        if not (a["emb_vec"].any() and b["emb_vec"].any()):
            continue
        cc = pair_corrupt("sibling-rewire", ia, ib, t, nodes, pyrng, root_pool, parent_pool, all_positioned)
        if cc is not None:
            al_p.append((a, b, t))
            al_n.append(cc)
    Xp2 = np.stack([fs2.features(a, b, t) for a, b, t in al_p])
    Xn2 = np.stack([fs2.features(a, b, t) for a, b, t in al_n])
    pp = 1.0 / (1.0 + np.exp(-head2.predict(Xp2)))
    pn = 1.0 / (1.0 + np.exp(-head2.predict(Xn2)))
    hard2 = [i for i in range(len(al_p)) if pn[i] > pp[i]]
    Xh2 = np.stack([fs2.features(a, b, t) for a, b, t in [al_n[i] for i in hard2]])
    Xf2 = np.concatenate([X2, np.concatenate([Xh2] * 3)])
    yf2 = np.concatenate([y2, np.zeros(len(Xh2) * 3, dtype=np.float32)])
    head2.fit(Xf2, yf2, epochs=15, lr=1e-3, bs=256, seed=args.head_seed % 100000)
    print(json.dumps({"pair_head": "trained+mined", "mined": len(hard2)}))

    def pair_hop_energy(x, y_, t):
        return -float(np.log(max(1.0 / (1.0 + np.exp(-head2.predict(
            np.stack([fs2.features(x, y_, t)]))))[0], 1e-12)))

    # ---- joint path head training
    Xs, ys = [], []
    for tr in train:
        ia, ib, ic, t1, t2 = tr
        pos = pfs.features(nodes[ia], nodes[ib], nodes[ic], t1, t2)
        Xs.append(pos)
        ys.append(1.0)
        for op in OPS:
            for _ in range(args.neg_per_pos):
                ct = corrupt_traversal(op, tr)
                if ct is None:
                    continue
                Xs.append(pfs.features(nodes[ct[0]], nodes[ct[1]], nodes[ct[2]], ct[3], ct[4]))
                ys.append(0.0)
    X3 = np.stack(Xs)
    y3 = np.array(ys, dtype=np.float32)
    head3 = MLP2(pfs.dim, hidden=32, layers=1, seed=args.head_seed % 100000)
    head3.fit(X3, y3, epochs=120, lr=1e-2, bs=256, seed=args.head_seed % 100000)
    print(json.dumps({"path_head": "trained", "rows": list(X3.shape)}))

    # hard-negative mining on sibling traversal train negatives
    hp, hn = [], []
    for tr in train:
        ct = corrupt_traversal("sibling-rewire", tr)
        if ct is not None:
            hp.append(tr)
            hn.append(ct)
    Xph = np.stack([pfs.features(nodes[t[0]], nodes[t[1]], nodes[t[2]], t[3], t[4]) for t in hp])
    Xnh = np.stack([pfs.features(nodes[t[0]], nodes[t[1]], nodes[t[2]], t[3], t[4]) for t in hn])
    php = 1.0 / (1.0 + np.exp(-head3.predict(Xph)))
    phn = 1.0 / (1.0 + np.exp(-head3.predict(Xnh)))
    hard3 = [i for i in range(len(hp)) if phn[i] > php[i]]
    X3h = np.stack([pfs.features(nodes[hn[i][0]], nodes[hn[i][1]], nodes[hn[i][2]], hn[i][3], hn[i][4]) for i in hard3])
    Xf3 = np.concatenate([X3, np.concatenate([X3h] * 3)])
    yf3 = np.concatenate([y3, np.zeros(len(X3h) * 3, dtype=np.float32)])
    head3.fit(Xf3, yf3, epochs=15, lr=1e-3, bs=256, seed=args.head_seed % 100000)
    print(json.dumps({"path_head": "mined", "mined": len(hard3)}))

    # ---- evaluation on held-out traversals
    def eval_traversals(head_eval, subset):
        pos_rows, neg_rows_by_op = [], {op: [] for op in OPS}
        for tr in subset:
            ia, ib, ic, t1, t2 = tr
            pos_rows.append(tr)
            for op in OPS:
                ct = corrupt_traversal(op, tr)
                if ct is not None:
                    neg_rows_by_op[op].append(ct)
        out = {}
        for op in OPS:
            negs = neg_rows_by_op[op]
            if not negs or not pos_rows:
                continue

            def cos3(tr):
                ia, ib, ic, t1, t2 = tr
                s = 0.0
                for x, y_ in ((nodes[ia], nodes[ib]), (nodes[ib], nodes[ic])):
                    u, v = x["emb_vec"], y_["emb_vec"]
                    s += 1.0 - float(u @ v / ((np.linalg.norm(u) * np.linalg.norm(v)) or 1.0))
                return s

            def comb3(tr):
                ia, ib, ic, t1, t2 = tr
                return (lineage_energy(nodes[ia]["parsed"], nodes[ib]["parsed"])
                        + lineage_energy(nodes[ib]["parsed"], nodes[ic]["parsed"]) + cos3(tr))

            def pair_head3(tr):
                ia, ib, ic, t1, t2 = tr
                return (pair_hop_energy(nodes[ia], nodes[ib], t1)
                        + pair_hop_energy(nodes[ib], nodes[ic], t2))

            Xp3 = np.stack([pfs.features(nodes[t[0]], nodes[t[1]], nodes[t[2]], t[3], t[4]) for t in pos_rows])
            Xn3 = np.stack([pfs.features(nodes[t[0]], nodes[t[1]], nodes[t[2]], t[3], t[4]) for t in negs])
            pp3 = 1.0 / (1.0 + np.exp(-head3.predict(Xp3)))
            pn3 = 1.0 / (1.0 + np.exp(-head3.predict(Xn3)))
            out[op] = {
                "n_paths": len(pos_rows),
                "joint_head": rank_auc((-head3.predict(Xn3)).tolist(), (-head3.predict(Xp3)).tolist()),
                "summed_pair_head": rank_auc([pair_head3(t) for t in negs], [pair_head3(t) for t in pos_rows]),
                "summed_cosine": rank_auc([cos3(t) for t in negs], [cos3(t) for t in pos_rows]),
                "summed_combined": rank_auc([comb3(t) for t in negs], [comb3(t) for t in pos_rows]),
                "calibration_joint": calib_readout(pp3, pn3),
            }
        return out

    results = eval_traversals(head3, test)
    for op, d in results.items():
        print(json.dumps({op: {k: (round(v, 3) if isinstance(v, float) else v)
                               for k, v in d.items() if k != "calibration_joint"}}))

    evidence = {
        "schema": "actuation.model-types-ebm-path-head/v1",
        "created": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "experiment": "E-MT-EBM-FIELD phase-3 (joint traversal head vs summed scoring)",
        "features": {
            "joint": ("structure over all three coordinates (per-hop lineage, pairwise+triple LCP, directed "
                      "ancestor flags, depths/families/roots/primes), PCA-64 of e_a/e_b/e_c and e_a*e_c, "
                      "cos(a,b), cos(b,c), chord cos(a,c), both hop types hashed 128, endpoint type profiles"),
            "feature_dim": pfs.dim,
            "arch": "1x32 (capacity lesson of phase 2.6: small generalises)",
            "pair_baseline": "v2b summed pairwise head (semantic channel + typed + mining), identical protocol",
        },
        "data": {"traversals": len(traversals), "train": len(train), "test": len(test)},
        "results": results,
        "honest_limits": [
            "negatives are synthetic corruptions; AUC measures separation from the ladder, not correctness of real transitions",
            "loop-proposed transitions remain untested until the composed loop runs (EBM-GATE-DESIGN.md, shadow mode first)",
            "calibration describes agreement with the real-vs-corrupted traversal label, not with human judgement",
        ],
        "runtime_seconds": round(time.time() - t0, 1),
        "provenance": {"promotion": "none", "reading_class": "semantic-stochastic"},
    }
    args.out.mkdir(parents=True, exist_ok=True)
    out_path = args.out / f"path-head-{time.strftime('%Y%m%d-%H%M%S')}.json"
    out_path.write_text(json.dumps(evidence, indent=2, default=float) + "\n")
    print(json.dumps({"wrote": str(out_path)}))


if __name__ == "__main__":
    import argparse  # noqa: F401  (placed here to keep the docstring importable)

    sys.exit(main())
