"""Cooperative cancellation protocol for the host's narrow Tart owner.

Used inside the upstream-owned process group. Only Tart itself may be detached
by the shared lifecycle. A terminal acknowledgement is written after operation
unwinding AND the caller's concrete cleanup verifier, never in an atexit guess.
There is no capture/VM CLI here and no signal is sent to a saved numeric PID.
"""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import re
import signal
import stat
import tempfile
import threading
import time

from vm_lifecycle import Cancelled, _signals


class ControlError(RuntimeError):
    pass


def _unique(pairs):
    value = {}
    for key, item in pairs:
        if key in value:
            raise ControlError("duplicate control JSON key")
        value[key] = item
    return value


def _read(path: Path) -> tuple[bytes, dict]:
    if path.resolve() != path or path.is_symlink():
        raise ControlError("canonical control path required")
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(fd, "rb") as handle:
        meta = os.fstat(handle.fileno())
        # Atomic no-replace publishers may briefly use two hard links. The
        # private path and request digest, not transient link count, bind data.
        if not stat.S_ISREG(meta.st_mode) or meta.st_size > 16384:
            raise ControlError("invalid control file")
        raw = handle.read(16385)
    if len(raw) > 16384:
        raise ControlError("control file grew beyond bound")
    value = json.loads(raw, object_pairs_hook=_unique)
    if not isinstance(value, dict):
        raise ControlError("control object required")
    return raw, value


def _publish(path: Path, value: dict) -> None:
    fd, name = tempfile.mkstemp(prefix=".terminal-", dir=path.parent)
    temporary = Path(name)
    try:
        with os.fdopen(fd, "w") as handle:
            json.dump(value, handle, sort_keys=True)
            handle.flush()
            os.fsync(handle.fileno())
        # Atomic no-replace publication. Remove only this private temporary link.
        os.link(temporary, path)
    finally:
        temporary.unlink()


def run(control: Path, request_sha256: str, operation, verify_cleanup):
    """Run the lifecycle callable and acknowledge only observed clean teardown.

    verify_cleanup must check that the owned VM/key/disk/process are absent.
    Exceptions with a clean teardown are acknowledged as failed/cancelled so
    the host may release its gate, but never seal capture evidence.
    """
    if (not control.is_absolute() or control.resolve() != control or not control.is_dir()
            or control.stat().st_mode & 0o077 or (control / "terminal.json").exists()):
        raise ControlError("private canonical unfinished control directory required")
    raw, request = _read(control / "request.json")
    if (not re.fullmatch("[0-9a-f]{64}", request_sha256)
            or hashlib.sha256(raw).hexdigest() != request_sha256
            or set(request) != {"schema", "control", "payload_sha256", "operation_ms", "grace_ms"}
            or type(request["schema"]) is not int or request["schema"] != 1
            or request["control"] != str(control)
            or not isinstance(request["payload_sha256"], str)
            or not re.fullmatch("[0-9a-f]{64}", request["payload_sha256"])
            or any(type(request[k]) is not int or not 0 < request[k] <= 86_400_000
                   for k in ("operation_ms", "grace_ms"))):
        raise ControlError("control request identity/limits differ")
    if threading.current_thread() is not threading.main_thread():
        raise ControlError("Tart owner must run on the main thread")
    stopped = threading.Event()
    errors = []
    deadline = time.monotonic() + request["operation_ms"] / 1000

    def monitor():
        try:
            while not stopped.wait(0.01):
                cancel = control / "cancel.json"
                if cancel.exists() or cancel.is_symlink():
                    _, value = _read(cancel)
                    if (set(value) != {"schema", "request_sha256", "reason"}
                            or type(value["schema"]) is not int or value["schema"] != 1
                            or value["request_sha256"] != request_sha256
                            or value["reason"] not in {"cancelled", "deadline"}):
                        raise ControlError("cancel request identity differs")
                    break
                if time.monotonic() >= deadline:
                    break
            else:
                return
        except BaseException as error:
            errors.append(error)
        if not stopped.is_set():
            # The handler is installed before this thread starts. Signal the
            # current process only, never a detached child or stale PID.
            os.kill(os.getpid(), signal.SIGTERM)

    with _signals():
        worker = threading.Thread(target=monitor, name="visual-cancellation", daemon=True)
        failure, result = None, None
        try:
            worker.start()
            result = operation()
        except BaseException as error:
            failure = error
        finally:
            # Further cancellation cannot interrupt cleanup verification or
            # publication. The Rust hard deadline remains the ultimate bound.
            for sig in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
                signal.signal(sig, signal.SIG_IGN)
            stopped.set()
            if worker.ident is not None:
                worker.join(timeout=1)
        if worker.is_alive() or errors:
            raise ControlError("cancellation monitor failed; cleanup unacknowledged") from (errors[0] if errors else None)
        verify_cleanup()
        current, _ = _read(control / "request.json")
        if hashlib.sha256(current).hexdigest() != request_sha256:
            raise ControlError("control request changed during operation")
        _publish(control / "terminal.json", {
            "schema": 1, "request_sha256": request_sha256,
            "outcome": "cancelled" if isinstance(failure, Cancelled) else "failed" if failure else "completed",
        })
        if failure:
            raise failure
        return result
