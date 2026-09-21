#!/usr/bin/env python3
"""E-MT EBM Phase 2.6 — head redesign + sibling-frontier ablations (R1, R2).

A rung ladder over the Phase-2 energy head where each rung adds exactly one
idea, so the effect of each idea is reported separately (NEXT-SESSION.md R1/R2;
seeds fixed: probe 20260917, head 20260918):

  rung0  base       replication of the Phase-2 head (scalar semantic channel,
                    32 hidden, 32-dim type hash) — sanity anchor vs
                    train_energy_head.py
  rung1  semantic   R1(a)+R2 semantic channel: PCA-64 projections of e_a, e_b,
                    e_a−e_b, e_a⊙e_b plus cosine, fed directly (not one scalar)
  rung2  typed      R1(b) relation-type conditioning: type hash widened to 128
                    dims + hashed incident-relation-type profile and log-degree
                    of both endpoints (the pair read inside its type neighbourhood)
  rung3  capacity   R2 capacity + calibration: 2×128 hidden; held-out Brier, ECE
                    and threshold table so a loop gate can threshold the output
  rung4  hardneg    R1(c) hard-negative mining: sibling train negatives the
                    rung3 model ranks wrongly are oversampled for a low-LR
                    fine-tune of the rung3 weights
  m0     m0only     R1(d) per-root head: rung3 architecture trained on M0 edges
                    only (M0-restricted corruption pools), eval on M0 held-out
                    against the global head and raw cosine on the same split

Honesty law unchanged: learned readings are semantic-stochastic, promotion
none, never a value or correctness claim. Split is by edge digest (mod 5) as
in Phase 2; corruptions for fitting are drawn from train edges only.
"""

import argparse
import json
import sys
import time
from collections import defaultdict
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent))
from field_probe import (  # noqa: E402
    as_vector,
    detect_vector_key,
    lineage_energy,
    parent_key,
    parse_coord,
    rank_auc,
    root_of,
    TYPE_LETTERS,
)
from train_energy_head import corrupt, edge_id, h, seg_prefix  # noqa: E402

FAMILIES = list(TYPE_LETTERS)
PCA_DIM = 64
TYPE_HASH_DIMS = 128
PROFILE_HASH_DIMS = 64
CACHE = Path("/tmp/mt-graph-cache-v2")


# ---------------------------------------------------------------- graph load

def load_graph(uri, database):
    """Load nodes (coord/embedding) + edges + type profiles once; cache in /tmp."""
    if CACHE.with_suffix(".done").exists():
        return (
            json.loads(CACHE.with_suffix(".meta.json").read_text()),
            np.load(CACHE.with_suffix(".emb.npy")),
            json.loads(CACHE.with_suffix(".edges.json").read_text()),
            json.loads(CACHE.with_suffix(".profiles.json").read_text()),
        )
    from neo4j import GraphDatabase

    driver = GraphDatabase.driver(uri, auth=("neo4j", "bimba"))
    with driver.session(database=database) as session:
        vector_key = detect_vector_key(session)
        nodes, order = {}, []
        for rec in session.run(
            "MATCH (n:Bimba) RETURN elementId(n) AS id, "
            "coalesce(n.bimbaCoordinate, n.coordinate) AS coord, "
            "coalesce(n.c_1_name, n.title) AS name, "
            "n.description AS description" + (
                f", n.{vector_key} AS embedding" if vector_key else ""
            )
        ):
            parsed = parse_coord(rec["coord"])
            if parsed is None:
                continue
            nodes[rec["id"]] = {
                "coord": rec["coord"],
                "parsed": parsed,
                "emb_idx": len(order),
            }
            order.append(as_vector(rec.get("embedding")) if vector_key else None)
        edges = [
            (rec["ia"], rec["ib"], rec["t"])
            for rec in session.run(
                "MATCH (a:Bimba)-[r]->(b:Bimba) "
                "RETURN elementId(a) AS ia, elementId(b) AS ib, type(r) AS t"
            )
        ]
        # incident relation-type profile per node: which types touch it, how often
        inc = defaultdict(lambda: defaultdict(int))
        for ia, ib, t in edges:
            if ia in nodes:
                inc[ia][t] += 1
            if ib in nodes:
                inc[ib][t] += 1
        profiles = {
            nid: {"deg": sum(tc.values()), "types": dict(tc)}
            for nid, tc in inc.items()
        }
    driver.close()

    dim = max((len(v) for v in order if v), default=0)
    emb = np.zeros((len(order), dim), dtype=np.float32)
    for i, v in enumerate(order):
        if v and len(v) == dim:
            emb[i] = v

    CACHE.parent.mkdir(exist_ok=True)
    np.save(CACHE.with_suffix(".emb.npy"), emb)
    CACHE.with_suffix(".meta.json").write_text(json.dumps(
        {"vector_key": vector_key, "dim": dim,
         "nodes": {k: {"coord": v["coord"], "emb_idx": v["emb_idx"]} for k, v in nodes.items()}}))
    CACHE.with_suffix(".edges.json").write_text(json.dumps(edges))
    CACHE.with_suffix(".profiles.json").write_text(json.dumps(profiles))
    CACHE.with_suffix(".done").write_text("ok")
    return (
        {"vector_key": vector_key, "dim": dim,
         "nodes": {k: {"coord": v["coord"], "emb_idx": v["emb_idx"]} for k, v in nodes.items()}},
        emb, edges, profiles,
    )


# ---------------------------------------------------------------- features

def fit_pca(mat, k):
    """Deterministic PCA to k dims; returns (mean, components[k])."""
    mean = mat.mean(axis=0)
    xc = mat - mean
    cov = xc.T @ xc / len(xc)
    w, v = np.linalg.eigh(cov)
    comps = v[:, ::-1][:, :k]  # top-k eigenvectors, descending eigenvalue
    return mean.astype(np.float32), comps.astype(np.float32)


def type_hash(vec, rel_type, dims):
    for i in range(2):
        d = h(f"{rel_type}#{i}")
        idx = int.from_bytes(d[:4], "little") % dims
        sign = 1.0 if d[4] % 2 else -1.0
        vec[idx] += sign


def profile_hash(vec, types_counter, dims):
    """Signed hashing of a node's incident relation-type bag (type:count)."""
    for t, c in sorted(types_counter.items()):
        d = h(f"prof|{t}")
        idx = int.from_bytes(d[:4], "little") % dims
        sign = 1.0 if d[4] % 2 else -1.0
        vec[idx] += sign * float(min(c, 8)) / 8.0


class FeatureSpace:
    """Feature builder parameterised by rung so each idea can be isolated.

    rung0: Phase-2 features verbatim (32-dim type hash, scalar semantic).
    rung1: + PCA semantic channel (e_a, e_b, e_a−e_b, e_a⊙e_b, cos).
    rung2: + 128-dim type hash (replacing the 32-dim one) + incident-type
           profile hashes + log-degree for both endpoints.
    """

    def __init__(self, nodes, emb, profiles, rung, pcas=None):
        self.nodes = nodes
        self.emb = emb
        self.profiles = profiles
        self.rung = rung
        self.pcab, self.pcad, self.pcap = pcas if pcas else (None, None, None)
        self.type_dims = TYPE_HASH_DIMS if rung >= 2 else 32
        self.dim = self._dim()

    def _dim(self):
        d = 15 + 2 * len(FAMILIES) + 2 * 6 + self.type_dims
        if self.rung >= 1:
            d += 4 * PCA_DIM + 1
        if self.rung >= 2:
            d += 2 * PROFILE_HASH_DIMS + 2
        return d

    def pca_block(self, a, b):
        ea = self.emb[a["emb_idx"]][None, :]
        eb = self.emb[b["emb_idx"]][None, :]
        diff = ea - eb
        prod = ea * eb
        blocks = [
            (ea - self.pcab[0]) @ self.pcab[1],
            (eb - self.pcab[0]) @ self.pcab[1],
            (diff - self.pcad[0]) @ self.pcad[1],
            (prod - self.pcap[0]) @ self.pcap[1],
        ]
        cos = float((ea * eb).sum() / ((np.linalg.norm(ea) * np.linalg.norm(eb)) or 1.0))
        return np.concatenate([*blocks, [[cos]]], axis=1).ravel().astype(np.float32)

    def features(self, a, b, rel_type):
        """Directed feature vector for state (a, r, b)."""
        pa, pb = a["parsed"], b["parsed"]
        sem_ok = bool(a["emb_vec"].any()) and bool(b["emb_vec"].any())
        lcp = seg_prefix(pa["segments"], pb["segments"])
        a_anc = (pa["type"] == pb["type"] and lcp == len(pa["segments"])
                 and len(pa["segments"]) < len(pb["segments"]))
        b_anc = (pa["type"] == pb["type"] and lcp == len(pb["segments"])
                 and len(pb["segments"]) < len(pa["segments"]))
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
        if sem_ok:
            cos = float(self.emb[a["emb_idx"]] @ self.emb[b["emb_idx"]]
                        / ((np.linalg.norm(self.emb[a["emb_idx"]])
                            * np.linalg.norm(self.emb[b["emb_idx"]])) or 1.0))
            sem = 1.0 - cos
        else:
            cos, sem = 0.0, None

        dense = [
            lin,
            0.0 if sem is None else sem,
            0.0 if sem is None else 1.0,
            float(lcp),
            float(a_anc),
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
        parts = [np.array(dense + fam_a + fam_b + root_a + root_b, dtype=np.float32)]
        if self.rung >= 1:
            if sem_ok:
                parts.append(self.pca_block(a, b))
            else:
                parts.append(np.zeros(4 * PCA_DIM + 1, dtype=np.float32))

        th = np.zeros(self.type_dims, dtype=np.float32)
        type_hash(th, rel_type, self.type_dims)
        parts.append(th)

        if self.rung >= 2:
            prof = np.zeros(2 * PROFILE_HASH_DIMS + 2, dtype=np.float32)
            for side, node in ((0, a), (1, b)):
                p = self.profiles.get(node["nid"])
                if p:
                    off = side * PROFILE_HASH_DIMS
                    sub = prof[off:off + PROFILE_HASH_DIMS]
                    profile_hash(sub, p["types"], PROFILE_HASH_DIMS)
                    prof[2 * PROFILE_HASH_DIMS + side] = float(np.log1p(p["deg"]))
            parts.append(prof)
        return np.concatenate(parts)


# ---------------------------------------------------------------- model

class MLP2:
    """ReLU MLP (1 or 2 hidden layers); logit = -(energy)."""

    def __init__(self, dim, hidden, layers, seed):
        rng = np.random.default_rng(seed)
        self.layers = layers
        dims = [dim] + [hidden] * layers + [1]
        self.W = [rng.normal(0, 0.05, (dims[i], dims[i + 1])).astype(np.float32)
                  for i in range(len(dims) - 1)]
        self.b = [np.zeros(d, dtype=np.float32) for d in dims[1:]]

    def _forward(self, X):
        acts, Hs = X, []
        for i in range(self.layers):
            H = np.maximum(acts @ self.W[i] + self.b[i], 0.0)
            Hs.append(H)
            acts = H
        z = (acts @ self.W[-1] + self.b[-1]).ravel()
        return z, Hs

    def predict(self, X):
        return self._forward(X)[0]

    def fit(self, X, y, epochs=120, lr=1e-2, bs=256, seed=7):
        rng = np.random.default_rng(seed)
        n = len(X)
        for _ in range(epochs):
            order = rng.permutation(n)
            for i in range(0, n, bs):
                idx = order[i:i + bs]
                xb, yb = X[idx], y[idx]
                z, Hs = self._forward(xb)
                g = (1.0 / (1.0 + np.exp(-z)) - yb).astype(np.float32)
                grads_W = [None] * len(self.W)
                grads_b = [None] * len(self.b)
                grads_W[-1] = (Hs[-1].T @ g[:, None]) / len(idx) if Hs else None
                grads_b[-1] = g.mean(keepdims=True)
                gact = g[:, None] @ self.W[-1].T
                for li in range(self.layers - 1, -1, -1):
                    acts_prev = xb if li == 0 else Hs[li - 1]
                    gH = gact * (Hs[li] > 0)
                    grads_W[li] = (acts_prev.T @ gH) / len(idx)
                    grads_b[li] = gH.mean(axis=0)
                    if li > 0:
                        gact = gH @ self.W[li].T
                for li in range(len(self.W)):
                    self.W[li] -= lr * grads_W[li]
                    self.b[li] -= lr * grads_b[li]


# ---------------------------------------------------------------- helpers

def calib_readout(p_pos, p_neg):
    """Brier, ECE-10, Youden threshold, recall and FPR at that threshold."""
    y = np.concatenate([np.ones(len(p_pos)), np.zeros(len(p_neg))])
    p = np.concatenate([p_pos, p_neg])
    brier = float(np.mean((p - y) ** 2))
    ece = 0.0
    for b in range(10):
        lo, hi = b / 10, (b + 1) / 10
        m = (p >= lo) & ((p < hi) if b < 9 else (p <= hi))
        if m.any():
            ece += m.mean() * abs(y[m].mean() - p[m].mean())
    order = np.argsort(-p)
    ys = y[order]
    tp = np.cumsum(ys)
    fp = np.cumsum(1 - ys)
    tpr = tp / max(tp[-1], 1)
    fpr = fp / max(fp[-1], 1)
    j = tpr - fpr
    k = int(np.argmax(j))
    return {
        "brier": round(brier, 4),
        "ece_10bin": round(ece, 4),
        "mean_p_real": round(float(p_pos.mean()), 4),
        "mean_p_corrupted": round(float(p_neg.mean()), 4),
        "youden_threshold": round(float(p[order][k]), 4),
        "recall_at_threshold": round(float(tpr[k]), 4),
        "false_positive_rate_at_threshold": round(float(fpr[k]), 4),
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--uri", default="bolt://100.92.62.101:7687")
    ap.add_argument("--database", default="neo4j")
    ap.add_argument("--neg-per-pos", type=int, default=2)
    ap.add_argument("--seed", type=int, default=20260918)
    ap.add_argument("--out", type=Path, default=Path(__file__).resolve().parent / "evidence")
    args = ap.parse_args()

    import random as pyrandom

    pyrng = pyrandom.Random(args.seed)
    rng = np.random.default_rng(args.seed)
    t0 = time.time()

    meta, emb, edges, profiles = load_graph(args.uri, args.database)
    nodes = {}
    for nid, m in meta["nodes"].items():
        parsed = parse_coord(m["coord"])
        if parsed is None:
            continue
        nodes[nid] = {
            "nid": nid,
            "coord": m["coord"],
            "parsed": parsed,
            "emb_idx": m["emb_idx"],
            "emb_vec": emb[m["emb_idx"]],
        }

    vector_key = meta["vector_key"]
    valid = [
        (ia, ib, t) for ia, ib, t in edges
        if ia in nodes and ib in nodes
        and nodes[ia]["emb_vec"].any() and nodes[ib]["emb_vec"].any()
    ]
    train = [e for e in valid if int.from_bytes(h(edge_id(*e))[:4], "little") % 5 != 0]
    test = [e for e in valid if e not in train]

    root_pool, parent_pool = defaultdict(list), defaultdict(list)
    for n in nodes.values():
        root_pool[root_of(n["parsed"])].append(n)
        pk = parent_key(n["parsed"])
        if pk is not None:
            parent_pool[pk].append(n)
    all_positioned = list(nodes.values())

    m0_nodes = {nid: n for nid, n in nodes.items() if root_of(n["parsed"]) == ("M", (0,))}
    m0_root_pool, m0_parent_pool = defaultdict(list), defaultdict(list)
    for n in m0_nodes.values():
        m0_root_pool[root_of(n["parsed"])].append(n)
        pk = parent_key(n["parsed"])
        if pk is not None:
            m0_parent_pool[pk].append(n)
    m0_all = list(m0_nodes.values())
    m0_test = [e for e in test if e[0] in m0_nodes and e[1] in m0_nodes]
    m0_train = [e for e in train if e[0] in m0_nodes and e[1] in m0_nodes]

    OPS = ["endpoint-rewire", "hard-rewire", "sibling-rewire", "direction-reversal"]

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
        return {
            "cosine": rank_auc(cos_n, cos_p),
            "lineage": rank_auc(lin_n, lin_p),
            "combined": rank_auc([l + c for l, c in zip(lin_n, cos_n)],
                                 [l + c for l, c in zip(lin_p, cos_p)]),
        }

    def eval_head(head, fs, pos, neg):
        Xp = np.stack([fs.features(a, b, t) for a, b, t in pos])
        Xn = np.stack([fs.features(a, b, t) for a, b, t in neg])
        return {
            "learned": rank_auc((-head.predict(Xn)).tolist(), (-head.predict(Xp)).tolist()),
            "calibration": calib_readout(
                1.0 / (1.0 + np.exp(-head.predict(Xp))),
                1.0 / (1.0 + np.exp(-head.predict(Xn))),
            ),
        }

    # ---- PCA bases fitted once on the field's own node vectors
    idx = [n["emb_idx"] for n in nodes.values() if n["emb_vec"].any()]
    mat = emb[idx].astype(np.float64)
    n_pairs = min(len(mat) // 2, 2000)
    diff_mat = mat[0:2 * n_pairs:2] - mat[1:2 * n_pairs:2]
    prod_mat = mat[0:2 * n_pairs:2] * mat[1:2 * n_pairs:2]
    print(json.dumps({"pca": "fitting 3 bases", "n": len(mat)}))
    pcas = (fit_pca(mat, PCA_DIM), fit_pca(diff_mat, PCA_DIM), fit_pca(prod_mat, PCA_DIM))

    rungs = {}

    # ---- rung0/1/2: feature ladder at 1x32 capacity
    for rung, name in ((0, "base"), (1, "semantic_channel"), (2, "typed")):
        fs = FeatureSpace(nodes, emb, profiles, rung, pcas=pcas)
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
        res, m0 = {}, {}
        for op in OPS:
            pos, neg = make_pairs(op, test, nodes, root_pool, parent_pool, all_positioned)
            if not pos or not neg:
                res[op] = {"n_pos": len(pos)}
                continue
            res[op] = {"n_pos": len(pos), **eval_head(head, fs, pos, neg),
                       **{f"baseline_{k}": v for k, v in baselines(pos, neg).items()}}
        for op in ("hard-rewire", "sibling-rewire", "direction-reversal"):
            pos, neg = make_pairs(op, m0_test, m0_nodes, m0_root_pool, m0_parent_pool, m0_all)
            if not pos or not neg:
                m0[op] = {"n_pos": len(pos)}
                continue
            m0[op] = {"n_pos": len(pos), **eval_head(head, fs, pos, neg),
                      **{f"baseline_{k}": v for k, v in baselines(pos, neg).items()}}
        rungs[name] = {"rung": rung, "feature_dim": fs.dim, "arch": "1x32",
                       "full": res, "m0": m0}
        sib = res.get("sibling-rewire", {})
        print(json.dumps({"rung": name, "feature_dim": fs.dim,
                          "full_sibling_learned": sib.get("learned")}))

    # ---- rung3: capacity 2x128 on typed features + calibration
    fs3 = FeatureSpace(nodes, emb, profiles, 2, pcas=pcas)
    Xs, ys = [], []
    for op in OPS:
        pos, neg = make_pairs(op, train, nodes, root_pool, parent_pool, all_positioned)
        for a, b, t in pos:
            Xs.append(fs3.features(a, b, t))
            ys.append(1.0)
        for a, b, t in neg:
            Xs.append(fs3.features(a, b, t))
            ys.append(0.0)
    X3 = np.stack(Xs)
    y3 = np.array(ys, dtype=np.float32)
    head3 = MLP2(fs3.dim, hidden=128, layers=2, seed=args.seed % 100000)
    head3.fit(X3, y3, epochs=120, lr=3e-3, bs=256, seed=args.seed % 100000)

    def eval_cap(head, res_into, key, op, subset, nodes_map, rp, pp, al):
        pos, neg = make_pairs(op, subset, nodes_map, rp, pp, al)
        if not pos or not neg:
            res_into[key] = {"n_pos": len(pos)}
            return
        res_into[key] = {"n_pos": len(pos), **eval_head(head, fs3, pos, neg),
                         **{f"baseline_{k}": v for k, v in baselines(pos, neg).items()}}

    res3, m03 = {}, {}
    for op in OPS:
        eval_cap(head3, res3, op, op, test, nodes, root_pool, parent_pool, all_positioned)
    for op in ("hard-rewire", "sibling-rewire", "direction-reversal"):
        eval_cap(head3, m03, op, op, m0_test, m0_nodes, m0_root_pool, m0_parent_pool, m0_all)
    rungs["capacity"] = {"rung": 3, "feature_dim": fs3.dim, "arch": "2x128",
                         "full": res3, "m0": m03}
    print(json.dumps({"rung": "capacity",
                      "full_sibling_learned": res3.get("sibling-rewire", {}).get("learned")}))

    # ---- rung4: hard-negative mining on sibling (train edges only)
    # aligned per-edge: (real edge, its own sibling corruption) kept only as a pair
    al_pos, al_neg = [], []
    for ia, ib, t in train:
        a, b = nodes[ia], nodes[ib]
        if not (a["emb_vec"].any() and b["emb_vec"].any()):
            continue
        c = corrupt("sibling-rewire", ia, ib, t, nodes, pyrng, root_pool, parent_pool, all_positioned)
        if c is not None and c[0]["emb_vec"].any() and c[1]["emb_vec"].any():
            al_pos.append((a, b, t))
            al_neg.append(c)
    Xp_tr = np.stack([fs3.features(a, b, t) for a, b, t in al_pos])
    Xn_tr = np.stack([fs3.features(a, b, t) for a, b, t in al_neg])
    p_pos = 1.0 / (1.0 + np.exp(-head3.predict(Xp_tr)))
    p_neg = 1.0 / (1.0 + np.exp(-head3.predict(Xn_tr)))
    hard_idx = [i for i in range(len(al_pos)) if p_neg[i] > p_pos[i]]
    Xh = np.stack([fs3.features(a, b, t) for a, b, t in [al_neg[i] for i in hard_idx]])
    Xf2 = np.concatenate([X3, np.concatenate([Xh] * 3)])
    yf2 = np.concatenate([y3, np.zeros(len(Xh) * 3, dtype=np.float32)])
    head3.fit(Xf2, yf2, epochs=15, lr=1e-3, bs=256, seed=args.seed % 100000)

    res4, m04 = {}, {}
    for op in OPS:
        eval_cap(head3, res4, op, op, test, nodes, root_pool, parent_pool, all_positioned)
    for op in ("hard-rewire", "sibling-rewire", "direction-reversal"):
        eval_cap(head3, m04, op, op, m0_test, m0_nodes, m0_root_pool, m0_parent_pool, m0_all)
    rungs["hardneg"] = {
        "rung": 4, "feature_dim": fs3.dim, "arch": "2x128+mined",
        "mined_hard_train_negatives": len(hard_idx),
        "mining_note": ("sibling train negatives the rung3 model ranked above its real edge "
                        "were oversampled x3 in a 15-epoch low-LR fine-tune of the rung3 weights"),
        "full": res4, "m0": m04,
    }
    print(json.dumps({"rung": "hardneg", "mined": len(hard_idx),
                      "full_sibling_learned": res4.get("sibling-rewire", {}).get("learned")}))

    # ---- M0-only head (rung3 arch, M0 train edges, M0 pools)
    Xs, ys = [], []
    for op in ("hard-rewire", "sibling-rewire", "direction-reversal"):
        pos, neg = make_pairs(op, m0_train, m0_nodes, m0_root_pool, m0_parent_pool, m0_all)
        for a, b, t in pos:
            Xs.append(fs3.features(a, b, t))
            ys.append(1.0)
        for a, b, t in neg:
            Xs.append(fs3.features(a, b, t))
            ys.append(0.0)
    Xm = np.stack(Xs)
    ym = np.array(ys, dtype=np.float32)
    headm = MLP2(fs3.dim, hidden=128, layers=2, seed=args.seed % 100000)
    headm.fit(Xm, ym, epochs=150, lr=3e-3, bs=128, seed=args.seed % 100000)
    m0only = {}
    for op in ("hard-rewire", "sibling-rewire", "direction-reversal"):
        pos, neg = make_pairs(op, m0_test, m0_nodes, m0_root_pool, m0_parent_pool, m0_all)
        if not pos or not neg:
            m0only[op] = {"n_pos": len(pos)}
            continue
        m0only[op] = {"n_pos": len(pos), **eval_head(headm, fs3, pos, neg),
                      **{f"baseline_{k}": v for k, v in baselines(pos, neg).items()}}
    rungs["m0only"] = {"rung": 5, "arch": "2x128 m0-trained",
                       "m0_train_edges": len(m0_train), "m0": m0only}
    print(json.dumps({"rung": "m0only",
                      "m0_sibling_learned": m0only.get("sibling-rewire", {}).get("learned")}))

    evidence = {
        "schema": "actuation.model-types-ebm-energy-head-v2/v1",
        "created": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "experiment": "E-MT-EBM-FIELD phase-2.6 (head redesign + sibling frontier, R1/R2)",
        "model": {
            "rungs": {
                "base": "phase-2 replication: 1x32, scalar semantic, 32-dim type hash",
                "semantic_channel": "+ PCA-64 of e_a, e_b, e_a-e_b, e_a*e_b + cosine (R1a/R2 semantic channel)",
                "typed": "+ 128-dim type hash + incident-type profile hashes + log-degree both endpoints (R1b)",
                "capacity": "typed features at 2x128 capacity + calibration readout (R2)",
                "hardneg": "capacity + sibling hard-negative mining x3 oversample, low-LR fine-tune (R1c)",
                "m0only": "capacity arch trained on M0 edges only, M0 pools (R1d)",
            },
            "pca": (f"three PCA bases ({PCA_DIM} comps) fitted on the field's own node vectors; "
                    "difference/product bases estimated on consecutive node pairings (unsupervised, no labels)"),
            "split": "held-out edges by sha256 digest bucket mod 5, unchanged from phase 2",
            "seeds": {"head": args.seed, "probe": 20260917},
        },
        "data": {
            "vector_key": vector_key,
            "embedding_dim": meta["dim"],
            "edges_valid": len(valid),
            "train_edges": len(train),
            "test_edges": len(test),
            "m0_test_edges": len(m0_test),
            "m0_train_edges": len(m0_train),
        },
        "rungs": rungs,
        "honest_limits": [
            "learned readings are semantic-stochastic; never promoted, never the determination",
            "negatives are synthetic corruptions; AUC measures separation from the ladder, not value or correctness",
            "PCA components are fitted on the field's own node vectors (unsupervised); no label information is used",
            "the 'combined' baseline is lineage + cosine (probe convention); phase-2 tables called this 'semantic+lineage'",
            "hard-negative mining uses train edges only; held-out negatives stay untouched by mining",
            "calibration statistics (Brier/ECE/threshold) describe agreement of the head's probability output with the "
            "real-vs-corrupted label on the corruption ladder, not with human judgement of real loop transitions",
        ],
        "runtime_seconds": round(time.time() - t0, 1),
        "provenance": {"promotion": "none", "reading_class": "semantic-stochastic"},
    }
    args.out.mkdir(parents=True, exist_ok=True)
    out_path = args.out / f"energy-head-v2-{time.strftime('%Y%m%d-%H%M%S')}.json"
    out_path.write_text(json.dumps(evidence, indent=2, default=float) + "\n")
    np.savez(args.out / "energy-head-v2-weights.npz",
             **{f"rung3_W{i}": W for i, W in enumerate(head3.W)},
             **{f"rung3_b{i}": b for i, b in enumerate(head3.b)})

    summary = {}
    for name, r in rungs.items():
        full_sib = r.get("full", {}).get("sibling-rewire", {})
        m0_sib = r.get("m0", {}).get("sibling-rewire", {})
        summary[name] = {
            "full_sibling_learned": full_sib.get("learned"),
            "full_sibling_cosine": full_sib.get("baseline_cosine"),
            "m0_sibling_learned": m0_sib.get("learned"),
            "m0_sibling_cosine": m0_sib.get("baseline_cosine"),
        }
    print(json.dumps({"wrote": str(out_path), "summary": summary}, indent=2))


if __name__ == "__main__":
    sys.exit(main())
