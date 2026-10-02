"""Hash-bound offline Nix export/import for one already hardened disposable guest.

The source lock is the only runtime authority. No host/Golden Image store writes,
substitution, builders, repair, alternate Python or signature-policy overrides.
The existing transport supervises all direct children and destroys the clone on
failure; this module creates no VM, SSH route or capture evidence.
"""
from dataclasses import dataclass
import hashlib
import os
from pathlib import Path
import shlex
import shutil
import stat
import time

import vm_install
import vm_runtime

ARCHIVE_LIMIT = vm_runtime.TOTAL_LIMIT + 8 * 1024 * 1024
REMOTE = '/Users/admin/bbr-visual-runtime.export'


def fingerprint(path: Path) -> tuple[int, str]:
    vm_install._absolute(path)
    until = time.monotonic() + 120
    with os.fdopen(os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK), 'rb') as source:
        meta = os.fstat(source.fileno())
        if not stat.S_ISREG(meta.st_mode) or meta.st_nlink != 1 or not 0 < meta.st_size <= ARCHIVE_LIMIT:
            raise vm_runtime.RuntimeError('invalid runtime export size/type')
        digest, size = hashlib.sha256(), 0
        while block := source.read(65536):
            size += len(block)
            if size > meta.st_size or time.monotonic() >= until:
                raise vm_runtime.RuntimeError('runtime export grew/deadline')
            digest.update(block)
        if size != meta.st_size:
            raise vm_runtime.RuntimeError('runtime export truncated')
        return size, digest.hexdigest()


@dataclass(frozen=True)
class Export:
    path: Path
    payload_sha256: str
    runtime_lock_sha256: str
    bytes: int
    sha256: str
    paths: tuple[str, ...]

    def recheck(self):
        if fingerprint(self.path) != (self.bytes, self.sha256):
            raise vm_runtime.RuntimeError('runtime export changed')


def prepare(share, digest, repo, suite, workspace, execute) -> Export:
    payload = vm_install.inspect(share, digest, repo, suite)
    lock_path = share / 'source' / vm_runtime.LOCK
    handle, _ = vm_install._open(lock_path, 1024 * 1024)
    with handle: raw = handle.read(1024 * 1024 + 1)
    lock_hash = hashlib.sha256(raw).hexdigest()
    if lock_hash != payload['sources'].get(vm_runtime.LOCK):
        raise vm_runtime.RuntimeError('runtime lock differs from payload')
    lock = vm_runtime.validate(vm_runtime._json(raw))
    vm_runtime.verify(share, digest, repo, suite)
    store = shutil.which('nix-store')
    if not store:
        raise vm_runtime.RuntimeError('existing nix-store required')
    archive = workspace / 'runtime.export'
    if archive.exists() or archive.is_symlink():
        raise vm_runtime.RuntimeError('fresh runtime export required')
    # --export sorts the explicit closure topologically, never realises it.
    execute([store, '--export', *sorted(lock['paths'])], archive,
            workspace / 'runtime-export.stderr', script=None,
            seconds=240, output_limit=ARCHIVE_LIMIT)
    size, sha256 = fingerprint(archive)
    archive.chmod(0o400)
    vm_runtime.verify(share, digest, repo, suite)
    value = Export(archive, digest, lock_hash, size, sha256, tuple(sorted(lock['paths'])))
    value.recheck()
    return value


def import_script(value: Export) -> str:
    value.recheck()
    q = shlex.quote
    # Passed through -c: stdin is only the hash-bound binary export, never shell.
    return '\n'.join([
        'set -euo pipefail', 'umask 077',
        '[[ "$(/usr/bin/id -un)" == admin && "$(/usr/bin/id -u)" == 501 ]] || { print -u2 "runtime import host forbidden"; exit 76; }',
        '[[ "$(/usr/sbin/sysctl -n hw.model)" == VirtualMac* ]] || { print -u2 "runtime import host forbidden"; exit 76; }',
        f'[[ ! -e {REMOTE} && ! -L {REMOTE} ]]',
        'setopt noclobber',
        f'/bin/cat > {REMOTE}',
        f'[[ "$(/usr/bin/stat -f %z {REMOTE})" == {value.bytes} ]]',
        f'[[ "$(/usr/bin/shasum -a 256 {REMOTE})" == {q(value.sha256 + "  " + REMOTE)} ]]',
        f'/usr/bin/sudo -n /usr/bin/env -i PATH=/usr/bin:/bin:/usr/sbin:/sbin NIX_REMOTE=local /nix/var/nix/profiles/default/bin/nix-store --import --option substitute false --option builders {q("")} < {REMOTE}',
        f'/bin/rm {REMOTE}',
    ]) + '\n'
