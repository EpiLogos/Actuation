#!/usr/bin/env python3
"""Build Jev states for QL/MEF lensing and classification (E-MT-1).

Per the owner's scoping: Jev is tested on QL positions and MEF lenses — the
lensing/classifying layer — not on the M-coordinate census work. Ground truth
comes from two places:

  ql-position          — the source_position recorded on each act by the QL
                         circuits in the retained Series-1 runs (ql-direct /
                         ql-deep conditions). Standing: the loop's own recorded
                         position state, not human-verified labels.
  mef-lens-refraction  — no ground truth by design. MEF is a manifold of
                         disclosure, not a bucket taxonomy; the question asks
                         which lens most coherently refracts a subject, and the
                         answer is an instrument reading (research standing).
                         The retained runs carry the runtime's standing
                         refraction pair (L1 + L4′) on every event, so the
                         harness reports Jev's mass on that pair as
                         agreement-of-readings, not accuracy.

Emits states.jsonl / truth.jsonl / manifest.json.
"""

import argparse
import glob
import hashlib
import json
import sys
from pathlib import Path

LENS_OPTIONS = [
    ("L0", "Quaternal — why/what/how questioning articulation"),
    ("L0'", "Archetypal-Numerical — one through six as archetypal number"),
    ("L1", "Causal — svatantrya, material/efficient/formal/final cause, will"),
    ("L1'", "Phenomenal — introversion, sensation, feeling, thinking, intuition, extroversion"),
    ("L2", "Logical — tetralemmaic ground: IS, IS-NOT, BOTH, NEITHER, SILENCE"),
    ("L2'", "Alchemical-Elemental — elemental correspondence articulation"),
    ("L3", "Processual — concrescent desire, actual occasion (Whitehead)"),
    ("L3'", "Chronological — Spirit (Geist), spring, summer, autumn, winter, life (Aufhebung)"),
    ("L4", "Phenomenological — Sein, Geworfenheit, Dasein, Zeit, Besorge, Gelassenheit"),
    ("L4'", "Scientific — prompts, traces, challenges, patterns, discovery, insight"),
    ("L5", "Para Vāk — anuttara/asambhava, para vāk, paśyantī, madhyamā, vaikharī, mātṛkā"),
    ("L5'", "Divine Logos — arche, apokalypsis, dynamis and the logological articulation"),
]
STANDING_REFRACTION_PAIR = ["L1", "L4'"]


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def bounded(text, n):
    return str(text)[:n] if text else ""


def iter_act_events(runs_dir: Path):
    """Yield (run_file, record, event) for events carrying an act, in QL conditions."""
    for run_file in sorted(glob.glob(str(runs_dir / "*" / "*.json"))):
        try:
            doc = json.loads(Path(run_file).read_text())
        except json.JSONDecodeError:
            continue
        for rec in doc.get("records") or []:
            if not str(rec.get("condition", "")).startswith("ql-"):
                continue
            for event in rec.get("record", {}).get("events") or []:
                payload = event.get("payload") or {}
                if isinstance(payload.get("act"), dict):
                    yield run_file, rec, event


def build_ql_position(runs_dir, limit, states, truth):
    n = 0
    for run_file, rec, event in iter_act_events(runs_dir):
        if n >= limit:
            return n
        act = event["payload"]["act"]
        position = act.get("source_position")
        if not position:
            continue
        carrier = act.get("carrier") or {}
        ql = event.get("ql") or {}
        record_id = f"ql-position:{event.get('event_id')}"
        states.append({
            "record_id": record_id,
            "family": "ql-mef-lensing",
            "question_id": "ql-position",
            "state": {
                "act_intent": bounded(act.get("intent"), 400),
                "carrier_kind": carrier.get("kind"),
                "carrier_name": carrier.get("name"),
                "face": act.get("face") or event.get("face"),
                "preceding_from_position": ql.get("from"),
                "task_success_conditions": [
                    bounded(c, 160) for c in (rec.get("success_conditions") or [])[:6]
                ],
                "task_id": rec.get("task_id"),
            },
        })
        truth.append({"record_id": record_id, "question_id": "ql-position", "answer": position})
        n += 1
    return n


def build_mef_lens(runs_dir, limit, states):
    n = 0
    seen = set()
    for run_file, rec, event in iter_act_events(runs_dir):
        if n >= limit:
            return n
        act = event["payload"]["act"]
        intent = bounded(act.get("intent"), 500)
        key = (rec.get("task_id"), intent[:120])
        if key in seen or not intent:
            continue
        seen.add(key)
        record_id = f"mef-lens-refraction:{event.get('event_id')}"
        states.append({
            "record_id": record_id,
            "family": "ql-mef-lensing",
            "question_id": "mef-lens-refraction",
            "state": {
                "subject": intent,
                "carrier_kind": (act.get("carrier") or {}).get("kind"),
                "task_success_conditions": [
                    bounded(c, 160) for c in (rec.get("success_conditions") or [])[:6]
                ],
                "standing_refraction_pair": STANDING_REFRACTION_PAIR,
            },
            "expected_standing_pair": STANDING_REFRACTION_PAIR,
        })
        n += 1
    return n


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--runs-dir", type=Path,
        default=Path(__file__).resolve().parents[2] / "ql-runtime/comparison/series1/runs",
    )
    ap.add_argument("--out", type=Path, default=Path(__file__).resolve().parent)
    ap.add_argument("--limit", type=int, default=100, help="max records per question id (cost bound)")
    args = ap.parse_args()

    states, truth = [], []
    counts = {
        "ql-position": build_ql_position(args.runs_dir, args.limit, states, truth),
        "mef-lens-refraction": build_mef_lens(args.runs_dir, args.limit, states),
    }

    sources = {
        str(p): sha256_file(Path(p))
        for p in sorted(glob.glob(str(args.runs_dir / "*" / "*.json")))
    }
    manifest = {
        "schema": "actuation.model-types-jev-states/v1",
        "created": "2026-09-17",
        "scope": "QL/MEF lensing and classification; the M-coordinate census work is deliberately out of scope",
        "sources": sources,
        "counts": counts,
        "truth_standing": {
            "ql-position": "loop-recorded position state from the retained ql-condition runs; not human-verified",
            "mef-lens-refraction": "none — instrument reading, research standing; report mass on the standing L1+L4' pair as agreement-of-readings",
        },
    }

    for name, payload in (("states.jsonl", states), ("truth.jsonl", truth)):
        with (args.out / name).open("w") as f:
            for rec_ in payload:
                f.write(json.dumps(rec_, ensure_ascii=False) + "\n")
    (args.out / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(json.dumps({"counts": counts, "out": str(args.out)}))


if __name__ == "__main__":
    sys.exit(main())
