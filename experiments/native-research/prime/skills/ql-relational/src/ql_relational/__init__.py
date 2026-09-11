"""Prime's Python import ABI over native Actuation. No formal algebra, source
search, compiler invocation, comparison or evidence-store implementation lives here."""
from __future__ import annotations

import asyncio
import json
import os
from pathlib import Path
import subprocess
import tempfile
from typing import Any

RETURN_SCHEMA = "actuation.prime-return/v0"
_LIMIT = 16 * 1024 * 1024


def _invoke(value: dict[str, Any]) -> dict[str, Any]:
    binary = Path(os.environ.get("ACTUATION_RESEARCH_BIN", ""))
    if not binary.is_absolute() or not binary.is_file():
        raise RuntimeError("ACTUATION_RESEARCH_BIN must name the installed native executable")
    data = json.dumps(value, ensure_ascii=False).encode("utf-8")
    if len(data) > _LIMIT:
        raise ValueError("native faculty request exceeds transport bound")
    # File-backed transport prevents an unbounded stdout allocation. The native
    # executable owns operation bounds; this is only its installed-language ABI.
    with tempfile.TemporaryFile() as out, tempfile.TemporaryFile() as err:
        try:
            result = subprocess.run([str(binary)], input=data, stdout=out, stderr=err,
                                    env={}, timeout=60, check=False)
        except subprocess.TimeoutExpired:
            raise RuntimeError("native faculty transport timed out") from None
        out.seek(0)
        raw = out.read(_LIMIT + 1)
        if len(raw) > _LIMIT:
            raise RuntimeError("native faculty response exceeds transport bound")
        try:
            response = json.loads(raw)
        except (ValueError, UnicodeError):
            raise RuntimeError("native faculty returned invalid JSON") from None
        if result.returncode != 0:
            raise RuntimeError("native faculty operation failed; no Python fallback")
        return response


async def _faculty(operation: str, **arguments: Any) -> dict[str, Any]:
    configuration = os.environ.get("ACTUATION_RESEARCH_FACULTY_CONFIG")
    if not configuration:
        raise RuntimeError("ACTUATION_RESEARCH_FACULTY_CONFIG is required")
    response = await asyncio.to_thread(_invoke, {
        "operation": "faculty.invoke", "configuration": configuration,
        "trace_ref": os.environ.get("ACTUATION_RESEARCH_TRACE_REF"),
        "declared_locus_ref": os.environ.get("ACTUATION_RESEARCH_LOCUS_REF"),
        "request": {"operation": operation, **arguments},
    })
    if response.get("schema") != "actuation.prime-faculty-result/v1" or response.get("success") is not True:
        raise RuntimeError("native faculty refused operation; consult retained native evidence")
    return response["result"]


async def capabilities() -> dict[str, Any]:
    return await _faculty("capabilities")


async def kernel_apply(operator: str, address: str) -> dict[str, Any]:
    return await _faculty("kernel-apply", operator=operator, address=address)


async def mef_lenses() -> dict[str, Any]:
    return await _faculty("mef-lenses")


async def context_frames() -> dict[str, Any]:
    return await _faculty("context-frames")


async def vak_locate(vak_ref: str) -> dict[str, Any]:
    return await _faculty("vak-locate", vak_ref=vak_ref)


async def negotiate(operation: str) -> dict[str, Any]:
    return await _faculty("negotiate", service_operation=operation)


async def wiki_refract(request: dict[str, Any]) -> dict[str, Any]:
    return await _faculty("wiki-refract", request=request)


async def source_state() -> dict[str, Any]:
    return await _faculty("source-state")


async def constellation_contract() -> dict[str, Any]:
    return await _faculty("constellation-contract")


async def harmonic_search(query: str, max_matches: int = 8) -> dict[str, Any]:
    return await _faculty("harmonic-search", query=query, max_matches=max_matches)


async def harmonic_snapshot(basis: str = "chromatic") -> dict[str, Any]:
    return await _faculty("harmonic-snapshot", basis=basis)


def return_envelope(subject_ref: str, relation_to_parent: str, determination: str,
                    result: str, difference: str, evidence_refs: list[str] | None = None,
                    ql_reading_refs: list[str] | None = None, unresolved: list[str] | None = None,
                    next_relations: list[str] | None = None, child_ref: str | None = None,
                    parent_ref: str | None = None, provenance: dict[str, Any] | None = None) -> dict[str, Any]:
    """Marshal the existing synchronous helper to Rust's Return constructor."""
    value = {"schema": RETURN_SCHEMA, "subject_ref": subject_ref,
             "relation_to_parent": relation_to_parent, "determination": determination,
             "result": result, "difference": difference, "evidence_refs": evidence_refs or [],
             "ql_reading_refs": ql_reading_refs or [], "unresolved": unresolved or [],
             "next_relations": next_relations or [], "provenance": provenance or {}}
    if child_ref is not None:
        value["child_ref"] = child_ref
    if parent_ref is not None:
        value["parent_ref"] = parent_ref
    return _invoke({"operation": "prime.return", "return": value, "require_schema": True})
