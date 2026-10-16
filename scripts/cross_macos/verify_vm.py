#!/usr/bin/env python3
"""Run checksum-bound Linux artifacts in the established disposable Tart profile.

Never builds on macOS, uses the host GUI, adopts baselines or installs Xcode.
Use an outer timeout with at least 120 seconds of cleanup grace.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import shlex
import sys
import tarfile
import time

try:
    from . import apple_sdk
except ImportError:
    import apple_sdk

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))
sys.path.insert(0, str(SCRIPTS / "visual"))
import e2e_vm
import tart_vm
import vm_lifecycle


def inspect(payload, expected):
    path = payload / "payload.json"
    if not apple_sdk.sha256_value(expected) or apple_sdk.digest(path) != expected:
        raise ValueError("payload manifest hash differs from independent expectation")
    manifest = json.loads(path.read_text(), object_pairs_hook=apple_sdk.unique_object)
    if manifest.get("schema") != 1:
        raise ValueError("unsupported payload schema")
    repo = manifest["repo"]
    if not re.fullmatch(r"/tmp/bbr-cross-[a-zA-Z0-9-]+/bbr", repo):
        raise ValueError("compiled source must be a fresh /tmp/bbr-cross-<id>/bbr path")
    for relative in manifest["files"]:
        apple_sdk.relative_path(relative)
        if not re.fullmatch(r"[a-zA-Z0-9_./+@-]+", relative):
            raise ValueError("unsafe checksum/shell source path")
    for name, relative in manifest["tests"].items():
        if (not re.fullmatch(r"e2e_[a-z0-9_]+|bbr-gallery", name)
                or not relative.startswith("target/aarch64-apple-darwin/")
                or relative not in manifest["files"]
                or manifest["files"][relative]["mode"] != 0o755):
            raise ValueError("invalid test executable binding")
    archive = payload / "payload.tar.gz"
    if apple_sdk.digest(archive) != manifest["archive_sha256"]:
        raise ValueError("payload archive hash differs")
    with tarfile.open(archive) as tar:
        members = tar.getmembers()
        if len(members) != len(manifest["files"]) or {m.name for m in members} != set(manifest["files"]):
            raise ValueError("archive membership differs")
        for member in members:
            if not member.isfile() or member.mode not in (0o644, 0o755):
                raise ValueError("unsupported archive member")
            record = manifest["files"][member.name]
            with tar.extractfile(member) as handle:
                sha = hashlib.file_digest(handle, "sha256").hexdigest()
            if {"sha256": sha, "bytes": member.size, "mode": member.mode} != record:
                raise ValueError("payload file bytes/mode differ")
    checks = "".join(f'{r["sha256"]}  {p}\n' for p, r in manifest["files"].items())
    if (payload / "SHA256SUMS").read_text() != checks:
        raise ValueError("checksum list differs from bound payload")
    return manifest


def plan_for(names, functional):
    if functional and names:
        raise ValueError("choose explicit recipes or the functional aggregate, not both")
    if not functional and not names:
        raise ValueError("explicit recipes or --functional required")
    result = []
    for recipe, cargo, test in e2e_vm.plan_for(["e2e", "e2e-peek", "e2e-gallery"] if functional else names):
        if functional and recipe == "e2e-perf":
            continue  # Explicit existing execution deferral, never reported as full perf pass.
        if "perf" in recipe:
            raise ValueError("performance execution is deferred; this driver claims functional evidence only")
        args = shlex.split(cargo)
        name = args[args.index("--test") + 1] if "--test" in args else "bbr-gallery"
        limit = 480 if recipe in ("e2e-shell-dense", "e2e-shell-responsiveness", "e2e-web-navigation-gestures") else 120
        result.append((recipe, name, shlex.split(test), limit))
    return result


def bootstrap(manifest):
    repo = manifest["repo"]
    root = str(Path(repo).parent)
    archive_sha = manifest["archive_sha256"]
    modes = "\n".join(f'[[ "$(/usr/bin/stat -f %Lp {shlex.quote(p)})" == {r["mode"]:o} ]]'
                      for p, r in manifest["files"].items() if r["mode"] == 0o755)
    return f'''set -euo pipefail
[[ "$(/usr/sbin/sysctl -n hw.model)" == VirtualMac* ]]
[[ "$(/usr/bin/id -un)" == admin ]]
[[ ! -e {root} && ! -L {root} ]]
/usr/bin/sudo -n /usr/bin/pmset -a displaysleep 0 sleep 0 disksleep 0
/usr/bin/defaults -currentHost write com.apple.screensaver idleTime -int 0
console=$(/usr/sbin/ioreg -n Root -d1)
[[ "$console" == *'kCGSSessionOnConsoleKey"=Yes'* ]]
[[ "$console" != *'CGSSessionScreenIsLocked"=Yes'* ]]
/bin/mkdir -m 700 /Users/admin/cross-payload
/usr/bin/sudo -n /sbin/mount_virtiofs -r bbr-visual-payload /Users/admin/cross-payload
cd /Users/admin/cross-payload/visual
printf '%s  payload.tar.gz\\n' {archive_sha} | /usr/bin/shasum -a 256 -c -
/bin/mkdir -m 700 {root} {repo}
cd {repo}
/usr/bin/tar -xzf /Users/admin/cross-payload/visual/payload.tar.gz
/usr/bin/shasum -a 256 -c /Users/admin/cross-payload/visual/SHA256SUMS
{modes}
'''


def production_smoke(repo):
    # Observation is a liveness-only package smoke, not presentation/performance.
    # SIGINT is the app's established clean shutdown command. Exactly the spawned
    # child is checked by command and start time before signalling; no PID search.
    return f'''set -euo pipefail
cd {repo}
/usr/bin/codesign --verify --deep --strict BBR.app
exe={repo}/BBR.app/Contents/MacOS/bbr
/usr/bin/env -i HOME=/Users/admin USER=admin LOGNAME=admin PATH=/usr/bin:/bin:/usr/sbin:/sbin TMPDIR=/tmp "$exe" --bbr-hidden --bbr-profile-dir /Users/admin/cross-production-profile > /tmp/bbr-cross-production.log 2>&1 &
pid=$!
/bin/sleep 1
[[ "$(/bin/ps -p "$pid" -o comm=)" == "$exe" ]]
started=$(/bin/ps -p "$pid" -o lstart=)
/bin/sleep 5
[[ "$(/bin/ps -p "$pid" -o comm=)" == "$exe" ]]
[[ "$(/bin/ps -p "$pid" -o lstart=)" == "$started" ]]
/bin/ps -p "$pid" -o pid,ppid,lstart,command
/bin/kill -INT "$pid"
wait "$pid"
printf 'PRODUCTION BUNDLE SMOKE PASSED\\n'
'''


def validate_test_output(text):
    results = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)
    if len(results) != 1 or int(results[0][0]) == 0 or results[0][1:] != ("0", "0"):
        raise ValueError("test executable did not report nonempty complete successful execution")
    return int(results[0][0])


def execute(payload, expected, artifacts, names, functional):
    if sys.platform != "darwin":
        raise ValueError("Tart validation requires macOS")
    manifest = inspect(payload, expected)
    plan = plan_for(names, functional)
    if not all(name in manifest["tests"] for _, name, _, _ in plan):
        raise ValueError("payload lacks a required test executable")
    artifacts.mkdir(mode=0o700)  # Existing lifecycle rejects overlap/dirty/linked paths.
    summary = []
    with vm_lifecycle.session(payload, artifacts) as session:
        def guest(label, script, limit):
            started = time.monotonic()
            result = tart_vm.guest_script(session.ssh, script, timeout=limit)
            (artifacts / f"{label}.log").write_text(result.stdout + result.stderr)
            summary.append({"stage": label, "exit": result.returncode,
                            "seconds": time.monotonic() - started})
            (artifacts / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
            print(json.dumps(summary[-1]), flush=True)
            if result.returncode:
                raise RuntimeError(f"{label} failed; see {artifacts / (label + '.log')}")
            return result.stdout + result.stderr
        guest("prepare", bootstrap(manifest), 180)
        repo = manifest["repo"]
        if manifest["production_bundle"]:
            guest("production-smoke", production_smoke(repo), 60)
        for index, (recipe, name, flags, limit) in enumerate(plan):
            package = "crates/bbr-gallery" if name == "bbr-gallery" else "crates/bbr-app"
            exe = f'{repo}/{manifest["tests"][name]}'
            script = f'''set -euo pipefail
cd {repo}/{package}
/usr/bin/codesign --verify --strict {exe}
/usr/bin/env -i HOME=/Users/admin USER=admin LOGNAME=admin PATH=/usr/bin:/bin:/usr/sbin:/sbin TMPDIR=/tmp CARGO_MANIFEST_DIR={repo}/{package} RUST_BACKTRACE=1 {exe} {shlex.join(flags)}
'''
            text = guest(f"{index:02d}-{recipe}-{name}", script, limit)
            summary[-1]["passed"] = validate_test_output(text)
            (artifacts / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
        guest("post-hashes", f"set -eu\ncd {repo}\n/usr/bin/shasum -a 256 -c /Users/admin/cross-payload/visual/SHA256SUMS\n", 60)
    inspect(payload, expected)
    (artifacts / "verified.json").write_text(json.dumps({"payload_sha256": expected,
        "source": manifest["source"], "test_programs": len(plan),
        "passed_tests": sum(stage.get("passed", 0) for stage in summary),
        "functional_aggregate": functional, "performance": "deferred, unpassed",
        "cleanup": "independently observed by established lifecycle"}, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--payload", type=Path, required=True)
    parser.add_argument("--sha256", required=True, help="independently retained payload.json hash")
    parser.add_argument("--artifacts", type=Path, required=True)
    parser.add_argument("--functional", action="store_true", help="canonical functional order plus Peek/Gallery; perf deferred")
    parser.add_argument("recipes", nargs="*")
    args = parser.parse_args()
    execute(args.payload, args.sha256, args.artifacts, args.recipes, args.functional)


if __name__ == "__main__":
    main()
