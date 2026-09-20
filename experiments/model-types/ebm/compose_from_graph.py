#!/usr/bin/env python3
"""Compose embed text for bimba nodes from the graph's OWN properties.

The map carries its content in the current coordinate-tagged schema
(c_0_/c_1_/c_2_/c_3_/c_5_ content keys, l_/m_/p_/s_/t_ family keys — 3,026
distinct keys, mean ~1,670 chars text per node, surveyed 2026-09-19). This
composes one text per node: the common essential set first (coordinate, name,
description, QL metadata, family/layer/subsystem), then every available
content property in key order — bounded, provenance-recorded.

Source of truth is the live graph. No vault, no datasets, no sync.
"""

import argparse
import hashlib
import json
import sys
import time
from pathlib import Path

META_KEY_PARTS = (
    "uuid", "updated", "created", "sync", "embedding", "dataset_branch",
    "dataset_branch_label", "source_dataset", "projected", "graph_node",
    "artifact_role", "access_level", "notion_page_id", "lastupdated",
)
ESSENTIAL = [
    "coordinate", "c_1_name", "c_1_description", "c_4_family", "c_4_layer",
    "c_4_subsystem", "c_4_ql_position", "c_4_ql_category",
    "c_4_ql_operator_types", "c_4_ql_variant", "c_3_context_frame",
    "c_1_primary_designation",
]
CONTENT_PREFIXES = ("c_0_", "c_1_", "c_2_", "c_3_", "c_5_",
                    "l_2_", "l_3_", "m_2_4_", "m_3_3_", "m_3_5_",
                    "p_1_", "p_3_", "s_2_", "s_4_", "t_3_")


def is_meta(key):
    k = key.lower()
    return any(part in k for part in META_KEY_PARTS)


def render(value):
    if isinstance(value, list):
        return " | ".join(str(x) for x in value if str(x).strip())
    return str(value)


def compose(prop):
    parts = []
    used = set()
    for key in ESSENTIAL:
        if key in prop:
            text = render(prop[key]).strip()
            if text and text.lower() not in ("none", "null"):
                parts.append(text)
            used.add(key)
    for key in sorted(prop.keys()):
        if key in used or is_meta(key):
            continue
        if not key.startswith(CONTENT_PREFIXES):
            continue
        text = render(prop[key]).strip()
        if not text or text.lower() in ("none", "null"):
            continue
        label = key.split("_", 2)[-1].replace("_", " ")
        parts.append(f"{label}: {text}")
    return " . ".join(parts)[:6000]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--uri", default="bolt://100.92.62.101:7687")
    ap.add_argument("--user", default="neo4j")
    ap.add_argument("--password", default="bimba")
    ap.add_argument("--database", default="neo4j")
    ap.add_argument("--scope", type=Path, default=Path("/tmp/embed-scope.json"))
    ap.add_argument("--out", type=Path, default=Path(__file__).resolve().parent / "evidence")
    args = ap.parse_args()

    from neo4j import GraphDatabase

    driver = GraphDatabase.driver(args.uri, auth=(args.user, args.password))
    items, chars = [], []
    with driver.session(database=args.database) as session:
        for rec in session.run(
            "MATCH (n:Bimba) "
            "RETURN coalesce(n.c_2_uuid, n.uuid) AS uuid, "
            "coalesce(n.bimbaCoordinate, n.coordinate) AS coord, properties(n) AS p"
        ):
            coord = str(rec["coord"] or "").strip()
            if coord.startswith("#"):
                coord = "M" + coord[1:]
            if not coord or coord[0] not in "ML" or not rec["uuid"]:
                continue
            text = compose(rec["p"] or {})
            if text.strip():
                chars.append(len(text))
                items.append({"uuid": rec["uuid"], "coord": coord, "text": text})
    driver.close()

    manifest = {
        "schema": "actuation.model-types-embed-text-provenance/v2",
        "created": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "source": "live bimba graph properties (current tagged schema) — no vault, no datasets, no sync",
        "nodes": len(items),
        "mean_text_chars": round(sum(chars) / max(len(chars), 1), 1),
        "nodes_under_120_chars": sum(1 for c in chars if c < 120),
        "composition": "essential common set first, then all content-family keys in key order; bounded 6000 chars",
        "sources_digest": hashlib.sha256(json.dumps(items, sort_keys=True).encode()).hexdigest()[:16],
    }
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "embed-text-provenance.json").write_text(json.dumps(manifest, indent=2) + "\n")
    json.dump({"items": items}, open(args.scope, "w"))
    print(json.dumps({k: manifest[k] for k in ("nodes", "mean_text_chars", "nodes_under_120_chars")}))


if __name__ == "__main__":
    sys.exit(main())
