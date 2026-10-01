"""Observe owned Tart resource absence after lifecycle cleanup, never repair it.

Only the exact recorded clone is examined. No delete/stop/retry, key contents,
foreign worktree or numeric-PID signaling. Unknown observations fail closed.
"""
from __future__ import annotations

import json
import os
from pathlib import Path
import plistlib
import re
import socket

from vm_lifecycle import tart_vm
from vm_policy import validate_ip


class CleanupError(RuntimeError):
    pass


def validate_inventory(raw: str, name: str) -> None:
    if len(raw) > 8 * 1024 * 1024:
        raise CleanupError("Tart inventory exceeds bound")
    inventory = json.loads(raw)
    if not isinstance(inventory, list):
        raise CleanupError("unknown Tart inventory schema")
    names = set()
    for entry in inventory:
        if (not isinstance(entry, dict) or not isinstance(entry.get("Name"), str)
                or entry.get("Source") != "local" or type(entry.get("Running")) is not bool
                or entry.get("State") not in {"running", "stopped", "suspended"}
                or entry["Name"] in names):
            raise CleanupError("incomplete/ambiguous Tart inventory")
        names.add(entry["Name"])
    if name in names:
        raise CleanupError("owned clone still in Tart inventory")


def validate_mounts(raw: str, disk_image: Path) -> None:
    if len(raw) > 16 * 1024 * 1024:
        raise CleanupError("mounted image inventory exceeds bound")
    value = plistlib.loads(raw.encode())
    if not isinstance(value, dict) or not isinstance(value.get("images"), list):
        raise CleanupError("unknown mounted image inventory")
    for entry in value["images"]:
        if not isinstance(entry, dict) or not isinstance(entry.get("image-path"), str):
            raise CleanupError("missing mounted image identity")
        # Compare canonical paths, not just basename/device ids, which may be
        # reused after detach. The clone path itself must already be absent.
        path = Path(entry["image-path"])
        if not path.is_absolute():
            raise CleanupError("nonabsolute mounted image identity")
        if path.resolve() == disk_image:
            raise CleanupError("owned clone disk image still attached")


def observe(clone, ip: str) -> dict:
    """Must run after Clone.cleanup returned successfully, within cleanup grace."""
    if not re.fullmatch(r"bbr-visual-[0-9]+-[0-9a-f]{12}", clone.vm):
        raise CleanupError("invalid owned visual clone identity")
    if clone.cleaned is not True or clone.keep_vm or clone.base_disk is not None:
        raise CleanupError("lifecycle cleanup incomplete")
    ip = validate_ip(ip)
    process = clone.tart_process
    if process is None or process.poll() is None:
        raise CleanupError("owned Tart process not reaped")
    _absence(clone.vm, clone.artifacts, ip)
    return {"schema": 1, "vm": clone.vm, "ip": ip, "tart_pid": process.pid,
            "tart_exit": process.returncode, "clone_absent": True, "disk_detached": True,
            "private_key_absent": True, "owned_process_reaped": True, "ssh_unreachable": True}


def recheck(vm: str, artifacts: Path, ip: str, pid: int) -> None:
    """Independent observer: only read resources, never signal a recorded PID."""
    if (not re.fullmatch(r"bbr-visual-[0-9]+-[0-9a-f]{12}", vm)
            or type(pid) is not int or not 1 < pid <= 2**31 - 1):
        raise CleanupError("invalid recorded resource identity")
    ip = validate_ip(ip)
    _process_absent(pid)
    _absence(vm, artifacts, ip)
    _process_absent(pid)


def _process_absent(pid: int) -> None:
    try:
        os.kill(pid, 0)  # Observation only. Reused PID is ambiguity, never success.
    except ProcessLookupError:
        return
    raise CleanupError("recorded Tart process still exists or PID was reused")


def _absence(vm: str, artifacts: Path, ip: str) -> None:
    key = artifacts / "runner_ed25519"
    if key.exists() or key.is_symlink():
        raise CleanupError("run-specific private key remains")
    directory = Path.home() / ".tart/vms" / vm
    if directory.exists() or directory.is_symlink():
        raise CleanupError("owned clone directory remains")
    disk_image = directory.resolve() / "disk.img"
    inventory = tart_vm.run(["tart", "list", "--source", "local", "--format", "json"], timeout=15).stdout
    validate_inventory(inventory, vm)
    mounts = tart_vm.run(["hdiutil", "info", "-plist"], timeout=15).stdout
    validate_mounts(mounts, disk_image)
    try:
        with socket.create_connection((ip, 22), timeout=2):
            pass
    except (ConnectionRefusedError, TimeoutError):
        pass
    else:
        raise CleanupError("removed guest SSH still reachable")
