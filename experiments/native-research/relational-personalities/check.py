#!/usr/bin/env python3
"""Check package bytes and experiment input consistency; never certify live behaviour."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]


def checked_file(path: str) -> Path:
    candidate = (ROOT / path).resolve()
    if not candidate.is_relative_to(ROOT) or not candidate.is_file():
        raise ValueError(f"not a repository file: {path}")
    return candidate


def validate() -> dict[str, object]:
    intake = ROOT / "docs/agent-self/source-intake/v0.2"
    manifest = json.loads((intake / "MANIFEST.json").read_text())
    for name, expected in manifest.items():
        body = (intake / name).read_bytes()
        if len(body) != expected["bytes"] or hashlib.sha256(body).hexdigest() != expected["sha256"]:
            raise ValueError(f"source intake changed: {name}")
    spec = json.loads((HERE / "experiment.json").read_text())
    if spec.get("schema") != "actuation.relational-personality-experiment-spec/v1":
        raise ValueError("wrong experiment input schema")
    peers = spec["participants"]
    if len(peers) != 4 or {p["seed_key"] for p in peers} != {"kite", "cairn", "lilt", "flint"}:
        raise ValueError("the experiment requires the four specified peers")
    if len({p["display_name"] for p in peers}) != 4:
        raise ValueError("display names must be distinct")
    for pin in [p["self_source"] for p in peers] + [spec["shared_logos"], spec["room"]]:
        body = checked_file(pin["path"]).read_bytes()
        if hashlib.sha256(body).hexdigest() != pin["sha256"]:
            raise ValueError(f"input digest mismatch: {pin['path']}")
    if len({p["self_source"]["sha256"] for p in peers}) != 4:
        raise ValueError("the four selves must have distinct source bytes")
    if (HERE / "personas/kite.md").read_bytes() != (intake / "SELF-KITE.md").read_bytes():
        raise ValueError("initial Kite must preserve the supplied source")
    logos = checked_file(spec["shared_logos"]["path"]).read_text()
    for marker in ["## 0/1", "## 0 —", "## 1 —", "## 2 —", "## 3 —", "## 4 —", "## 5 —", "## 5 → 0", "Each fuller relation", "same identity", "implicate"]:
        if marker not in logos:
            raise ValueError(f"shared Logos missing retained structural marker: {marker}")
    primary = spec["primary"]
    total = sum(primary[key] for key in ("first_meeting_opportunities", "continuity_opportunities", "return_meeting_opportunities", "reflection_opportunities"))
    if total != spec["limits"]["peer_act_opportunities"] or total != 48:
        raise ValueError("primary opportunities do not match the bounded plan")
    if spec["required_engineering"] != [f"E{i}" for i in range(1, 11)]:
        raise ValueError("engineering obligation coverage changed")
    if spec["comparative"]["auto_run"] is not False:
        raise ValueError("comparative batches require a separate native budget")
    if spec["standing"] != "planned-not-executed":
        raise ValueError("the source specification must not pretend to be an execution receipt")
    return {"package_integrity": "passed", "participant_sources": 4, "native_or_behavioural_verification": "not performed by this check"}


if __name__ == "__main__":
    try:
        print(json.dumps(validate(), indent=2))
    except (OSError, ValueError, KeyError, TypeError) as error:
        raise SystemExit(f"package check failed: {error}")
