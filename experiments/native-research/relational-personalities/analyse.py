#!/usr/bin/env python3
"""Derived arithmetic over independently annotated native evidence, not a judge/runtime."""
from __future__ import annotations

import argparse
import json
import math
from collections import Counter
from pathlib import Path
from typing import Any

METRICS = (
    "character_discriminability", "contingent_uptake", "attribution",
    "acquaintance_grounding", "correction_uptake", "continuity", "rubric_leakage",
)


def text(value: Any, field: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise ValueError(f"{field} must be nonempty text")
    return value


def summarise(document: dict[str, Any]) -> dict[str, Any]:
    if not isinstance(document, dict) or document.get("schema") != "actuation.acquaintance-evidence-projection/v1":
        raise ValueError("expected an acquaintance evidence projection")
    episode = text(document.get("episode_ref"), "episode_ref")
    refs = document.get("participant_refs")
    if not isinstance(refs, list) or len(refs) != 4:
        raise ValueError("exactly four participant refs are required")
    refs = [text(r, "participant_ref") for r in refs]
    if len(set(refs)) != 4:
        raise ValueError("participant identities must be distinct")
    if not isinstance(document.get("native_basis_refs"), list) or not document["native_basis_refs"]:
        raise ValueError("native_basis_refs are required (the operator verifies them)")
    for ref in document["native_basis_refs"]:
        text(ref, "native_basis_ref")
    external = document.get("nonparticipant_event_refs", [])
    if not isinstance(external, list):
        raise ValueError("nonparticipant_event_refs must be a list")
    external = {text(r, "nonparticipant_event_ref") for r in external}
    messages = document.get("messages")
    if not isinstance(messages, list):
        raise ValueError("messages must be a list")
    index: dict[str, dict[str, Any]] = {}
    counts: Counter[str] = Counter({r: 0 for r in refs})
    directed: Counter[tuple[str, str]] = Counter()
    for item in messages:
        if not isinstance(item, dict):
            raise ValueError("message must be an object")
        ref = text(item.get("native_ref"), "message.native_ref")
        sender = text(item.get("sender_ref"), "message.sender_ref")
        if sender not in refs:
            raise ValueError("unknown message sender")
        if ref in external:
            raise ValueError("a peer message cannot also be a nonparticipant event")
        if ref in index:
            raise ValueError("duplicate logical message; preserve and reconcile transport evidence first")
        index[ref] = item
        if item.get("segment") not in ("first-meeting", "return-meeting", "continuity", "reflection"):
            raise ValueError("unknown message segment")
        if item["segment"] in ("first-meeting", "return-meeting"):
            counts[sender] += 1
    for item in messages:
        reply = item.get("reply_to")
        if reply is None:
            continue
        if not isinstance(reply, str) or (reply not in index and reply not in external):
            raise ValueError("reply_to must name a retained message or declared native opening event")
        if reply in external:
            continue
        if reply == item["native_ref"]:
            raise ValueError("a message cannot reply to itself")
        target = index[reply]["sender_ref"]
        if item["segment"] in ("first-meeting", "return-meeting") and target != item["sender_ref"]:
            directed[(item["sender_ref"], target)] += 1
    observations = document.get("observations", [])
    if not isinstance(observations, list):
        raise ValueError("observations must be a list")
    totals = {m: {"numerator": 0, "denominator": 0, "ineligible": 0} for m in METRICS}
    seen: set[tuple[str, str]] = set()
    for item in observations:
        if not isinstance(item, dict) or item.get("metric") not in totals:
            raise ValueError("unknown metric observation")
        metric = item["metric"]
        opportunity = text(item.get("opportunity_ref"), "opportunity_ref")
        key = (metric, opportunity)
        if key in seen:
            raise ValueError("duplicate metric opportunity; adjudicate multiple raters separately")
        seen.add(key)
        text(item.get("annotator_ref"), "annotator_ref")
        evidence = item.get("evidence_refs")
        if not isinstance(evidence, list) or not evidence:
            raise ValueError("each observation requires cited evidence")
        for ref in evidence:
            text(ref, "evidence_ref")
        eligible = item.get("eligible")
        if type(eligible) is not bool:
            raise ValueError("eligible must be boolean")
        if eligible:
            if type(item.get("satisfied")) is not bool:
                raise ValueError("eligible observations require a boolean judgement")
            totals[metric]["denominator"] += 1
            totals[metric]["numerator"] += int(item["satisfied"])
        else:
            if item.get("satisfied") is not None:
                raise ValueError("ineligible observations cannot claim success")
            totals[metric]["ineligible"] += 1
    for row in totals.values():
        row["rate"] = row["numerator"] / row["denominator"] if row["denominator"] else None
    usage = document.get("usage", {})
    if not isinstance(usage, dict):
        raise ValueError("usage must be an object")
    for key, value in usage.items():
        if value is not None and (type(value) not in (int, float) or not math.isfinite(value) or value < 0):
            raise ValueError(f"usage.{key} must be a nonnegative finite number or null")
    reciprocal = sum(1 for i, a in enumerate(refs) for b in refs[i + 1:] if directed[(a, b)] and directed[(b, a)])
    return {
        "schema": "actuation.acquaintance-derived-analysis/v1",
        "episode_ref": episode,
        "native_integrity_assessed_by_this_tool": False,
        "semantic_judgements": "supplied by named independent annotation, not inferred by this script",
        "metrics": totals,
        "public_acts_by_participant": dict(counts),
        "directed_reply_counts": [{"from": a, "to": b, "count": directed[(a, b)]} for a in refs for b in refs if a != b],
        "reciprocal_dyads": {"count": reciprocal, "possible": 6},
        "usage": usage,
        "limits": "No combined score, automatic pass, causal claim or utterance-independent confidence interval.",
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path)
    parser.add_argument("--out", type=Path)
    args = parser.parse_args()
    try:
        result = summarise(json.loads(args.input.read_text(encoding="utf-8")))
        rendered = json.dumps(result, ensure_ascii=False, indent=2, allow_nan=False) + "\n"
        if args.out:
            args.out.write_text(rendered, encoding="utf-8")
        else:
            print(rendered, end="")
    except (OSError, ValueError, TypeError) as error:
        parser.exit(2, f"analysis refused: {error}\n")


if __name__ == "__main__":
    main()
