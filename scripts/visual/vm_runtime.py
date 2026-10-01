"""Offline, source-bound runtime prerequisite for the visual guest bootstrap.

No Nix realization, substitution, repair, VM, install or producer execution. The
caller supplies an already verified payload identity. Read-only probes remain in
the Rust caller's upstream-owned process group (no detached children). This module
reaps only its direct probe child; upstream owns whole-group cancellation. It
cannot open the capture guard and never executes a native producer.
"""
from __future__ import annotations

import base64
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import tempfile
import time

import vm_install

LOCK = "scripts/visual/guest-runtime.json"
STORE = re.compile(r"/nix/store/[0-9abcdfghijklmnpqrsvwxyz]{32}-[A-Za-z0-9][A-Za-z0-9+._?=-]*\Z")
TOTAL_LIMIT = 2 * 1024 * 1024 * 1024
OUTPUT_LIMIT = 4 * 1024 * 1024
MAX_PATHS = 256


class RuntimeError(ValueError):
    pass


def _need(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def _json(raw: str | bytes):
    try:
        return json.loads(raw, object_pairs_hook=vm_install._unique)
    except (ValueError, RecursionError, vm_install.InstallError) as error:
        raise RuntimeError("invalid/duplicate runtime JSON") from error


def _store(path) -> None:
    _need(isinstance(path, str) and STORE.fullmatch(path) is not None, "invalid Nix store root")


def _absolute(path: str) -> None:
    _need(isinstance(path, str) and path.startswith("/") and
          all(part and part not in {".", ".."} for part in path[1:].split("/")) and
          not any(ord(c) < 33 or ord(c) == 127 for c in path), "noncanonical runtime path")


def validate(value: dict) -> dict:
    _need(isinstance(value, dict) and set(value) == {"schema", "arch", "python", "roots", "paths"},
          "invalid runtime lock fields")
    _need(type(value["schema"]) is int and value["schema"] == 1 and value["arch"] == "arm64",
          "runtime schema/architecture differs")
    python, roots, paths = value["python"], value["roots"], value["paths"]
    _need(isinstance(python, dict) and set(python) == {"path", "version", "sha256"}, "invalid Python identity")
    _absolute(python["path"])
    _need(isinstance(python["version"], str) and re.fullmatch(r"3\.(?:1[3-9]|[2-9][0-9])\.[0-9]+", python["version"]),
          "explicit supported Python version required")
    _need(isinstance(python["sha256"], str) and re.fullmatch("[0-9a-f]{64}", python["sha256"]),
          "invalid Python hash")
    root = str(Path(python["path"]).parents[1])
    _store(root)
    _need(python["path"] == root + "/bin/python" + python["version"].rsplit(".", 1)[0] and
          root.endswith("-python3-" + python["version"]), "Python path/version differs")
    _need(isinstance(roots, list) and 1 <= len(roots) <= MAX_PATHS, "invalid runtime roots")
    for item in roots: _store(item)
    _need(len(set(roots)) == len(roots) and root in roots, "duplicate roots or missing Python root")
    _need(isinstance(paths, dict) and 1 <= len(paths) <= MAX_PATHS, "invalid runtime closure")
    total = 0
    for path, record in paths.items():
        _store(path)
        _need(isinstance(record, dict) and set(record) == {"narHash", "narSize", "references"},
              "invalid closure record")
        _need(type(record["narSize"]) is int and 0 < record["narSize"] <= TOTAL_LIMIT, "invalid NAR size")
        total += record["narSize"]
        digest = record["narHash"]
        _need(isinstance(digest, str) and re.fullmatch(r"sha256-[A-Za-z0-9+/]{43}=", digest), "invalid NAR hash")
        decoded = base64.b64decode(digest[7:], validate=True)
        _need(len(decoded) == 32 and base64.b64encode(decoded).decode() == digest[7:], "noncanonical NAR hash")
        refs = record["references"]
        _need(isinstance(refs, list) and len(refs) <= MAX_PATHS, "invalid NAR references")
        for ref in refs: _store(ref)
        _need(len(set(refs)) == len(refs) and set(refs) <= paths.keys(), "duplicate/missing closure reference")
    _need(total <= TOTAL_LIMIT and set(roots) <= paths.keys(), "excessive/incomplete runtime closure")
    seen, pending = set(), list(roots)
    while pending:
        path = pending.pop()
        if path not in seen:
            seen.add(path)
            pending.extend(paths[path]["references"])
    _need(seen == paths.keys(), "unreachable runtime closure entry")
    return value


def dependencies(raw: str, artifact: Path, lock: dict) -> list[str]:
    """Parse one exact thin-arm64 otool listing; unknown load paths fail closed."""
    lines = raw.splitlines()
    _need(len(lines) > 1 and lines[0] == str(artifact) + ":", "missing/ambiguous Mach-O dependency listing")
    nix = []
    seen = set()
    for line in lines[1:]:
        match = re.fullmatch(r"\s+(.+) \(compatibility version [0-9.]+, current version [0-9.]+\)", line)
        _need(match is not None, "unknown Mach-O dependency record")
        name = match[1]
        _absolute(name)
        _need(name not in seen, "duplicate Mach-O dependency")
        seen.add(name)
        if name.startswith(("/usr/lib/", "/System/Library/Frameworks/", "/System/Library/PrivateFrameworks/")):
            continue
        _need(name.startswith("/nix/store/") and len(Path(name).parts) > 4, "unapproved dylib origin")
        root = "/".join(name.split("/")[:4])
        _store(root)
        _need(root in lock["paths"], "unlocked Mach-O runtime dependency")
        nix.append(name)
    return sorted(nix)


def metadata(raw: str) -> dict:
    value = _json(raw)
    _need(isinstance(value, dict) and 1 <= len(value) <= MAX_PATHS, "missing runtime metadata")
    result = {}
    for path, record in value.items():
        _store(path)
        _need(isinstance(record, dict) and {"narHash", "narSize", "references"} <= record.keys(),
              "incomplete Nix metadata")
        _need(type(record["narSize"]) is int and isinstance(record["narHash"], str) and
              isinstance(record["references"], list) and
              all(isinstance(ref, str) for ref in record["references"]), "invalid Nix metadata types")
        result[path] = {key: record[key] for key in ("narHash", "narSize", "references")}
    return result


def compare_metadata(raw: str, expected: dict) -> None:
    _need(metadata(raw) == expected, "Nix runtime closure differs from source-bound lock")


def _command(argv: list[str], *, seconds: float = 30) -> str:
    """Bound a direct read-only probe; remain in upstream's owned process group."""
    env = {"PATH": "/usr/bin:/bin:/usr/sbin:/sbin", "HOME": str(Path.home()),
           "TZ": "UTC", "LANG": "C", "LC_ALL": "C"}
    with tempfile.TemporaryFile() as log:
        child = subprocess.Popen(argv, stdin=subprocess.DEVNULL, stdout=log, stderr=log,
                                 env=env)
        try:
            deadline = time.monotonic() + seconds
            while child.poll() is None:
                _need(time.monotonic() < deadline, "runtime probe deadline")
                _need(os.fstat(log.fileno()).st_size <= OUTPUT_LIMIT, "runtime probe output limit")
                try: child.wait(timeout=min(0.02, max(0.001, deadline - time.monotonic())))
                except subprocess.TimeoutExpired: pass
            _need(child.returncode == 0, f"runtime probe failed: {argv[0]} rc={child.returncode}")
            _need(os.fstat(log.fileno()).st_size <= OUTPUT_LIMIT, "runtime probe output limit")
            log.seek(0)
            return log.read(OUTPUT_LIMIT + 1).decode("utf-8")
        finally:
            if child.poll() is None:
                child.terminate()
                try: child.wait(timeout=3)
                except subprocess.TimeoutExpired:
                    child.kill()
                    child.wait(timeout=3)


def _file_hash(path: Path) -> str:
    # Nix optimization legitimately hardlinks store files. Reject writable or
    # symlink executable paths, but do not apply payload's single-link rule here.
    _need(path.resolve() == path, "symlink in pinned interpreter path")
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(fd, "rb") as source:
        meta = os.fstat(source.fileno())
        _need(stat.S_ISREG(meta.st_mode) and 0 < meta.st_size <= 64 * 1024 * 1024 and
              stat.S_IMODE(meta.st_mode) == 0o555, "bounded immutable executable required")
        digest, size = hashlib.sha256(), 0
        while block := source.read(65536):
            size += len(block)
            _need(size <= meta.st_size, "interpreter grew")
            digest.update(block)
        _need(size == meta.st_size, "interpreter truncated")
        return digest.hexdigest()


def _python(lock: dict) -> dict:
    python = lock["python"]
    _need(_file_hash(Path(python["path"])) == python["sha256"], "pinned Python bytes differ")
    probe = ("import sys, platform, hashlib, json, tarfile, subprocess, plistlib; "
             "print(json.dumps({'path':sys.executable,'version':platform.python_version(),"
             "'arch':platform.machine(),'system':platform.system()}))")
    actual = _json(_command([python["path"], "-I", "-B", "-c", probe]))
    _need(actual == {"path": python["path"], "version": python["version"],
                     "arch": lock["arch"], "system": "Darwin"}, "pinned Python observation differs")
    _need(_file_hash(Path(python["path"])) == python["sha256"], "Python changed after probe")
    return python


def verify(share: Path, digest: str, repo: Path, suite: str) -> dict:
    """Host prerequisite, not guest evidence. All commands are read-only/offline."""
    payload = vm_install.inspect(share, digest, repo, suite)
    _need(LOCK in payload["sources"], "payload lacks source-bound guest runtime lock")
    lock_path = share / "source" / LOCK
    handle, _ = vm_install._open(lock_path, 1024 * 1024)
    with handle: raw = handle.read(1024 * 1024 + 1)
    _need(hashlib.sha256(raw).hexdigest() == payload["sources"][LOCK], "runtime lock bytes differ")
    lock = validate(_json(raw))
    observed = {}
    for entry in payload["files"]:
        if entry["executable"]:
            artifact = share / entry["payload"]
            _need(_command(["/usr/bin/lipo", "-archs", str(artifact)]).strip() == lock["arch"],
                  "producer/consumer architecture differs")
            observed[entry["guest"]] = dependencies(_command(["/usr/bin/otool", "-L", str(artifact)]), artifact, lock)
    nix, store = shutil.which("nix"), shutil.which("nix-store")
    _need(nix is not None and store is not None, "documented existing Nix tools required; never install a fallback")
    query = [nix, "--extra-experimental-features", "nix-command", "path-info", "--offline", "--json", "--recursive", *lock["roots"]]
    compare_metadata(_command(query), lock["paths"])
    _command([store, "--verify-path", *sorted(lock["paths"])], seconds=180)
    python = _python(lock)
    compare_metadata(_command(query), lock["paths"])
    _need(vm_install.inspect(share, digest, repo, suite) == payload, "payload changed during runtime verification")
    return {"schema": 1, "origin": "host-runtime-prerequisite", "payload_sha256": digest,
            "runtime_lock_sha256": payload["sources"][LOCK], "python": python,
            "dependencies": observed, "closure_paths": len(lock["paths"]),
            "closure_nar_bytes": sum(record["narSize"] for record in lock["paths"].values())}
