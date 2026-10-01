"""Verify and install the Rust-staged payload at its compiled guest paths.

Transport only: no producer execution, environment observation or capture seal.
The caller supplies the host-verified manifest hash, repo and suite independently;
none is inferred from guest bytes. Production callers must run in the approved
throwaway guest. There is deliberately no host CLI or destination remapping.
"""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import re
import stat
import time
import unicodedata

FILE_LIMIT = 1024 * 1024 * 1024
TOTAL_LIMIT = 4 * FILE_LIMIT
MANIFEST_LIMIT = 8 * 1024 * 1024
MAX_FILES = 20_000
DEADLINE_SECONDS = 120


class InstallError(RuntimeError):
    pass


def _tick(deadline: float) -> None:
    if time.monotonic() >= deadline:
        raise InstallError("payload installation deadline")


def _absolute(path: Path) -> None:
    if not path.is_absolute() or path.resolve() != path:
        raise InstallError("canonical absolute path without symlinks required")
    for item in (path, *path.parents):
        if item.is_symlink():
            raise InstallError("symlink in installation path")


def _relative(name: str) -> None:
    if not isinstance(name, str) or not name or any(ord(c) < 32 or ord(c) == 127 for c in name):
        raise InstallError("invalid payload path")
    for part in name.split("/"):
        if (not part or part.startswith(".") or part in {
                "secrets", "credentials", "reports", "baselines", "gallery-baselines"}
                or part.endswith((".pem", ".key", "@git"))):
            raise InstallError("noncanonical/non-source payload path")


def _digest(value: str) -> None:
    if not isinstance(value, str) or not re.fullmatch("[0-9a-f]{64}", value):
        raise InstallError("invalid payload digest")


def _unique(pairs):
    value = {}
    for key, item in pairs:
        if key in value:
            raise InstallError("duplicate payload JSON key")
        value[key] = item
    return value


def _open(path: Path, limit: int):
    _absolute(path)
    # NONBLOCK prevents a raced FIFO/device from blocking before fstat rejects it.
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    handle = os.fdopen(fd, "rb")
    meta = os.fstat(handle.fileno())
    if not stat.S_ISREG(meta.st_mode) or meta.st_nlink != 1 or meta.st_size > limit:
        handle.close()
        raise InstallError("bounded regular file without links required")
    return handle, meta


def _copy_hash(path: Path, record: dict, deadline: float, output=None) -> str:
    _tick(deadline)
    handle, meta = _open(path, FILE_LIMIT)
    with handle:
        if meta.st_size != record["bytes"] or bool(meta.st_mode & 0o111) != record["executable"]:
            raise InstallError("payload size/executable mode differs")
        digest, count = hashlib.sha256(), 0
        while block := handle.read(65536):
            _tick(deadline)
            count += len(block)
            if count > record["bytes"]:
                raise InstallError("payload grew while copying")
            digest.update(block)
            if output is not None:
                output.write(block)
        if count != record["bytes"] or digest.hexdigest() != record["sha256"]:
            raise InstallError("payload bytes differ")
    return digest.hexdigest()


def _tree(root: Path, expected: set[str], deadline: float) -> None:
    directories = {str(parent) for name in expected for parent in Path(name).parents if str(parent) != "."}
    found, queue = set(), [root]
    visited = 0
    while queue:
        _tick(deadline)
        with os.scandir(queue.pop()) as entries:
            for entry in entries:
                _tick(deadline)
                visited += 1
                if visited > len(expected) + len(directories):
                    raise InstallError("unexpected payload tree entries")
                name = str(Path(entry.path).relative_to(root))
                if entry.is_symlink():
                    raise InstallError("symlink in payload tree")
                if entry.is_dir(follow_symlinks=False):
                    if name not in directories:
                        raise InstallError("unexpected payload directory")
                    queue.append(Path(entry.path))
                elif entry.is_file(follow_symlinks=False) and name in expected:
                    found.add(name)
                else:
                    raise InstallError("unexpected/nonregular payload file")
    if found != expected:
        raise InstallError("missing payload file")


def _manifest(raw: bytes, expected_repo: Path, suite: str) -> dict:
    try:
        value = json.loads(raw, object_pairs_hook=_unique)
    except (ValueError, RecursionError) as error:
        raise InstallError("invalid payload JSON") from error
    if not isinstance(value, dict) or set(value) != {"schema", "suite", "repo", "sources", "files"}:
        raise InstallError("invalid payload fields")
    if (type(value["schema"]) is not int or value["schema"] != 1 or value["suite"] != suite
            or suite not in {"gallery", "browser"} or value["repo"] != str(expected_repo)):
        raise InstallError("payload schema/suite/compiled repository differs")
    sources, files = value["sources"], value["files"]
    if (not isinstance(sources, dict) or not sources or not isinstance(files, list)
            or not 1 <= len(files) <= MAX_FILES):
        raise InstallError("empty/excessive payload inventory")
    for name, digest in sources.items():
        _relative(name)
        _digest(digest)
    seen, guests, mapped, roles, total = set(), set(), {}, set(), 0
    for entry in files:
        if not isinstance(entry, dict) or set(entry) != {"payload", "guest", "sha256", "bytes", "executable"}:
            raise InstallError("invalid payload entry fields")
        name, guest = entry["payload"], entry["guest"]
        _relative(name)
        if not isinstance(guest, str) or not guest.startswith(str(expected_repo) + "/"):
            raise InstallError("guest path outside compiled repository")
        relative = guest[len(str(expected_repo)) + 1:]
        _relative(relative)
        _digest(entry["sha256"])
        if type(entry["bytes"]) is not int or not 0 <= entry["bytes"] <= FILE_LIMIT or type(entry["executable"]) is not bool:
            raise InstallError("invalid payload size/mode")
        total += entry["bytes"]
        if total > TOTAL_LIMIT or name in seen or guest in guests:
            raise InstallError("duplicate/excessive payload inventory")
        seen.add(name)
        guests.add(guest)
        if name.startswith("source/"):
            if entry["executable"] or name != "source/" + relative or relative.startswith("target/"):
                raise InstallError("source mapping differs")
            mapped[relative] = entry["sha256"]
        else:
            if not re.fullmatch(r"executables/(0|[1-9][0-9]*)", name) or not entry["executable"]:
                raise InstallError("invalid executable mapping")
            if relative == "target/release/bbr-gallery":
                role = "gallery"
            elif relative == "target/release/bbr":
                role = "browser"
            elif re.fullmatch(r"target/release/deps/e2e_visual_shell_capture-[a-zA-Z0-9]+", relative):
                role = "capture-test"
            else:
                raise InstallError("unknown executable role")
            if role in roles:
                raise InstallError("duplicate executable role")
            roles.add(role)
    required = {"gallery"} if suite == "gallery" else {"browser", "capture-test"}
    if mapped != sources or roles != required:
        raise InstallError("incomplete source/executable inventory")
    # APFS may be case-insensitive and normalization-insensitive. Reject aliases
    # and file/directory conflicts before any destination is created.
    for names in (seen, guests):
        folded = {unicodedata.normalize("NFD", name).casefold() for name in names}
        if len(folded) != len(names) or any(str(parent) in folded for name in folded for parent in Path(name).parents):
            raise InstallError("colliding payload paths")
    return value


def inspect(root: Path, expected_hash: str, repo: Path, suite: str, *, deadline=None) -> dict:
    """Verify the entire read-only share before installing anything."""
    deadline = time.monotonic() + DEADLINE_SECONDS if deadline is None else deadline
    _tick(deadline)
    _digest(expected_hash)
    _absolute(root)
    _absolute(repo)
    if root == repo or root in repo.parents or repo in root.parents:
        raise InstallError("shared payload and compiled repo must be disjoint")
    handle, _ = _open(root / "payload.json", MANIFEST_LIMIT)
    with handle:
        raw = handle.read(MANIFEST_LIMIT + 1)
    if len(raw) > MANIFEST_LIMIT or hashlib.sha256(raw).hexdigest() != expected_hash:
        raise InstallError("payload manifest hash differs")
    value = _manifest(raw, repo, suite)
    _tree(root, {"payload.json", *(entry["payload"] for entry in value["files"])}, deadline)
    for entry in value["files"]:
        _copy_hash(root / entry["payload"], entry, deadline)
    return value


def _installed(value: dict, deadline: float) -> dict[str, str]:
    repo = Path(value["repo"])
    _absolute(repo)
    _tree(repo, {str(Path(entry["guest"]).relative_to(repo)) for entry in value["files"]}, deadline)
    result = {}
    for entry in value["files"]:
        path = Path(entry["guest"])
        if stat.S_IMODE(path.lstat().st_mode) != (0o500 if entry["executable"] else 0o400):
            raise InstallError("installed file permissions differ")
        result[str(path)] = _copy_hash(path, entry, deadline)
    return result


def verify_installed(root: Path, expected_hash: str, repo: Path, suite: str) -> dict[str, str]:
    """Recheck shared and installed bytes before/after guest execution."""
    deadline = time.monotonic() + DEADLINE_SECONDS
    value = inspect(root, expected_hash, repo, suite, deadline=deadline)
    return _installed(value, deadline)


def install(root: Path, expected_hash: str, repo: Path, suite: str) -> dict[str, str]:
    """Fresh destination only. Failure leaves incomplete evidence, never resumes.

    The transport caller prepares the compiled repo's parent only inside its
    disposable guest. No sudo, ancestor ownership change or executable launch
    happens here. The returned observed hashes feed the separate guest receipt.
    """
    deadline = time.monotonic() + DEADLINE_SECONDS
    value = inspect(root, expected_hash, repo, suite, deadline=deadline)
    if repo.exists() or not repo.parent.is_dir():
        raise InstallError("fresh compiled repo under an existing parent required")
    _tick(deadline)
    repo.mkdir(mode=0o700)
    for entry in value["files"]:
        _tick(deadline)
        target = Path(entry["guest"])
        target.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
        _absolute(target)
        with target.open("xb") as output:
            os.fchmod(output.fileno(), 0o500 if entry["executable"] else 0o400)
            _copy_hash(root / entry["payload"], entry, deadline, output)
    # Shared bytes and manifest must still match, and installation must not
    # contain another runtime source/target authority or unexpected files.
    inspect(root, expected_hash, repo, suite, deadline=deadline)
    return _installed(value, deadline)
