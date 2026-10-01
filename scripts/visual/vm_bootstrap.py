"""Generate fixed source-bound guest shell input; never execute it here.

Caller must hold the approved VM lifecycle/supervisor and verified payload. This
module is neither a CLI nor permission to boot, install runtimes or enable capture.
"""
import hashlib
import json
from pathlib import Path
import shlex

import vm_install
import vm_runtime

REPO = Path("/Users/patwoz/dev/Personal/macos/bbr")
MOUNT = Path("/Users/admin/bbr-visual-payload-mount")
SHARE = MOUNT / "visual"
WORK = Path("/Users/admin/bbr-visual-work")
MODULES = ("vm_install", "vm_observe", "vm_transfer", "vm_guest", "vm_bootstrap_guest")
NIX = "/nix/var/nix/profiles/default/bin/nix"
DELIMITER = "BBR_VISUAL_BOOTSTRAP"
ENV = "/usr/bin/env -i HOME=/Users/admin USER=admin LOGNAME=admin PATH=/usr/bin:/bin:/usr/sbin:/sbin TMPDIR=/private/tmp TZ=UTC LANG=C LC_ALL=C SEED=0"


class BootstrapError(ValueError):
    pass


def _source(share, payload, name):
    relative = "scripts/visual/" + name
    expected = payload["sources"].get(relative)
    if expected is None:
        raise BootstrapError("missing source-bound bootstrap helper: " + relative)
    handle, _ = vm_install._open(share / "source" / relative, 1024 * 1024)
    with handle:
        raw = handle.read(1024 * 1024 + 1)
    if hashlib.sha256(raw).hexdigest() != expected:
        raise BootstrapError("bootstrap helper bytes differ")
    return raw, expected


def plan(share: Path, digest: str, repo: Path, suite: str) -> str:
    """Pure/read-only: render shell only for the actual compiled checkout path."""
    if repo != REPO:
        raise BootstrapError("bootstrap compiled repository differs")
    payload = vm_install.inspect(share, digest, repo, suite)
    lock_raw, _ = _source(share, payload, "guest-runtime.json")
    lock = vm_runtime.validate(vm_runtime._json(lock_raw))
    loader, _ = _source(share, payload, "vm_bootstrap_loader.py")
    # Do not manufacture bootstrap authority from foreign payload source alone.
    # Current source identity is required, not an old payload or arbitrary code.
    names = (*MODULES, "vm_bootstrap_loader", "vm_bootstrap", "vm_runtime")
    modules = {}
    for name in names:
        raw, expected = _source(share, payload, name + ".py")
        local = Path(__file__).parent / (name + ".py")
        if raw != local.read_bytes():
            raise BootstrapError("bootstrap source differs from current host helper: " + name)
        if name in MODULES:
            modules[name] = expected
    binding = {"share": str(SHARE), "payload_sha256": digest, "repo": str(repo),
               "suite": suite, "work": str(WORK), "modules": modules}
    q = shlex.quote
    # Only literal fixed commands and validated/quoted source-bound values. No
    # interactive profiles, Xcode python3 placeholder, arbitrary argv or fallback.
    lines = [
        "set -euo pipefail", "umask 077",
        '[[ "$(/usr/bin/id -un)" == admin && "$(/usr/bin/id -u)" == 501 ]] || { print -u2 "bootstrap host forbidden"; exit 76; }',
        '[[ "$(/usr/sbin/sysctl -n hw.model)" == VirtualMac* ]] || { print -u2 "bootstrap host forbidden"; exit 76; }',
        f"[[ ! -e {q(str(MOUNT))} && ! -L {q(str(MOUNT))} && ! -e {q(str(WORK))} && ! -L {q(str(WORK))} ]]",
        # Actual NAR bytes, not merely local DB metadata. --offline rules out
        # substitution; hash does not install, repair or mutate store paths.
    ]
    for path, record in sorted(lock["paths"].items()):
        lines.append(f'[[ "$({NIX} --extra-experimental-features nix-command hash path --offline --type sha256 --sri {q(path)})" == {q(record["narHash"])} ]]')
    python = lock["python"]
    lines.extend([
        f'[[ "$(/usr/bin/shasum -a 256 {q(python["path"])})" == {q(python["sha256"] + "  " + python["path"])} ]]',
        f"/bin/mkdir -m 700 {q(str(MOUNT))}",
        f"/usr/bin/sudo -n /sbin/mount_virtiofs -r bbr-visual-payload {q(str(MOUNT))}",
        f"exec {ENV} {q(python['path'])} -I -B -c {q(loader.decode('utf-8'))} {q(json.dumps(binding, sort_keys=True))}",
    ])
    body = "\n".join(lines)
    if DELIMITER in body.splitlines():
        raise BootstrapError("bootstrap source collides with shell delimiter")
    if vm_install.inspect(share, digest, repo, suite) != payload:
        raise BootstrapError("bootstrap payload changed")
    return f"exec {ENV} /bin/zsh -f <<'{DELIMITER}'\n" + body + "\n" + DELIMITER + "\n"
