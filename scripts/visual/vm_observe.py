"""Guest-only rendering observations; no build-host environment fallback.

Invoked by the future installed-payload guest runner, not the host CLI. Callers
must validate the receipt and exact before/after equality in the Rust consumer.
"""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import plistlib
import subprocess
import time


class ObservationError(RuntimeError):
    pass


def console(plist: bytes) -> dict:
    roots = plistlib.loads(plist)
    if not isinstance(roots, list) or len(roots) != 1:
        raise ObservationError("one IORegistry root required")
    users = roots[0].get("IOConsoleUsers", [])
    active = [u for u in users if u.get("kCGSSessionOnConsoleKey") is True]
    if len(active) != 1 or active[0].get("kCGSSessionUserNameKey") != "admin":
        raise ObservationError("one admin guest console required")
    if active[0].get("CGSSessionScreenIsLocked", False) is not False:
        raise ObservationError("guest screen locked")
    return {"console_user": "admin", "on_console": True, "screen_locked": False}


def fonts(roots: tuple[Path, ...], user_root: Path, deadline: float) -> dict[str, str]:
    if user_root.exists() and any(user_root.iterdir()):
        raise ObservationError("user-installed guest fonts are not part of the canonical environment")
    result = {}
    for root in roots:
        if not root.is_dir() or root.is_symlink():
            raise ObservationError("missing/symlink guest font root")
        for directory, subdirs, files in os.walk(root, followlinks=False):
            for name in subdirs:
                if (Path(directory) / name).is_symlink():
                    raise ObservationError("symlink font directory")
            for name in files:
                path = Path(directory) / name
                if time.monotonic() >= deadline:
                    raise ObservationError("guest font inventory deadline")
                if path.is_symlink() or not path.is_file():
                    raise ObservationError("nonregular guest font entry")
                # Hash all regular files, including supplemental catalogs. Never
                # follow user aliases, convert fonts or infer identity from OS alone.
                if path.stat().st_size > 256 * 1024 * 1024:
                    raise ObservationError("guest font file exceeds bound")
                digest = hashlib.sha256()
                with path.open("rb") as handle:
                    while block := handle.read(65536):
                        if time.monotonic() >= deadline:
                            raise ObservationError("guest font hashing deadline")
                        digest.update(block)
                result[str(path)] = digest.hexdigest()
    if not result:
        raise ObservationError("empty guest font inventory")
    return dict(sorted(result.items()))


def observe() -> dict:
    """Actual commands must execute in the throwaway guest's admin session."""
    if os.environ.get("TZ") != "UTC" or os.environ.get("LC_ALL") != "C":
        raise ObservationError("explicit UTC/C guest process environment required")
    if Path.home() != Path("/Users/admin"):
        raise ObservationError("not the disposable guest admin home")
    deadline = time.monotonic() + 60

    def command(*argv: str) -> bytes:
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise ObservationError("guest environment deadline")
        result = subprocess.run(argv, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                check=False, timeout=remaining)
        if result.returncode or len(result.stdout) > 16 * 1024 * 1024:
            raise ObservationError(f"guest probe failed: {argv[0]}")
        return result.stdout

    def text(*argv: str) -> str:
        value = command(*argv).decode().strip()
        if not value:
            raise ObservationError("empty guest probe observation")
        return value

    state = console(command("/usr/sbin/ioreg", "-a", "-n", "Root", "-d1"))
    command("/usr/bin/pgrep", "-x", "WindowServer")
    displays = json.loads(command("/usr/sbin/system_profiler", "-json", "SPDisplaysDataType"))
    return {
        "schema": 1, "origin": "tart-guest",
        "os": text("/usr/bin/sw_vers", "-productVersion"),
        "os_build": text("/usr/bin/sw_vers", "-buildVersion"),
        "arch": text("/usr/bin/uname", "-m"),
        "model": text("/usr/sbin/sysctl", "-n", "hw.model"),
        "displays": displays["SPDisplaysDataType"],
        "fonts": fonts((Path("/System/Library/Fonts"), Path("/Library/Fonts")),
                       Path.home() / "Library/Fonts", deadline),
        **state, "window_server": True, "timezone": "UTC", "locale": "C",
    }
