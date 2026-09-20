#!/usr/bin/env python3
"""E-MT EBM Phase 2.6b — hard-negative mining on the best ladder config.

The v2 ladder's best full-graph sibling rung was typed features at 1x32
capacity (0.619). Mining was only applied to the 2x128 rung there. This run
applies the identical mining step (aligned per-edge sibling train negatives
the model ranks above its real edge, oversampled x3, low-LR fine-tune) to the
1x32 typed head, and reports the full operator ladder + calibration for the
resulting recommended configuration. Seeds unchanged (head 20260918).
"""

import json
import sys
import time
from collections import defaultdict
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent))
from field_probe import lineage_energy, rank_auc  # noqa: E402
from train_energy_head import corrupt, h, edge_id  # noqa: E402
from train_energy_head_v2 import (  # noqa: E402
    FeatureSpace,
    MLP2,
    calib_readout,
    load_graph,
)


def main():
    import argparse
    import random as pyrandom

    ap = argparse.ArgumentParser()
    ap.add_argument("--seed", type=int, default=20260918)
    ap.add_argument("--out", type=Path, default=Path(__file__).resolve().parent / "evidence")
    args = ap.parse_args()

    pyrng = pyrandom.Random(args.seed)
    t0 = time.time()

    meta, emb, edges, profiles = load_graph("bolt://100.92.62.101:7687", "neo4j")
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    from field_probe import parse_coord, root_of

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
    train = [e for e in valid if int.from_bytes(h(edge_id(*e))[:4], "little") % 5 != 0]
    test = [e for e in valid if e not in train]

    root_pool, parent_pool = defaultdict(list), defaultdict(list)
    for n in nodes.values():
        root_pool[root_of(n["parsed"])].append(n)
        pk = None
        segs = n["parsed"]["segments"]
        if len(segs) >= 2:
            pk = (n["parsed"]["type"], tuple(segs[:-1]))
        if pk is not None:
            parent_pool[pk].append(n)
    all_positioned = list(nodes.values())

    m0_nodes = {nid: n for nid, n in nodes.items()
                if root_of(n["parsed"]) == ("M", (0,))}
    m0_root_pool, m0_parent_pool = defaultdict(list), defaultdict(list)
    for n in m0_nodes.values():
        m0_root_pool[root_of(n["parsed"])].append(n)
        segs = n["parsed"]["segments"]
        if len(segs) >= 2:
            m0_parent_pool[(n["parsed"]["type"], tuple(segs[:-1]))].append(n)
    m0_all = list(m0_nodes.values())
    m0_test = [e for e in test if e[0] in m0_nodes and e[1] in m0_nodes]

    OPS = ["endpoint-rewire", "hard-rewire", "sibling-rewire", "direction-reversal"]

    # identical PCA bases to the v2 run (same field, same fit)
    idx = [n["emb_idx"] for n in nodes.values() if n["emb_vec"].any()]
    from train_energy_head_v2 import fit_pca, PCA_DIM
    mat = emb[idx].astype(np.float64)
    n_pairs = min(len(mat) // 2, 2000)
    pcas = (fit_pca(mat, PCA_DIM),
            fit_pca(mat[0:2 * n_pairs:2] - mat[1:2 * n_pairs:2], PCA_DIM),
            fit_pca(mat[0:2 * n_pairs:2] * mat[1:2 * n_pairs:2], PCA_DIM))

    fs = FeatureSpace(nodes, emb, profiles, 2, pcas=pcas)

    def make_pairs(op, subset, nodes_map, rp, pp, al):
        pos, neg = [], []
        for ia, ib, t in subset:
            a, b = nodes_map[ia], nodes_map[ib]
            if not (a["emb_vec"].any() and b["emb_vec"].any()):
                continue
            pos.append((a, b, t))
            c = corrupt(op, ia, ib, t, nodes_map, pyrng, rp, pp, al)
            if c is not None and c[0]["emb_vec"].any() and c[1]["emb_vec"].any():
                neg.append(c)
        return pos, neg

    def baselines(pos, neg):
        lin_p = [lineage_energy(a["parsed"], b["parsed"]) for a, b, _ in pos]
        lin_n = [lineage_energy(a["parsed"], b["parsed"]) for a, b, _ in neg]

        def cos_en(x, y):
            return 1.0 - float(x["emb_vec"] @ y["emb_vec"]
                               / ((np.linalg.norm(x["emb_vec"]) * np.linalg.norm(y["emb_vec"])) or 1.0))

        cos_p = [cos_en(a, b) for a, b, _ in pos]
        cos_n = [cos_en(a, b) for a, b, _ in neg]
        return {"cosine": rank_auc(cos_n, cos_p),
                "combined": rank_auc([l + c for l, c in zip(lin_n, cos_n)],
                                     [l + c for l, c in zip(lin_p, cos_p)])}

    def eval_head(head, pos, neg):
        Xp = np.stack([fs.features(a, b, t) for a, b, t in pos])
        Xn = np.stack([fs.features(a, b, t) for a, b, t in neg])
        return {"learned": rank_auc((-head.predict(Xn)).tolist(), (-head.predict(Xp)).tolist()),
                "calibration": calib_readout(1.0 / (1.0 + np.exp(-head.predict(Xp))),
                                             1.0 / (1.0 + np.exp(-head.predict(Xn))))}

    # train typed 1x32 (same data order/seeds as v2 rung 'typed')
    Xs, ys = [], []
    for op in OPS:
        pos, neg = make_pairs(op, train, nodes, root_pool, parent_pool, all_positioned)
        for a, b, t in pos:
            Xs.append(fs.features(a, b, t))
            ys.append(1.0)
        for a, b, t in neg:
            Xs.append(fs.features(a, b, t))
            ys.append(0.0)
    X = np.stack(Xs)
    y = np.array(ys, dtype=np.float32)
    head = MLP2(fs.dim, hidden=32, layers=1, seed=args.seed % 100000)
    head.fit(X, y, epochs=120, lr=1e-2, bs=256, seed=args.seed % 100000)

    # mine aligned per-edge sibling negatives on train edges only
    al_pos, al_neg = [], []
    for ia, ib, t in train:
        a, b = nodes[ia], nodes[ib]
        if not (a["emb_vec"].any() and b["emb_vec"].any()):
            continue
        c = corrupt("sibling-rewire", ia, ib, t, nodes, pyrng, root_pool, parent_pool, all_positioned)
        if c is not None and c[0]["emb_vec"].any() and c[1]["emb_vec"].any():
            al_pos.append((a, b, t))
            al_neg.append(c)
    Xp_tr = np.stack([fs.features(a, b, t) for a, b, t in al_pos])
    Xn_tr = np.stack([fs.features(a, b, t) for a, b, t in al_neg])
    p_pos = 1.0 / (1.0 + np.exp(-head.predict(Xp_tr)))
    p_neg = 1.0 / (1.0 + np.exp(-head.predict(Xn_tr)))
    hard_idx = [i for i in range(len(al_pos)) if p_neg[i] > p_pos[i]]
    Xh = np.stack([fs.features(a, b, t) for a, b, t in [al_neg[i] for i in hard_idx]])
    Xf = np.concatenate([X, np.concatenate([Xh] * 3)])
    yf = np.concatenate([y, np.zeros(len(Xh) * 3, dtype=np.float32)])
    head.fit(Xf, yf, epochs=15, lr=1e-3, bs=256, seed=args.seed % 100000)

    res, m0 = {}, {}
    for op in OPS:
        pos, neg = make_pairs(op, test, nodes, root_pool, parent_pool, all_positioned)
        if pos and neg:
            res[op] = {"n_pos": len(pos), **eval_head(head, pos, neg),
                       **{f"baseline_{k}": v for k, v in baselines(pos, neg).items()}}
    for op in ("hard-rewire", "sibling-rewire", "direction-reversal"):
        pos, neg = make_pairs(op, m0_test, m0_nodes, m0_root_pool, m0_parent_pool, m0_all)
        if pos and neg:
            m0[op] = {"n_pos": len(pos), **eval_head(head, pos, neg),
                      **{f"baseline_{k}": v for k, v in baselines(pos, neg).items()}}

    evidence = {
        "schema": "actuation.model-types-ebm-energy-head-v2b/v1",
        "created": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "experiment": "E-MT-EBM-FIELD phase-2.6b (mining on the typed 1x32 rung)",
        "config": "typed features (semantic channel + 128-dim type hash + incident-type profiles), 1x32, then sibling hard-negative mining x3, 15 epochs lr 1e-3",
        "mined_hard_train_negatives": len(hard_idx),
        "full": res,
        "m0": m0,
        "honest_limits": [
            "learned readings are semantic-stochastic; never promoted, never the determination",
            "negatives are synthetic corruptions; AUC measures separation from the ladder, not value or correctness",
            "mining uses train edges only; held-out negatives stay untouched by mining",
        ],
        "runtime_seconds": round(time.time() - t0, 1),
        "provenance": {"promotion": "none", "reading_class": "semantic-stochastic"},
    }
    args.out.mkdir(parents=True, exist_ok=True)
    out_path = args.out / f"energy-head-v2b-{time.strftime('%Y%m%d-%H%M%S')}.json"
    out_path.write_text(json.dumps(evidence, indent=2, default=float) + "\n")
    print(json.dumps({"wrote": str(out_path),
                      "full": {op: res[op] for op in res},
                      "m0": {op: m0[op] for op in m0}}, indent=2, default=float))


if __name__ == "__main__":
    sys.exit(main())
