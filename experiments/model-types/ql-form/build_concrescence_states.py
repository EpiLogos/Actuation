#!/usr/bin/env python3
"""E-MT-CONCRESCENCE — build typed states for the QL determination question.

For each closed circuit with a determination residue in the retained Series-1
ql-* records, emit one state per question per degradation variant:

  intact                 the circuit as recorded
  strip-P1               material residues (#1) removed
  strip-P4               evaluation residues (#4) removed
  strip-middle           all of #1-#4 removed (determination floats on the frame)
  shallow-determination  synthesis replaced with "Done."

The degradation ladder is the instrument validation: if Jev reads type
fullness, warrantedness must FALL as the circuit is gutted — a flat curve
invalidates the instrument before any agreement number counts. Degraded
variants carry NO truth (the recorded closure is not truth for a gutted
circuit); only intact closure-appropriate states carry truth (the recorded
close, standing pending-human-review).

The state deliberately excludes the loop's bookkeeping (closure state,
verification flags, requested_outcome): this asks about the whole, not about
what the loop recorded having decided.

Both host prompt shapes are handled (deeplane frame/circuit/native shapes).
Output: concrescence/states.jsonl, truth.jsonl, manifest.json.
"""

import glob
import json
import sys
import time
from pathlib import Path

RUNS = Path(__file__).resolve().parents[2] / "ql-runtime/comparison/series1/runs"
OUT = Path(__file__).resolve().parent / "concrescence"
QUESTIONS = ["determination-warranted", "middle-maturity", "closure-appropriate"]
VARIANTS = ["intact", "strip-P1", "strip-P4", "strip-middle", "shallow-determination"]
POSITIONS_MIDDLE = {"P1", "P2", "P3", "P4"}


def bounded(x, n):
    return str(x)[:n] if x else ""


def first_ql_next_act_doc(events):
    pending = None
    for e in sorted(events, key=lambda x: x.get("sequence", 0)):
        if e.get("event_type") == "model_requested":
            pending = e
        elif e.get("event_type") == "model_returned" and pending is not None:
            if (pending.get("payload") or {}).get("purpose") == "ql-next-act":
                try:
                    return json.loads(((pending["payload"] or {}).get("input") or {}).get("prompt") or "")
                except json.JSONDecodeError:
                    return {}
            pending = None
    return {}


def circuit_state(record, variant):
    rec = record.get("record") or {}
    circuit = (rec.get("result") or {}).get("circuit") or {}
    events = rec.get("events") or []
    doc = first_ql_next_act_doc(events)
    frame = (doc.get("frame") or {}) or (doc.get("circuit") or {})

    residues = circuit.get("residues") or []
    middle, determination = [], None
    for r in residues:
        pos = str(r.get("position") or "")
        val = r.get("value") or {}
        if r.get("kind") == "determination":
            determination = val
            continue
        if pos in POSITIONS_MIDDLE:
            middle.append({
                "position": pos, "kind": r.get("kind"),
                "summary": bounded(val.get("semantic_summary") or val.get("difference") or val, 220),
            })

    if determination is None or not closed(record, circuit):
        return None
    synthesis = determination.get("synthesis")
    if not (synthesis or "").strip():
        return None
    if variant == "shallow-determination":
        synthesis = "Done."
    if variant in ("strip-P1", "strip-middle"):
        middle = [m for m in middle if m["position"] != "P1"]
    if variant in ("strip-P4", "strip-middle"):
        middle = [m for m in middle if m["position"] != "P4"]

    return {
        "task_id": record.get("task_id"),
        "circuit_id": circuit.get("id"),
        "condition": record.get("condition"),
        "initiating_intent": bounded(frame.get("initiating_intent") or doc.get("task"), 500),
        "success_conditions": [bounded(c, 160) for c in (record.get("success_conditions") or [])[:6]],
        "middle_residues": middle,
        "determination": {
            "synthesis": bounded(synthesis, 500),
            "claimed_adequacy": determination.get("claimed_adequacy"),
            "evidence_ref_count": len(determination.get("evidence_refs") or []),
        },
    }


def closed(record, circuit):
    rec = record.get("record") or {}
    result = (rec.get("result") or {})
    return circuit.get("closureState") == "closed" or bool(rec.get("closure")) or bool(result.get("closure"))


def main():
    states, truth = [], []
    circuits = 0
    for run_file in sorted(glob.glob(str(RUNS / "2026-09-13-glm" / "*.json"))
                           + sorted(glob.glob(str(RUNS / "2026-09-13-smoke-restraint" / "*.json")))):
        path = Path(run_file)
        try:
            doc = json.loads(path.read_text())
        except json.JSONDecodeError:
            continue
        for idx, record in enumerate(doc.get("records") or []):
            if not str(record.get("condition", "")).startswith("ql-"):
                continue
            for variant in VARIANTS:
                state = circuit_state(record, variant)
                if state is None:
                    continue
                if variant == "intact":
                    circuits += 1
                stem = path.stem
                cid = state["circuit_id"]
                for qid in QUESTIONS:
                    rid = f"{qid}|{stem}|{cid}|{variant}"
                    states.append({"record_id": rid, "family": "concrescence",
                                   "question_id": qid, "variant": variant, "state": state})
                    if qid == "closure-appropriate" and variant == "intact":
                        truth.append({"record_id": rid, "question_id": qid, "answer": "close"})

    OUT.mkdir(exist_ok=True)
    with (OUT / "states.jsonl").open("w") as f:
        for s in states:
            f.write(json.dumps(s, ensure_ascii=False) + "\n")
    with (OUT / "truth.jsonl").open("w") as f:
        for t in truth:
            f.write(json.dumps(t, ensure_ascii=False) + "\n")
    (OUT / "manifest.json").write_text(json.dumps({
        "schema": "actuation.model-types-concrescence-states/v1",
        "created": time.strftime("%Y-%m-%d"),
        "experiment": "E-MT-CONCRESCENCE",
        "circuits": circuits,
        "states": len(states),
        "variants": VARIANTS,
        "truth_standing": {
            "closure-appropriate|intact": "the circuit's recorded closure — the incumbent's own outcome, agreement not ground truth, pending-human-review",
            "all degraded variants": "no truth by design: the degradation curve is the instrument validation",
            "determination-warranted": "no truth by design: warrantedness of a determination is exactly what human review exists to judge",
        },
        "provenance": {"promotion": "none", "reading_class": "semantic-stochastic"},
    }, indent=2) + "\n")
    print(json.dumps({"circuits": circuits, "states": len(states),
                      "by_variant": {v: sum(1 for s in states if s["variant"] == v) for v in VARIANTS}}))


if __name__ == "__main__":
    sys.exit(main())
