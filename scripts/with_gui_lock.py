#!/usr/bin/env python3
"""Run one GUI command while holding BBR's host-wide AppKit gate lock."""

from __future__ import annotations

import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

DEFAULT_TIMEOUT_SECONDS = 30 * 60


def repository_key() -> str:
    override = os.environ.get("BBR_GUI_LOCK_KEY")
    if override:
        return override

    result = subprocess.run(
        ["git", "rev-parse", "--path-format=absolute", "--git-common-dir"],
        check=True,
        capture_output=True,
        text=True,
    )
    common_dir = Path(result.stdout.strip()).resolve()
    return hashlib.sha256(str(common_dir).encode()).hexdigest()[:16]


def lock_path() -> Path:
    root = Path(os.environ.get("BBR_GUI_LOCK_DIR", "/tmp"))
    return root / f"bbr-gui-gate-{repository_key()}.lock"


def timeout_seconds() -> float:
    raw = os.environ.get("BBR_GUI_LOCK_TIMEOUT_SECONDS")
    if raw is None:
        return DEFAULT_TIMEOUT_SECONDS
    value = float(raw)
    if value <= 0:
        raise ValueError("BBR_GUI_LOCK_TIMEOUT_SECONDS must be greater than zero")
    return value


def read_holder(lock_file) -> str:
    lock_file.seek(0)
    value = lock_file.read().strip()
    return value or "unknown holder"


def acquire(lock_file, path: Path, timeout: float) -> None:
    deadline = time.monotonic() + timeout
    while True:
        try:
            fcntl.flock(lock_file.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
            break
        except BlockingIOError:
            if time.monotonic() >= deadline:
                holder = read_holder(lock_file)
                raise TimeoutError(
                    f"timed out after {timeout:g}s waiting for GUI gate {path}; "
                    f"current holder: {holder}"
                )
            time.sleep(min(0.1, max(0.0, deadline - time.monotonic())))

    metadata = {
        "pid": os.getpid(),
        "session_id": os.environ.get("PI_SESSION_ID"),
        "cwd": str(Path.cwd()),
        "acquired_monotonic": time.monotonic(),
    }
    lock_file.seek(0)
    lock_file.truncate()
    lock_file.write(json.dumps(metadata, sort_keys=True))
    lock_file.flush()


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Serialize BBR AppKit/WKWebView GUI gates across worktrees."
    )
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    if args.command and args.command[0] == "--":
        args.command = args.command[1:]
    if not args.command:
        parser.error("a command is required after --")
    return args


def main() -> int:
    args = parse_args()
    path = lock_path()
    path.parent.mkdir(parents=True, exist_ok=True)

    try:
        timeout = timeout_seconds()
    except ValueError as error:
        print(error, file=sys.stderr)
        return 2

    with path.open("a+", encoding="utf-8") as lock_file:
        try:
            acquire(lock_file, path, timeout)
        except TimeoutError as error:
            print(error, file=sys.stderr)
            return 75

        print(f"BBR GUI gate acquired: {path}", file=sys.stderr)
        try:
            return subprocess.run(args.command, check=False).returncode
        finally:
            fcntl.flock(lock_file.fileno(), fcntl.LOCK_UN)


if __name__ == "__main__":
    raise SystemExit(main())
