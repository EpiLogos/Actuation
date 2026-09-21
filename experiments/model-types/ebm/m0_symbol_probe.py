#!/usr/bin/env python3
"""E-MT EBM — M0 at symbol granularity: symbol-only vs full-content embeddings.

The owner's correction: M0 is the compressed VAK syntax — a mathematical-
symbolic notation meant to self-compute — so the M0 test should read the
symbol and formulation properties (c_1_symbol, c_1_complete_formulation,
formulation_breakdown, m_0_3_* matrix keys, operator/symbolics fields), not
generic embeddings over all narrative content.

Method: symbol-only embed texts were composed from those properties alone and
embedded READ-ONLY (bimba_embed without store_for — no graph write; the
authoritative in-graph embeddings are untouched). This script scores the same
M0-internal corruption ladder twice with identical seeds/pools: once with the
in-graph full-content embeddings, once with the symbol-only vectors (matched
by coordinate). Semantic readout = cosine; plus a small M0-local head on the
symbol vectors as a secondary reading (data starvation caveat applies).

Validity note: this is a property-set A/B on one coordinate family with ~109
nodes and 121-ish held-out edges — directional, not decisive. The question is
whether the symbolic layer carries MORE structure than the diluted mix, and
by how much.
"""

import json
import sys
import time
from collections import defaultdict
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent))
from field_probe import lineage_energy, parse_coord, rank_auc, root_of, parent_key  # noqa: E402
from train_energy_head_v2 import MLP2, calib_readout  # noqa: E402
from train_energy_head import h, edge_id  # noqa: E402
from train_energy_head_v2 import load_graph  # noqa: E402


def cos_energy(u, v):
    return 1.0 - float(u @ v / ((np.linalg.norm(u) * np.linalg.norm(v)) or 1.0))


def run_ladder(nodes, edges, ops, rng):
    root_pool, parent_pool = defaultdict(list), defaultdict(list)
    for n in nodes.values():
        if n.get("vec") is None:
            continue
        root_pool[root_of(n["parsed"])].append(n)
        pk = parent_key(n["parsed"])
        if pk is not None:
            parent_pool[pk].append(n)
    out = {}
    for op in ops:
        pos_e, neg_e = [], []
        for ia, ib, t in edges:
            a, b = nodes[ia], nodes[ib]
            pos_e.append((a, b))
            if op == "direction-reversal":
                neg_e.append((b, a))
                continue
            if op == "hard-rewire":
                pool = [n for n in root_pool.get(root_of(b["parsed"]), []) if n is not b]
            else:
                pk = parent_key(b["parsed"])
                pool = [n for n in parent_pool.get(pk, []) if n is not b]
            if not pool:
                pos_e.pop()
                continue
            neg_e.append((a, rng.choice(pool)))
        if not pos_e:
            out[op] = None
            continue
        lin_p = [lineage_energy(a["parsed"], b["parsed"]) for a, b in pos_e]
        lin_n = [lineage_energy(a["parsed"], b["parsed"]) for a, b in neg_e]
        sem_p = [cos_energy(a["vec"], b["vec"]) for a, b in pos_e]
        sem_n = [cos_energy(a["vec"], b["vec"]) for a, b in neg_e]
        out[op] = {
            "n_pairs": len(pos_e),
            "cosine": rank_auc(sem_n, sem_p),
            "combined": rank_auc([l + s for l, s in zip(lin_n, sem_n)],
                                 [l + s for l, s in zip(lin_p, sem_p)]),
        }
    return out


def main():
    import argparse
    import random as pyrandom

    ap = argparse.ArgumentParser()
    ap.add_argument("--seed", type=int, default=20260917)
    ap.add_argument("--out", type=Path, default=Path(__file__).resolve().parent / "evidence")
    args = ap.parse_args()
    rng = pyrandom.Random(args.seed)
    pyrng_np = np.random.default_rng(20260918)
    t0 = time.time()

    meta, emb, edges, profiles = load_graph("bolt://100.92.62.101:7687", "neo4j")
    symbol_vecs = {k: np.asarray(v, dtype=np.float32)
                   for k, v in json.load(open("/tmp/m0-symbol-embeddings.json")).items() if v}

    nodes = {}
    for nid, m in meta["nodes"].items():
        parsed = parse_coord(m["coord"])
        if parsed is None or root_of(parsed) != ("M", (0,)):
            continue
        nodes[nid] = {"nid": nid, "coord": m["coord"], "parsed": parsed,
                      "emb_idx": m["emb_idx"],
                      "full_vec": emb[m["emb_idx"]],
                      "vec": None}
    n_sym = 0
    for nid, n in nodes.items():
        v = symbol_vecs.get(n["coord"])
        if v is not None and v.size:
            n["vec"] = v
            n_sym += 1
    m0_edges = [(ia, ib, t) for ia, ib, t in edges if ia in nodes and ib in nodes]
    sym_edges = [(ia, ib, t) for ia, ib, t in m0_edges
                 if nodes[ia]["vec"] is not None and nodes[ib]["vec"] is not None]

    # A: full-content embeddings, all M0 edges
    for n in nodes.values():
        n["vec"] = n["full_vec"]
    full = run_ladder(nodes, m0_edges, ["hard-rewire", "sibling-rewire", "direction-reversal"],
                      pyrandom.Random(args.seed))
    # B: symbol-only embeddings, edges where both sides have symbol vectors
    for nid, n in nodes.items():
        n["vec"] = symbol_vecs.get(n["coord"])
    sym = run_ladder(nodes, sym_edges, ["hard-rewire", "sibling-rewire", "direction-reversal"],
                     pyrandom.Random(args.seed))

    # C: small M0-local head on symbol vectors (secondary; starvation caveat)
    train = [e for e in sym_edges if int.from_bytes(h(edge_id(*e))[:4], "little") % 5 != 0]
    test = [e for e in sym_edges if e not in train]
    head_res = {}
    if train and test:
        dim = len(next(iter(symbol_vecs.values())))
        root_pool, parent_pool = defaultdict(list), defaultdict(list)
        for n in nodes.values():
            if n["vec"] is None:
                continue
            root_pool[root_of(n["parsed"])].append(n)
            pk = parent_key(n["parsed"])
            if pk is not None:
                parent_pool[pk].append(n)

        def feats(a, b):
            u, v = a["vec"], b["vec"]
            pk = parent_key(a["parsed"])
            return np.array([
                lineage_energy(a["parsed"], b["parsed"]),
                cos_energy(u, v),
                float(pk is not None and pk == parent_key(b["parsed"])),
                float(root_of(a["parsed"]) == root_of(b["parsed"])),
                float(len(a["parsed"]["segments"])), float(len(b["parsed"]["segments"])),
            ], dtype=np.float32)

        X, y = [], []
        for op in ("hard-rewire", "sibling-rewire", "direction-reversal"):
            for ia, ib, t in train:
                a, b = nodes[ia], nodes[ib]
                X.append(feats(a, b)); y.append(1.0)
                if op == "direction-reversal":
                    X.append(feats(b, a)); y.append(0.0); continue
                if op == "hard-rewire":
                    pool = [n for n in root_pool.get(root_of(b["parsed"]), []) if n is not b]
                else:
                    pool = [n for n in parent_pool.get(parent_key(b["parsed"]), []) if n is not b]
                if not pool:
                    continue
                nb = rng.choice(pool)
                X.append(feats(a, nb)); y.append(0.0)
        X, y = np.stack(X), np.array(y, dtype=np.float32)
        head = MLP2(6, hidden=16, layers=1, seed=18)
        head.fit(X, y, epochs=200, lr=1e-2, bs=64, seed=18)
        for op in ("hard-rewire", "sibling-rewire", "direction-reversal"):
            pp, nn = [], []
            for ia, ib, t in test:
                a, b = nodes[ia], nodes[ib]
                pp.append((a, b))
                if op == "direction-reversal":
                    nn.append((b, a))
                elif op == "hard-rewire":
                    pool = [n for n in root_pool.get(root_of(b["parsed"]), []) if n is not b]
                    if pool: nn.append((a, rng.choice(pool)))
                else:
                    pool = [n for n in parent_pool.get(parent_key(b["parsed"]), []) if n is not b]
                    if pool: nn.append((a, rng.choice(pool)))
            if not pp or not nn:
                continue
            Xp, Xn = np.stack([feats(a, b) for a, b in pp]), np.stack([feats(a, b) for a, b in nn])
            pp_p = 1.0 / (1.0 + np.exp(-head.predict(Xp)))
            pn_p = 1.0 / (1.0 + np.exp(-head.predict(Xn)))
            head_res[op] = {"n_pairs": len(pp),
                            "head": rank_auc((-head.predict(Xn)).tolist(), (-head.predict(Xp)).tolist()),
                            "calibration": calib_readout(pp_p, pn_p)}

    evidence = {
        "schema": "actuation.model-types-ebm-m0-symbols/v1",
        "created": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "experiment": "E-MT-EBM-FIELD M0 at symbol granularity (owner hypothesis: the compressed VAK syntax carries the structure; generic embeddings dilute it)",
        "symbol_property_set": [
            "c_1_symbol", "c_1_complete_formulation", "c_1_formulation_breakdown",
            "c_1_structure", "c_1_metaphysical_names", "c_4_ql_operator_types",
            "c_4_key_operators", "c_1_operational_symbolics", "c_1_mother_daughter_formula",
            "c_1_wholeness_formula_seed", "c_0_logical_operator_nature",
            "c_0_void_grammar_structure", "c_1_consciousness_structure",
            "all m_0_<n>_<*> matrix keys", "all q_<n>_<*> structure keys", "coordinate + name",
        ],
        "data": {
            "m0_nodes": len(nodes), "m0_nodes_with_symbol_vectors": n_sym,
            "m0_edges_full": len(m0_edges), "m0_edges_symbol": len(sym_edges),
        },
        "results_full_content_embeddings": full,
        "results_symbol_only_embeddings": sym,
        "results_symbol_only_m0_local_head": head_res,
        "honest_limits": [
            "graph untouched: symbol vectors fetched via bimba_embed WITHOUT store_for (read-only path)",
            "different edge counts (full vs symbol) mean cross-set comparison is directional, not exact",
            "small n: one coordinate family, ~109 nodes; treat deltas within +/-0.05 as noise",
            "the M0-local head trains on ~4/5 of an already small edge set (starvation caveat from phase 2.6 applies)",
            "readings are instrument output; promotion none; low energy is compatibility, nothing else",
        ],
        "seeds": {"probe": args.seed},
        "runtime_seconds": round(time.time() - t0, 1),
        "provenance": {"promotion": "none", "reading_class": "research"},
    }
    args.out.mkdir(parents=True, exist_ok=True)
    out_path = args.out / f"m0-symbol-probe-{time.strftime('%Y%m%d-%H%M%S')}.json"
    out_path.write_text(json.dumps(evidence, indent=2, default=float) + "\n")
    print(json.dumps({
        "wrote": str(out_path),
        "full_content": full,
        "symbol_only": sym,
        "symbol_head": {k: v.get("head") for k, v in head_res.items()},
    }, indent=1, default=float))


if __name__ == "__main__":
    sys.exit(main())
