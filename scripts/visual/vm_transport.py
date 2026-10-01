"""Internal Tart transport composition; no capture CLI or baseline authority.

The Rust caller owns the process group, leases and original request/payload
identity. This module owns only direct SSH children and the existing clone
lifecycle. Tests substitute resource access, never launch SSH/Tart/native code.
"""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time

import vm_bootstrap
import vm_control
import vm_evidence
import vm_install
import vm_lifecycle
import vm_transfer

LOG_LIMIT = 16 * 1024 * 1024
ARCHIVE_LIMIT = vm_transfer.TOTAL_LIMIT + vm_transfer.MAX_FILES * 1024 + vm_transfer.MANIFEST_LIMIT


class TransportError(RuntimeError):
    pass


def _ssh(session) -> list[str]:
    """Only the lifecycle's exact isolated command; no caller-supplied switches."""
    from vm_policy import validate_ip
    ip = validate_ip(session.ip)
    expected = vm_lifecycle.tart_vm.ssh_args(session.clone.artifacts / "runner_ed25519",
                                            session.clone.artifacts / "known_hosts",
                                            ip, isolated=True)
    if session.ssh != expected:
        raise TransportError("isolated lifecycle SSH command differs")
    return expected


def _execute(argv: list[str], stdout: Path, stderr: Path, *, script: str | None,
             seconds: float, output_limit: int) -> None:
    """Bound direct SSH output with files (no pipe deadlock or detached group).

    Rust owns whole-group hard cancellation; remote processes are finally bounded
    by destruction of their one owned VM. Local SSH exit alone proves neither
    native producer reaping nor remote cleanup.
    """
    for path in (stdout, stderr):
        vm_install._absolute(path)
        if path.exists() or path.is_symlink():
            raise TransportError("fresh transport diagnostics required")
    with tempfile.TemporaryFile() as source, stdout.open("xb") as out, stderr.open("xb") as err:
        os.fchmod(out.fileno(), 0o600)
        os.fchmod(err.fileno(), 0o600)
        if script is not None:
            raw = script.encode()
            if len(raw) > 256 * 1024:
                raise TransportError("bootstrap input exceeds bound")
            source.write(raw)
            source.seek(0)
        # Do not inherit SSH_AUTH_SOCK, DISPLAY/askpass, DYLD or proxy variables.
        # Existing route already disables config/agent/forwarding/multiplexing.
        env = {"HOME": str(Path.home()), "PATH": "/usr/bin:/bin:/usr/sbin:/sbin",
               "TZ": "UTC", "LANG": "C", "LC_ALL": "C"}
        child = subprocess.Popen(argv, stdin=source, stdout=out, stderr=err, env=env)
        try:
            deadline = time.monotonic() + seconds
            while child.poll() is None:
                if time.monotonic() >= deadline:
                    raise TransportError("SSH transport deadline")
                if os.fstat(out.fileno()).st_size > output_limit or os.fstat(err.fileno()).st_size > LOG_LIMIT:
                    raise TransportError("SSH transport output limit")
                try:
                    child.wait(timeout=min(0.02, max(0.001, deadline - time.monotonic())))
                except subprocess.TimeoutExpired:
                    pass
            if (child.returncode != 0 or os.fstat(out.fileno()).st_size > output_limit
                    or os.fstat(err.fileno()).st_size > LOG_LIMIT):
                raise TransportError(f"SSH transport failed: rc={child.returncode}; see {stderr}")
        finally:
            if child.poll() is None:
                child.terminate()
                try:
                    child.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    child.kill()
                    child.wait(timeout=5)
                    raise TransportError("SSH hard termination; remote cleanup requires VM teardown")


def _exchange(session, script: str, workspace: Path) -> None:
    ssh = _ssh(session)
    ssh = ["/usr/bin/ssh", *ssh[1:]]
    _execute([*ssh, "/bin/zsh", "-f", "-s"], workspace / "bootstrap.stdout",
             workspace / "bootstrap.stderr", script=script, seconds=960, output_limit=LOG_LIMIT)
    # Never scp a caller-selected path or run a producer through a second runner.
    # The archive is already written/closed by the fixed guest producer chain.
    _execute([*ssh, "/usr/bin/env", "-i", "PATH=/usr/bin:/bin", "/bin/cat",
              str(vm_bootstrap.WORK / "return.tar")], workspace / "return.tar",
             workspace / "return.stderr", script=None, seconds=120, output_limit=ARCHIVE_LIMIT)


def run(share: Path, digest: str, repo: Path, suite: str, workspace: Path, vm: str,
        control: Path, request_sha256: str) -> dict:
    """Called by vm_supervision only, after external network approval.

    No retries or runtime installation. Payload/build/coverage/seal authority stays
    in Rust. The terminal acknowledgement cannot precede archive validation and
    observed cleanup; returned candidates are still unsealed staging diagnostics.
    """
    for path in (share, repo, workspace, control):
        vm_install._absolute(path)
    for left, right in ((share, repo), (share, workspace), (repo, workspace)):
        if left == right or left in right.parents or right in left.parents:
            raise TransportError("transport workspace/payload/repo must be disjoint")
    if (not workspace.is_dir() or workspace.stat().st_mode & 0o077
            or control != workspace / "control" or set(p.name for p in workspace.iterdir()) != {"control", "vm"}):
        raise TransportError("fresh private transport workspace required")
    request, fields = vm_control._read(control / "request.json")
    if (hashlib.sha256(request).hexdigest() != request_sha256 or fields.get("payload_sha256") != digest):
        raise TransportError("transport request payload/identity differs")
    payload = vm_install.inspect(share, digest, repo, suite)
    script = vm_bootstrap.plan(share, digest, repo, suite)
    binding = vm_evidence.Binding(workspace / "vm", vm, request_sha256, digest)
    binding.fields()

    def operation():
        with vm_lifecycle.session(share, binding.artifacts, binding=binding) as session:
            _exchange(session, script, workspace)
            vm_transfer.inspect(workspace / "return.tar", digest)
            if vm_install.inspect(share, digest, repo, suite) != payload:
                raise TransportError("payload changed during SSH execution")
        # Lifecycle has unwound. Commands succeeding is not proof: re-observe
        # owned resource absence before exposing raw files even as diagnostics.
        vm_evidence.verify(binding)
        manifest = vm_transfer.unpack(workspace / "return.tar", workspace / "raw", digest)
        return {"schema": 1, "payload_sha256": digest, "raw": str(workspace / "raw"),
                "files": manifest["files"]}

    return vm_control.run(control, request_sha256, operation, lambda: vm_evidence.verify(binding))
