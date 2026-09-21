#!/usr/bin/env python3
"""Full graph content survey — the honest measurement the Phase-2.5 story needs.

The earlier "zero content" claim only checked a fixed list of legacy property
names on :Bimba nodes. This surveys EVERYTHING: all labels, all property keys
(with sample values), relationship property keys, and a per-node text-volume
distribution. Its output decides how embed text must be composed from the
graph's own (authoritative, owner-verified data-complete) content.

Read-only. Run when bolt is reachable; no writes, no sync, no imports.
"""

import json
import sys
from collections import Counter

from neo4j import GraphDatabase

URI = "bolt://100.92.62.101:7687"


def sample(session, query, **params):
    return session.run(query, **params).data()


def main():
    driver = GraphDatabase.driver(URI, auth=("neo4j", "bimba"), connection_timeout=15)
    with driver.session(database="neo4j") as s:
        labels = sample(s, "MATCH (n) UNWIND labels(n) AS l RETURN l, count(*) AS c ORDER BY c DESC")
        prop_keys = sample(
            s,
            "MATCH (n:Bimba) UNWIND keys(n) AS k RETURN k, count(*) AS c, "
            "count(CASE WHEN n[k] IS NOT NULL AND n[k] <> '' THEN 1 END) AS nonempty "
            "ORDER BY c DESC",
        )
        rel_keys = sample(
            s,
            "MATCH (:Bimba)-[r]->(:Bimba) UNWIND keys(r) AS k "
            "RETURN k, count(*) AS c ORDER BY c DESC LIMIT 20",
        )
        rel_types = sample(
            s,
            "MATCH (:Bimba)-[r]->(:Bimba) RETURN type(r) AS t, count(*) AS c "
            "ORDER BY c DESC LIMIT 5",
        )
        samples = {}
        for row in prop_keys:
            key = row["k"]
            if row["nonempty"] < 20:
                continue
            rec = sample(
                s,
                f"MATCH (n:Bimba) WHERE n[{json.dumps(key)}] IS NOT NULL "
                f"RETURN n[{json.dumps(key)}] AS v LIMIT 1",
            )
            if rec:
                v = rec[0]["v"]
                if isinstance(v, list):
                    v = " | ".join(str(x) for x in v)
                v = str(v)
                if v.strip():
                    samples[key] = {"len": len(v), "head": v[:160]}
        # text volume per node over string properties (excluding metadata keys)
        META = {"embedding", "embedding_dimensions", "embedding_task_type",
                "embedding_model", "embedding_generated_at", "c_2_uuid",
                "c_3_dataset_branch", "c_3_dataset_branch_label", "c_3_source_dataset",
                "c_4_family", "c_4_layer", "c_4_ql_position", "c_3_projected_at",
                "c_3_projected_from", "c_4_graph_node", "c_4_artifact_role",
                "lastUpdated", "updatedAt", "updated_at", "accessLevel", "id",
                "c_3_created_at", "c_3_updated_at"}
        volumes = []
        for rec in s.run("MATCH (n:Bimba) RETURN properties(n) AS p"):
            total = 0
            for k, v in (rec["p"] or {}).items():
                if k in META:
                    continue
                if isinstance(v, str):
                    total += len(v)
                elif isinstance(v, list):
                    total += sum(len(str(x)) for x in v)
            volumes.append(total)
        buckets = Counter()
        for v in volumes:
            buckets["0" if v == 0 else "<=100" if v <= 100 else "<=1000" if v <= 1000 else ">1000"] += 1
    driver.close()

    print(json.dumps({
        "labels": labels,
        "node_property_keys": prop_keys,
        "samples_of_nonempty_keys": samples,
        "relationship_property_keys": rel_keys,
        "top_relation_types": rel_types,
        "per_node_text_volume": {
            "nodes": len(volumes),
            "buckets": dict(buckets),
            "mean_chars": round(sum(volumes) / max(len(volumes), 1), 1),
        },
    }, indent=2, default=str))


if __name__ == "__main__":
    sys.exit(main())
