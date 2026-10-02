"""Verified guest bootstrap tail; only fixed compiled-path parents are created."""
import os
from pathlib import Path
import pwd
import subprocess

import vm_guest
import vm_install
import vm_selection

REPO = Path("/Users/patwoz/dev/Personal/macos/bbr")


def identity():
    if (Path.home() != Path("/Users/admin") or pwd.getpwuid(os.getuid()).pw_name != "admin"
            or os.getuid() != 501):
        raise vm_guest.GuestError("bootstrap requires actual guest admin; host forbidden")
    result = subprocess.run(["/usr/sbin/sysctl", "-n", "hw.model"], check=True,
                            capture_output=True, text=True, timeout=5)
    if not result.stdout.strip().startswith("VirtualMac"):
        raise vm_guest.GuestError("bootstrap requires actual VirtualMac hardware")


def prepare_parent(repo):
    if repo != REPO:
        raise vm_guest.GuestError("bootstrap compiled repository differs")
    vm_install._absolute(repo)
    root = Path("/Users/patwoz")
    if root.exists() or root.is_symlink():
        raise vm_guest.GuestError("fresh compiled-path ancestor required; never take ownership")
    # No -p, recursive chown or existing-directory reuse. A partially completed
    # bootstrap is diagnostic only and this throwaway VM must not be resumed.
    subprocess.run(["/usr/bin/sudo", "-n", "/bin/mkdir", "-m", "700", str(root)],
                   check=True, timeout=10)
    subprocess.run(["/usr/bin/sudo", "-n", "/usr/sbin/chown", "501:20", str(root)],
                   check=True, timeout=10)
    vm_install._absolute(root)
    if root.stat().st_uid != 501 or root.stat().st_mode & 0o7777 != 0o700:
        raise vm_guest.GuestError("compiled-path ancestor ownership/mode differs")
    current = root
    for part in ("dev", "Personal", "macos"):
        current /= part
        current.mkdir(mode=0o700)


def run(share, digest, repo, suite, work, selected=None, selected_sha256=None):
    if suite == "gallery-semantic":
        raise vm_guest.GuestError("semantic guest execution is not wired")
    if suite not in {"gallery", "browser"}:
        raise vm_guest.GuestError("unsupported bootstrap suite")
    identity()  # Must precede source probes, sudo and writes.
    if repo != REPO:
        raise vm_guest.GuestError("bootstrap compiled repository differs")
    vm_selection.decode(selected, selected_sha256, digest, suite)
    vm_install.inspect(share, digest, repo, suite)
    if work.exists() or work.is_symlink() or not work.parent.is_dir():
        raise vm_guest.GuestError("fresh guest capture work required")
    vm_install._absolute(work)
    for left, right in ((share, repo), (share, work), (repo, work)):
        if left == right or left in right.parents or right in left.parents:
            raise vm_guest.GuestError("bootstrap source/share/work must be disjoint")
    prepare_parent(repo)
    argv = ["vm_guest.py", str(share), digest, str(repo), suite, str(work)]
    if selected is not None:
        argv.extend([selected, selected_sha256])
    vm_guest.main(argv)
