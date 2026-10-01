"""Narrow visual consumer of the existing Tart clone/boot/cleanup lifecycle.

No CLI entrypoint: the Rust capture guard stays closed until the VM pilot is
explicitly authorized and integrated. Tests inject the transport, never boot.
"""
from __future__ import annotations

from contextlib import contextmanager
from dataclasses import dataclass
import os
from pathlib import Path
import signal
import socket
import sys
import uuid

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import tart_vm
from vm_policy import harden_offline, validate_ip, validate_listeners, validate_sshd


class Cancelled(BaseException):
    pass


@dataclass(frozen=True)
class Session:
    clone: tart_vm.Clone
    ssh: list[str]
    ip: str
    audit: Path


def _guest(ssh: list[str], script: str, log: Path) -> str:
    result = tart_vm.guest_script(ssh, script, timeout=30)
    log.write_text(result.stdout + result.stderr)
    if result.returncode:
        raise tart_vm.RunnerError(f"guest audit failed ({result.returncode}); see {log}")
    return result.stdout


def audit(ssh: list[str], ip: str, artifacts: Path) -> None:
    """Observe effective authentication and TCP listeners before any producer."""
    effective = _guest(ssh, "set -eu\nsudo /usr/sbin/sshd -T -C user=admin,host=localhost,addr=127.0.0.1\n",
                       artifacts / "sshd-audit.log")
    validate_sshd(effective)
    listeners = _guest(ssh, "set -eu\nsudo /usr/sbin/lsof -nP -iTCP -sTCP:LISTEN -Fpcn\n",
                       artifacts / "listeners-audit.log")
    validate_listeners(listeners)
    results = []
    for port in (22, 88, 5900):
        try:
            with socket.create_connection((ip, port), timeout=2):
                reachable = True
        except (ConnectionRefusedError, TimeoutError):
            reachable = False
        results.append(f"{ip}:{port} reachable={reachable}")
        (artifacts / "network-audit.log").write_text("\n".join(results) + "\n")
        if reachable != (port == 22):
            raise tart_vm.RunnerError(f"unexpected host-to-guest reachability: TCP {port}")


@contextmanager
def _signals():
    """Deliver cancellation through finally, then shield bounded cleanup."""
    previous = {}

    def interrupt(number, _frame):
        # Further INT/TERM/HUP must not interrupt owned-resource cleanup.
        for sig in previous:
            signal.signal(sig, signal.SIG_IGN)
        raise Cancelled(f"visual VM cancelled by signal {number}")

    try:
        for sig in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
            previous[sig] = signal.signal(sig, interrupt)
        yield
    finally:
        for sig, handler in previous.items():
            signal.signal(sig, handler)


@contextmanager
def session(payload: Path, artifacts: Path):
    """Own exactly one headless NAT clone. Called only after external approval.

    Share only the already verified immutable payload, read-only. Never accept
    user-provided source VM names, forwarding switches or a keep-vm shortcut.
    """
    for path in (payload, artifacts):
        if not path.is_absolute() or path.resolve() != path or any(c in str(path) for c in ":,\n\r"):
            raise tart_vm.RunnerError("canonical path without Tart share delimiters required")
        if not path.is_dir():
            raise tart_vm.RunnerError("existing payload and private diagnostics directories required")
    if payload == artifacts or payload in artifacts.parents or artifacts in payload.parents:
        raise tart_vm.RunnerError("payload and diagnostics must be disjoint")
    if artifacts.stat().st_mode & 0o077 or any(artifacts.iterdir()):
        raise tart_vm.RunnerError("fresh private diagnostics directory required")
    if not (payload / "payload.json").is_file():
        raise tart_vm.RunnerError("completed payload required before clone creation")
    name = f"bbr-visual-{os.getpid()}-{uuid.uuid4().hex[:12]}"
    with _signals():
        clone, ssh, banner = tart_vm.boot(
            "macos-27-golden-gate", name, artifacts, False, "3200x1800",
            [f"visual:{payload}:ro,tag=bbr-visual-payload"],
            exclusive_key=True, prepare_data=harden_offline,
        )
        try:
            ip = validate_ip(ssh[-1].removeprefix("admin@"))
            # Do not inherit a personal SSH config, agent, proxy, multiplexed
            # connection or forwarding. Initial boot SSH uses the same options.
            (artifacts / "session.log").write_text(banner + "\n")
            audit(ssh, ip, artifacts)
            yield Session(clone, ssh, ip, artifacts)
        finally:
            # Cleanup failure propagates and must prevent capture sealing.
            # No retry of writes, no host capture fallback.
            for sig in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
                signal.signal(sig, signal.SIG_IGN)
            clone.cleanup()
