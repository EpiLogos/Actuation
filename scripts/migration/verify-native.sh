#!/usr/bin/env bash
# Deterministic native gates. No provider or owner-machine acceptance is
# inferred from this build environment. Since the R7 cutover the served product
# is the native executable. The frozen Node-era migration corpora and their
# native replay gate were retired in cleanup/retire-node-oracle-2026-09-22:
# the product's own contract, conformance and verify suites are the standing
# acceptance. Evidence file names are unchanged.
set -euo pipefail
cd "$(dirname "$0")/../.."
evidence="${1:-native-evidence}"
# The hosted evidence directory is exclusive, ordinary, and within this actual
# checkout. Refuse an existing/symlinked destination instead of overwriting it.
evidence="$(python3 - "$evidence" <<'ADMIT'
import os
from pathlib import Path
import sys
root = Path.cwd().resolve(strict=True)
requested = Path(sys.argv[1])
assert requested.name not in ("", ".", "..") and ".." not in requested.parts
selected = requested if requested.is_absolute() else root / requested
parent = selected.parent.resolve(strict=True)
assert parent.is_relative_to(root), "evidence destination is outside the hosted checkout"
flags = os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW | os.O_DIRECTORY
fd = os.open(parent, flags)
try:
    os.mkdir(selected.name, mode=0o700, dir_fd=fd)
    held = os.open(selected.name, flags, dir_fd=fd)
    try:
        actual = os.fstat(held)
        named = os.stat(selected.name, dir_fd=fd, follow_symlinks=False)
        assert (actual.st_dev, actual.st_ino) == (named.st_dev, named.st_ino)
        print(parent / selected.name)
    finally:
        os.close(held)
finally:
    os.close(fd)
ADMIT
)"
# Evidence is ordinary hosted output, not a second native state/owner store.
# Fixtures are never recursively uploaded or deleted by this gate.
cat > "$evidence/qualification-driver.py" <<'PY'
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys
import time

ROOT = Path.cwd().resolve(strict=True)
EVIDENCE = Path(sys.argv[1]).resolve(strict=True)
MODE = sys.argv[2]
FLAGS = os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW
DIRECTORY = FLAGS | os.O_DIRECTORY
SHA_COST = {"bytes": 0, "seconds": 0.0,
            "profile": "actual wall time hashing data; image measurements include read/copy IO, not isolated CPU"}
MODULES = [
    ("research", "crates/actuation-research/src/process.rs", "native_retirement_tests", 12),
    ("research", "crates/actuation-research/src/sdk.rs", "native_finish_tests", 2),
    ("core", "crates/actuation-core/src/wire.rs", "causal_error_tests", 1),
]
BOOLS = {
    "spawned", "direct_child_reaped", "signal_forbidden", "term_signal_attempted",
    "kill_signal_attempted", "direct_kill_attempted", "group_absent",
    "terminal_group_owner_only_observed",
    "unreaped_owner_observed_before_signal", "retirement_deadline_exhausted",
    "reply_observed", "capture_truncated", "capture_read_failed",
}
OPTIONAL_INTS = {"exit_code", "signal", "stdout_observed_bytes", "stderr_observed_bytes"}
INTS = {"captured_stdout_bytes", "captured_stderr_bytes"}


def write(name, value):
    (EVIDENCE / name).parent.mkdir(parents=True, exist_ok=True)
    (EVIDENCE / name).write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


def basis(s):
    return {"device": s.st_dev, "inode": s.st_ino, "mode": s.st_mode,
            "uid": s.st_uid, "nlink": s.st_nlink, "size": s.st_size,
            "mtime_ns": s.st_mtime_ns, "ctime_ns": s.st_ctime_ns}


def same(fd, path, expected):
    assert basis(os.fstat(fd)) == expected, "held executable/material basis changed"
    assert basis(os.stat(path, follow_symlinks=False)) == expected, "named basis changed"


def digest(data):
    started = time.monotonic()
    result = hashlib.sha256(data).hexdigest()
    SHA_COST["bytes"] += len(data)
    SHA_COST["seconds"] += time.monotonic() - started
    return result


def read_regular(parent, name, limit):
    fd = os.open(name, FLAGS | os.O_NONBLOCK, dir_fd=parent)
    try:
        initial = os.fstat(fd)
        assert stat.S_ISREG(initial.st_mode), "selected evidence member is not regular"
        assert initial.st_size <= limit, "selected evidence member exceeds read profile"
        chunks = []
        count = 0
        while count <= limit:
            data = os.read(fd, min(65536, limit + 1 - count))
            if not data:
                break
            chunks.append(data)
            count += len(data)
        assert count <= limit, "selected evidence member grew beyond read profile"
        assert basis(os.fstat(fd)) == basis(initial), "held evidence member changed"
        assert basis(os.stat(name, dir_fd=parent, follow_symlinks=False)) == basis(initial), \
            "named evidence member changed"
        data = b"".join(chunks)
        assert len(data) == initial.st_size, "evidence length changed"
        return data, basis(initial)
    finally:
        os.close(fd)


def phase(value):
    allowed = {"running", "spawn", "execution", "capture", "retirement", "rpc_setup",
               "rpc_finish", "owner_identity", "group_term", "group_kill", "grace_wait",
               "direct_kill", "reap", "capture_metadata", "capture_stdout", "capture_stderr",
               "late_rpc_capture", "group_membership"}
    assert type(value) is str and value in allowed, "unknown native owner phase"
    return value


def observation(value):
    assert type(value) is dict
    assert set(value) == BOOLS | OPTIONAL_INTS | INTS | {"phase"}, "unknown observation shape"
    result = {"phase": phase(value["phase"])}
    for key in BOOLS:
        assert type(value[key]) is bool
        result[key] = value[key]
    for key in OPTIONAL_INTS:
        assert value[key] is None or type(value[key]) is int
        result[key] = value[key]
    for key in INTS:
        assert type(value[key]) is int and value[key] >= 0
        result[key] = value[key]
    return result


def io_fact(value):
    if value is None:
        return None
    assert type(value) is dict and set(value) == {"kind", "raw_os_error"}
    assert type(value["kind"]) is str and re.fullmatch(r"[A-Za-z]{1,64}", value["kind"])
    assert value["raw_os_error"] is None or type(value["raw_os_error"]) is int
    return {"kind": value["kind"], "raw_os_error": value["raw_os_error"]}


def native_facts(value):
    keys = {"attachment", "observation", "primary_io", "cause_projection_limit",
            "supplemental_count", "secondary_count", "observed_group_absence_count",
            "cause_projection_truncated", "supplemental_io", "secondary", "observed_group_absence"}
    assert type(value) is dict and set(value) == keys, "unknown native facts shape"
    assert value["attachment"] in ("primary_source", "supplemental_source")
    result = {"attachment": value["attachment"], "observation": observation(value["observation"]),
              "primary_io": io_fact(value["primary_io"])}
    for key in ("cause_projection_limit", "supplemental_count", "secondary_count",
                "observed_group_absence_count"):
        assert type(value[key]) is int and value[key] >= 0
        result[key] = value[key]
    assert result["cause_projection_limit"] == 32
    assert type(value["cause_projection_truncated"]) is bool
    result["cause_projection_truncated"] = value["cause_projection_truncated"]
    assert type(value["supplemental_io"]) is list and len(value["supplemental_io"]) <= 32
    result["supplemental_io"] = [io_fact(row) for row in value["supplemental_io"]]
    for key in ("secondary", "observed_group_absence"):
        assert type(value[key]) is list and len(value[key]) <= 32
        result[key] = []
        for row in value[key]:
            assert type(row) is dict and set(row) == {"phase", "io"}
            result[key].append({"phase": phase(row["phase"]), "io": io_fact(row["io"])})
    return result


def terminal_causes(value, allowed=None):
    assert type(value) is list and len(value) <= 32
    result = []
    for row in value:
        assert type(row) is dict and set(row) == {"phase", "io"}
        label = row["phase"]
        if allowed is None:
            label = phase(label)
        else:
            assert type(label) is str and label in allowed
        result.append({"phase": label, "io": io_fact(row["io"])})
    return result


def optional_boolean(value):
    assert value is None or type(value) is bool
    return value


def optional_integer(value):
    assert value is None or type(value) is int
    return value


def terminal_case_record(value):
    # These are actual test-emitted scalar profiles, not arbitrary returned
    # Values. Never copy fixture paths, scripts, requests, capture or messages.
    case = value["case"]
    assert type(case) is str and case in ("terminal-spawn-group", "terminal-external-reap",
                                          "terminal-membership-probe")
    if case == "terminal-membership-probe":
        keys = {"case", "slots", "probe_refused", "owner_only", "actual_io", "observation"}
        assert set(value) == keys, "unknown native membership probe shape"
        assert type(value["slots"]) is int and value["slots"] in (1, 2)
        assert type(value["probe_refused"]) is bool
        owner_only = optional_boolean(value["owner_only"])
        assert (owner_only is None) == value["probe_refused"]
        actual = io_fact(value["actual_io"])
        assert actual is None or value["probe_refused"]
        return {"case": case, "slots": value["slots"], "probe_refused": value["probe_refused"],
                "owner_only": owner_only, "actual_io": actual,
                "observation": observation(value["observation"])}
    if set(value) == {"case", "phase", "actual_io"}:
        expected = {"terminal-spawn-group": "leader_spawn", "terminal-external-reap": "spawn"}
        assert value["phase"] == expected[case]
        actual = io_fact(value["actual_io"])
        assert actual is not None, "native spawn failure must retain actual IO"
        return {"case": case, "phase": expected[case], "actual_io": actual}
    if case == "terminal-spawn-group":
        keys = {"case", "terminal_observed", "member_group_matches", "leader_retirement",
                "leader_secondary", "leader_group_absence", "member_reaped", "member_exit_code",
                "member_signal", "unrelated_was_live_after_original_group_effect",
                "unrelated_retirement", "unrelated_secondary", "actual_errors"}
        assert set(value) == keys, "unknown terminal group scalar shape"
        for key in ("terminal_observed", "member_reaped"):
            assert type(value[key]) is bool
        result = {"case": case, "terminal_observed": value["terminal_observed"],
                  "member_reaped": value["member_reaped"],
                  "member_group_matches": optional_boolean(value["member_group_matches"]),
                  "member_exit_code": optional_integer(value["member_exit_code"]),
                  "member_signal": optional_integer(value["member_signal"]),
                  "unrelated_was_live_after_original_group_effect":
                      optional_boolean(value["unrelated_was_live_after_original_group_effect"]),
                  "leader_retirement": observation(value["leader_retirement"]),
                  "leader_secondary": terminal_causes(value["leader_secondary"]),
                  "leader_group_absence": terminal_causes(value["leader_group_absence"],
                                                           {"group_term", "group_kill"})}
        assert type(value["actual_errors"]) is list and len(value["actual_errors"]) <= 3
        result["actual_errors"] = terminal_causes(value["actual_errors"],
                                                  {"native_setup", "unrelated_observation", "member_reap"})
        result["unrelated_retirement"] = (None if value["unrelated_retirement"] is None
                                           else observation(value["unrelated_retirement"]))
        result["unrelated_secondary"] = (None if value["unrelated_secondary"] is None
                                          else terminal_causes(value["unrelated_secondary"]))
        return result
    keys = {"case", "terminal_observed", "external_wait_actually_reaped_exact_child", "external_io",
            "prerequisite_io", "actual_retirement", "actual_secondary", "memoized_same_record",
            "fixture_disposition"}
    assert set(value) == keys, "unknown terminal external-reap scalar shape"
    for key in ("terminal_observed", "external_wait_actually_reaped_exact_child", "memoized_same_record"):
        assert type(value[key]) is bool
    assert value["fixture_disposition"] == "retained-owner-unavailable"
    return {"case": case, "terminal_observed": value["terminal_observed"],
            "external_wait_actually_reaped_exact_child": value["external_wait_actually_reaped_exact_child"],
            "external_io": io_fact(value["external_io"]),
            "prerequisite_io": io_fact(value["prerequisite_io"]),
            "actual_retirement": observation(value["actual_retirement"]),
            "actual_secondary": terminal_causes(value["actual_secondary"]),
            "memoized_same_record": value["memoized_same_record"],
            "fixture_disposition": "retained-owner-unavailable"}


def scalar_record(value):
    assert type(value) is dict
    if "case" in value:
        return terminal_case_record(value)
    if "attachment" in value:
        return native_facts(value)
    if "phase" in value:
        return observation(value)
    if set(value) == {"actual_retirement", "external_wait_observed", "fixture_disposition"}:
        assert type(value["external_wait_observed"]) is bool
        assert value["fixture_disposition"] == "retained-owner-unavailable"
        return {"actual_retirement": observation(value["actual_retirement"]),
                "external_wait_observed": value["external_wait_observed"],
                "fixture_disposition": value["fixture_disposition"]}
    if set(value) == {"native_failure", "retained_supplemental"}:
        return {key: native_facts(value[key]) for key in value}
    if set(value) == {"status", "observation"}:
        assert value["status"] in ("completed", "failed")
        return {"status": value["status"], "observation": observation(value["observation"])}
    raise AssertionError("unavailable: unrecognised scalar record profile")


def refusal(error, stage):
    return {"stage": stage, "error_type": type(error).__name__,
            "errno": error.errno if isinstance(error, OSError) else None}


def snapshot(label):
    # Original fixtures stay with their owner. Never walk/copy arbitrary files,
    # follow links, read pipes or export scripts/private capture/reply bodies.
    descriptors = []
    report = {"label": label, "original_fixtures_deleted": False, "entries": []}
    try:
        fd = os.open(ROOT, DIRECTORY)
        descriptors.append(fd)
        for component in ("ProjectCentral", "now", "tmp", "actuation-process-native"):
            try:
                fd = os.open(component, DIRECTORY, dir_fd=fd)
            except FileNotFoundError as error:
                report["parent_observation"] = {"present": False, **refusal(error, "parent")}
                write(f"fixture-snapshot/{label}.json", report)
                return report
            descriptors.append(fd)
        initial = basis(os.fstat(fd))
        names = sorted(os.listdir(fd))
        for index, name in enumerate(names):
            row = {"entry": index}
            child = None
            try:
                st = os.stat(name, dir_fd=fd, follow_symlinks=False)
                row["basis"] = basis(st)
                if not stat.S_ISDIR(st.st_mode):
                    row["scalar_observation"] = "UNAVAILABLE_NON_DIRECTORY; not opened"
                    report["entries"].append(row)
                    continue
                child = os.open(name, DIRECTORY, dir_fd=fd)
                assert basis(os.fstat(child)) == basis(st), "fixture directory changed"
                data, record_basis = read_regular(child, "custody.json", 65536)
                custody = json.loads(data)
                assert type(custody) is dict and set(custody) == {
                    "root", "device", "inode", "disposition", "semantic_world_identity_inferred"}
                assert custody["root"] == str(ROOT / "ProjectCentral/now/tmp/actuation-process-native" / name)
                assert type(custody["device"]) is int and type(custody["inode"]) is int
                assert (custody["device"], custody["inode"]) == (st.st_dev, st.st_ino)
                assert custody["disposition"] == "retained-before-effects"
                assert custody["semantic_world_identity_inferred"] is False
                row["custody"] = {"record_basis": record_basis, "sha256": digest(data),
                                  "device": custody["device"], "inode": custody["inode"],
                                  "disposition": custody["disposition"],
                                  "semantic_world_identity_inferred": False}
                try:
                    data, record_basis = read_regular(child, "actual-result.json", 65536)
                except FileNotFoundError as error:
                    row["scalar_observation"] = "NOT_EMITTED_OR_REMOVED; no values reconstructed"
                    row["result_read"] = refusal(error, "actual_result")
                else:
                    row["scalar_observation"] = "ACTUAL_RETAINED_TYPED_RECORD"
                    row["result_basis"] = record_basis
                    row["result_sha256"] = digest(data)
                    row["facts"] = scalar_record(json.loads(data))
                assert basis(os.fstat(child)) == basis(st), "fixture affiliation changed"
                assert basis(os.stat(name, dir_fd=fd, follow_symlinks=False)) == basis(st)
            except (OSError, ValueError, AssertionError) as error:
                row.pop("facts", None)
                row["scalar_observation"] = "UNAVAILABLE_CURRENT_READ_OR_PROFILE"
                row["refusal"] = refusal(error, "fixture")
            finally:
                if child is not None:
                    os.close(child)
            report["entries"].append(row)
        assert basis(os.fstat(fd)) == initial, "fixture parent membership/affiliation changed"
        report["parent_observation"] = {"present": True, "basis": initial,
                                        "same_final_membership": names == sorted(os.listdir(fd))}
        assert report["parent_observation"]["same_final_membership"]
    except (OSError, AssertionError) as error:
        report["parent_refusal"] = refusal(error, "parent")
    finally:
        for fd in reversed(descriptors):
            os.close(fd)
    write(f"fixture-snapshot/{label}.json", report)
    return report


def source_roster():
    cases = []
    for role, relative, module, count in MODULES:
        data = (ROOT / relative).read_text(encoding="utf-8")
        marker = f"mod {module} {{"
        assert data.count(marker) == 1, "selected native test module missing/ambiguous"
        suffix = data.split(marker, 1)[1]
        names = re.findall(r"#\[test\]\s*fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(", suffix)
        assert len(names) == count and len(set(names)) == count, "source roster changed"
        assert "#[ignore" not in suffix, "selected native tests must be default"
        owner = Path(relative).stem
        cases.extend({"role": role, "name": f"{owner}::{module}::{name}",
                      "source": relative} for name in names)
    assert len(cases) == 15
    write("source-native-roster.json", {"research": 14, "core": 1, "ignored": 0, "cases": cases})
    return cases


def run_raw(argv, prefix):
    started = time.monotonic()
    with (EVIDENCE / f"{prefix}.stdout").open("xb") as out, \
            (EVIDENCE / f"{prefix}.stderr").open("xb") as err:
        result = subprocess.run(argv, stdin=subprocess.DEVNULL, stdout=out, stderr=err, check=False)
    (EVIDENCE / f"{prefix}.status").write_text(str(result.returncode) + "\n")
    return result.returncode, time.monotonic() - started


def listing(path, prefix, ignored=False):
    argv = [str(path), "--list", "--format", "terse"]
    if ignored:
        argv.append("--ignored")
    code, elapsed = run_raw(argv, prefix)
    assert code == 0, "compiled list failed; not native execution success"
    text = (EVIDENCE / f"{prefix}.stdout").read_text(encoding="utf-8")
    return {line[:-6] for line in text.splitlines() if line.endswith(": test")}


def compiler_role(role, target):
    path = EVIDENCE / f"compile-{role}.stdout"
    matches = []
    for line in path.read_text(encoding="utf-8").splitlines():
        row = json.loads(line)
        if row.get("reason") == "compiler-artifact" and row["target"]["name"] == target \
                and row["target"]["kind"] == ["lib"] and row["profile"]["test"] \
                and row.get("executable"):
            matches.append(row)
    assert len(matches) == 1, "actual libtest CompilerArtifact missing/ambiguous"
    artifact = matches[0]
    path = Path(artifact["executable"])
    assert path.is_absolute() and path.resolve(strict=True).is_relative_to(ROOT / "target")
    fd = os.open(path, FLAGS | os.O_NONBLOCK)
    try:
        initial = basis(os.fstat(fd))
        assert stat.S_ISREG(initial["mode"]) and initial["mode"] & 0o111
        same(fd, path, initial)
        copy = EVIDENCE / "images" / f"{role}-libtest"
        copy.parent.mkdir(exist_ok=True)
        h = hashlib.sha256()
        started = time.monotonic()
        count = 0
        with copy.open("xb") as out:
            while True:
                data = os.read(fd, 1024 * 1024)
                if not data:
                    break
                count += len(data)
                h.update(data)
                out.write(data)
        SHA_COST["bytes"] += count
        SHA_COST["seconds"] += time.monotonic() - started
        assert count == initial["size"]
        same(fd, path, initial)
        os.chmod(copy, stat.S_IMODE(initial["mode"]))
        row = {"role": role, "compiler_artifact": artifact, "held_basis": initial,
               "sha256": h.hexdigest(), "copied_image": str(copy.relative_to(EVIDENCE)),
               "copied_bytes": count, "execution": "original mapped CompilerArtifact name; FD held throughout",
               "atomic_external_writer_exclusion": False}
        write(f"image-{role}.json", row)
        return fd, path, initial, row
    except BaseException:
        os.close(fd)
        raise


def final_source():
    original = json.loads((EVIDENCE / "source-custody.json").read_text(encoding="utf-8"))
    current_commit = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
    current_tree = subprocess.check_output(["git", "rev-parse", "HEAD^{tree}"], text=True).strip()
    tracked = subprocess.run(["git", "diff", "--quiet"], check=False).returncode
    index = subprocess.run(["git", "diff", "--cached", "--quiet"], check=False).returncode
    current_root = basis(os.stat(ROOT, follow_symlinks=False))
    record = {"actual_commit": current_commit, "actual_tree": current_tree,
              "lock_sha256": digest((ROOT / "Cargo.lock").read_bytes()),
              "tracked_diff_status": tracked, "index_diff_status": index,
              "same_root_affiliation": (current_root["device"], current_root["inode"]) ==
                  (original["root_basis"]["device"], original["root_basis"]["inode"])}
    write("source-final-custody.json", record)
    assert current_commit == original["actual_commit"] and current_tree == original["actual_tree"]
    assert record["lock_sha256"] == original["lock_sha256"] and record["same_root_affiliation"]
    assert tracked == 0 and index == 0, "tracked source/index changed during native qualification"


def execute():
    assert sys.platform in ("linux", "darwin") and os.geteuid() != 0, "native host/nonroot prerequisite"
    cases = source_roster()
    roles = {}
    results = []
    try:
        for role, target in (("research", "actuation_research"), ("core", "actuation_core")):
            roles[role] = compiler_role(role, target)
        for role, (fd, path, initial, image) in roles.items():
            same(fd, path, initial)
            compiled = listing(path, f"list-{role}")
            ignored = listing(path, f"list-{role}-ignored", True)
            prefixes = tuple(f"{Path(relative).stem}::{module}::"
                             for selected, relative, module, _ in MODULES if selected == role)
            selected = {name for name in compiled if name.startswith(prefixes)}
            expected = {row["name"] for row in cases if row["role"] == role}
            assert selected == expected and not (expected & ignored), "compiled native census mismatch"
            same(fd, path, initial)
            write(f"compiled-{role}-roster.json", {"selected": sorted(selected),
                  "expected": sorted(expected), "ignored_selected": sorted(expected & ignored),
                  "whole_compiled_count": len(compiled), "whole_ignored_count": len(ignored)})
        for index, case in enumerate(cases):
            fd, path, initial, image = roles[case["role"]]
            same(fd, path, initial)
            prefix = f"case-{index:02}"
            before = snapshot(prefix + "-before")
            code, elapsed = run_raw([str(path), case["name"], "--exact", "--nocapture",
                                     "--test-threads=1"], prefix)
            after = snapshot(prefix + "-after")
            same(fd, path, initial)
            raw = (EVIDENCE / f"{prefix}.stdout").read_text(encoding="utf-8")
            summaries = re.findall(r"^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;",
                                   raw, re.MULTILINE)
            body = re.findall(r"^test " + re.escape(case["name"]) + r" \.\.\. (ok|FAILED)$",
                              raw, re.MULTILINE)
            passed = code == 0 and summaries == [("ok", "1", "0", "0")] and body == ["ok"]
            prior = {(row.get("basis", {}).get("device"), row.get("basis", {}).get("inode"))
                     for row in before["entries"]}
            retained = [row for row in after["entries"]
                        if (row.get("basis", {}).get("device"), row.get("basis", {}).get("inode")) not in prior]
            record = {**case, "case_index": index, "actual_status": code,
                      "elapsed_seconds": elapsed, "actual_summary": summaries,
                      "actual_named_body": body, "exact_one_pass_no_failure_no_ignore": passed,
                      "image_sha256": image["sha256"], "same_final_named_and_held_basis": True,
                      "new_retained_fixture_observations": retained,
                      "scalar_custody_limit": "success-disposed actual-result files unavailable; no values inferred",
                      "outer_libtest_return_is_not_inner_retirement": True}
            results.append(record)
            write("native-case-results.json", {"required": 15, "executed": len(results),
                  "remaining_not_executed": [c["name"] for c in cases[len(results):]], "cases": results})
            assert passed, "actual native case failed/skipped/wrong census; raw failure retained, no retry"
        for role, (fd, path, initial, image) in roles.items():
            same(fd, path, initial)
            os.lseek(fd, 0, os.SEEK_SET)
            started = time.monotonic()
            h = hashlib.sha256()
            count = 0
            while True:
                data = os.read(fd, 1024 * 1024)
                if not data:
                    break
                h.update(data)
                count += len(data)
            SHA_COST["bytes"] += count
            SHA_COST["seconds"] += time.monotonic() - started
            same(fd, path, initial)
            assert h.hexdigest() == image["sha256"], "held image hash changed"
            write(f"image-{role}-final.json", {"same_held_and_named_basis": True,
                  "same_sha256": True, "sha256": h.hexdigest(), "bytes": count})
        final_source()
        # Historical filename retained for artifact consumers; contents qualify exact15, not13.
        write("native-13-qualified.json", {"executed": 15, "research": 14, "core": 1,
              "passed": 15, "failed": 0, "ignored": 0, "standing": "actual hosted native unit cases only",
              "required_case_count": 15, "original_cases": 13, "new_terminal_cases": 2,
              "historical_filename": "native-13-qualified.json; count is declared by contents",
              "full_scalar_observation_emission": False,
              "provider_model_installed_original_H_claim": False})
    finally:
        for fd, _, _, _ in roles.values():
            os.close(fd)
        write("sha2-case-cost.json", SHA_COST)


if MODE == "initial":
    tracked_clean = subprocess.run(["git", "diff", "--quiet"], check=False).returncode == 0
    index_clean = subprocess.run(["git", "diff", "--cached", "--quiet"], check=False).returncode == 0
    assert tracked_clean and index_clean, "dirty tracked checkout is not actual HEAD evidence"
    locked = (ROOT / "Cargo.lock").read_bytes()
    archive = EVIDENCE / "source.tar"
    archive_fd = os.open(archive, FLAGS)
    try:
        archive_basis = basis(os.fstat(archive_fd))
        assert stat.S_ISREG(archive_basis["mode"])
        h = hashlib.sha256()
        archive_count = 0
        started = time.monotonic()
        while True:
            data = os.read(archive_fd, 1024 * 1024)
            if not data:
                break
            archive_count += len(data)
            h.update(data)
        SHA_COST["bytes"] += archive_count
        SHA_COST["seconds"] += time.monotonic() - started
        same(archive_fd, archive, archive_basis)
        assert archive_count == archive_basis["size"]
        archive_sha = h.hexdigest()
    finally:
        os.close(archive_fd)
    write("source-custody.json", {
        "actual_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip(),
        "actual_tree": subprocess.check_output(["git", "rev-parse", "HEAD^{tree}"], text=True).strip(),
        "actual_parents": subprocess.check_output(["git", "show", "-s", "--format=%P", "HEAD"], text=True).split(),
        "github_sha": os.environ.get("GITHUB_SHA"),
        "requested_source": os.environ.get("ACTUATION_NATIVE_REQUESTED_SOURCE"),
        "lock_sha256": digest(locked), "source_archive_sha256": archive_sha,
        "source_archive_bytes": archive_count, "root_basis": basis(os.stat(ROOT, follow_symlinks=False)),
        "tracked_and_index_clean": True, "fixture_is_not_semantic_world_identity": True,
        "host_job_timeout_minutes": 30, "host_timeout_not_inner_retirement": True})
    (EVIDENCE / "Cargo.lock").write_bytes(locked)
    script = (ROOT / "scripts/migration/verify-native.sh").read_text(encoding="utf-8")
    boundaries = list(re.finditer(
        r"(?m)^# FINAL_IMAGE_ADMISSION: no later cargo/rustc invocation\.$", script))
    assert len(boundaries) == 1, "actual shell image-admission boundary missing/ambiguous"
    later = script[boundaries[0].end():]
    assert not re.search(r"(?m)^\s*(?:cargo|rustc|compile_role|run_gate)\b", later), \
        "owned script compiles after image admission"
    write("source-ordering-invariant.json", {"owned_post_boundary_compiler_calls": 0,
          "final_roles": ["actuation_research/lib/test", "actuation_core/lib/test"],
          "source_rule": "all quality, regressions, release certification precede final role admission",
          "external_concurrent_process_exclusion": False})
    source_roster()
    snapshot("initial")
    write("sha2-source-cost.json", SHA_COST)
elif MODE == "execute":
    execute()
elif MODE == "final":
    report = snapshot("final")
    assert "parent_refusal" not in report, "final fixture census unavailable; raw prior failure stays retained"
else:
    raise AssertionError("unknown evidence operation")
PY
finish_evidence() {
  original=$?
  trap - EXIT
  set +e
  python3 "$evidence/qualification-driver.py" "$evidence" final \
    > "$evidence/final-custody.stdout" 2> "$evidence/final-custody.stderr"
  custody=$?
  printf '%s\n' "$custody" > "$evidence/final-custody.status"
  printf '%s\n' "$original" > "$evidence/gate-primary.status"
  # Never replace an original quality/compiler/native failure with retention IO.
  effective="$original"
  if [[ "$original" -eq 0 && "$custody" -ne 0 ]]; then
    effective="$custody"
  fi
  printf '%s\n' "$effective" > "$evidence/gate.status"
  exit "$effective"
}
trap finish_evidence EXIT
run_gate() {
  local label="$1"
  shift
  set +e
  "$@" 2>&1 | tee "$evidence/$label.log"
  local outcomes=("${PIPESTATUS[@]}")
  set -e
  printf '%s\n' "${outcomes[0]}" > "$evidence/$label.status"
  printf '%s\n' "${outcomes[1]}" > "$evidence/$label-retention.status"
  if [[ "${outcomes[0]}" -ne 0 ]]; then
    return "${outcomes[0]}"
  fi
  return "${outcomes[1]}"
}
compile_role() {
  local role="$1" package="$2"
  set +e
  cargo test --locked -p "$package" --lib --no-run --message-format=json \
    > "$evidence/compile-$role.stdout" 2> "$evidence/compile-$role.stderr"
  local outcome=$?
  set -e
  printf '%s\n' "$outcome" > "$evidence/compile-$role.status"
  cat "$evidence/compile-$role.stderr"
  return "$outcome"
}
git rev-parse HEAD > "$evidence/source-commit.txt"
git write-tree > "$evidence/index-tree.txt"
git ls-files -s > "$evidence/source-modes.txt"
git archive --format=tar HEAD > "$evidence/source.tar"
rustc -Vv > "$evidence/rustc.txt"
cargo -V > "$evidence/cargo.txt"
python3 "$evidence/qualification-driver.py" "$evidence" initial
printf 'clean tracked/index; generated evidence and owned test scratch excluded\n' \
  > "$evidence/worktree-state.txt"
# Full standing mandatory gates all precede final role compilation/admission.
run_gate fmt cargo fmt --all -- --check
run_gate clippy cargo clippy --locked --workspace --all-targets -- -D warnings
run_gate tests cargo test --locked --workspace
# The default suite has genuine ignored QL cases: preserve its result without
# presenting two ignored bodies as executed native integration. The receipt
# bridge is a scripted protocol regression, separate from real QL/provider proof.
run_gate ql-agent-default cargo test --locked -p actuation-research --test ql_agent_native -- --nocapture
run_gate prime-faculty-scripted-receipt cargo test --locked -p actuation-research --test prime_faculty_receipt_bridge -- --nocapture
# Use only each original test's actual declared prerequisites. Missing optional
# owners are explicitly unqualified; partial configuration is a real refusal.
python3 - "$evidence" <<'QL_SCOPE'
import json, os
from pathlib import Path
import sys
root = Path(sys.argv[1])
cases = [
    ("ql-owner", "prime_faculty_uses_the_actual_event_and_harmonic_owner_without_a_model",
     ["QL_AGENT_TEST_BIN", "QL_AGENT_OWNER_REVISION", "QL_AGENT_EVENT_FIXTURE"]),
    ("ql-client-retirement", "prime_decision_timeout_reaps_the_real_separately_grouped_native_client",
     ["QL_AGENT_ADAPTER_ROOT", "QL_AGENT_PYTHON"]),
]
plan = []
for label, name, variables in cases:
    present = [key for key in variables if os.environ.get(key)]
    absent = [key for key in variables if not os.environ.get(key)]
    state = "SELECTED_UNEXECUTED" if not absent else (
        "PARTIAL_PREREQUISITES_REFUSED" if present else "NOT_EXECUTED_UNQUALIFIED")
    row = {"label": label, "case": name, "present_variables": present,
           "absent_variables": absent, "standing": state,
           "executed_total": 0, "pass_credit": 0,
           "source": "crates/actuation-research/tests/ql_agent_native.rs"}
    (root / (label + "-scope.json")).write_text(json.dumps(row, indent=2) + "\n")
    plan.append("\t".join([label, name, state]))
(root / "native-ql-plan.tsv").write_text("\n".join(plan) + "\n")
QL_SCOPE
while IFS=$'\t' read -r label case_name standing; do
  case "$standing" in
    NOT_EXECUTED_UNQUALIFIED) continue ;;
    PARTIAL_PREREQUISITES_REFUSED)
      printf '%s\n' "Actual QL prerequisite set is partial; inspect $evidence/$label-scope.json" >&2
      exit 1 ;;
    SELECTED_UNEXECUTED) ;;
    *) printf '%s\n' 'Unknown native QL prerequisite standing' >&2; exit 1 ;;
  esac
  # Parse and retain every actual exit, including a real Cargo/test failure.
  if run_gate "$label" cargo test --locked -p actuation-research --test ql_agent_native "$case_name" -- --ignored --exact --nocapture --test-threads=1; then :; else :; fi
  python3 - "$evidence" "$label" "$case_name" <<'QL_RESULT'
import hashlib, json, re
from pathlib import Path
import sys
root = Path(sys.argv[1]); label = sys.argv[2]; case = sys.argv[3]
# Existing run_gate retains one combined log, the command exit and tee exit.
log = (root / (label + ".log")).read_bytes()
try:
    body = log.decode("utf-8")
    decode_failure = None
except UnicodeDecodeError:
    body = ""
    decode_failure = "UnicodeDecodeError"
status = int((root / (label + ".status")).read_text())
retention_status = int((root / (label + "-retention.status")).read_text())
census = re.findall(r"^test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", body, re.M)
# Serial libtest prints the selected name before the body. Actual body stdout
# or inherited child stderr may split its eventual result; retain that log.
# --ignored --exact selects one original case; the actual exit and one-pass
# census qualify its result, while this original start prefix binds its name.
named = re.findall(r"^test " + re.escape(case) + r" \.\.\.", body, re.M)
scope = json.loads((root / (label + "-scope.json")).read_text())
admitted = decode_failure is None and status == 0 and retention_status == 0 and census == [("1", "0", "0")] and len(named) == 1
scope.update({"actual_exit": status, "log_decode_failure": decode_failure, "actual_log_retention_exit": retention_status, "actual_census": census,
              "actual_named_start_count": len(named),
              "combined_log_sha256": hashlib.sha256(log).hexdigest(),
              "standing": "ACTUAL_DECLARED_NATIVE_CASE_PASSED" if admitted else "ACTUAL_CASE_UNQUALIFIED",
              "command_attempted": True, "executed_total": 1 if len(named) == 1 else 0,
              "pass_credit": 1 if admitted else 0,
              "limit": "Exact original configured body; one selected named start plus actual zero exits and one-pass census; not independent installed owner-image or model/H acceptance"})
(root / (label + "-scope.json")).write_text(json.dumps(scope, indent=2) + "\n")
assert admitted, "Actual configured native QL case was not one pass/zero failed/zero ignored"
QL_RESULT
done < "$evidence/native-ql-plan.tsv"
run_gate examples-bins cargo build --locked --examples --bins
run_gate release-build cargo build --locked --release -p actuation-cli
# Preserve actual JSON stdout separately from actual stderr and exit status.
set +e
./target/release/actuation verify --json > "$evidence/R7-native-verify.json" \
  2> "$evidence/R7-native-verify.stderr"
verification=$?
set -e
printf '%s\n' "$verification" > "$evidence/R7-native-verify.status"
cat "$evidence/R7-native-verify.json"
cat "$evidence/R7-native-verify.stderr" >&2
if [[ "$verification" -ne 0 ]]; then
  exit "$verification"
fi
# Final compiler operations. Neither this script nor its evidence driver compiles
# anything after the image boundary. Images are admitted only after both builds.
compile_role research actuation-research
compile_role core actuation-core
# FINAL_IMAGE_ADMISSION: no later cargo/rustc invocation.
python3 "$evidence/qualification-driver.py" "$evidence" execute
