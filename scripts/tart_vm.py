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

import os
import re
import subprocess
import time
from dataclasses import dataclass
from pathlib import Path


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

    def cleanup(self) -> None:
        records: list[str] = []
        if self.base_disk:
            result = run(["hdiutil", "detach", self.base_disk], check=False)
            records.append(f"detach_rc={result.returncode}")
            self.base_disk = None
        records.append(f"stop_rc={run(['tart', 'stop', self.vm], check=False).returncode}")
        if self.tart_process and self.tart_process.poll() is None:
            self.tart_process.terminate()
            try:
                self.tart_process.wait(timeout=20)
            except subprocess.TimeoutExpired:
                self.tart_process.kill()
                self.tart_process.wait(timeout=10)
        if self.keep_vm:
            records.append("delete=skipped_keep_vm")
        else:
            result = run(["tart", "delete", self.vm], check=False)
            records.append(f"delete_rc={result.returncode}")
            if result.returncode == 0:
                (self.artifacts / "runner_ed25519").unlink(missing_ok=True)
                records.append("private_key=deleted")
        try:
            (self.artifacts / "cleanup.log").write_text("\n".join(records) + "\n")
        except OSError:
            pass


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


def inject_key(clone: Clone) -> tuple[Path, str]:
    """Write a fresh key into the stopped clone's `admin` account."""
    key = clone.artifacts / "runner_ed25519"
    run(["ssh-keygen", "-q", "-t", "ed25519", "-N", "", "-C", clone.vm, "-f", str(key)])
    os.chmod(key, 0o600)
    data = _attach_data_volume(clone, Path.home() / ".tart/vms" / clone.vm / "disk.img")
    auth = data / "Users/admin/.ssh/authorized_keys"
    auth.parent.mkdir(parents=True, exist_ok=True)
    existing = auth.read_text() if auth.exists() else ""
    public = key.with_suffix(".pub").read_text().strip()
    if public not in existing.splitlines():
        with auth.open("a") as handle:
            handle.write(public + "\n")
    os.chmod(auth.parent, 0o700)
    os.chmod(auth, 0o600)
    os.chown(auth.parent, 501, 20)
    os.chown(auth, 501, 20)
    run(["diskutil", "unmount", str(data)])
    run(["hdiutil", "detach", clone.base_disk])
    clone.base_disk = None
    return key, run(["ssh-keygen", "-lf", str(key.with_suffix(".pub"))]).stdout.strip()


def ssh_args(key: Path, known_hosts: Path, ip: str) -> list[str]:
    options = [
        "IdentitiesOnly=yes",
        "BatchMode=yes",
        "StrictHostKeyChecking=accept-new",
        f"UserKnownHostsFile={known_hosts}",
        "ConnectTimeout=5",
        "ServerAliveInterval=10",
        "ServerAliveCountMax=3",
    ]
    return ["ssh", "-i", str(key), *[part for option in options for part in ("-o", option)], f"admin@{ip}"]


def guest_script(ssh: list[str], script: str, *, timeout: int = 600) -> subprocess.CompletedProcess[str]:
    return run([*ssh, "/bin/zsh", "-s"], input_text=script, check=False, timeout=timeout)


def boot(
    source_vm: str, vm: str, artifacts: Path, keep_vm: bool, display: str, shares: list[str]
) -> tuple[Clone, list[str], str]:
    """Clone `source_vm`, boot it headless with the read-only `shares`
    (`name:path:ro,tag=…`), and return the clone, the SSH command and the
    key fingerprint once key-only SSH answers."""
    clone = Clone(vm, artifacts, keep_vm)
    run(["tart", "clone", source_vm, vm])
    try:
        run(["tart", "set", vm, "--cpu", "8", "--memory", "16384", "--display", display,
             "--no-display-refit", "--random-mac"])
        key, fingerprint = inject_key(clone)
        log = (artifacts / "tart-run.log").open("w")
        clone.tart_process = subprocess.Popen(
            ["tart", "run", "--no-graphics", "--no-audio", "--no-clipboard",
             *[f"--dir={share}" for share in shares], vm],
            stdout=log, stderr=subprocess.STDOUT, text=True, start_new_session=True,
        )
        ip = wait_until("Tart NAT IP", 240, lambda: run(["tart", "ip", vm], check=False).stdout.strip())
        ssh = ssh_args(key, artifacts / "known_hosts", ip)
        wait_until("key-only SSH", 300,
                   lambda: "ready" if run([*ssh, "true"], check=False).returncode == 0 else "")
    except BaseException:
        clone.cleanup()
        raise
    return clone, ssh, f"vm={vm} ip={ip} key={fingerprint}"
