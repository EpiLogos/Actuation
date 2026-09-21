#!/usr/bin/env python3
"""E-MT EBM Phase 2 — learned energy head over the bimba field.

Trains a small energy model on the live graph's real relations, using the
Phase-1 corruption ladder as the negative sampler, and evaluates against the
Phase-1 declared potentials on held-out edges. Targets the gaps Phase 1
measured: direction sensitivity, potential weights, and sibling separation.

Honesty: the learned reading is semantic-stochastic, never promoted; low
energy means compatibility with learned structure, nothing else. Train/eval
splits are by edge digest so no eval edge (or a corruption of one) is trained
on. Direction-reversal negatives share the unordered pair with their positive,
so the asymmetric features (ancestor-of in each direction, per-side depth and
family) are what the head must use — the pair-symmetric Phase-1 potentials
score exactly 0.5 there by construction.
"""

import argparse
import hashlib
import json
import sys
import time
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent))
from field_probe import (  # noqa: E402
    as_vector,
    detect_vector_key,
    lineage_energy,
    parse_coord,
    rank_auc,
    root_of,
    parent_key,
    TYPE_LETTERS,
)

FAMILIES = list(TYPE_LETTERS)
HASH_DIMS = 32


def h(s):
    return hashlib.sha256(s.encode()).digest()


def edge_id(ia, ib, t):
    return f"{ia}|{ib}|{t}"


def type_hash_feats(rel_type):
    """Signed feature hashing of the live relation type into HASH_DIMS dims."""
    v = np.zeros(HASH_DIMS, dtype=np.float32)
    for i in range(2):
        d = h(f"{rel_type}#{i}")
        idx = int.from_bytes(d[:4], "little") % HASH_DIMS
        sign = 1.0 if d[4] % 2 else -1.0
        v[idx] += sign
    return v


def seg_prefix(sa, sb):
    lcp = 0
    for x, y in zip(sa, sb):
        if x != y:
            break
        lcp += 1
    return lcp


def features(a, b, rel_type, vector_key):
    """Directed feature vector for state (a, r, b). None semantic if vectors missing."""
    pa, pb = a["parsed"], b["parsed"]
    sem_ok = True
    if vector_key:
        if not a["embedding"] or not b["embedding"]:
            sem_ok = False
    lcp = seg_prefix(pa["segments"], pb["segments"])
    a_anc = pa["type"] == pb["type"] and lcp == len(pa["segments"]) and len(pa["segments"]) < len(pb["segments"])
    b_anc = pa["type"] == pb["type"] and lcp == len(pb["segments"]) and len(pb["segments"]) < len(pa["segments"])
    same_parent = parent_key(pa) is not None and parent_key(pa) == parent_key(pb)
    same_root = root_of(pa) == root_of(pb)

    fam_a = [0.0] * len(FAMILIES)
    fam_a[FAMILIES.index(pa["type"])] = 1.0
    fam_b = [0.0] * len(FAMILIES)
    fam_b[FAMILIES.index(pb["type"])] = 1.0
    root_a = [0.0] * 6
    if pa["segments"]:
        root_a[pa["segments"][0] % 6] = 1.0
    root_b = [0.0] * 6
    if pb["segments"]:
        root_b[pb["segments"][0] % 6] = 1.0

    lin = lineage_energy(pa, pb)
    if vector_key:
        if sem_ok:
            u, v = np.asarray(a["embedding"], dtype=np.float32), np.asarray(b["embedding"], dtype=np.float32)
            cos = float(u @ v / ((np.linalg.norm(u) * np.linalg.norm(v)) or 1.0))
            sem = 1.0 - cos
        else:
            sem = None
    else:
        inter = len(a["tok"] & b["tok"])
        union = len(a["tok"] | b["tok"]) or 1
        sem = 1.0 - inter / union

    dense = [
        lin,
        0.0 if sem is None else sem,          # semantic energy (imputed 0 when missing; mask carries honesty)
        0.0 if sem is None else 1.0,          # semantic-availability mask
        float(lcp),
        float(a_anc),                          # asymmetric: direction-sensitive
        float(b_anc),
        float(same_parent),
        float(same_root),
        float(pa["type"] == pb["type"]),
        float(len(pa["segments"])),
        float(len(pb["segments"])),
        float(abs(len(pa["segments"]) - len(pb["segments"]))),
        float(pa["is_prime"]),
        float(pb["is_prime"]),
        float(pa["is_prime"] == pb["is_prime"]),
    ]
    x = np.array(dense + fam_a + fam_b + root_a + root_b, dtype=np.float32)
    return np.concatenate([x, type_hash_feats(rel_type)]), sem


FEATURE_DIM = 15 + 2 * len(FAMILIES) + 2 * 6 + HASH_DIMS


class MLP:
    """1-hidden-layer energy head; logits are the (negative) energy."""

    def __init__(self, dim, hidden=32, seed=7):
        rng = np.random.default_rng(seed)
        self.W1 = rng.normal(0, 0.05, (dim, hidden)).astype(np.float32)
        self.b1 = np.zeros(hidden, dtype=np.float32)
        self.W2 = rng.normal(0, 0.05, (hidden, 1)).astype(np.float32)
        self.b2 = np.zeros(1, dtype=np.float32)

    def _forward(self, X):
        H = np.maximum(X @ self.W1 + self.b1, 0.0)
        return (H @ self.W2 + self.b2).ravel(), H

    def forward(self, X):
        return self._forward(X)[0]

    def predict(self, X):
        return self.forward(X)

    def fit(self, X, y, epochs=120, lr=1e-2, bs=256, seed=7):
        rng = np.random.default_rng(seed)
        n = len(X)
        for _ in range(epochs):
            order = rng.permutation(n)
            for i in range(0, n, bs):
                idx = order[i:i + bs]
                xb, yb = X[idx], y[idx]
                z, Hb = self._forward(xb)
                p = 1.0 / (1.0 + np.exp(-z))
                g = (p - yb).astype(np.float32)          # dL/dz for BCE
                gW2 = (Hb.T @ g[:, None]) / len(idx)
                gb2 = g.mean(keepdims=True)
                gH = g[:, None] @ self.W2.T
                gH = gH * (Hb > 0)
                gW1 = (xb.T @ gH) / len(idx)
                gb1 = gH.mean(axis=0)
                self.W2 -= lr * gW2
                self.b2 -= lr * gb2
                self.W1 -= lr * gW1
                self.b1 -= lr * gb1


def corrupt(op, ia, ib, t, nodes, rng, root_pool, parent_pool, all_positioned):
    """Return (a, b_corrupt, type) for a negative, or None."""
    a, b = nodes[ia], nodes[ib]
    if op == "direction-reversal":
        return b, a, t
    if op == "endpoint-rewire":
        pool = [n for n in all_positioned if root_of(n["parsed"]) != root_of(b["parsed"])]
    elif op == "hard-rewire":
        pool = [n for n in root_pool.get(root_of(b["parsed"]), []) if n is not b]
    else:  # sibling-rewire
        pool = [n for n in parent_pool.get(parent_key(b["parsed"]), []) if n is not b]
    if not pool:
        return None
    return a, rng.choice(pool), t


def build_xy(pairs, nodes, vector_key):
    X, sems = [], []
    for a, b, t in pairs:
        x, sem = features(a, b, t, vector_key)
        X.append(x)
        sems.append(sem)
    return np.stack(X), sems


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--uri", default="bolt://100.92.62.101:7687")
    ap.add_argument("--user", default="neo4j")
    ap.add_argument("--password", default="bimba")
    ap.add_argument("--database", default="neo4j")
    ap.add_argument("--neg-per-pos", type=int, default=2, help="negatives per positive, per operator")
    ap.add_argument("--seed", type=int, default=20260918)
    ap.add_argument("--out", type=Path, default=Path(__file__).resolve().parent / "evidence")
    args = ap.parse_args()

    from neo4j import GraphDatabase

    rng = np.random.default_rng(args.seed)
    pyrng = __import__("random").Random(args.seed)
    t0 = time.time()

    driver = GraphDatabase.driver(args.uri, auth=(args.user, args.password))
    with driver.session(database=args.database) as session:
        vector_key = detect_vector_key(session)
        vec_prop = f", n.{vector_key} AS embedding" if vector_key else ""
        nodes = {}
        for rec in session.run(
            "MATCH (n:Bimba) RETURN elementId(n) AS id, "
            "coalesce(n.bimbaCoordinate, n.coordinate) AS coord, "
            "coalesce(n.c_1_name, n.title) AS name, "
            "n.description AS description" + vec_prop
        ):
            parsed = parse_coord(rec["coord"])
            if parsed is None:
                continue
            nodes[rec["id"]] = {
                "coord": rec["coord"],
                "parsed": parsed,
                "embedding": as_vector(rec.get("embedding")),
                "tok": set(__import__("re").findall(r"[a-z0-9]{3,}", (str(rec.get("name") or "") + " " + str(rec.get("description") or "")).lower())),
            }
        edges = [
            (rec["ia"], rec["ib"], rec["t"])
            for rec in session.run(
                "MATCH (a:Bimba)-[r]->(b:Bimba) "
                "RETURN elementId(a) AS ia, elementId(b) AS ib, type(r) AS t"
            )
        ]
    driver.close()

    valid = [
        (ia, ib, t) for ia, ib, t in edges
        if ia in nodes and ib in nodes and nodes[ia]["embedding"] and nodes[ib]["embedding"]
    ]
    # held-out split by edge digest: no eval edge is ever trained on
    train = [e for e in valid if int.from_bytes(h(edge_id(*e))[:4], "little") % 5 != 0]
    test = [e for e in valid if e not in train]
    print(json.dumps({"edges_valid": len(valid), "train": len(train), "test": len(test),
                      "vector_key": vector_key}))

    root_pool, parent_pool, all_positioned = {}, {}, list(nodes.values())
    from collections import defaultdict
    root_pool = defaultdict(list)
    parent_pool = defaultdict(list)
    for n in nodes.values():
        root_pool[root_of(n["parsed"])].append(n)
        pk = parent_key(n["parsed"])
        if pk is not None:
            parent_pool[pk].append(n)

    OPS = ["endpoint-rewire", "hard-rewire", "sibling-rewire", "direction-reversal"]

    # ---- training set: train edges (+) and their corruptions (-)
    Xs, ys = [], []
    for op in OPS:
        for ia, ib, t in train:
            pos = (nodes[ia], nodes[ib], t)
            xp, _ = features(pos[0], pos[1], t, vector_key)
            Xs.append(xp)
            ys.append(1.0)
            for _ in range(args.neg_per_pos):
                c = corrupt(op, ia, ib, t, nodes, pyrng, root_pool, parent_pool, all_positioned)
                if c is None:
                    continue
                xn, _ = features(c[0], c[1], c[2], vector_key)
                Xs.append(xn)
                ys.append(0.0)
    X = np.stack(Xs)
    y = np.array(ys, dtype=np.float32)
    print(json.dumps({"train_matrix": list(X.shape), "pos_frac": round(float(y.mean()), 3)}))

    head = MLP(FEATURE_DIM, hidden=32, seed=args.seed % 100000)
    head.fit(X, y, epochs=120, lr=1e-2, bs=256, seed=args.seed % 100000)

    # energy = -logit; separation_auc reads P(real energy < corrupted energy)
    # via rank_auc with the corrupted list in the positives role.
    def auc_for(op, subset):
        pos_pairs, neg_pairs = [], []
        for ia, ib, t in subset:
            a, b = nodes[ia], nodes[ib]
            if not (a["embedding"] and b["embedding"]):
                continue
            pos_pairs.append((a, b, t))
            c = corrupt(op, ia, ib, t, nodes, pyrng, root_pool, parent_pool, all_positioned)
            if c is not None and c[0]["embedding"] and c[1]["embedding"]:
                neg_pairs.append(c)
        if not pos_pairs or not neg_pairs:
            return None, 0
        Xp, sems_p = build_xy(pos_pairs, nodes, vector_key)
        Xn, sems_n = build_xy(neg_pairs, nodes, vector_key)
        learned = rank_auc((-head.predict(Xn)).tolist(), (-head.predict(Xp)).tolist())
        lin_p = [lineage_energy(a["parsed"], b["parsed"]) for a, b, _ in pos_pairs]
        lin_n = [lineage_energy(a["parsed"], b["parsed"]) for a, b, _ in neg_pairs]
        sem_p = [0.0 if s is None else s for s in sems_p]
        sem_n = [0.0 if s is None else s for s in sems_n]
        base = {
            "lineage": rank_auc(lin_n, lin_p),
            "semantic": rank_auc(sem_n, sem_p),
            "combined": rank_auc(
                [l + s for l, s in zip(lin_n, sem_n)],
                [l + s for l, s in zip(lin_p, sem_p)],
            ),
        }
        return {"learned": learned, **{f"baseline_{k}": v for k, v in base.items()}}, len(pos_pairs)

    results = {}
    for op in OPS:
        res, n_pos = auc_for(op, test)
        results[op] = {"n_pos": n_pos, **(res or {})}

    # M0 focus on held-out edges: self-contained field — corruption pools
    # restricted to M0 nodes, so every operator stays comparable to the
    # probe's --root M0 run (endpoint-rewire goes vacuous there too).
    m0_test = [e for e in test
               if root_of(nodes[e[0]]["parsed"]) == ("M", (0,)) and root_of(nodes[e[1]]["parsed"]) == ("M", (0,))]
    m0_nodes = {nid: n for nid, n in nodes.items()
                if root_of(n["parsed"]) == ("M", (0,))}
    m0_root_pool, m0_parent_pool = defaultdict(list), defaultdict(list)
    for n in m0_nodes.values():
        m0_root_pool[root_of(n["parsed"])].append(n)
        pk = parent_key(n["parsed"])
        if pk is not None:
            m0_parent_pool[pk].append(n)
    m0_all = list(m0_nodes.values())
    m0 = {}
    for op in OPS:
        def auc_m0(subset, op=op):
            pos_pairs, neg_pairs = [], []
            for ia, ib, t in subset:
                a, b = nodes[ia], nodes[ib]
                if not (a["embedding"] and b["embedding"]):
                    continue
                pos_pairs.append((a, b, t))
                c = corrupt(op, ia, ib, t, m0_nodes, pyrng, m0_root_pool, m0_parent_pool, m0_all)
                if c is not None and c[0]["embedding"] and c[1]["embedding"]:
                    neg_pairs.append(c)
            if not pos_pairs or not neg_pairs:
                return None, 0
            Xp, sems_p = build_xy(pos_pairs, nodes, vector_key)
            Xn, sems_n = build_xy(neg_pairs, nodes, vector_key)
            learned = rank_auc((-head.predict(Xn)).tolist(), (-head.predict(Xp)).tolist())
            lin_p = [lineage_energy(a["parsed"], b["parsed"]) for a, b, _ in pos_pairs]
            lin_n = [lineage_energy(a["parsed"], b["parsed"]) for a, b, _ in neg_pairs]
            sem_p = [0.0 if s is None else s for s in sems_p]
            sem_n = [0.0 if s is None else s for s in sems_n]
            base = {
                "lineage": rank_auc(lin_n, lin_p),
                "semantic": rank_auc(sem_n, sem_p),
                "combined": rank_auc(
                    [l + s for l, s in zip(lin_n, sem_n)],
                    [l + s for l, s in zip(lin_p, sem_p)],
                ),
            }
            return {"learned": learned, **{f"baseline_{k}": v for k, v in base.items()}}, len(pos_pairs)
        res, n_pos = auc_m0(m0_test)
        m0[op] = {"n_pos": n_pos, **(res or {})}

    evidence = {
        "schema": "actuation.model-types-ebm-energy-head/v1",
        "created": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "experiment": "E-MT-EBM-FIELD phase-2 (learned energy head)",
        "model": {
            "kind": "mlp-1x32-bce",
            "features": "lineage, semantic(cos)+mask, lcp, directed ancestor flags, sibling/root/family one-hots, depths, prime flags, 32-dim signed hash of live relation type",
            "feature_dim": FEATURE_DIM,
            "negatives": "corruption ladder on train edges",
            "split": "held-out edges by sha256 digest bucket (mod 5), corruptions drawn only from train edges for fitting; eval negatives from held-out edges",
        },
        "data": {
            "vector_key": vector_key,
            "edges_valid": len(valid),
            "train_edges": len(train),
            "test_edges": len(test),
            "m0_test_edges": len(m0_test),
        },
        "results_full": results,
        "results_m0_focus": m0,
        "honest_limits": [
            "learned readings are semantic-stochastic; never promoted, never the determination",
            "negatives are synthetic corruptions; AUC measures separation from the ladder, not value or correctness",
            "loop-proposed transitions (Phase 3 gate) remain untested until the composed loop runs",
        ],
        "runtime_seconds": round(time.time() - t0, 1),
        "provenance": {"promotion": "none", "reading_class": "semantic-stochastic"},
    }

    args.out.mkdir(parents=True, exist_ok=True)
    out_path = args.out / f"energy-head-{time.strftime('%Y%m%d-%H%M%S')}.json"
    npz_path = args.out / "energy-head-weights.npz"
    np.savez(npz_path, W1=head.W1, b1=head.b1, W2=head.W2, b2=head.b2)
    evidence["artifact"] = str(npz_path)
    out_path.write_text(json.dumps(evidence, indent=2) + "\n")
    print(json.dumps({
        "wrote": str(out_path),
        "full": {op: results[op] for op in OPS},
        "m0": {op: m0[op] for op in OPS},
    }, indent=2))


if __name__ == "__main__":
    sys.exit(main())
