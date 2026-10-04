#!/usr/bin/env python3
"""Seven real old/current Central + SDK cases, requiring an admitted outer owner.

Run through the existing strict, finite native capture owner. The SDK's inherited
communicate() has no cancellation/retirement contract: wait_for below detects a
failed deadline, and does not certify child retirement. Every fixture is kept
before operations, including on failure; this script never sweeps native roots.
Compiler/source/image manifests are host evidence, not semantic owner identity.
"""
from __future__ import annotations

import argparse
import asyncio
import errno
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import stat
import tempfile
import time
from typing import Any

LEGACY_SOURCE = "5e4510a6bd61d6e151c84d755f884b61b693db27"
CASES = (
    "root_roundtrip_current_native_source",
    "project_member_and_old_owner_compatibility",
    "exact_selected_source_preserves_sibling",
    "current_owner_withdrawal_and_unsafe_forms",
    "actual_nonroot_eacces_then_reopen",
    "old_root_without_route_is_unavailable_not_resubmitted",
    "real_selected_malformed_and_budget_material",
)


def basis(path: Path) -> tuple[int, int, int, int, int, int, int]:
    value = path.lstat()
    return (value.st_dev, value.st_ino, value.st_mode, value.st_nlink,
            value.st_size, value.st_mtime_ns, value.st_ctime_ns)


def ordinary_bytes(path: Path, limit: int, *, single_link: bool = True) -> bytes:
    """No-follow held/named ordinary-file observation, not another native reader."""
    before = basis(path)
    if not stat.S_ISREG(before[2]) or before[3] < 1 or (single_link and before[3] != 1):
        raise RuntimeError("required test evidence is not an ordinary single-link file")
    descriptor = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK | os.O_CLOEXEC)
    try:
        value = os.fstat(descriptor)
        held = (value.st_dev, value.st_ino, value.st_mode, value.st_nlink,
                value.st_size, value.st_mtime_ns, value.st_ctime_ns)
        if held != before or value.st_size > limit:
            raise RuntimeError("required test evidence changed or exceeds its finite profile")
        with os.fdopen(descriptor, "rb", closefd=False) as handle:
            body = handle.read(limit + 1)
        after = os.fstat(descriptor)
        after_basis = (after.st_dev, after.st_ino, after.st_mode, after.st_nlink,
                       after.st_size, after.st_mtime_ns, after.st_ctime_ns)
        if len(body) > limit or after_basis != held or basis(path) != held:
            raise RuntimeError("required test evidence changed during observation")
        return body
    finally:
        os.close(descriptor)


def build_image(path: Path, role: str) -> tuple[Path, dict[str, Any]]:
    manifest = json.loads(ordinary_bytes(path, 1024 * 1024))
    if not isinstance(manifest, dict) or manifest.get("role") != role:
        raise RuntimeError("actual compiled-image provenance is required for each role")
    image = Path(manifest["executable"])
    if not image.is_absolute() or image.resolve(strict=True) != image:
        raise RuntimeError("compiled image requires its actual ordinary absolute coordinate")
    artifact = manifest.get("compiler_artifact")
    if (not isinstance(artifact, dict) or artifact.get("reason") != "compiler-artifact"
            or artifact.get("executable") != str(image)
            or artifact.get("target", {}).get("name") != "ctrl"
            or artifact.get("target", {}).get("kind") != ["bin"]):
        raise RuntimeError("actual ctrl binary CompilerArtifact is required, not a test image")
    if role == "legacy" and manifest.get("source_revision") != LEGACY_SOURCE:
        raise RuntimeError("old compatibility must use the actual retained 5e Source")
    revision = manifest.get("source_revision")
    if (not isinstance(revision, str) or len(revision) != 40
            or any(value not in "0123456789abcdef" for value in revision)):
        raise RuntimeError("actual source revision is required")
    for field in ("source_archive_sha256", "cargo_lock_sha256", "sha256"):
        value = manifest.get(field)
        if (not isinstance(value, str) or len(value) != 64
                or any(character not in "0123456789abcdef" for character in value)):
            raise RuntimeError("source, lock and image custody hashes are required")
    start = time.monotonic()
    # Host owns actual compile-before-execute attribution. This verifies bytes;
    # it cannot independently prove the compiler consumed an arbitrary manifest.
    body = ordinary_bytes(image, 256 * 1024 * 1024, single_link=False)
    digest = hashlib.sha256(body).hexdigest()
    if digest != manifest["sha256"] or not os.access(image, os.X_OK):
        raise RuntimeError("actual required image bytes or execute form differ")
    observation = dict(manifest)
    observation["sha2_cost"] = {"bytes": len(body), "seconds": time.monotonic() - start}
    observation["physical_basis"] = basis(image)
    return image, observation


class NativeCases:
    def __init__(self, sdk: Any, current: Path, legacy: Path, root: Path):
        self.sdk = sdk
        self.current = current
        self.legacy = legacy
        self.owner = root
        self.owner_basis = basis(root)[:2]
        self.fixtures: list[dict[str, Any]] = []
        self.calls = 0
        self.unknown_retirement = False

    def owner_admit(self) -> None:
        value = basis(self.owner)
        if (not stat.S_ISDIR(value[2]) or value[:2] != self.owner_basis
                or self.owner.resolve(strict=True) != self.owner):
            raise RuntimeError("admitted test owner affiliation changed")

    def admit(self) -> None:
        self.owner_admit()
        for image in (self.current, self.legacy):
            if basis(image) != IMAGE_BASES[str(image)]:
                raise RuntimeError("required native image changed before operation")

    def fixture(self, label: str) -> Path:
        self.admit()
        base = Path(tempfile.mkdtemp(prefix=label + "-", dir=self.owner))
        root = base / "Central"
        (root / "Control").mkdir(parents=True)
        row = {"case": label, "base": str(base), "basis": basis(base),
               "root": str(root), "disposition": "retained-before-effects"}
        self.fixtures.append(row)
        # Durable host output identifies custody before the first SDK child.
        self.write_custody()
        return root

    def write_custody(self) -> None:
        self.admit()
        (self.owner / "fixture-custody.json").write_text(
            json.dumps({"fixtures": self.fixtures,
                        "unknown_retirement": self.unknown_retirement}, sort_keys=True) + "\n"
        )

    async def call(self, awaitable: Any) -> Any:
        try:
            self.admit()
            self.calls += 1
            try:
                result = await asyncio.wait_for(awaitable, timeout=20)
            except asyncio.TimeoutError:
                self.unknown_retirement = True
                self.write_custody()
                raise RuntimeError("SDK deadline failed; native child retirement is unknown")
            self.admit()
            return result
        finally:
            # Admission failure can happen before a newly constructed coroutine
            # is awaited. Close only that coroutine, never resubmit the action.
            if getattr(awaitable, "cr_frame", None) is not None:
                awaitable.close()

    def select(self, root: Path, image: Path | None = None) -> None:
        self.admit()
        os.environ["CENTRAL_CTRL_BIN"] = str(image or self.current)
        os.environ["CENTRAL_ROOT"] = str(root)
        os.environ.pop("CENTRAL_PROJECT", None)

    async def returned(self, root: Path, label: str, *, project: str | None = None,
                       image: Path | None = None) -> dict[str, Any]:
        self.select(root, image)
        return await self.call(self.sdk.central_now_handover(
            "real native " + label, "owned fixture canary " + label,
            actor="actuation-native-handoff-proof", project=project,
            session_ref="agent-session/actuation-native-handoff-proof",
        ))

    @staticmethod
    def path(root: Path, returned: dict[str, Any], project: str | None = None) -> Path:
        source = returned["data"]["source"]
        return ((root / "Work" / project) if project else root) / source

    async def selected(self, returned: dict[str, Any], project: str | None = None) -> dict[str, Any]:
        data = returned["data"]
        return await self.call(self.sdk.central_now_handoff_read(
            data["handoff"]["id"], project, read_path=data["read_path"]
        ))

    async def refused(self, returned: dict[str, Any], project: str | None = None) -> str:
        try:
            await self.selected(returned, project)
        except RuntimeError as error:
            observed = str(error)
            assert returned["data"]["handoff"]["result"] not in observed
            return observed
        raise AssertionError("actual current owner or SDK material refusal was required")

    async def init_project(self, root: Path, project: str, image: Path | None = None) -> None:
        self.select(root, image)
        (root / "Work" / project).mkdir(parents=True)
        await self.call(self.sdk._central_action(
            "projectcentral.init", {"project": project, "project_id": "opaque/different-id"}
        ))
        await self.call(self.sdk._central_action("projectcentral.now.init", {"project": project}))

    async def root_roundtrip_current_native_source(self) -> None:
        root = self.fixture("root-roundtrip")
        returned = await self.returned(root, "root-roundtrip")
        path = self.path(root, returned)
        body = ordinary_bytes(path, 4 * 1024 * 1024)
        before = basis(path)
        read = await self.selected(returned)
        assert read["handoff"] == returned["data"]["handoff"]
        assert json.loads(body) == read["handoff"]
        metadata = read["source_reading"]
        assert metadata["location"] == returned["data"]["read_path"]["input"]["location"]
        assert metadata["byte_len"] == len(body) and isinstance(metadata["revision"], str)
        assert metadata["location"]["path"] == returned["data"]["source"]
        assert ordinary_bytes(path, 4 * 1024 * 1024) == body and basis(path) == before
        assert not (root / "Control/agents/now/clearings").exists()

    async def project_member_and_old_owner_compatibility(self) -> None:
        for role, image in (("current", self.current), ("legacy", self.legacy)):
            root = self.fixture("project-" + role)
            await self.init_project(root, "member", image)
            returned = await self.returned(root, "project-" + role, project="member", image=image)
            path = self.path(root, returned, "member")
            before, body = basis(path), ordinary_bytes(path, 4 * 1024 * 1024)
            legacy = await self.call(self.sdk.central_now_handoff_read(
                returned["data"]["handoff"]["id"], project="member"
            ))
            assert legacy["handoff"] == returned["data"]["handoff"]
            if role == "current":
                route = returned["data"]["read_path"]
                assert route["input"]["location"]["path"].startswith("Work/member/")
                read = await self.selected(returned, "member")
                assert read["handoff"] == legacy["handoff"]
            else:
                assert "read_path" not in returned["data"]
            assert ordinary_bytes(path, 4 * 1024 * 1024) == body and basis(path) == before
            assert not (root / "Control/agents/now/agents").exists()

    async def exact_selected_source_preserves_sibling(self) -> None:
        root = self.fixture("selected-sibling")
        first = await self.returned(root, "selected-first")
        second = await self.returned(root, "selected-second")
        sibling = self.path(root, second)
        before, body = basis(sibling), ordinary_bytes(sibling, 4 * 1024 * 1024)
        reading = await self.selected(first)
        assert reading["handoff"]["id"] == first["data"]["handoff"]["id"]
        assert reading["handoff"]["id"] != second["data"]["handoff"]["id"]
        assert second["data"]["handoff"]["result"] not in json.dumps(reading)
        assert ordinary_bytes(sibling, 4 * 1024 * 1024) == body and basis(sibling) == before
        assert len(list(sibling.parent.glob("*.json"))) == 2

    async def current_owner_withdrawal_and_unsafe_forms(self) -> None:
        root = self.fixture("current-refusals")
        returned = await self.returned(root, "current-refusals")
        path = self.path(root, returned)
        before, body = basis(path), ordinary_bytes(path, 4 * 1024 * 1024)
        marker = path.parent / ".no-agent-retrieval"
        marker.write_bytes(b"")
        try:
            await self.refused(returned)
        finally:
            marker.unlink()
        assert (await self.selected(returned))["handoff"] == returned["data"]["handoff"]
        saved = path.with_suffix(".retained")
        path.rename(saved)
        try:
            await self.refused(returned)  # Genuine absent final member.
            for form in ("symlink", "directory", "fifo"):
                if form == "symlink":
                    path.symlink_to(saved)
                elif form == "directory":
                    path.mkdir()
                else:
                    os.mkfifo(path, 0o600)
                try:
                    await self.refused(returned)
                finally:
                    path.rmdir() if form == "directory" else path.unlink()
        finally:
            saved.rename(path)
        foreign = self.fixture("foreign-root")
        self.select(foreign)
        await self.refused(returned)
        self.select(root)
        assert (await self.selected(returned))["handoff"] == returned["data"]["handoff"]
        assert ordinary_bytes(path, 4 * 1024 * 1024) == body
        assert basis(path)[:2] == before[:2]

    async def actual_nonroot_eacces_then_reopen(self) -> None:
        assert os.geteuid() != 0, "actual EACCES prerequisite is mandatory, never skipped"
        root = self.fixture("eacces")
        returned = await self.returned(root, "eacces")
        path = self.path(root, returned)
        body, identity = ordinary_bytes(path, 4 * 1024 * 1024), basis(path)[:2]
        parent, mode = path.parent, stat.S_IMODE(path.parent.stat().st_mode)
        parent_identity = basis(parent)[:2]
        parent.chmod(0)
        try:
            try:
                with path.open("rb") as handle:
                    handle.read(1)
            except OSError as oracle:
                assert oracle.errno == errno.EACCES
            else:
                raise AssertionError("actual owned OS EACCES oracle is required")
            await self.refused(returned)
            # CLI/SDK currently flatten native typed cause. Do not fabricate an
            # errno guarantee from the real OS oracle above or an error string.
        finally:
            assert basis(parent)[:2] == parent_identity
            parent.chmod(mode)
        assert (await self.selected(returned))["handoff"] == returned["data"]["handoff"]
        assert ordinary_bytes(path, 4 * 1024 * 1024) == body and basis(path)[:2] == identity

    async def old_root_without_route_is_unavailable_not_resubmitted(self) -> None:
        root = self.fixture("old-root")
        returned = await self.returned(root, "old-root", image=self.legacy)
        assert "read_path" not in returned["data"]
        path = self.path(root, returned)
        before, body = basis(path), ordinary_bytes(path, 4 * 1024 * 1024)
        entries = sorted(value.name for value in path.parent.iterdir())
        calls = self.calls
        try:
            await self.sdk.central_now_handoff_read(returned["data"]["handoff"]["id"])
        except RuntimeError as error:
            assert "without the native return read_path" in str(error)
        else:
            raise AssertionError("actual old root has no selected-read route")
        assert self.calls == calls
        assert sorted(value.name for value in path.parent.iterdir()) == entries
        assert ordinary_bytes(path, 4 * 1024 * 1024) == body and basis(path) == before

    async def real_selected_malformed_and_budget_material(self) -> None:
        root = self.fixture("malformed-budget")
        returned = await self.returned(root, "malformed-budget")
        path = self.path(root, returned)
        body, identity = ordinary_bytes(path, 4 * 1024 * 1024), basis(path)[:2]
        replacements = [b"{invalid-json", b"{}", b"x" * (4 * 1024 * 1024 + 1)]
        wrong_id = dict(returned["data"]["handoff"])
        wrong_id["id"] = "another-owned-material-id"
        replacements.insert(2, json.dumps(wrong_id).encode())
        try:
            for replacement in replacements:
                path.write_bytes(replacement)  # Actual owned existing file, not a fake response.
                await self.refused(returned)
                assert basis(path)[:2] == identity
        finally:
            path.write_bytes(body)
        assert (await self.selected(returned))["handoff"] == returned["data"]["handoff"]
        assert ordinary_bytes(path, 4 * 1024 * 1024) == body and basis(path)[:2] == identity


IMAGE_BASES: dict[str, tuple[int, int, int, int, int, int, int]] = {}


async def main(args: argparse.Namespace) -> None:
    if os.name != "posix" or os.geteuid() == 0:
        raise RuntimeError("supported nonroot Linux/Mac native prerequisites are mandatory")
    root = Path(args.artifact_root)
    if not root.is_absolute() or root.resolve(strict=True) != root or not stat.S_ISDIR(basis(root)[2]):
        raise RuntimeError("existing ordinary admitted native artifact root is required")
    if any(root.iterdir()):
        raise RuntimeError("exclusive admitted artifact root must start empty")
    current, current_proof = build_image(Path(args.current_build), "current")
    legacy, legacy_proof = build_image(Path(args.legacy_build), "legacy")
    if current == legacy or current_proof["source_revision"] == legacy_proof["source_revision"]:
        raise RuntimeError("actual current and historical owner build inputs must be distinct")
    IMAGE_BASES.update({str(current): basis(current), str(legacy): basis(legacy)})
    sdk_path = Path(args.sdk_source)
    sdk_body = ordinary_bytes(sdk_path, 1024 * 1024)
    if hashlib.sha256(sdk_body).hexdigest() != args.sdk_sha256:
        raise RuntimeError("actual selected SDK Source differs before import")
    spec = importlib.util.spec_from_file_location("native_handoff_sdk", sdk_path)
    if spec is None or spec.loader is None:
        raise RuntimeError("actual SDK Source import is unavailable")
    sdk = importlib.util.module_from_spec(spec)
    # Avoid writing __pycache__ into the frozen Source tree.
    import sys
    sys.dont_write_bytecode = True
    spec.loader.exec_module(sdk)
    for field in ("CENTRAL_PROJECT", "CENTRAL_NATIVE_TOKEN", "QL_RELATIONAL_EVIDENCE_LOG",
                  "ACTUATION_RESEARCH_BIN", "ACTUATION_RESEARCH_FACULTY_CONFIG",
                  "ACTUATION_RESEARCH_TRACE_REF"):
        os.environ.pop(field, None)
    (root / "build-custody.json").write_text(json.dumps({
        "current": current_proof, "legacy": legacy_proof,
        "sdk_sha256": args.sdk_sha256,
        "compiled_source_attribution": "host compile/Source/lock receipts required; image bytes independently rehashed here",
    }, sort_keys=True) + "\n")
    cases = NativeCases(sdk, current, legacy, root)
    completed: list[dict[str, Any]] = []
    try:
        for name in CASES:
            start = time.monotonic()
            await getattr(cases, name)()
            completed.append({"case": name, "result": "passed", "seconds": time.monotonic() - start})
            (root / "case-results.json").write_text(json.dumps(completed, sort_keys=True) + "\n")
            print(json.dumps(completed[-1], sort_keys=True), flush=True)
        assert len(completed) == 7 and {row["case"] for row in completed} == set(CASES)
        cases.admit()
        assert ordinary_bytes(sdk_path, 1024 * 1024) == sdk_body
        cases.write_custody()
        print(json.dumps({"selected": 7, "passed": 7, "skipped": 0,
                          "fixtures": "retained; no inner process retirement inferred",
                          "native_calls": cases.calls}, sort_keys=True), flush=True)
    except BaseException as error:
        # Controlled fixture facts only; do not dump owner replies or document bodies.
        cases.owner_admit()
        (root / "failure.json").write_text(json.dumps({
            "type": type(error).__name__, "message": str(error),
            "completed": completed, "native_calls": cases.calls,
            "unknown_retirement": cases.unknown_retirement,
        }, sort_keys=True) + "\n")
        raise


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--current-build", required=True)
    parser.add_argument("--legacy-build", required=True)
    parser.add_argument("--artifact-root", required=True)
    parser.add_argument("--sdk-source", required=True)
    parser.add_argument("--sdk-sha256", required=True)
    asyncio.run(main(parser.parse_args()))
