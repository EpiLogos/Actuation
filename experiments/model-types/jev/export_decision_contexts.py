#!/usr/bin/env python3
"""Build E-MT-2 replay states from retained Series-1 run records.

Maps the QL loop's five model-facing decision points (question-catalogue.json
families.loop-refereeing.decision_points) onto typed Jev questions, replayed
offline from the retained run records (ql-* conditions only).

One state per historical decision point. The state carries what the loop saw
at that point — parsed from the loop's own `ql-next-act` / `ql-conjugate-scope`
controller prompts, plus the record's verification and outcome fields — minus
the answer (the answer is the paired `model_returned` control, used as truth
where the record derives it honestly).

Truth standing (all loop-recorded, not human-verified; determination remains
pending-human-review):

  next-carrier            truth = the recorded act's carrier.kind (the loop's
                          own decision at that point).
  next-position           truth = the recorded transition's `to` position.
  requested-outcome       truth = 'close' if the circuit closed
                          (closureState == 'closed' or a closure object is
                          recorded) else 'reopen'.
  intent-already-achieved truth derived only where the record forces it:
                          the loop chose an exterior act (model/capability)
                          -> false (it did not treat the intent as achieved);
                          the loop chose internal_control AND the run ended
                          completed with closureState closed and
                          verification.objective_checks_pass -> true.
                          Internal_control decisions on runs that did not end
                          that way are omitted (not honestly derivable).
  subject-verifiable      truth = claimed_subject (from the recorded P5
                          determination residue) equals the task_id exactly
                          AND verification.objective_checks_pass. Mechanical,
                          record-derived; where claimed_subject is absent the
                          state is omitted.
  stipulation-violated,
  genuinely-local-whole,
  act-adequacy            NO truth — instrument readings only (calibration /
                          confidence data, not accuracy). The recorded
                          verdicts on these points are the incumbent's own
                          evaluations, not ground truth.

Deterministic ordering: files sorted by path, records in document order,
events by sequence; selection interleaves question ids round-robin in a fixed
order up to --limit total states.

Emits states.jsonl / truth.jsonl / manifest.json into --out (default emt2/,
so the E-MT-1 files are never overwritten).
"""

import argparse
import glob
import hashlib
import json
import re
import sys
from pathlib import Path

QUESTION_ORDER = [
    "next-carrier",
    "next-position",
    "requested-outcome",
    "intent-already-achieved",
    "subject-verifiable",
    "stipulation-violated",
    "genuinely-local-whole",
    "act-adequacy",
]

POSITIONS = {"P0", "P1", "P2", "P3", "P4", "P5"}


def bounded(text, n):
    return str(text)[:n] if text else ""


def sha256_obj(obj):
    return hashlib.sha256(json.dumps(obj, ensure_ascii=False, sort_keys=True).encode()).hexdigest()


def parse_prompt_json(prompt_text):
    try:
        return json.loads(prompt_text)
    except (json.JSONDecodeError, TypeError):
        return None


def residue_summaries(circuit, max_items=3, width=220):
    out = []
    for res in (circuit.get("residues") or [])[-max_items:]:
        val = res.get("value") or {}
        out.append({
            "position": res.get("position"),
            "kind": res.get("kind"),
            "summary": bounded(val.get("semantic_summary"), width),
        })
    return out


def recent_events_digest(events, upto_seq, max_items=4, width=220):
    """Bounded recent event window (payload facts only, before the decision)."""
    window = []
    for e in events:
        if e.get("sequence", 0) >= upto_seq:
            continue
        p = e.get("payload") or {}
        item = None
        if isinstance(p.get("act"), dict):
            act = p["act"]
            item = {
                "type": e.get("event_type"),
                "position": act.get("source_position"),
                "carrier_kind": (act.get("carrier") or {}).get("kind"),
                "summary": bounded(act.get("intent"), width),
            }
        elif isinstance(p.get("residue"), dict):
            res = p["residue"]
            val = res.get("value") or {}
            item = {
                "type": e.get("event_type"),
                "position": res.get("position"),
                "kind": res.get("kind"),
                "summary": bounded(val.get("semantic_summary"), width),
            }
        if item:
            window.append(item)
    window = window[-max_items:]
    return window, sha256_obj(window)


def base_context(record, circuit, prompt_doc, provenance, events, upto_seq):
    budget = prompt_doc.get("budget") or {}
    allowance = budget.get("allowance") or {}
    rec = record.get("record") or {}
    recent, digest = recent_events_digest(events, upto_seq)
    return {
        "run_id": rec.get("run_id"),
        "task_id": record.get("task_id"),
        "circuit_id": circuit.get("id"),
        "active_position": (allowance.get("active_position") or {}).get("id")
        if isinstance(allowance.get("active_position"), dict)
        else allowance.get("active_position"),
        "mode": prompt_doc.get("mode"),
        "allowance_state": {
            "max_steps": budget.get("max_steps"),
            "steps_used": budget.get("steps_used"),
            "schedule": allowance.get("schedule"),
            "consumed": allowance.get("consumed"),
        },
        "stipulations": [
            {"id": s.get("id"), "kind": s.get("kind"), "statement": bounded(s.get("text"), 200)}
            for s in (prompt_doc.get("stipulations") or [])
        ][:8],
        "success_conditions": [bounded(c, 160) for c in (record.get("success_conditions") or [])[:6]],
        "capabilities": [c for c in (prompt_doc.get("capabilities") or []) if isinstance(c, str)],
        "recent_residues": residue_summaries(circuit),
        "recent_transcript_digest": digest,
        "recent_events": recent,
        "provenance": provenance,
    }


def carrier_kind_of(ctrl):
    """Carrier kind from a control answer; older runs store a plain string."""
    if not isinstance(ctrl, dict):
        return None
    carrier = ctrl.get("carrier")
    if isinstance(carrier, dict):
        return carrier.get("kind")
    if isinstance(carrier, str):
        return carrier
    return None


def pair_model_calls(events):
    """Pair model_requested with the next model_returned, in sequence order."""
    pairs = []
    pending = None
    for e in sorted(events, key=lambda x: x.get("sequence", 0)):
        if e.get("event_type") == "model_requested":
            pending = e
        elif e.get("event_type") == "model_returned" and pending is not None:
            pairs.append((pending, e))
            pending = None
    return pairs


def run_closed(record, circuit):
    rec = record.get("record") or {}
    result = rec.get("result") or {}
    if circuit.get("closureState") == "closed":
        return True
    return bool(rec.get("closure")) or bool(result.get("closure"))


def iter_ql_records(runs_dir):
    for run_file in sorted(glob.glob(str(runs_dir / "*" / "*.json"))):
        path = Path(run_file)
        try:
            doc = json.loads(path.read_text())
        except json.JSONDecodeError:
            continue
        for rec_index, record in enumerate(doc.get("records") or []):
            if not str(record.get("condition", "")).startswith("ql-"):
                continue
            yield path, rec_index, record


def collect(run_file, rec_index, record):
    """Return dict of question_id -> list of {record_id, state, truth}."""
    path = run_file
    stem = path.stem
    rec = record.get("record") or {}
    events = rec.get("events") or []
    result = rec.get("result") or {}
    circuit = result.get("circuit") or {}
    verification = record.get("verification") or {}
    checks_pass = bool(verification.get("objective_checks_pass"))
    condition = record.get("condition")

    def prov(extra):
        d = {"source_file": path.name, "record_index": rec_index, "condition": condition}
        d.update(extra)
        return d

    out = {q: [] for q in QUESTION_ORDER}

    # --- first parseable ql-next-act prompt: stipulations/task-level facts
    pairs = pair_model_calls(events)
    base_prompt_doc = None
    for req, _ in pairs:
        if req["payload"].get("purpose") != "ql-next-act":
            continue
        doc = parse_prompt_json((req["payload"].get("input") or {}).get("prompt"))
        if doc and doc.get("stipulations") is not None:
            base_prompt_doc = doc
            break

    # --- next-carrier / intent-already-achieved at each ql-next-act decision
    n = 0
    for req, ret in pairs:
        if req["payload"].get("purpose") != "ql-next-act":
            continue
        prompt_doc = parse_prompt_json((req["payload"].get("input") or {}).get("prompt"))
        if not prompt_doc:
            continue
        ctrl = ((ret.get("payload") or {}).get("output") or {}).get("control")
        carrier_kind = carrier_kind_of(ctrl)
        ctx = base_context(
            record, prompt_doc.get("circuit") or circuit, prompt_doc,
            prov({"event_id": req.get("event_id"), "purpose": "ql-next-act"}),
            events, req.get("sequence", 0),
        )
        # previous act intent, if any
        prev_intent = None
        for e in events:
            if e.get("sequence", 0) >= req.get("sequence", 0):
                break
            act = (e.get("payload") or {}).get("act")
            if isinstance(act, dict):
                prev_intent = bounded(act.get("intent"), 400)
        ctx["previous_act_intent"] = prev_intent

        rid = f"next-carrier|{stem}|{ctx['circuit_id']}|{n}"
        entry = {"record_id": rid, "state": ctx, "truth": carrier_kind}
        out["next-carrier"].append(entry)

        # intent-already-achieved: honest derivation only
        ia_truth = None
        if carrier_kind in ("model", "capability"):
            ia_truth = False
        elif carrier_kind == "internal_control" and run_closed(record, circuit) \
                and record.get("execution_status") == "completed" and checks_pass:
            ia_truth = True
        if ia_truth is not None:
            out["intent-already-achieved"].append({
                "record_id": f"intent-already-achieved|{stem}|{ctx['circuit_id']}|{n}",
                "state": ctx,
                "truth": ia_truth,
            })
        n += 1

    # --- act-adequacy readings at each recorded act (no truth)
    for e in events:
        act = (e.get("payload") or {}).get("act")
        if not isinstance(act, dict) or not act.get("intent"):
            continue
        carrier = act.get("carrier") or {}
        out["act-adequacy"].append({
            "record_id": f"act-adequacy|{stem}|{circuit.get('id')}|{act.get('id', e.get('event_id'))}",
            "state": {
                "task_id": record.get("task_id"),
                "circuit_id": circuit.get("id"),
                "active_position": act.get("source_position"),
                "mode": condition.replace("ql-", "") if condition else None,
                "success_conditions": [bounded(c, 160) for c in (record.get("success_conditions") or [])[:6]],
                "act_intent": bounded(act.get("intent"), 400),
                "carrier_kind": carrier.get("kind"),
                "carrier_name": carrier.get("name"),
                "stipulations": [
                    {"id": s.get("id"), "kind": s.get("kind"), "statement": bounded(s.get("text"), 200)}
                    for s in ((base_prompt_doc or {}).get("stipulations") or [])
                ][:8],
                "provenance": prov({"event_id": e.get("event_id"), "purpose": "act-adequacy"}),
            },
            "truth": None,
        })

    # --- next-position at each trajectory transition (truth = transition `to`)
    # R4 fix (2026-09-19): the state now carries the loop's mechanical state —
    # the per-position allowance schedule and its consumption as last restated
    # by the controller prompt that produced the act (policy.rs restates
    # budget.allowance in every ql-next-act payload). Transitions whose
    # destination was set by a routing rule (closure-request law, reopen
    # routing, allowance exhaustion) are classified `mechanical-*` and belong
    # to Rust, not to any model; only `model-interpretation` transitions form
    # the semantic remainder a typed referee may fairly be asked. The class is
    # recorded per record_id in destination-basis.json for the re-measure —
    # it is evaluation metadata only and is never placed in the state.
    trajectory = circuit.get("trajectory") or []
    acts_by_id = {}
    act_seq_by_id = {}
    for e in events:
        act = (e.get("payload") or {}).get("act")
        if isinstance(act, dict) and act.get("id"):
            acts_by_id[act["id"]] = act
            act_seq_by_id[act["id"]] = e.get("sequence", 0)

    def allowance_state_for(seq):
        """Schedule + consumption as last restated by a ql-next-act prompt at
        or before `seq` (the freshest recorded allowance state)."""
        best = None
        for req, _ in pairs:
            if req["payload"].get("purpose") != "ql-next-act":
                continue
            if req.get("sequence", 0) > seq:
                continue
            doc = parse_prompt_json((req["payload"].get("input") or {}).get("prompt"))
            if doc:
                best = doc
        if best is None:
            return None
        budget = best.get("budget") or {}
        allowance = budget.get("allowance") or {}
        active = allowance.get("active_position")
        return {
            "max_steps": budget.get("max_steps"),
            "steps_used": budget.get("steps_used"),
            "schedule": allowance.get("schedule"),
            "consumed": allowance.get("consumed"),
            "active_position": active.get("id") if isinstance(active, dict) else active,
            "source": "last ql-next-act controller prompt at or before the act",
        }

    def destination_basis(tr):
        """mechanical-* when a routing rule set the destination (Rust-owned);
        model-interpretation when the incumbent's ql-interpret-return decided."""
        witness = tr.get("witness_state") or {}
        facts = witness.get("structural_facts") or {}
        if isinstance(facts, dict):
            if facts.get("closure_request") is True:
                return "mechanical-closure-request"
            if facts.get("closure_status") in ("reopen", "closed"):
                return "mechanical-reopen-routing"
        if "allowance_refusal" in witness:
            return "mechanical-allowance-exhaustion"
        if "compressed_control" in witness or "vak_control" in witness:
            return "mechanical-rule-routing"
        if "claimed_position" in witness or "observed_position" in witness:
            return "model-interpretation"
        return "unclassified"

    for i, tr in enumerate(trajectory):
        to_pos = tr.get("to")
        if to_pos not in POSITIONS:
            continue
        interp = tr.get("interpretation_id") or ""
        act_id = interp[: -len(":return")] if interp.endswith(":return") else interp
        act = acts_by_id.get(act_id) or {}
        carrier = act.get("carrier") or {}
        witness = tr.get("witness_state") or {}
        allowance_state = allowance_state_for(act_seq_by_id.get(act_id, 0))
        out["next-position"].append({
            "record_id": f"next-position|{stem}|{circuit.get('id')}|{tr.get('id', i)}",
            "state": {
                "task_id": record.get("task_id"),
                "circuit_id": circuit.get("id"),
                "mode": condition.replace("ql-", "") if condition else None,
                "current_position": tr.get("from"),
                "allowance_state": allowance_state,
                "prior_trajectory": [
                    {"from": t.get("from"), "to": t.get("to"), "relation": t.get("relation")}
                    for t in trajectory[:i]
                ],
                "producing_act": {
                    "source_position": act.get("source_position"),
                    "intent": bounded(act.get("intent"), 300),
                    "carrier_kind": carrier.get("kind"),
                    "carrier_name": carrier.get("name"),
                    "claimed_position": witness.get("claimed_position") or act.get("claimed_position"),
                },
                "success_conditions": [bounded(c, 160) for c in (record.get("success_conditions") or [])[:6]],
                "provenance": prov({"event_id": tr.get("id"), "purpose": "resolve_Rij"}),
            },
            "truth": to_pos,
            "destination_basis": destination_basis(tr),
        })

    # --- requested-outcome at the determination point (one per record)
    outcome_state = {
        "task_id": record.get("task_id"),
        "circuit_id": circuit.get("id"),
        "mode": condition.replace("ql-", "") if condition else None,
        "success_conditions": [bounded(c, 160) for c in (record.get("success_conditions") or [])[:6]],
        "stipulations": [
            {"id": s.get("id"), "kind": s.get("kind"), "statement": bounded(s.get("text"), 200)}
            for s in ((base_prompt_doc or {}).get("stipulations") or [])
        ][:8],
        "verification": {
            "protocol": verification.get("protocol"),
            "observations": verification.get("observations"),
            "objective_checks_pass": checks_pass,
        },
        "execution_status": record.get("execution_status"),
        "semantic_status": record.get("semantic_status"),
        "outcome": bounded(record.get("outcome"), 900),
        "residue_path": [
            {"position": r.get("position"), "kind": r.get("kind")}
            for r in (circuit.get("residues") or [])
        ],
        "trajectory": [
            {"from": t.get("from"), "to": t.get("to")} for t in trajectory
        ],
        "provenance": prov({"purpose": "ql-propose-determination"}),
    }
    out["requested-outcome"].append({
        "record_id": f"requested-outcome|{stem}|{circuit.get('id')}",
        "state": outcome_state,
        "truth": "close" if run_closed(record, circuit) else "reopen",
    })

    # --- subject-verifiable at the determination residue (mechanical truth)
    claimed_subject = None
    for res in reversed(circuit.get("residues") or []):
        if res.get("kind") == "determination":
            claimed_subject = (res.get("value") or {}).get("claimed_subject")
            if claimed_subject:
                break
    if claimed_subject:
        sv_truth = bool(str(claimed_subject).strip() == record.get("task_id") and checks_pass)
        out["subject-verifiable"].append({
            "record_id": f"subject-verifiable|{stem}|{circuit.get('id')}",
            "state": {
                "task_id": record.get("task_id"),
                "circuit_id": circuit.get("id"),
                "mode": condition.replace("ql-", "") if condition else None,
                "claimed_subject": bounded(claimed_subject, 300),
                "verification": {
                    "observations": verification.get("observations"),
                    "objective_checks_pass": checks_pass,
                },
                "execution_status": record.get("execution_status"),
                "success_conditions": [bounded(c, 160) for c in (record.get("success_conditions") or [])[:6]],
                "provenance": prov({"purpose": "ql-propose-determination"}),
            },
            "truth": sv_truth,
        })

    # --- stipulation-violated readings, per stipulation (no truth)
    if base_prompt_doc:
        observed = {
            "task_id": record.get("task_id"),
            "circuit_id": circuit.get("id"),
            "mode": condition.replace("ql-", "") if condition else None,
            "execution_status": record.get("execution_status"),
            "verification_observations": verification.get("observations"),
            "objective_checks_pass": checks_pass,
            "residues": residue_summaries(circuit, max_items=5, width=200),
            "carrier_kinds_used": [
                a.get("carrier", {}).get("kind")
                for a in (acts_by_id.values())
                if isinstance(a, dict)
            ],
            "success_conditions": [bounded(c, 160) for c in (record.get("success_conditions") or [])[:6]],
        }
        for s in (base_prompt_doc.get("stipulations") or [])[:8]:
            out["stipulation-violated"].append({
                "record_id": f"stipulation-violated|{stem}|{circuit.get('id')}|{s.get('id')}",
                "state": {
                    **observed,
                    "stipulation": {
                        "id": s.get("id"),
                        "kind": s.get("kind"),
                        "statement": bounded(s.get("text"), 250),
                    },
                    "provenance": prov({"purpose": "classify_stipulations", "stipulation_id": s.get("id")}),
                },
                "truth": None,
            })

    # --- genuinely-local-whole readings (deep mode; no truth)
    glw_ctxs = []
    for req, _ in pairs:
        if req["payload"].get("purpose") != "ql-conjugate-scope":
            continue
        prompt_doc = parse_prompt_json((req["payload"].get("input") or {}).get("prompt"))
        if not prompt_doc:
            continue
        circ_doc = prompt_doc.get("circuit") or circuit
        glw_ctxs.append({
            "task_id": record.get("task_id"),
            "circuit_id": circ_doc.get("id"),
            "mode": prompt_doc.get("mode"),
            "active_position": (prompt_doc.get("budget", {}).get("allowance") or {}).get("active_position"),
            "trajectory": [
                {"from": t.get("from"), "to": t.get("to"), "relation": t.get("relation")}
                for t in (circ_doc.get("trajectory") or [])
            ][:12],
            "recent_residues": residue_summaries(circ_doc, max_items=4),
            "success_conditions": [bounded(c, 160) for c in (record.get("success_conditions") or [])[:6]],
            "decision_point": "ql-conjugate-scope",
            "provenance": prov({"event_id": req.get("event_id"), "purpose": "ql-conjugate-scope"}),
        })
    if condition == "ql-deep":
        for req, _ in pairs:
            if req["payload"].get("purpose") != "ql-next-act":
                continue
            prompt_doc = parse_prompt_json((req["payload"].get("input") or {}).get("prompt"))
            if not prompt_doc:
                continue
            allowance = (prompt_doc.get("budget") or {}).get("allowance") or {}
            active = allowance.get("active_position")
            active = active.get("id") if isinstance(active, dict) else active
            if active != "P4":
                continue
            ctx = base_context(
                record, prompt_doc.get("circuit") or circuit, prompt_doc,
                prov({"event_id": req.get("event_id"), "purpose": "deep-operator-depth"}),
                events, req.get("sequence", 0),
            )
            ctx["deep_operator_option"] = True
            glw_ctxs.append(ctx)
    for i, ctx in enumerate(glw_ctxs):
        out["genuinely-local-whole"].append({
            "record_id": f"genuinely-local-whole|{stem}|{circuit.get('id')}|{i}",
            "state": ctx,
            "truth": None,
        })

    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--runs-dir", type=Path,
        default=Path(__file__).resolve().parents[2] / "ql-runtime/comparison/series1/runs",
    )
    ap.add_argument("--out", type=Path, default=Path(__file__).resolve().parent / "emt2")
    ap.add_argument("--limit", type=int, default=150, help="max total states across all question ids")
    args = ap.parse_args()

    per_question = {q: [] for q in QUESTION_ORDER}
    sources = {}
    for run_file, rec_index, record in iter_ql_records(args.runs_dir):
        sources[str(run_file)] = hashlib.sha256(Path(run_file).read_bytes()).hexdigest()
        try:
            collected = collect(run_file, rec_index, record)
        except Exception as exc:  # fail loud per record, keep going deterministically
            print(f"record-failed: {run_file}[{rec_index}]: {exc}", file=sys.stderr)
            continue
        for qid, items in collected.items():
            per_question[qid].extend(items)

    # deterministic round-robin selection up to --limit total
    selected = []
    cursor = {q: 0 for q in QUESTION_ORDER}
    while len(selected) < args.limit:
        progressed = False
        for qid in QUESTION_ORDER:
            if len(selected) >= args.limit:
                break
            items = per_question[qid]
            if cursor[qid] < len(items):
                selected.append(items[cursor[qid]])
                cursor[qid] += 1
                progressed = True
        if not progressed:
            break

    counts = {}
    destination_basis = {}
    for qid in QUESTION_ORDER:
        picked = [s for s in selected if s["record_id"].split("|", 1)[0] == qid]
        counts[qid] = {
            "generated": len(per_question[qid]),
            "selected": len(picked),
            "with_truth": sum(1 for s in picked if s.get("truth") is not None),
        }
        if qid == "next-position":
            for s in picked:
                destination_basis[s["record_id"]] = s.get("destination_basis")
            counts[qid]["destination_basis"] = dict(
                __import__("collections").Counter(
                    s.get("destination_basis") for s in picked))

    args.out.mkdir(parents=True, exist_ok=True)
    with (args.out / "states.jsonl").open("w") as f:
        for s in selected:
            f.write(json.dumps(
                {"record_id": s["record_id"], "family": "loop-refereeing",
                 "question_id": s["record_id"].split("|", 1)[0], "state": s["state"]},
                ensure_ascii=False) + "\n")
    with (args.out / "truth.jsonl").open("w") as f:
        for s in selected:
            if s.get("truth") is None:
                continue
            f.write(json.dumps(
                {"record_id": s["record_id"],
                 "question_id": s["record_id"].split("|", 1)[0],
                 "answer": s["truth"]},
                ensure_ascii=False) + "\n")

    manifest = {
        "schema": "actuation.model-types-jev-states/v1",
        "created": "2026-09-19",
        "experiment": "E-MT-2 loop decision refereeing",
        "sources": sources,
        "selection": f"deterministic round-robin over question ids {QUESTION_ORDER}, cap {args.limit}",
        "counts": counts,
        "truth_standing": {
            "next-carrier": "loop-recorded carrier kind (the incumbent's own decision); agreement, not ground truth",
            "next-position": "loop-recorded transition destination; agreement, not ground truth. R4: state now carries allowance_state; transitions are classified mechanical-* vs model-interpretation in destination-basis.json — the semantic remainder is the fair re-measure set",
            "requested-outcome": "derived from the record's closure state (closureState/closure objects)",
            "intent-already-achieved": "derived: exterior act -> false; internal_control on completed+closed+objective_checks_pass -> true; otherwise omitted",
            "subject-verifiable": "derived mechanically: claimed_subject == task_id and objective_checks_pass; omitted where claimed_subject absent",
            "stipulation-violated": "none — instrument reading only (recorded verdicts are the incumbent's own evaluations)",
            "genuinely-local-whole": "none — instrument reading only",
            "act-adequacy": "none — instrument reading only",
        },
        "determination": "pending-human-review",
        "provenance": {"promotion": "none", "reading_class": "semantic-stochastic"},
    }
    (args.out / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    # R4: per-transition destination class (evaluation metadata, never in the
    # state). mechanical-* transitions are Rust-owned routing outcomes and are
    # excluded from the semantic-remainder re-measure.
    (args.out / "destination-basis.json").write_text(json.dumps({
        "schema": "actuation.model-types-jev-destination-basis/v1",
        "note": "mechanical-* = destination set by a routing rule (closure-request law, reopen routing, allowance exhaustion, compressed/vak control): Rust-owned, not a model question. model-interpretation = incumbent ql-interpret-return decided; the semantic remainder.",
        "basis": destination_basis,
    }, indent=2) + "\n")
    print(json.dumps({
        "out": str(args.out),
        "states": len(selected),
        "truth": sum(1 for s in selected if s.get("truth") is not None),
        "counts": counts,
    }, indent=2))


if __name__ == "__main__":
    sys.exit(main())
