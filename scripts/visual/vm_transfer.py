"""Bounded raw-artifact return for the Tart adapter, not comparison or sealing.

Only flat producer PNG/JSON/TSV evidence travels back. No tar extraction helper,
links, sparse files, path rewriting, image conversions or implicit acceptance.
The Rust producer validator remains the sole application coverage authority.
"""
from __future__ import annotations

import hashlib
import io
import json
import os
from pathlib import Path
import re
import tarfile
import time

FILE_LIMIT = 64 * 1024 * 1024
TOTAL_LIMIT = 4 * 1024 * 1024 * 1024
MANIFEST_LIMIT = 8 * 1024 * 1024
MAX_FILES = 20_000
DEADLINE_SECONDS = 120
NAME = re.compile(r"[a-z0-9][a-z0-9-]*(?:\.png|\.snapshot\.json)\Z")
METADATA = {"coverage.tsv", "inventory.json", "selection.json", "guest-receipt.json", "live-inventory.tsv"}


class TransferError(RuntimeError):
    pass


def _tick(deadline: float) -> None:
    if time.monotonic() >= deadline:
        raise TransferError("artifact transfer deadline")


def _plain(path: Path) -> None:
    for item in (path, *path.parents):
        if item.is_symlink():
            raise TransferError("symlink in artifact transfer path")


def _hash(value: str) -> None:
    if not isinstance(value, str) or not re.fullmatch("[0-9a-f]{64}", value):
        raise TransferError("invalid transfer hash")


def _name(name: str) -> None:
    if name not in METADATA and not NAME.fullmatch(name):
        raise TransferError(f"invalid raw artifact name: {name}")


def _digest(handle, size: int, deadline: float, output=None) -> str:
    digest = hashlib.sha256()
    count = 0
    while block := handle.read(65536):
        _tick(deadline)
        count += len(block)
        if count > size:
            raise TransferError("artifact grew while reading")
        digest.update(block)
        if output is not None:
            output.write(block)
    if count != size:
        raise TransferError("artifact truncated")
    return digest.hexdigest()


def _unique(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise TransferError("duplicate JSON key")
        result[key] = value
    return result


def _manifest(data: bytes, payload_sha256: str) -> dict:
    if len(data) > MANIFEST_LIMIT:
        raise TransferError("return manifest too large")
    value = json.loads(data, object_pairs_hook=_unique)
    if not isinstance(value, dict) or set(value) != {"schema", "payload_sha256", "files"}:
        raise TransferError("invalid return manifest fields")
    if type(value["schema"]) is not int or value["schema"] != 1 or value["payload_sha256"] != payload_sha256:
        raise TransferError("return manifest schema/payload differs")
    files = value["files"]
    if not isinstance(files, dict) or not 1 <= len(files) <= MAX_FILES:
        raise TransferError("empty or excessive return inventory")
    total = 0
    for name, record in files.items():
        _name(name)
        if not isinstance(record, dict) or set(record) != {"bytes", "sha256"}:
            raise TransferError("invalid return file fields")
        size = record["bytes"]
        if type(size) is not int or not 0 < size <= FILE_LIMIT:
            raise TransferError("invalid returned artifact size")
        total += size
        _hash(record["sha256"])
    if total > TOTAL_LIMIT or "guest-receipt.json" not in files:
        raise TransferError("excessive return size or missing guest receipt")
    return value


def pack(output: Path, archive: Path, payload_sha256: str) -> dict:
    """Guest: package raw output only after producer exit and receipt creation."""
    _hash(payload_sha256)
    _plain(output)
    _plain(archive)
    if not output.is_dir() or archive.exists() or archive == output or output in archive.parents:
        raise TransferError("fresh archive outside producer directory required")
    deadline = time.monotonic() + DEADLINE_SECONDS
    files = {}
    total = 0
    for path in sorted(output.iterdir()):
        _tick(deadline)
        _name(path.name)
        if path.is_symlink() or not path.is_file():
            raise TransferError("nonregular raw producer artifact")
        size = path.stat().st_size
        if not 0 < size <= FILE_LIMIT or len(files) >= MAX_FILES:
            raise TransferError("unbounded raw producer artifact")
        total += size
        if total > TOTAL_LIMIT:
            raise TransferError("raw artifact set exceeds total limit")
        with path.open("rb") as handle:
            digest = _digest(handle, size, deadline)
        files[path.name] = {"bytes": size, "sha256": digest}
    manifest = {"schema": 1, "payload_sha256": payload_sha256, "files": files}
    raw = json.dumps(manifest, sort_keys=True, separators=(",", ":")).encode()
    _manifest(raw, payload_sha256)
    # create_new keeps interruption evidence; a partial tar is never retried in place.
    with archive.open("xb") as stream:
        os.chmod(archive, 0o600)
        with tarfile.open(fileobj=stream, mode="w", format=tarfile.USTAR_FORMAT) as tar:
            meta = tarfile.TarInfo("return.json")
            meta.size, meta.mode = len(raw), 0o400
            tar.addfile(meta, io.BytesIO(raw))
            for name, record in files.items():
                _tick(deadline)
                path = output / name
                _plain(path)
                meta = tarfile.TarInfo(name)
                meta.size, meta.mode = record["bytes"], 0o400
                with path.open("rb") as handle:
                    tar.addfile(meta, handle)
    # Verify final archive bytes, detecting producer output mutation between passes.
    inspect(archive, payload_sha256, deadline=deadline)
    return manifest


def inspect(archive: Path, payload_sha256: str, *, deadline: float | None = None) -> dict:
    """Host preflight: hash every archive member before creating any output."""
    _hash(payload_sha256)
    _plain(archive)
    deadline = deadline if deadline is not None else time.monotonic() + DEADLINE_SECONDS
    if not archive.is_file() or not 0 < archive.stat().st_size <= TOTAL_LIMIT + MAX_FILES * 1024 + MANIFEST_LIMIT:
        raise TransferError("unbounded/nonregular return archive")
    with tarfile.open(archive, "r:") as tar:
        found = {}
        manifest = None
        total = 0
        for member in tar:
            _tick(deadline)
            if len(found) > MAX_FILES or member.name in found:
                raise TransferError("duplicate or excessive archive members")
            if member.type != tarfile.REGTYPE or member.sparse or member.pax_headers or member.linkname:
                raise TransferError("nonregular/extended archive member")
            if member.name != "return.json":
                _name(member.name)
            size_limit = MANIFEST_LIMIT if member.name == "return.json" else FILE_LIMIT
            if not 0 < member.size <= size_limit:
                raise TransferError("invalid archive member size")
            total += member.size
            if total > TOTAL_LIMIT + MANIFEST_LIMIT:
                raise TransferError("archive expanded size exceeds limit")
            handle = tar.extractfile(member)
            if handle is None:
                raise TransferError("missing member content")
            with handle:
                if member.name == "return.json":
                    manifest = _manifest(handle.read(MANIFEST_LIMIT + 1), payload_sha256)
                    found[member.name] = None
                else:
                    found[member.name] = {"bytes": member.size, "sha256": _digest(handle, member.size, deadline)}
        if manifest is None or found.pop("return.json", "absent") is not None or found != manifest["files"]:
            raise TransferError("missing/extra/changed archive artifact")
    return manifest


def unpack(archive: Path, destination: Path, payload_sha256: str) -> dict:
    """Host: never overwrite; validate all bytes before exposing raw candidates.

    Use a staging path outside GPUI Visual's run directory. Rust coverage/receipt
    checks and clean VM teardown must still pass before the tool may seal images.
    """
    _plain(destination)
    if not destination.is_absolute() or destination.exists():
        raise TransferError("fresh absolute return staging path required")
    deadline = time.monotonic() + DEADLINE_SECONDS
    manifest = inspect(archive, payload_sha256, deadline=deadline)
    destination.mkdir(mode=0o700)
    with tarfile.open(archive, "r:") as tar:
        for member in tar:
            _tick(deadline)
            if member.name == "return.json":
                continue
            record = manifest["files"].get(member.name)
            if (record is None or member.type != tarfile.REGTYPE or member.size != record["bytes"]
                    or member.sparse or member.pax_headers or member.linkname):
                raise TransferError("archive changed after preflight")
            with tar.extractfile(member) as handle, (destination / member.name).open("xb") as output:
                digest = _digest(handle, record["bytes"], deadline, output)
            if digest != record["sha256"]:
                raise TransferError("archive bytes changed after preflight")
    actual = {path.name for path in destination.iterdir()}
    if actual != set(manifest["files"]):
        raise TransferError("incomplete transferred inventory")
    with (destination / "return.json").open("x") as output:
        json.dump(manifest, output, sort_keys=True)
    return manifest
