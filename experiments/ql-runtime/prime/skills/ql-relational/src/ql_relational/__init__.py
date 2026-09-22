from __future__ import annotations

import asyncio
import hashlib
import json
import os
import tempfile
from pathlib import Path
from typing import Any

RETURN_SCHEMA = "actuation.prime-return/v0"
FACULTY_RESULT_SCHEMA = "actuation.prime-faculty-result/v1"


def _root() -> Path:
    value = os.environ.get("QL_MEF_ROOT")
    if not value:
        raise RuntimeError("QL_MEF_ROOT is required for ql_relational.")
    root = Path(value).expanduser().resolve()
    if not (root / "Cargo.toml").exists():
        raise RuntimeError(f"QL_MEF_ROOT does not contain Cargo.toml: {root}")
    return root


async def _run(*args: str, stdin: str | None = None, cwd: Path | None = None) -> dict[str, Any]:
    proc = await asyncio.create_subprocess_exec(
        *args,
        cwd=str(cwd) if cwd is not None else None,
        stdin=asyncio.subprocess.PIPE if stdin is not None else asyncio.subprocess.DEVNULL,
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE,
    )
    stdout, stderr = await proc.communicate(None if stdin is None else stdin.encode())
    if proc.returncode != 0:
        raise RuntimeError(f"{' '.join(args)} failed ({proc.returncode}): {stderr.decode().strip()}")
    text = stdout.decode()
    try:
        return json.loads(text)
    except json.JSONDecodeError:
        return {"stdout": text, "stderr": stderr.decode(), "exit_code": proc.returncode}


async def _git_revision() -> str:
    declared = os.environ.get("QL_OWNER_REVISION", "").strip()
    if declared:
        if len(declared) != 40 or any(ch not in "0123456789abcdef" for ch in declared):
            raise RuntimeError("QL_OWNER_REVISION must be a lowercase 40-hex revision")
        return declared
    result = await _run("git", "rev-parse", "HEAD", cwd=_root())
    return str(result.get("stdout", "")).strip()


def _digest(value: Any) -> str:
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":"), default=str).encode()
    return hashlib.sha256(encoded).hexdigest()


async def _record(operation: str, request: Any, response: Any) -> None:
    target = os.environ.get("QL_RELATIONAL_EVIDENCE_LOG")
    if not target:
        return
    revision = await _git_revision()
    row = {
        "schema": "actuation.prime-ql-operation/v1",
        "operation": operation,
        "ql_mef_revision": revision,
        "request_digest": _digest(request),
        "response_digest": _digest(response),
        "harmonic_enabled": os.environ.get("QL_PRIME_HARMONIC") == "1",
    }
    path = Path(target)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(row, sort_keys=True) + "\n")


def _runtime_locus_ref() -> str | None:
    """Attribute a faculty call to Prime's current runtime locus without leaking paths.

    Prime's host supplies every descendant with RLM_DEPTH and its own
    RLM_SESSION_DIR.  The directory is material execution evidence but may
    contain a private/local path, so receipts carry only its SHA-256.  Root
    calls retain the Actuation-supplied locus ref.
    """
    declared = os.environ.get("ACTUATION_RESEARCH_LOCUS_REF")
    depth_raw = os.environ.get("RLM_DEPTH", "").strip()
    try:
        depth = int(depth_raw) if depth_raw else 0
    except ValueError as exc:
        raise RuntimeError("Prime supplied a non-integer RLM_DEPTH") from exc
    if depth <= 0:
        return declared
    session_dir = os.environ.get("RLM_SESSION_DIR", "").strip()
    if not session_dir:
        return f"prime-rlm-depth:{depth}:session-dir-unobserved"
    digest = hashlib.sha256(session_dir.encode("utf-8")).hexdigest()
    return f"prime-rlm-session-sha256:{digest}:depth:{depth}"


async def _receipt(native_request: dict[str, Any], response: Any) -> None:
    """File the native Actuation faculty receipt for one executed operation.

    When the research binary is available (ACTUATION_RESEARCH_BIN), the same
    operation is replayed through the engine's `faculty.invoke` so a
    content-addressed native receipt lands in the faculty configuration's
    evidence World — the receipts the experiment driver's claims collector
    reads. The request shape mirrors crates/actuation-research/src/faculty.rs
    exactly. The JSONL evidence log above is independent and stays. A
    success:false envelope still carries its receipt (the native refusal is
    itself evidence); only a missing or malformed receipt is a bridge failure.
    """
    research_bin = os.environ.get("ACTUATION_RESEARCH_BIN")
    if not research_bin:
        return
    configuration = os.environ.get("ACTUATION_RESEARCH_FACULTY_CONFIG")
    if not configuration:
        raise RuntimeError(
            "ACTUATION_RESEARCH_FACULTY_CONFIG is required to file native faculty receipts"
        )
    payload = {
        "operation": "faculty.invoke",
        "configuration": configuration,
        "request": native_request,
        "trace_ref": os.environ.get("ACTUATION_RESEARCH_TRACE_REF"),
        "declared_locus_ref": _runtime_locus_ref(),
    }
    proc = await asyncio.create_subprocess_exec(
        research_bin,
        stdin=asyncio.subprocess.PIPE,
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE,
    )
    stdout, stderr = await proc.communicate(json.dumps(payload).encode())
    if proc.returncode != 0:
        raise RuntimeError(
            f"native faculty receipt filing failed ({proc.returncode}): "
            f"{stderr.decode(errors='replace').strip()}"
        )
    try:
        result = json.loads(stdout.decode())
    except json.JSONDecodeError as e:
        raise RuntimeError("native faculty receipt reply is not JSON") from e
    if result.get("schema") != FACULTY_RESULT_SCHEMA or not isinstance(
        result.get("receipt"), dict
    ):
        raise RuntimeError(f"native faculty receipt is missing: {result}")


async def _ql(*args: str) -> dict[str, Any]:
    request = list(args)
    binary = os.environ.get("QL_BIN", "").strip()
    if binary:
        ql = Path(binary).expanduser().resolve()
        if not ql.is_file():
            raise RuntimeError(f"QL_BIN is not an installed executable file: {ql}")
        response = await _run(str(ql), *args, "--json")
    else:
        response = await _run(
            "cargo", "run", "--quiet", "--manifest-path", str(_root() / "Cargo.toml"),
            "-p", "ql-cli", "--", *args, "--json", cwd=_root()
        )
    await _record("ql-cli:" + ".".join(args[:2]), request, response)
    return response


async def capabilities() -> dict[str, Any]:
    """Return accepted QL/MEF CLI, kernel, MEF, Context Frame, VĀK and service capability disclosure."""
    result = await _ql("capabilities")
    await _receipt({"operation": "capabilities"}, result)
    return result


async def kernel_apply(operator: str, address: str) -> dict[str, Any]:
    """Apply one accepted deterministic QL kernel operator to one QL address."""
    result = await _ql("kernel", "apply", operator, address)
    await _receipt(
        {"operation": "kernel-apply", "operator": operator, "address": address}, result
    )
    return result


async def mef_lenses() -> dict[str, Any]:
    """Return the source-locked twelve-lens MEF registry."""
    result = await _ql("mef", "lenses")
    await _receipt({"operation": "mef-lenses"}, result)
    return result


async def context_frames() -> dict[str, Any]:
    """Return the accepted Context Frame registry."""
    result = await _ql("context-frame", "list")
    await _receipt({"operation": "context-frames"}, result)
    return result


async def vak_locate(vak_ref: str) -> dict[str, Any]:
    """Locate one source-backed VĀK entry."""
    result = await _ql("vak", "locate", vak_ref)
    await _receipt({"operation": "vak-locate", "vak_ref": vak_ref}, result)
    return result


async def negotiate(operation: str) -> dict[str, Any]:
    """Negotiate capabilities|locate|refract|relate|synthesise against the current QL service."""
    result = await _ql("service", "negotiate", operation)
    await _receipt({"operation": "negotiate", "service_operation": operation}, result)
    return result


async def wiki_refract(request: dict[str, Any]) -> dict[str, Any]:
    """Run the native ql-mef/wiki-refraction/v1 engine over a caller-owned Wiki target."""
    payload = json.dumps(request)
    response = await _run(
        "cargo", "run", "--quiet", "--manifest-path", str(_root() / "Cargo.toml"),
        "-p", "ql-wiki", "--bin", "ql-wiki-refraction",
        stdin=payload,
        cwd=_root(),
    )
    await _record("ql-wiki:refract", request, response)
    await _receipt({"operation": "wiki-refract", "request": request}, response)
    return response


async def source_state() -> dict[str, Any]:
    """Return exact QL-MEF revision and whether this run admits the source-locked harmonic development head."""
    revision = await _git_revision()
    result = {
        "ql_mef_root": str(_root()),
        "revision": revision,
        "harmonic_enabled": os.environ.get("QL_PRIME_HARMONIC") == "1",
    }
    await _record("source-state", {}, result)
    await _receipt({"operation": "source-state"}, result)
    return result


async def constellation_contract() -> dict[str, Any]:
    """Return the normative Wiki structural/constellation/Return contract with exact source revision."""
    path = _root() / "docs" / "wiki-structural-contract-v2.md"
    text = path.read_text(encoding="utf-8")
    result = {"path": str(path.relative_to(_root())), "revision": await _git_revision(), "content": text}
    logged = {"path": result["path"], "revision": result["revision"], "content_digest": _digest(text)}
    await _record("constellation-contract", {}, logged)
    await _receipt({"operation": "constellation-contract"}, logged)
    return result


async def harmonic_search(query: str, max_matches: int = 8) -> dict[str, Any]:
    """Search source-locked harmonic derivation and, when enabled, the executable #81 development carrier."""
    if not query or max_matches < 1:
        raise ValueError("query must be non-empty and max_matches positive")
    candidates = [
        _root() / "docs" / "sources" / "ql-musical-derivation-v3.md",
        _root() / "docs" / "music" / "PRE-M-MUSICAL-DERIVATION-v1.md",
    ]
    if os.environ.get("QL_PRIME_HARMONIC") == "1":
        candidates.append(_root() / "crates" / "ql-mef" / "src" / "music.rs")

    needle = query.casefold()
    matches: list[dict[str, Any]] = []
    for path in candidates:
        if not path.exists():
            continue
        lines = path.read_text(encoding="utf-8").splitlines()
        for index, line in enumerate(lines, start=1):
            if needle in line.casefold():
                start = max(0, index - 2)
                end = min(len(lines), index + 1)
                matches.append({
                    "path": str(path.relative_to(_root())),
                    "line": index,
                    "excerpt": "\n".join(lines[start:end]),
                })
                if len(matches) >= max_matches:
                    break
        if len(matches) >= max_matches:
            break

    result = {
        "query": query,
        "revision": await _git_revision(),
        "harmonic_enabled": os.environ.get("QL_PRIME_HARMONIC") == "1",
        "matches": matches,
    }
    await _record("harmonic-search", {"query": query, "max_matches": max_matches}, result)
    await _receipt(
        {"operation": "harmonic-search", "query": query, "max_matches": max_matches},
        result,
    )
    return result


async def harmonic_snapshot(basis: str = "chromatic") -> dict[str, Any]:
    """Execute the accepted-main pre-M harmonic derivation and return a compact numeric relational snapshot."""
    if os.environ.get("QL_PRIME_HARMONIC") != "1":
        raise RuntimeError("harmonic_snapshot requires QL_PRIME_HARMONIC=1 and the source-locked QL-MEF main checkout.")
    if basis not in {"chromatic", "fifths"}:
        raise ValueError("basis must be 'chromatic' or 'fifths'")

    source_lock_path = os.environ.get("QL_PRIME_SOURCE_LOCK")
    if source_lock_path:
        lock = json.loads(Path(source_lock_path).read_text(encoding="utf-8"))
        expected = lock["ql_mef"]["accepted_main_revision"]
        observed = await _git_revision()
        if observed != expected:
            raise RuntimeError(f"harmonic_snapshot source drift: expected {expected}, observed {observed}")

    rust = r"""
use ql_core::{QlFace, RelationFamily};
use ql_mef::{MusicalBasis, CrossOperator, derive_pre_m_music, pair_interval_deltas, cross_interval_deltas};

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_else(|| "chromatic".into());
    let basis = if arg == "fifths" { MusicalBasis::Fifths } else { MusicalBasis::Chromatic };
    let label = if arg == "fifths" { "fifths" } else { "chromatic" };
    let d = derive_pre_m_music(basis);
    let anchors: Vec<u8> = d.lens_anchors.iter().map(|a| a.pitch).collect();
    let a = pair_interval_deltas(basis, RelationFamily::A, QlFace::Direct);
    let b = pair_interval_deltas(basis, RelationFamily::B, QlFace::Direct);
    let c = pair_interval_deltas(basis, RelationFamily::C, QlFace::Direct);
    let d1 = cross_interval_deltas(basis, CrossOperator::SamePosition);
    let d2t = cross_interval_deltas(basis, CrossOperator::Transform);
    let d2r = cross_interval_deltas(basis, CrossOperator::Require);
    let d2c = cross_interval_deltas(basis, CrossOperator::Complete);
    println!(
        "{{\"basis\":\"{}\",\"direct_helix\":{:?},\"conjugate_helix\":{:?},\"lens_anchor_pitches\":{:?},\"A_direct_deltas\":{:?},\"B_direct_deltas\":{:?},\"C_direct_deltas\":{:?},\"D1_cross_deltas\":{:?},\"D2_transform_deltas\":{:?},\"D2_require_deltas\":{:?},\"D2_complete_deltas\":{:?},\"mode_tonic_count\":{}}}",
        label, d.direct_helix, d.conjugate_helix, anchors, a, b, c, d1, d2t, d2r, d2c, d.mode_tonic_landscape.len()
    );
}
"""
    with tempfile.TemporaryDirectory(prefix="ql-prime-harmonic-") as directory:
        probe = Path(directory)
        ql_path = str((_root() / "crates" / "ql-mef").resolve()).replace("\\", "\\\\")
        core_path = str((_root() / "crates" / "ql-core").resolve()).replace("\\", "\\\\")
        (probe / "src").mkdir()
        (probe / "Cargo.toml").write_text(
            "[package]\nname='ql-prime-harmonic-probe'\nversion='0.0.0'\nedition='2024'\n"
            f"[dependencies]\nql-mef={{path=\"{ql_path}\"}}\nql-core={{path=\"{core_path}\"}}\n",
            encoding="utf-8",
        )
        (probe / "src" / "main.rs").write_text(rust, encoding="utf-8")
        proc = await asyncio.create_subprocess_exec(
            "cargo", "run", "--quiet", "--manifest-path", str(probe / "Cargo.toml"), "--", basis,
            cwd=str(_root()),
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
        )
        stdout, stderr = await proc.communicate()
        if proc.returncode != 0:
            raise RuntimeError(f"harmonic probe failed ({proc.returncode}): {stderr.decode().strip()}")
        result = json.loads(stdout.decode())

    result["revision"] = await _git_revision()
    result["standing"] = "accepted-main"
    await _record("harmonic-snapshot", {"basis": basis}, result)
    await _receipt({"operation": "harmonic-snapshot", "basis": basis}, result)
    return result


async def epi_constitution() -> dict[str, Any]:
    """Return the native Epi-Logos Prime-QL constitution and #0..#5 faculty disclosure."""
    result = await _ql("epi-agent", "constitution")
    await _receipt({"operation": "epi-constitution"}, result)
    return result


async def epi_faculty(position: int) -> dict[str, Any]:
    """Return one source-qualified Epi faculty descriptor (#0..#5)."""
    if position not in range(6):
        raise ValueError("position must be 0..5")
    result = await _ql("epi-agent", "faculty", f"#{position}")
    await _receipt({"operation": "epi-faculty", "position": f"#{position}"}, result)
    return result


async def _epi_invoke(position: int, operation: str, input_value: dict[str, Any]) -> dict[str, Any]:
    if position not in range(6):
        raise ValueError("position must be 0..5")
    envelope = {
        "schema": "ql.epi-logos-agent-invocation/v1",
        "position": f"#{position}",
        "operation": operation,
        "input": input_value,
    }
    with tempfile.NamedTemporaryFile("w", suffix=".json", encoding="utf-8", delete=False) as handle:
        json.dump(envelope, handle)
        path = handle.name
    try:
        result = await _ql("epi-agent", "invoke", path)
    finally:
        Path(path).unlink(missing_ok=True)
    return result


async def anuttara_read(reference: str, max_relations: int = 128) -> dict[str, Any]:
    """Read one full Anuttara language row joined to current Bimba relations."""
    result = await _epi_invoke(0, "anuttara.read", {"reference": reference, "max_relations": max_relations})
    await _receipt({"operation": "anuttara-read", "reference": reference, "max_relations": max_relations}, result)
    return result


async def tda_vietoris_rips(request: dict[str, Any]) -> dict[str, Any]:
    """Run deterministic source-qualified Vietoris-Rips persistent H0/H1 over an explicit metric."""
    result = await _epi_invoke(1, "tda.vietoris-rips", request)
    await _receipt({"operation": "tda-vietoris-rips", "request": request}, result)
    return result


async def bimba_neighborhood(reference: str, max_relations: int = 256) -> dict[str, Any]:
    """Read exact source-graph adjacency without substituting GDS or learned inference."""
    result = await _epi_invoke(2, "bimba.neighborhood", {"reference": reference, "max_relations": max_relations})
    await _receipt({"operation": "bimba-neighborhood", "reference": reference, "max_relations": max_relations}, result)
    return result


async def representation_bind(request: dict[str, Any]) -> dict[str, Any]:
    """Bind source/form/representation/asset/temporal provenance for a Mahamaya representation."""
    result = await _epi_invoke(3, "representation.bind", request)
    await _receipt({"operation": "representation-bind", "request": request}, result)
    return result


async def nara_activity_validate(activity: dict[str, Any]) -> dict[str, Any]:
    """Validate protected Nara activity spans, provenance and protection semantics."""
    result = await _epi_invoke(4, "nara.activity.validate", {"activity": activity})
    await _receipt({"operation": "nara-activity-validate", "activity": activity}, result)
    return result


async def nara_elemental_map(request: dict[str, Any]) -> dict[str, Any]:
    """Map typed EFWA contributions to the native quaternion while retaining confidence separately."""
    result = await _epi_invoke(4, "nara.elemental-map", request)
    await _receipt({"operation": "nara-elemental-map", "request": request}, result)
    return result


async def nara_personal_receive(request: dict[str, Any]) -> dict[str, Any]:
    """Receive one exact coupled M1/M2/M3 event through the native Nara personal field."""
    result = await _epi_invoke(4, "nara.personal-receive", request)
    await _receipt({"operation": "nara-personal-receive", "request": request}, result)
    return result


async def logos_return(request: dict[str, Any]) -> dict[str, Any]:
    """Form the complete T/C/T-prime/C-prime Epii Return envelope without promoting it."""
    result = await _epi_invoke(5, "logos.return", request)
    await _receipt({"operation": "logos-return", "request": request}, result)
    return result


def return_envelope(
    subject_ref: str,
    relation_to_parent: str,
    determination: str,
    result: str,
    difference: str,
    evidence_refs: list[str] | None = None,
    ql_reading_refs: list[str] | None = None,
    unresolved: list[str] | None = None,
    next_relations: list[str] | None = None,
    child_ref: str | None = None,
    parent_ref: str | None = None,
    provenance: dict[str, Any] | None = None,
) -> dict[str, Any]:
    """Construct the experimental Actuation Prime Return envelope."""
    for name, value in {
        "subject_ref": subject_ref,
        "relation_to_parent": relation_to_parent,
        "determination": determination,
        "result": result,
        "difference": difference,
    }.items():
        if not isinstance(value, str) or not value.strip():
            raise ValueError(f"{name} must be a non-empty string")
    value: dict[str, Any] = {
        "schema": RETURN_SCHEMA,
        "subject_ref": subject_ref,
        "relation_to_parent": relation_to_parent,
        "determination": determination,
        "result": result,
        "difference": difference,
        "evidence_refs": list(evidence_refs or []),
        "ql_reading_refs": list(ql_reading_refs or []),
        "unresolved": list(unresolved or []),
        "next_relations": list(next_relations or []),
        "provenance": dict(provenance or {}),
    }
    if child_ref:
        value["child_ref"] = child_ref
    if parent_ref:
        value["parent_ref"] = parent_ref
    return value
