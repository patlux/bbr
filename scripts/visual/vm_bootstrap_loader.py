"""Inline bootstrap payload: standard library only until all helpers are verified.

The host embeds these source-bound bytes and independently expected helper hashes
into the fixed guest command. Never add a source/share directory to sys.path or
execute helpers by their mutable filenames. All helper bytes are frozen first.
"""
import hashlib
import importlib.abc
import importlib.util
import json
import os
from pathlib import Path
import stat
import sys

MODULES = ("vm_install", "vm_observe", "vm_transfer", "vm_guest", "vm_bootstrap_guest")
LIMIT = 1024 * 1024


def read(path, limit):
    if not path.is_absolute() or path.resolve() != path:
        raise ValueError("noncanonical bootstrap source")
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(fd, "rb") as stream:
        meta = os.fstat(stream.fileno())
        if not stat.S_ISREG(meta.st_mode) or meta.st_nlink != 1 or not 0 < meta.st_size <= limit:
            raise ValueError("bounded regular single-link bootstrap source required")
        raw = stream.read(limit + 1)
        if len(raw) != meta.st_size:
            raise ValueError("bootstrap source changed")
        return raw


def run(binding):
    if set(binding) != {"share", "payload_sha256", "repo", "suite", "work", "modules"}:
        raise ValueError("invalid bootstrap binding")
    share = Path(binding["share"])
    if not os.statvfs(share).f_flag & os.ST_RDONLY:
        raise ValueError("bootstrap payload mount must be read-only")
    if hashlib.sha256(read(share / "payload.json", 8 * LIMIT)).hexdigest() != binding["payload_sha256"]:
        raise ValueError("bootstrap payload differs")
    if set(binding["modules"]) != set(MODULES) or any(name in sys.modules for name in MODULES):
        raise ValueError("ambiguous bootstrap modules")
    code = {}
    for name in MODULES:
        path = share / "source/scripts/visual" / (name + ".py")
        raw = read(path, LIMIT)
        if hashlib.sha256(raw).hexdigest() != binding["modules"][name]:
            raise ValueError("bootstrap helper bytes differ: " + name)
        code[name] = compile(raw, str(path), "exec", dont_inherit=True)

    class FrozenModules(importlib.abc.MetaPathFinder, importlib.abc.Loader):
        def find_spec(self, fullname, path=None, target=None):
            return importlib.util.spec_from_loader(fullname, self) if fullname in code else None

        def create_module(self, spec):
            return None

        def exec_module(self, module):
            exec(code[module.__name__], module.__dict__)

    finder = FrozenModules()
    sys.meta_path.insert(0, finder)
    try:
        import vm_bootstrap_guest
        vm_bootstrap_guest.run(share, binding["payload_sha256"], Path(binding["repo"]),
                               binding["suite"], Path(binding["work"]))
    finally:
        sys.meta_path.remove(finder)


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise ValueError("one explicit bootstrap binding required")
    run(json.loads(sys.argv[1]))
