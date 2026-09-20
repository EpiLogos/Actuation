#!/usr/bin/env python3
"""Compose rich embed text for bimba graph nodes from the Idea/Bimba/Map vault.

Why: the Neo4j :Bimba nodes carry NO content properties (measured 2026-09-18:
description, keyPrinciples, resonances, ... = 0/2141; only names 1926/2141 plus
lineage metadata). The rich detail lives in the canonical vault tree
Epi-Logos-C-Experiments/Idea/Bimba/Map — one generated page per graph node
(35KB-scale: description, operational essence, principles, resonances, the
Quaternal Register). The earlier embedding therefore embedded names only.

This joins each graph node to its Map page by coordinate and rebuilds the
embed scope file with composed text (title + body, wikilinks stripped,
bounded). Writes a provenance manifest next to the experiment evidence.

Read-only on the vault; the graph write happens later through bimba_embed.
"""

import argparse
import hashlib
import json
import re
import sys
import time
from pathlib import Path

FRONTMATTER = re.compile(r"\A---\n(.*?)\n---\n", re.DOTALL)
COORD_KEY = re.compile(r'^coordinate:\s*"?([^"\n]+)"?', re.MULTILINE)
WIKILINK = re.compile(r"\[\[([^\]|]+)(\|[^\]]*)?\]\]")
BARE_LINK = re.compile(r"\[([^\]]+)\]\([^)]*\)")
MARKDOWN_NOISE = re.compile(r"^#{1,6} |^\s*[-*] \| ", re.MULTILINE)


def norm_coord(c):
    if not c:
        return None
    c = str(c).strip().strip('"')
    return ("M" + c[1:]) if c.startswith("#") else c


def compose_text(page_path: Path, coord: str):
    raw = page_path.read_text(errors="replace")
    m = FRONTMATTER.match(raw)
    title = ""
    if m:
        tm = re.search(r'^title:\s*"?([^"\n]+)"?', m.group(1), re.MULTILINE)
        if tm:
            title = tm.group(1).strip()
        raw = raw[m.end():]
    # strip the leading "# coord · title" line if present
    raw = re.sub(r"\A#\s*[^\n]+\n", "", raw)
    body = WIKILINK.sub(r"\1", raw)
    body = BARE_LINK.sub(r"\1", body)
    body = re.sub(r"\n{3,}", "\n\n", body)
    body = re.sub(r"[ \t]+\n", "\n", body).strip()
    head = f"{coord} · {title}. " if title else f"{coord}. "
    return (head + body)[:6000]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--map-root", type=Path,
                    default=Path.home() / "Central/Work/epi/Epi-Logos-C-Experiments/Idea/Bimba/Map")
    ap.add_argument("--scope", type=Path, default=Path("/tmp/embed-scope.json"))
    ap.add_argument("--out", type=Path, default=Path(__file__).resolve().parent / "evidence")
    args = ap.parse_args()

    datasets_root = args.map_root / "datasets"
    world_root = args.map_root.parent / "World"

    # source 1: Map pages, keyed by true coordinate from frontmatter
    by_coord = {}
    for page in args.map_root.rglob("*.md"):
        if page.name == "AGENTS.md":
            continue
        raw = page.read_text(errors="replace")[:1200]
        m = FRONTMATTER.match(raw)
        if not m:
            continue
        cm = COORD_KEY.search(m.group(1))
        if not cm:
            continue
        coord = norm_coord(cm.group(1))
        if coord:
            by_coord.setdefault(coord, ("map", page))

    # source 2: deep import datasets (full node detail per root)
    RICH_FIELDS = [
        ("name", "name"), ("description", "description"),
        ("operationalEssence", "operational essence"),
        ("keyPrinciples", "key principles"),
        ("architecturalFunction", "architectural function"),
        ("internalStructure", "internal structure"),
        ("consciousnessStructure", "consciousness structure"),
        ("practicalApplications", "practical applications"),
        ("resonances", "resonances"),
        ("primaryDesignation", "designation"),
        ("subsystem", "subsystem"), ("qlCategory", "QL category"),
        ("qlPosition", "QL position"), ("qlOperatorTypes", "QL operator types"),
        ("contextFrame", "context frame"),
    ]
    deep_by_coord = {}
    deep_files = sorted(datasets_root.glob("*/nodes-full-*.json"))
    for ds_file in deep_files:
        try:
            records = json.loads(ds_file.read_text(encoding="utf-8-sig"), strict=False)
        except (json.JSONDecodeError, OSError):
            continue
        if not isinstance(records, list):
            continue
        for rec in records:
            coord = norm_coord(rec.get("coordinate"))
            props = rec.get("filteredProps") or {}
            if not coord or not isinstance(props, dict):
                continue
            parts = []
            for key, label in RICH_FIELDS:
                v = props.get(key)
                if v is None:
                    continue
                s = re.sub(r"\s+", " ", str(v)).strip()
                if s and s.lower() not in ("none", "null", "n/a"):
                    parts.append(f"{label}: {s}" if label != "name" else s)
            if parts:
                deep_by_coord.setdefault(coord, (", ".join(parts))[:6000])

    # source 3: World crystallised forms (L/P/S/T/C families, flat files)
    for page in sorted(world_root.glob("*.md")):
        raw = page.read_text(errors="replace")
        m = FRONTMATTER.match(raw)
        if not m:
            continue
        cm = COORD_KEY.search(m.group(1))
        if not cm:
            continue
        coord = norm_coord(cm.group(1))
        if coord:
            by_coord.setdefault(coord, ("world", page))

    scope = json.loads(args.scope.read_text())["items"]
    items, joined, deep_only, fallback = [], 0, 0, 0
    for it in scope:
        coord = norm_coord(it["coord"])
        if coord in by_coord:
            kind, page = by_coord[coord]
            items.append({"uuid": it["uuid"], "coord": it["coord"], "text": compose_text(page, coord)})
            joined += 1
        elif coord in deep_by_coord:
            items.append({"uuid": it["uuid"], "coord": it["coord"], "text": f"{coord}. {deep_by_coord[coord]}"})
            deep_only += 1
        else:
            items.append(it)
            fallback += 1

    manifest = {
        "schema": "actuation.model-types-embed-text-provenance/v1",
        "created": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "map_root": str(args.map_root),
        "nodes": len(items),
        "joined_to_map_page": joined,
        "joined_to_deep_dataset": deep_only,
        "name_only_fallback": fallback,
        "sources": {
            "map_pages": str(args.map_root),
            "deep_datasets": [str(p) for p in deep_files],
            "world_forms": str(world_root),
        },
        "composition": "priority Map page > deep-dataset filteredProps > World form > name-only; "
                       "page text is title + body (wikilinks stripped); dataset text is the rich "
                       "property fields composed; bounded 6000 chars",
    }
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "embed-text-provenance.json").write_text(json.dumps(manifest, indent=2) + "\n")
    json.dump({"items": items}, open(args.scope, "w"))
    print(json.dumps({k: manifest[k] for k in ("nodes", "joined_to_map_page", "joined_to_deep_dataset", "name_only_fallback")}))


if __name__ == "__main__":
    sys.exit(main())
