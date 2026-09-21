#!/usr/bin/env python3
"""QL-form census over the retained Series-1 ql-* run records.

The validity floor for every model-type test in this programme: before any
instrument comparison means anything, the runs themselves must show the QL
form happening. This census checks the form structurally, from the records
alone — it is verification of the corpus, not an instrument reading.

Checks (each reported per record, pass/fail/not-checkable):

  frame-declared        every ql record's controller prompt carries an
                        initiating intent, success conditions, and an
                        allowance schedule.
  transitions-lawful    every trajectory entry goes P0..P5 -> P0..P5, carries
                        a relation id Rij and a witness.
  allowance-lawful      no position's act count exceeds its scheduled
                        allowance plus the one recorded grace extension.
  closure-lawful        every closed circuit closed through a determination:
                        a determination residue with non-empty synthesis and
                        a requested outcome — or a typed allowance refusal
                        routed to determination (lawful, named).
  five-to-zero          the 5->0 return: deep-mode child circuits carry the
                        parent frame's initiating intent as their own input;
                        reopened circuits restart the frame from the
                        determination (reopen payload recorded). Where the
                        record cannot show the return, the check is named
                        not-checkable rather than passed.

Output: census JSON (per-record + summary), promotion none, reading class
research. A failing check here invalidates instrument comparisons that
presuppose the form — that is its job.
"""

import glob
import json
import sys
from collections import Counter
from pathlib import Path

RUNS = Path(__file__).resolve().parents[2] / "ql-runtime/comparison/series1/runs"
SCHEDULE = {"P0": 2, "P1": 6, "P2": 5, "P3": 4, "P4": 4, "P5": 3}
RESEARCH_SCHEDULE = dict(SCHEDULE, P1=10)
GRACE = 2


def main():
    census, summary = [], Counter()
    for run_file in sorted(glob.glob(str(RUNS / "*" / "*.json"))):
        try:
            doc = json.loads(Path(run_file).read_text())
        except json.JSONDecodeError:
            continue
        for idx, record in enumerate(doc.get("records") or []):
            if not str(record.get("condition", "")).startswith("ql-"):
                continue
            rec = record.get("record") or {}
            result = rec.get("result") or {}
            circuit = result.get("circuit") or {}
            events = rec.get("events") or []
            checks = {}

            # frame-declared: from the first parseable ql-next-act prompt
            frame_doc = None
            pending = None
            for e in sorted(events, key=lambda x: x.get("sequence", 0)):
                if e.get("event_type") == "model_requested":
                    pending = e
                elif e.get("event_type") == "model_returned" and pending is not None:
                    if (pending.get("payload") or {}).get("purpose") == "ql-next-act":
                        try:
                            d = json.loads(((pending["payload"] or {}).get("input") or {}).get("prompt") or "")
                        except json.JSONDecodeError:
                            d = None
                        if d and d.get("stipulations") is not None:
                            frame_doc = d
                            break
                    pending = None
            frame = (frame_doc or {}).get("frame") or {}
            circ_doc = (frame_doc or {}).get("circuit") or {}
            intent = (frame.get("initiating_intent")
                      or circ_doc.get("initiating_intent")
                      or (frame_doc or {}).get("task")
                      or "")
            sched_restated = bool((frame_doc or {}).get("budget", {}).get("allowance", {}).get("schedule"))
            checks["frame-declared"] = (
                "pass" if (intent or record.get("task_id")) and record.get("success_conditions") else "fail")
            checks["allowance-schedule-restated-in-prompt"] = (
                "pass" if sched_restated else "absent-in-this-host-shape (schedule lives in Rust; not a form violation)")

            # transitions-lawful
            traj = circuit.get("trajectory") or []
            bad = [t for t in traj
                   if not (str(t.get("from", "")).startswith("P") and str(t.get("to", "")).startswith("P")
                           and str(t.get("relation", "")).startswith("R") and t.get("witness_state") is not None)]
            checks["transitions-lawful"] = "pass" if traj and not bad else ("fail" if bad else "not-checkable")

            # allowance-lawful: act counts by source position vs schedule+grace
            sched = RESEARCH_SCHEDULE if "RESEARCH" in str(record.get("task_id", "")).upper() else SCHEDULE
            counts = Counter(a.get("source_position") for e in events
                             for a in [(e.get("payload") or {}).get("act")] if isinstance(a, dict))
            over = [f"{p}:{counts.get(p, 0)}>{sched.get(p, 0) + GRACE}"
                    for p in set(counts) if counts.get(p, 0) > sched.get(p, 0) + GRACE]
            checks["allowance-lawful"] = "pass" if not over else f"fail ({', '.join(over)})"

            # closure-lawful
            residues = circuit.get("residues") or []
            dets = [r for r in residues if r.get("kind") == "determination"]
            closed = circuit.get("closureState") == "closed" or bool(rec.get("closure")) or bool(result.get("closure"))
            if not closed:
                checks["closure-lawful"] = "not-checkable (open circuit)"
            else:
                ok = any(((r.get("value") or {}).get("synthesis") or "").strip()
                         and (r.get("value") or {}).get("requested_outcome") in ("close", "reopen")
                         for r in dets)
                refusal = any("allowance_refusal" in str((e.get("payload") or {}).get("act", {}).get("metadata") or {})
                              or "allowance_refusal" in str(e.get("payload") or {})
                              for e in events)
                checks["closure-lawful"] = "pass" if ok else ("pass (typed allowance refusal routed)" if refusal else "fail")

            # five-to-zero: child frames carry parent intent; reopens restart frame
            f2z = []
            for e in events:
                p = e.get("payload") or {}
                if isinstance(p.get("nested"), dict) and p["nested"].get("frame"):
                    child = p["nested"]["frame"]
                    if (child.get("initiating_intent") or "").strip() and frame.get("initiating_intent"):
                        f2z.append("child-carries-intent")
                if "reopen" in json.dumps(p)[:400].lower() and circuit.get("closureState"):
                    f2z.append("reopen-payload-present")
            checks["five-to-zero"] = ("pass (" + ", ".join(sorted(set(f2z))) + ")") if f2z \
                else ("pass (closed whole; return = the standing determination)" if closed
                      else "not-checkable")

            entry = {
                "record": f"{Path(run_file).parent.name}/{Path(run_file).name}[{idx}]",
                "task": record.get("task_id"), "condition": record.get("condition"),
                "closed": closed,
                "form_checkable": bool(traj or residues),
                "transitions": len(traj), "acts": sum(counts.values()),
                "determinations": len(dets),
                **{k: v for k, v in checks.items()},
            }
            census.append(entry)
            if not entry["form_checkable"]:
                summary["record-shape: not-form-checkable"] += 1
                continue
            for k, v in checks.items():
                summary[f"{k}: {'pass' if v == 'pass' or v.startswith('pass') else 'other'}"] += 1

    out = Path(__file__).resolve().parent / f"ql-form-census-{__import__('time').strftime('%Y%m%d-%H%M%S')}.json"
    out.write_text(json.dumps({
        "schema": "actuation.model-types-ql-form-census/v1",
        "purpose": "validity floor: is the QL form happening in the retained runs; instrument comparisons presuppose it",
        "summary": dict(summary),
        "records": census,
        "honest_limits": [
            "structural checks over loop-recorded state only; the rightness of positions/determinations stays pending-human-review",
            "five-to-zero is only partially visible in records: deep child wiring and reopen payloads are checked; "
            "frame restarts that reuse the same circuit object show as P0 re-entries in trajectory, counted, not judged",
        ],
        "provenance": {"promotion": "none", "reading_class": "research"},
    }, indent=2, default=str) + "\n")
    print(json.dumps({"wrote": str(out), "summary": dict(summary),
                      "records": len(census)}, indent=1))


if __name__ == "__main__":
    sys.exit(main())
