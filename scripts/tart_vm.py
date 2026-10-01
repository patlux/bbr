"""Throwaway Tart macOS VM for the BBR E2E suites (see `scripts/e2e_vm.py`).

A copy-on-write clone of the golden image boots with `--no-graphics` on Tart
NAT only (no bridged interface, no port forwarding): the guest keeps its own
auto-logged-in WindowServer session, so no test window ever appears on the
host or takes its focus. A run-specific Ed25519 key is written into the
clone's offline Data volume and is the only way in; the clone and the key
are deleted when the run ends (unless `--keep-vm`).

Adapted from the unmerged `parallel/tart-e2e-runner` branch.
"""

from __future__ import annotations

import ipaddress
import os
import re
import subprocess
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Callable


class RunnerError(RuntimeError):
    pass


def run(
    args: list[str],
    *,
    check: bool = True,
    input_text: str | None = None,
    env: dict[str, str] | None = None,
    timeout: int = 600,
) -> subprocess.CompletedProcess[str]:
    try:
        result = subprocess.run(
            args,
            check=False,
            capture_output=True,
            text=True,
            input=input_text,
            env=env,
            timeout=timeout,
        )
    except subprocess.TimeoutExpired as exc:
        raise RunnerError(f"command timed out after {timeout}s: {' '.join(args)}") from exc
    if check and result.returncode != 0:
        output = "\n".join(part for part in (result.stdout, result.stderr) if part)
        raise RunnerError(f"command failed ({result.returncode}): {' '.join(args)}\n{output}")
    return result


@dataclass
class Clone:
    """One throwaway clone; `cleanup` stops and deletes it with its key."""

    vm: str
    artifacts: Path
    keep_vm: bool
    tart_process: subprocess.Popen[str] | None = None
    base_disk: str | None = None
    cleaned: bool = False

    def cleanup(self) -> None:
        """Attempt every owned-resource cleanup even after a command failure.

        Explicit callers must treat failure as unfinished evidence, not a clean
        capture. Successful repeated calls (e.g. finally + atexit) are no-ops.
        Keep the per-run key on deletion failure for recovery; never read it.
        """
        if self.cleaned:
            return
        records: list[str] = []
        errors: list[str] = []

        def attempt(label: str, argv: list[str]) -> bool:
            try:
                result = run(argv, check=False, timeout=30)
                records.append(f"{label}_rc={result.returncode}")
                if result.returncode == 0:
                    return True
                errors.append(f"{label} returned {result.returncode}")
            except (RunnerError, OSError) as exc:
                records.append(f"{label}_error={type(exc).__name__}")
                errors.append(f"{label}: {type(exc).__name__}")
            return False

        if self.base_disk and attempt("detach", ["hdiutil", "detach", self.base_disk]):
            self.base_disk = None
        attempt("stop", ["tart", "stop", self.vm])
        try:
            if self.tart_process and self.tart_process.poll() is None:
                self.tart_process.terminate()
                try:
                    self.tart_process.wait(timeout=20)
                except subprocess.TimeoutExpired:
                    self.tart_process.kill()
                    self.tart_process.wait(timeout=10)
            records.append("owned_process=reaped")
        except (OSError, subprocess.TimeoutExpired) as exc:
            records.append(f"owned_process_error={type(exc).__name__}")
            errors.append("owned Tart process cleanup failed")
        if self.keep_vm:
            records.append("delete=skipped_keep_vm")
        elif attempt("delete", ["tart", "delete", self.vm]):
            try:
                (self.artifacts / "runner_ed25519").unlink(missing_ok=True)
                records.append("private_key=deleted")
            except OSError:
                records.append("private_key=delete_failed")
                errors.append("run-specific key cleanup failed")
        try:
            log = self.artifacts / "cleanup.log"
            previous = log.read_text() if log.exists() else ""
            log.write_text(previous + "\n".join(records) + "\n")
        except OSError:
            errors.append("cleanup evidence could not be written")
        if errors:
            raise RunnerError("incomplete VM cleanup: " + "; ".join(errors))
        self.cleaned = True


def wait_until(label: str, seconds: int, probe) -> str:
    deadline = time.monotonic() + seconds
    last = ""
    while time.monotonic() < deadline:
        try:
            value = probe()
            if value:
                return value
        except (RunnerError, OSError) as exc:
            last = str(exc)
        time.sleep(2)
    raise RunnerError(f"deadline exceeded waiting for {label}: {last}")


def _attach_data_volume(clone: Clone, disk_image: Path) -> Path:
    attached = run(
        ["hdiutil", "attach", "-nomount", "-imagekey", "diskimage-class=CRawDiskImage", str(disk_image)]
    ).stdout
    clone.base_disk = attached.splitlines()[0].split()[0]
    for disk in re.findall(r"^(/dev/disk\d+)\s", attached, re.MULTILINE):
        listing = run(["diskutil", "apfs", "list", disk], check=False).stdout
        match = re.search(r"Volume Disk \(Role\):\s+(disk\d+s\d+)\s+\(Data\)", listing)
        if not match:
            continue
        run(["diskutil", "mount", match.group(1)])
        info = run(["diskutil", "info", match.group(1)]).stdout
        mount = re.search(r"Mount Point:\s+(.+)", info)
        if not mount:
            raise RunnerError("mounted Data volume has no mount point")
        return Path(mount.group(1).strip())
    raise RunnerError("clone Data volume not found")


def inject_key(
    clone: Clone, *, exclusive_key: bool = False,
    prepare_data: Callable[[Path], None] | None = None,
) -> tuple[Path, str]:
    """Write a fresh key into the stopped clone's `admin` account.

    A consumer can harden only this offline clone before its first boot; the
    default E2E path retains its existing policy. Never modify the source image.
    """
    key = clone.artifacts / "runner_ed25519"
    run(["ssh-keygen", "-q", "-t", "ed25519", "-N", "", "-C", clone.vm, "-f", str(key)])
    os.chmod(key, 0o600)
    data = _attach_data_volume(clone, Path.home() / ".tart/vms" / clone.vm / "disk.img")
    auth = data / "Users/admin/.ssh/authorized_keys"
    auth.parent.mkdir(parents=True, exist_ok=True)
    public = key.with_suffix(".pub").read_text().strip()
    if exclusive_key:
        auth.write_text(public + "\n")
    else:
        existing = auth.read_text() if auth.exists() else ""
        if public not in existing.splitlines():
            with auth.open("a") as handle:
                handle.write(public + "\n")
    os.chmod(auth.parent, 0o700)
    os.chmod(auth, 0o600)
    os.chown(auth.parent, 501, 20)
    os.chown(auth, 501, 20)
    if prepare_data is not None:
        prepare_data(data)
    run(["diskutil", "unmount", str(data)])
    run(["hdiutil", "detach", clone.base_disk])
    clone.base_disk = None
    return key, run(["ssh-keygen", "-lf", str(key.with_suffix(".pub"))]).stdout.strip()


def ssh_args(key: Path, known_hosts: Path, ip: str, *, isolated: bool = False) -> list[str]:
    options = [
        "IdentitiesOnly=yes",
        "BatchMode=yes",
        "StrictHostKeyChecking=accept-new",
        f"UserKnownHostsFile={known_hosts}",
        "ConnectTimeout=5",
        "ServerAliveInterval=10",
        "ServerAliveCountMax=3",
    ]
    if isolated:
        options.extend(["IdentityAgent=none", "ClearAllForwardings=yes", "ForwardAgent=no",
                        "ProxyCommand=none", "ProxyJump=none", "ControlMaster=no",
                        "ControlPath=none", "ControlPersist=no"])
    return ["ssh", *(["-F", "/dev/null"] if isolated else []), "-i", str(key),
            *[part for option in options for part in ("-o", option)], f"admin@{ip}"]


def guest_script(ssh: list[str], script: str, *, timeout: int = 600) -> subprocess.CompletedProcess[str]:
    return run([*ssh, "/bin/zsh", "-s"], input_text=script, check=False, timeout=timeout)


def boot(
    source_vm: str, vm: str, artifacts: Path, keep_vm: bool, display: str, shares: list[str],
    *, exclusive_key: bool = False, prepare_data: Callable[[Path], None] | None = None,
) -> tuple[Clone, list[str], str]:
    """Clone `source_vm`, boot it headless with the read-only `shares`
    (`name:path:ro,tag=…`), and return the clone, the SSH command and the
    key fingerprint once key-only SSH answers."""
    clone = Clone(vm, artifacts, keep_vm)
    run(["tart", "clone", source_vm, vm])
    try:
        run(["tart", "set", vm, "--cpu", "8", "--memory", "16384", "--display", display,
             "--no-display-refit", "--random-mac"])
        key, fingerprint = inject_key(clone, exclusive_key=exclusive_key, prepare_data=prepare_data)
        with (artifacts / "tart-run.log").open("w") as log:
            clone.tart_process = subprocess.Popen(
                ["tart", "run", "--no-graphics", "--no-audio", "--no-clipboard",
                 *[f"--dir={share}" for share in shares], vm],
                stdout=log, stderr=subprocess.STDOUT, text=True, start_new_session=True,
            )
        ip = wait_until("Tart NAT IP", 240, lambda: run(["tart", "ip", vm], check=False).stdout.strip())
        if exclusive_key:
            address = ipaddress.IPv4Address(ip)
            if not any(address in ipaddress.IPv4Network(net) for net in
                       ("10.0.0.0/8", "172.16.0.0/12", "192.168.0.0/16")):
                raise RunnerError("exclusive clone SSH requires a private NAT address")
        ssh = ssh_args(key, artifacts / "known_hosts", ip, isolated=exclusive_key)
        wait_until("key-only SSH", 300,
                   lambda: "ready" if run([*ssh, "true"], check=False).returncode == 0 else "")
    except BaseException:
        clone.cleanup()
        raise
    return clone, ssh, f"vm={vm} ip={ip} key={fingerprint}"
