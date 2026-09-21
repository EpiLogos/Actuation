#!/usr/bin/env python3
"""E-MT EBM Phase 3-preview — path-level energies (R3).

R3 asks: direction sensitivity and sibling discrimination exist at the pair
level; does scoring a whole traversal separate real traversals from corrupted
ones? A loop gate would score candidate transitions as traversals through the
field, not isolated pairs, so this is the probe that previews the gate's
actual readout.

State: a 2-hop traversal a -> b -> c taken from the graph's real edge set
(consecutive edges, a != c). Corruption operators (seeded, probe seed
20260917):
  endpoint-rewire   c replaced by a random different-root node (coarse)
  hard-rewire       c replaced by a random same-root node
  sibling-rewire    c replaced by a random sibling of c
  mid-rewire        b replaced by a random node (the traversal no longer exists)
  direction-reversal the traversal reversed: c -> b -> a

Scorers (all pairwise-summed, E(path) = E(a,b) + E(b,c)):
  cosine    1 - cos(u,v) per hop, in-graph 3072-dim embeddings
  lineage   coordinate lineage potential per hop (probe convention weights)
  combined  lineage + cosine per hop
  head      the phase-2.6b energy head (semantic channel + typed conditioning
            + sibling mining), retrained inline with the same fixed seeds

Readout: separation AUC, P(real path energy < corrupted path energy); 0.5 is
no separation. Pair potentials remain direction-blind per hop, so the
direction-reversal operator is answered only by the head's directed features.

Provenance: promotion none, reading class research (baselines) /
semantic-stochastic (head). Nothing here is a value or correctness claim.
"""

import argparse
import json
import sys
import time
from collections import defaultdict
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent))
from field_probe import lineage_energy, parse_coord, rank_auc, root_of  # noqa: E402
from train_energy_head import corrupt, h, edge_id  # noqa: E402
from train_energy_head_v2 import (  # noqa: E402
    FeatureSpace,
    MLP2,
    PCA_DIM,
    fit_pca,
    load_graph,
)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--samples", type=int, default=2000, help="corrupted paths per operator")
    ap.add_argument("--seed", type=int, default=20260917)
    ap.add_argument("--head-seed", type=int, default=20260918)
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
    valid_edges = [(ia, ib, t) for ia, ib, t in edges
                   if ia in nodes and ib in nodes
                   and nodes[ia]["emb_vec"].any() and nodes[ib]["emb_vec"].any()]

    # adjacency for 2-hop sampling
    out_edges = defaultdict(list)
    for ia, ib, t in valid_edges:
        out_edges[ia].append((ib, t))

    root_pool, parent_pool = defaultdict(list), defaultdict(list)
    for n in nodes.values():
        root_pool[root_of(n["parsed"])].append(n)
        segs = n["parsed"]["segments"]
        if len(segs) >= 2:
            parent_pool[(n["parsed"]["type"], tuple(segs[:-1]))].append(n)
    all_positioned = list(nodes.values())

    # ---- retrain the 2.6b head inline (same seeds/protocol as v2b)
    train = [e for e in valid_edges
             if int.from_bytes(h(edge_id(*e))[:4], "little") % 5 != 0]
    idx = [n["emb_idx"] for n in nodes.values() if n["emb_vec"].any()]
    mat = emb[idx].astype(np.float64)
    n_pairs = min(len(mat) // 2, 2000)
    pcas = (fit_pca(mat, PCA_DIM),
            fit_pca(mat[0:2 * n_pairs:2] - mat[1:2 * n_pairs:2], PCA_DIM),
            fit_pca(mat[0:2 * n_pairs:2] * mat[1:2 * n_pairs:2], PCA_DIM))
    fs = FeatureSpace(nodes, emb, profiles, 2, pcas=pcas)

    OPS_HEAD = ["endpoint-rewire", "hard-rewire", "sibling-rewire", "direction-reversal"]
    Xs, ys = [], []
    for op in OPS_HEAD:
        for ia, ib, t in train:
            a, b = nodes[ia], nodes[ib]
            if not (a["emb_vec"].any() and b["emb_vec"].any()):
                continue
            Xs.append(fs.features(a, b, t))
            ys.append(1.0)
            for _ in range(2):
                c = corrupt(op, ia, ib, t, nodes, pyrng, root_pool, parent_pool, all_positioned)
                if c is None or not (c[0]["emb_vec"].any() and c[1]["emb_vec"].any()):
                    continue
                Xs.append(fs.features(c[0], c[1], c[2]))
                ys.append(0.0)
    X = np.stack(Xs)
    y = np.array(ys, dtype=np.float32)
    head = MLP2(fs.dim, hidden=32, layers=1, seed=args.head_seed % 100000)
    head.fit(X, y, epochs=120, lr=1e-2, bs=256, seed=args.head_seed % 100000)

    # mining fine-tune (aligned per-edge sibling negatives, x3, low LR)
    al_pos, al_neg = [], []
    for ia, ib, t in train:
        a, b = nodes[ia], nodes[ib]
        if not (a["emb_vec"].any() and b["emb_vec"].any()):
            continue
        c = corrupt("sibling-rewire", ia, ib, t, nodes, pyrng, root_pool, parent_pool, all_positioned)
        if c is not None and c[0]["emb_vec"].any() and c[1]["emb_vec"].any():
            al_pos.append((a, b, t))
            al_neg.append(c)
    Xp = np.stack([fs.features(a, b, t) for a, b, t in al_pos])
    Xn = np.stack([fs.features(a, b, t) for a, b, t in al_neg])
    p_pos = 1.0 / (1.0 + np.exp(-head.predict(Xp)))
    p_neg = 1.0 / (1.0 + np.exp(-head.predict(Xn)))
    hard = [i for i in range(len(al_pos)) if p_neg[i] > p_pos[i]]
    Xh = np.stack([fs.features(a, b, t) for a, b, t in [al_neg[i] for i in hard]])
    Xf = np.concatenate([X, np.concatenate([Xh] * 3)])
    yf = np.concatenate([y, np.zeros(len(Xh) * 3, dtype=np.float32)])
    head.fit(Xf, yf, epochs=15, lr=1e-3, bs=256, seed=args.head_seed % 100000)
    print(json.dumps({"head": "trained+mined", "mined": len(hard)}))

    def cos_energy(u, v):
        return 1.0 - float(u @ v / ((np.linalg.norm(u) * np.linalg.norm(v)) or 1.0))

    def hop_energies(x, y_, t):
        lin = lineage_energy(x["parsed"], y_["parsed"])
        sem = cos_energy(x["emb_vec"], y_["emb_vec"])
        z = 1.0 / (1.0 + np.exp(-head.predict(np.stack([fs.features(x, y_, t)])))[0])
        return lin, sem, -float(np.log(max(z, 1e-12)))  # head energy = -log p

    def corrupt_path(op, ia, ib, ic, tb, tc):
        b, c = nodes[ib], nodes[ic]
        if op == "direction-reversal":
            return ic, ib, tc, ib  # reversed traversal reuses real edges' endpoints
        if op == "mid-rewire":
            pool = [n for n in all_positioned
                    if n is not nodes[ib] and n is not nodes[ia] and n is not nodes[ic]]
            if not pool:
                return None
            nb = pyrng.choice(pool)
            return ib, nb["emb_idx"] if False else nb, tb, tb  # replace b
        if op == "endpoint-rewire":
            pool = [n for n in all_positioned if root_of(n["parsed"]) != root_of(c["parsed"]) and n is not c and n is not nodes[ia]]
        elif op == "hard-rewire":
            pool = [n for n in root_pool.get(root_of(c["parsed"]), []) if n is not c and n is not nodes[ia] and n is not b]
        else:  # sibling-rewire
            segs = c["parsed"]["segments"]
            pk = (c["parsed"]["type"], tuple(segs[:-1])) if len(segs) >= 2 else None
            pool = [n for n in parent_pool.get(pk, []) if n is not c and n is not nodes[ia] and n is not b]
        if not pool:
            return None
        nc = pyrng.choice(pool)
        return ib, nc, tb, tc

    PATH_OPS = ["endpoint-rewire", "hard-rewire", "sibling-rewire", "mid-rewire",
                "direction-reversal"]
    pots = {name: ([], []) for name in ("cosine", "lineage", "combined", "head")}
    results, excluded = {}, 0
    for op in PATH_OPS:
        made, tries = 0, 0
        pots = {name: ([], []) for name in pots}
        while made < args.samples and tries < args.samples * 20:
            tries += 1
            ia, ib, tb = pyrng.choice(valid_edges)
            cands = out_edges.get(ib)
            if not cands:
                continue
            ic, tc = pyrng.choice(cands)
            if ic == ia:
                excluded += 1
                continue
            a, b, c = nodes[ia], nodes[ib], nodes[ic]
            if op == "direction-reversal":
                real_hops = [(a, b, tb), (b, c, tc)]
                neg_hops = [(c, b, tc), (b, a, tb)]
            else:
                r = corrupt_path(op, ia, ib, ic, tb, tc)
                if r is None:
                    continue
                _, nb_or_nc, tb2, tc2 = r
                if op == "mid-rewire":
                    nb = nb_or_nc
                    real_hops = [(a, b, tb), (b, c, tc)]
                    neg_hops = [(a, nb, tb), (nb, c, tc)]
                else:
                    nc = nb_or_nc
                    real_hops = [(a, b, tb), (b, c, tc)]
                    neg_hops = [(a, b, tb), (b, nc, tc2)]
            def path_scores(hops):
                lin = sem = hed = 0.0
                for x, y_, t in hops:
                    l, s, hd = hop_energies(x, y_, t)
                    lin += l
                    sem += s
                    hed += hd
                return lin, sem, lin + sem, hed
            rl, rs, rc, rh = path_scores(real_hops)
            nl, ns, nc_, nh = path_scores(neg_hops)
            pots["lineage"][0].append(rl)
            pots["lineage"][1].append(nl)
            pots["cosine"][0].append(rs)
            pots["cosine"][1].append(ns)
            pots["combined"][0].append(rc)
            pots["combined"][1].append(nc_)
            pots["head"][0].append(rh)
            pots["head"][1].append(nh)
            made += 1
        results[op] = {
            "n_paths": made,
            "n_excluded": excluded,
            "separation_auc_real_lower_energy": {
                name: rank_auc(n, p) for name, (p, n) in pots.items()
            },
            "mean_energy": {
                name: {"real": sum(p) / max(len(p), 1), "corrupted": sum(n) / max(len(n), 1)}
                for name, (p, n) in pots.items()
            },
        }
        print(json.dumps({op: results[op]["separation_auc_real_lower_energy"]}))

    evidence = {
        "schema": "actuation.model-types-ebm-path-probe/v1",
        "created": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "experiment": "E-MT-EBM-FIELD phase-3-preview (path-level energies, R3)",
        "state": "2-hop traversal a->b->c from consecutive real edges; corruption operators replace the last hop, the middle node, or reverse the traversal",
        "scorers": {
            "cosine": "sum of 1-cos per hop (3072-dim in-graph embeddings)",
            "lineage": "sum of coordinate lineage potential per hop (probe convention weights)",
            "combined": "lineage + cosine per hop",
            "head": "phase-2.6b head (semantic channel + typed + mining) energy per hop, energy = -log p_compatible",
        },
        "seeds": {"probe": args.seed, "head": args.head_seed},
        "results": results,
        "honest_limits": [
            "readings are instrument output; promotion none; low energy is compatibility, nothing else",
            "corruptions are synthetic; AUC measures separation from the corruption ladder, not correctness of real traversals",
            "pairwise-summed path energy is the simplest path readout; joint 3-node features are future work",
            "the head was retrained inline with the v2b protocol (typed features 1x32 + sibling mining) so path and pair results share one scorer",
            "direction-reversal negatives reuse the same edges reversed; per-hop pair potentials are blind there by construction",
        ],
        "runtime_seconds": round(time.time() - t0, 1),
        "provenance": {"promotion": "none", "reading_class": "research"},
    }
    args.out.mkdir(parents=True, exist_ok=True)
    out_path = args.out / f"path-probe-{time.strftime('%Y%m%d-%H%M%S')}.json"
    out_path.write_text(json.dumps(evidence, indent=2, default=float) + "\n")
    print(json.dumps({"wrote": str(out_path)}))


if __name__ == "__main__":
    sys.exit(main())
