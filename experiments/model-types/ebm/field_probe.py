#!/usr/bin/env python3
"""E-MT EBM field probe, Phase 1 — evaluative, no training.

Reads the live bimba graph and scores real relations against seeded corruptions
under two potentials: a coordinate-led structural potential (mirroring the
operative coordinate parser in bimba-portable/src/coordinates/parser.ts — type
letter CPMSLT, multi-digit segments, '-'/'.' descent, canonical context frames,
prime marker, legacy '#' to M family) and a semantic compatibility readout.

The bimba_schema relation-type vocabulary is out of line with the live graph
(owner-confirmed 2026-09-17: zero POSk_* types materialised among 1,378 live
types), so structure is taken from the coordinates themselves, not from
declared relation types.

Provenance: promotion none, reading class research. Low energy means
compatibility with declared or learned structure — nothing else.
"""

import argparse
import hashlib
import json
import math
import random
import re
import sys
import time
from collections import Counter, defaultdict
from pathlib import Path

TYPE_LETTERS = "CPMSLT"
TOKEN_RE = re.compile(r"[a-z0-9]{3,}")


def parse_coord(raw):
    """Mirror of parseCoordinate in bimba-portable/src/coordinates/parser.ts.

    Returns {"type": letter, "segments": [ints], "context_frame": str|None,
    "is_prime": bool} or None when the string is not a valid coordinate.
    """
    if not raw:
        return None
    s = str(raw).strip()
    if s.startswith("#"):
        s = "M" + s[1:]
    if not s or s[0] not in TYPE_LETTERS:
        return None
    type_letter = s[0]
    seg_match = re.match(r"^[CPMSLT]([\d.\-]+)", s)
    segments = []
    if seg_match:
        segments = [int(p) for p in re.split(r"[-.]", seg_match.group(1)) if p.isdigit()]
    context = re.search(r"[-.]\(([^)]+)\)", s)
    return {
        "type": type_letter,
        "segments": segments,
        "context_frame": context.group(1) if context else None,
        "is_prime": s.endswith("'"),
    }


def root_of(parsed):
    return (parsed["type"], tuple(parsed["segments"][:1]))


def parent_key(parsed):
    segs = parsed["segments"]
    if len(segs) < 2:
        return None
    return (parsed["type"], tuple(segs[:-1]))


def lineage_energy(a, b):
    """Structural potential from the coordinate tree.

    0 for ancestor/descendant or identical; cross-family sits above any
    same-family non-prefix at a flat 1.5; otherwise graded by longest common
    prefix: 1/(1+LCP). These weights are a declared convention of this probe,
    stated in the evidence — not owner-declared legality.
    """
    if a["type"] != b["type"]:
        return 1.5
    sa, sb = a["segments"], b["segments"]
    if sa == sb:
        return 0.0
    lcp = 0
    for x, y in zip(sa, sb):
        if x != y:
            break
        lcp += 1
    if lcp == len(sa) or lcp == len(sb):
        return 0.0  # ancestor or descendant
    return 1.0 / (1.0 + lcp)


def tokens(text):
    return set(TOKEN_RE.findall((text or "").lower()))


def jaccard(a, b):
    if not a or not b:
        return 0.0
    return len(a & b) / len(a | b)


def rank_auc(positives, negatives):
    """Mann-Whitney AUC with average ranks for ties."""
    if not positives or not negatives:
        return None
    labeled = sorted([(e, 1) for e in positives] + [(e, 0) for e in negatives],
                     key=lambda x: x[0])
    ranks = [0.0] * len(labeled)
    i = 0
    while i < len(labeled):
        j = i
        while j + 1 < len(labeled) and labeled[j + 1][0] == labeled[i][0]:
            j += 1
        avg = (i + j) / 2.0 + 1.0
        for k in range(i, j + 1):
            ranks[k] = avg
        i = j + 1
    r_pos = sum(r for r, (_, lab) in zip(ranks, labeled) if lab == 1)
    n1, n2 = len(positives), len(negatives)
    return (r_pos - n1 * (n1 + 1) / 2.0) / (n1 * n2)


def detect_vector_key(session):
    """Find the embedding property: bimba stores it as a JSON string
    (`n.embedding`); a plain float-list property is also accepted."""
    counts = Counter()
    for rec in session.run("MATCH (n:Bimba) WITH n LIMIT 25 RETURN properties(n) AS props"):
        for key, value in (rec["props"] or {}).items():
            if isinstance(value, str) and value.startswith("["):
                try:
                    parsed = json.loads(value)
                except json.JSONDecodeError:
                    continue
                if isinstance(parsed, list) and len(parsed) >= 256:
                    counts[key] += 1
            elif isinstance(value, list) and len(value) >= 256 and all(
                isinstance(x, (int, float)) for x in value[:8]
            ):
                counts[key] += 1
    return counts.most_common(1)[0][0] if counts else None


def as_vector(value):
    if isinstance(value, str):
        try:
            value = json.loads(value)
        except json.JSONDecodeError:
            return None
    if isinstance(value, list) and value and all(isinstance(x, (int, float)) for x in value[:4]):
        return value
    return None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--uri", default="bolt://100.92.62.101:7687")
    ap.add_argument("--user", default="neo4j")
    ap.add_argument("--password", default="bimba",
                    help="ignored by the auth-disabled instance; kept for driver parity")
    ap.add_argument("--database", default="neo4j")
    ap.add_argument("--samples", type=int, default=2000, help="corrupted pairs per operator")
    ap.add_argument("--seed", type=int, default=20260917)
    ap.add_argument("--root", default=None,
                    help="restrict the field to one coordinate family, e.g. M0 (nodes whose root matches)")
    ap.add_argument("--out", type=Path, default=Path(__file__).resolve().parent / "evidence")
    args = ap.parse_args()

    from neo4j import GraphDatabase

    rng = random.Random(args.seed)
    t0 = time.time()

    driver = GraphDatabase.driver(args.uri, auth=(args.user, args.password))
    with driver.session(database=args.database) as session:
        vector_key = detect_vector_key(session)
        vec_prop = f", n.{vector_key} AS embedding" if vector_key else ""
        nodes = {}
        skipped_nodes = 0
        for rec in session.run(
            "MATCH (n:Bimba) RETURN elementId(n) AS id, "
            "coalesce(n.bimbaCoordinate, n.coordinate) AS coord, "
            "coalesce(n.c_1_name, n.title) AS name, "
            "n.description AS description" + vec_prop
        ):
            parsed = parse_coord(rec["coord"])
            if parsed is None:
                skipped_nodes += 1
                continue
            nodes[rec["id"]] = {
                "coord": rec["coord"],
                "parsed": parsed,
                "embedding": as_vector(rec.get("embedding")),
                "tok": tokens(str(rec.get("name") or "") + " " + str(rec.get("description") or "")),
            }
        edges = [
            (rec["ia"], rec["ib"], rec["t"])
            for rec in session.run(
                "MATCH (a:Bimba)-[r]->(b:Bimba) "
                "RETURN elementId(a) AS ia, elementId(b) AS ib, type(r) AS t"
            )
        ]
    driver.close()

    if args.root:
        want = parse_coord(args.root)
        if want is None:
            print(json.dumps({"error": f"unparseable --root {args.root}"}))
            return 2
        root_tuple = (want["type"], tuple(want["segments"][:1]))
        nodes = {
            nid: n for nid, n in nodes.items()
            if root_of(n["parsed"]) == root_tuple
        }

    root_pool = defaultdict(list)
    parent_pool = defaultdict(list)
    for node in nodes.values():
        root_pool[root_of(node["parsed"])].append(node)
        pk = parent_key(node["parsed"])
        if pk is not None:
            parent_pool[pk].append(node)

    sem_mode = f"in-graph-embeddings:{vector_key}" if vector_key else "lexical-baseline-fixture-grade"
    embedded_count = sum(1 for n in nodes.values() if n["embedding"])

    def sem_energy(a, b):
        if vector_key:
            u, v = a["embedding"], b["embedding"]
            if not u or not v or len(u) != len(v):
                return None  # excluded pair when either side lacks a vector
            dot = sum(x * y for x, y in zip(u, v))
            nu = math.sqrt(sum(x * x for x in u))
            nv = math.sqrt(sum(x * x for x in v))
            return 1.0 - (dot / (nu * nv) if nu and nv else 0.0)
        return 1.0 - jaccard(a["tok"], b["tok"])

    def potentials(a, b):
        sem = sem_energy(a, b)
        lin = lineage_energy(a["parsed"], b["parsed"])
        return {
            "lineage": lin,
            "semantic": sem,
            "combined": None if sem is None else lin + sem,
        }

    valid_edges = [(ia, ib, t) for ia, ib, t in edges if ia in nodes and ib in nodes]

    graph_digest = hashlib.sha256(
        ("\n".join(sorted(f"{ia}|{ib}|{t}" for ia, ib, t in edges))
         + "\n".join(sorted(str(n["coord"]) for n in nodes.values()))).encode()
    ).hexdigest()

    operator_notes = {
        "endpoint-rewire": "target replaced with a random node of a different coordinate root (type + first segment)",
        "hard-rewire": "target replaced with a random node of the same coordinate root",
        "sibling-rewire": "target replaced with a random sibling (same type + parent segments, different final segment)",
        "direction-reversal": "endpoints swapped: (b, r, a) — identical unordered pair, direction only",
    }

    results = {}
    for op in operator_notes:
        pots = {"lineage": ([], []), "semantic": ([], []), "combined": ([], [])}
        made, excluded = 0, 0
        tries = 0

        def record(e_real, e_neg):
            """Append a pair to each potential that has values on both sides."""
            for name in pots:
                if e_real[name] is None or e_neg[name] is None:
                    continue
                pots[name][0].append(e_real[name])
                pots[name][1].append(e_neg[name])

        while made < args.samples and tries < args.samples * 20:
            tries += 1
            ia, ib, real_type = rng.choice(valid_edges)
            a, b = nodes[ia], nodes[ib]

            if op == "direction-reversal":
                e_real, e_neg = potentials(a, b), potentials(b, a)
                if e_real["semantic"] is None or e_neg["semantic"] is None:
                    excluded += 1
                    continue
                record(e_real, e_neg)
                made += 1
                continue

            if op == "endpoint-rewire":
                pool = [n for n in nodes.values() if root_of(n["parsed"]) != root_of(b["parsed"])]
            elif op == "hard-rewire":
                pool = [n for n in root_pool.get(root_of(b["parsed"]), []) if n is not b]
            else:  # sibling-rewire
                pool = [n for n in parent_pool.get(parent_key(b["parsed"]), []) if n is not b]
            if not pool:
                continue
            nb = rng.choice(pool)

            e_real, e_neg = potentials(a, b), potentials(a, nb)
            if e_real["semantic"] is None or e_neg["semantic"] is None:
                excluded += 1
                continue
            record(e_real, e_neg)
            made += 1

        results[op] = {
            "n_pairs": made,
            "n_excluded_missing_vector": excluded,
            "separation_auc_real_lower_energy": {
                name: rank_auc(n, p) for name, (p, n) in pots.items()
            },
            "mean_energy": {
                name: {"real": sum(p) / max(len(p), 1), "corrupted": sum(n) / max(len(n), 1)}
                for name, (p, n) in pots.items()
            },
        }

    honest_limits = [
        "structure is taken from the coordinate grammar (operative parser mirror), not from "
        "relation-type declarations: the schema-docs vocabulary is out of line with the live "
        "graph (zero POSk_* types among the live types) — owner-confirmed 2026-09-17",
        "lineage weights (prefix=0, cross-family=1.5, else 1/(1+LCP)) are this probe's declared "
        "convention, not owner-declared legality",
        "separation AUC reads P(real energy < corrupted energy); 0.5 is no separation",
        "AUC here is not a claim that the field models value or correctness",
        "direction-reversal negatives share the unordered pair with their real positive, so "
        "pair-potentials separate them only through direction-sensitive structure (none is "
        "direction-sensitive by construction)",
    ]
    honest_limits.append(
        "embedding readout uses in-graph vectors as stored; no re-embedding was performed"
        if vector_key
        else "lexical-baseline semantic readout is fixture-grade and supports no claim on its own"
    )

    evidence = {
        "schema": "actuation.model-types-ebm-field-probe/v1",
        "created": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "experiment": "E-MT-EBM-FIELD phase-1 (coordinate-led)",
        "uri": args.uri,
        "root_filter": args.root,
        "graph": {
            "nodes_total": len(nodes) + skipped_nodes,
            "nodes_with_parsed_coordinate": len(nodes),
            "nodes_unparseable": skipped_nodes,
            "nodes_with_embedding": embedded_count,
            "embedding_coverage": round(embedded_count / max(len(nodes), 1), 3),
            "edges_total_all_roots": len(edges),
            "edges_scoreable": len(valid_edges),
            "distinct_relation_types_in_scope": len({t for _, _, t in valid_edges}),
            "graph_capture_sha256": graph_digest,
        },
        "structural_potential": {
            "source": "bimba-portable/src/coordinates/parser.ts grammar, mirrored in probe",
            "weights": {"prefix_or_self": 0.0, "cross_family": 1.5, "other": "1/(1+LCP)"},
        },
        "semantic_readout": {"mode": sem_mode},
        "corruption_operators": operator_notes,
        "results": results,
        "honest_limits": honest_limits,
        "runtime_seconds": round(time.time() - t0, 1),
        "provenance": {"promotion": "none", "reading_class": "research"},
    }

    args.out.mkdir(parents=True, exist_ok=True)
    out_path = args.out / f"field-probe-{time.strftime('%Y%m%d-%H%M%S')}.json"
    out_path.write_text(json.dumps(evidence, indent=2) + "\n")
    print(json.dumps({
        "wrote": str(out_path),
        "semantic_readout": sem_mode,
        "nodes": len(nodes),
        "edges": len(edges),
        "separation_auc": {
            op: results[op]["separation_auc_real_lower_energy"] for op in results
        },
    }, indent=2))


if __name__ == "__main__":
    sys.exit(main())
