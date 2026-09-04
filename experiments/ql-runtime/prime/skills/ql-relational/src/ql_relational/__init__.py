from __future__ import annotations

import asyncio
import hashlib
import json
import os
from pathlib import Path
from typing import Any

RETURN_SCHEMA = "actuation.prime-return/v0"


def _root() -> Path:
    value = os.environ.get("QL_MEF_ROOT")
    if not value:
        raise RuntimeError("QL_MEF_ROOT is required for ql_relational.")
    root = Path(value).expanduser().resolve()
    if not (root / "Cargo.toml").exists():
        raise RuntimeError(f"QL_MEF_ROOT does not contain Cargo.toml: {root}")
    return root


async def _run(*args: str, stdin: str | None = None) -> dict[str, Any]:
    proc = await asyncio.create_subprocess_exec(
        *args,
        cwd=str(_root()),
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
    result = await _run("git", "rev-parse", "HEAD")
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


async def _ql(*args: str) -> dict[str, Any]:
    request = list(args)
    response = await _run(
        "cargo", "run", "--quiet", "--manifest-path", str(_root() / "Cargo.toml"),
        "-p", "ql-cli", "--", *args, "--json"
    )
    await _record("ql-cli:" + ".".join(args[:2]), request, response)
    return response


async def capabilities() -> dict[str, Any]:
    """Return accepted QL/MEF CLI, kernel, MEF, Context Frame, VĀK and service capability disclosure."""
    return await _ql("capabilities")


async def kernel_apply(operator: str, address: str) -> dict[str, Any]:
    """Apply one accepted deterministic QL kernel operator to one QL address."""
    return await _ql("kernel", "apply", operator, address)


async def mef_lenses() -> dict[str, Any]:
    """Return the source-locked twelve-lens MEF registry."""
    return await _ql("mef", "lenses")


async def context_frames() -> dict[str, Any]:
    """Return the accepted Context Frame registry."""
    return await _ql("context-frame", "list")


async def vak_locate(vak_ref: str) -> dict[str, Any]:
    """Locate one source-backed VĀK entry."""
    return await _ql("vak", "locate", vak_ref)


async def negotiate(operation: str) -> dict[str, Any]:
    """Negotiate capabilities|locate|refract|relate|synthesise against the current QL service."""
    return await _ql("service", "negotiate", operation)


async def wiki_refract(request: dict[str, Any]) -> dict[str, Any]:
    """Run the native ql-mef/wiki-refraction/v1 engine over a caller-owned Wiki target."""
    payload = json.dumps(request)
    response = await _run(
        "cargo", "run", "--quiet", "--manifest-path", str(_root() / "Cargo.toml"),
        "-p", "ql-wiki", "--bin", "ql-wiki-refraction",
        stdin=payload,
    )
    await _record("ql-wiki:refract", request, response)
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
    return result


async def constellation_contract() -> dict[str, Any]:
    """Return the normative Wiki structural/constellation/Return contract with exact source revision."""
    path = _root() / "docs" / "wiki-structural-contract-v2.md"
    text = path.read_text(encoding="utf-8")
    result = {"path": str(path.relative_to(_root())), "revision": await _git_revision(), "content": text}
    await _record("constellation-contract", {}, {"path": result["path"], "revision": result["revision"], "content_digest": _digest(text)})
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
