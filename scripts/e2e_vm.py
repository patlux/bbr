#!/usr/bin/env python3
"""Run BBR E2E suites in a throwaway Tart macOS VM (`just e2e-vm`).

GUI suites on the host show windows and take the user's keyboard focus.
This runner builds the test executables on the host from the current
working tree (the guest has no Xcode 27 Metal toolchain), boots a headless
clone of the golden image (`scripts/tart_vm.py`), copies the tree and the
executables into the guest at the same absolute path (the executables
carry `CARGO_BIN_EXE_bbr`/`CARGO_MANIFEST_DIR` as compile-time paths) and
runs every selected `just` recipe's test executables there, in the
canonical `just e2e` order. Nothing appears on the host screen.

    python3 scripts/e2e_vm.py e2e-shell-organize e2e-shell-persistence
    python3 scripts/e2e_vm.py e2e              # every suite of `just e2e`
    python3 scripts/e2e_vm.py e2e-shell-persistence --filter favicons
    python3 scripts/e2e_vm.py e2e-visual-capture --filter representative \
        --env BBR_VISUAL_CAPTURE_BIN=<repo>/target/debug/bbr \
        --env BBR_VISUAL_CAPTURE_DIR=<repo>/reports/visual/vm-capture \
        --collect <repo>/reports/visual/vm-capture

The exit status is 0 only if every executable passed. Logs per executable
and `summary.log` land in `/tmp/bbr-e2e-vm/run-<stamp>/`; `--collect`
copies one guest directory (review stills, never baselines) into
`collected/` there. `--env` values are guest paths: the tree lives at the
same absolute path in the guest.
"""

from __future__ import annotations

import argparse
import atexit
import json
import os
import re
import shlex
import subprocess
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import tart_vm  # noqa: E402

REPO = Path(__file__).resolve().parents[1]
WORK = Path("/tmp/bbr-e2e-vm")
TEST_LINE = re.compile(r"cargo test (?P<cargo>.*?) -- (?P<test>.*)$")
ENV_LINE = re.compile(r"^[A-Z_][A-Z0-9_]*=.*$", re.DOTALL)


def guest_exports(pairs: list[str]) -> str:
    """`export K=V; ` for every `--env K=V` (shell-quoted), empty without."""
    for pair in pairs:
        if not ENV_LINE.match(pair):
            raise tart_vm.RunnerError(f"--env expects KEY=VALUE with an upper-case KEY: {pair!r}")
    return "".join(f"export {k}={shlex.quote(v)}; " for k, v in (p.split("=", 1) for p in pairs))


def collect(ssh: list[str], guest_dir: str, destination: Path) -> int:
    """Copy `guest_dir` into `destination` as a tar stream over the run's SSH
    session; returns the number of files."""
    if not guest_dir.startswith("/") or ".." in Path(guest_dir).parts:
        raise tart_vm.RunnerError("--collect expects an absolute guest path without traversal")
    stream = subprocess.run(
        [*ssh, f"tar cf - --no-xattrs --no-mac-metadata -C {shlex.quote(guest_dir)} ."],
        capture_output=True, timeout=600,
    )
    if stream.returncode:
        raise tart_vm.RunnerError(f"collect failed ({stream.returncode}): {stream.stderr.decode(errors='replace')}")
    destination.mkdir(parents=True, exist_ok=False)
    subprocess.run(["tar", "xf", "-"], input=stream.stdout, cwd=destination, check=True, timeout=600)
    return sum(1 for p in destination.rglob("*") if p.is_file())


def recipes() -> tuple[dict[str, list[tuple[str, str]]], list[str]]:
    """Every justfile recipe's `cargo test` lines as (cargo args, test args),
    and the `e2e` recipe's dependency order."""
    found: dict[str, list[tuple[str, str]]] = {}
    order: list[str] = []
    current = None
    for line in (REPO / "justfile").read_text().splitlines():
        if line.startswith("e2e: "):
            order = line.split(":", 1)[1].split()
        match = re.match(r"^([a-z0-9-]+)(\s[^:=]*)?:(?!=)", line)
        if match:
            current = match.group(1)
            found.setdefault(current, [])
        elif current and line.startswith("    "):
            test = TEST_LINE.search(line)
            if test:
                found[current].append((test.group("cargo").strip(), test.group("test").strip()))
    return found, order


def plan_for(names: list[str]) -> list[tuple[str, str, str]]:
    """(recipe, cargo args, test args) in the requested order; `e2e`
    expands to its suites."""
    found, order = recipes()
    plan = []
    for name in names:
        for recipe in order if name == "e2e" else [name]:
            if not found.get(recipe):
                raise tart_vm.RunnerError(f"{recipe} is not a recipe with cargo test lines")
            plan.extend((recipe, cargo, test) for cargo, test in found[recipe])
    return plan


def build_env() -> dict[str, str]:
    """The justfile's Xcode environment for host builds."""
    default = re.search(
        r'BBR_XCODE_DEVELOPER_DIR", "([^"]+)"', (REPO / "justfile").read_text()
    ).group(1)
    developer = os.environ.get("BBR_XCODE_DEVELOPER_DIR", default)
    env = dict(os.environ, DEVELOPER_DIR=developer)

    def xcrun(*args: str) -> str:
        return tart_vm.run(["/usr/bin/xcrun", "--sdk", "macosx", *args], env=env).stdout.strip()

    env["SDKROOT"] = xcrun("--show-sdk-path")
    env["CC"] = xcrun("--find", "clang")
    env["CXX"] = xcrun("--find", "clang++")
    env["CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER"] = env["CC"]
    return env


def build(plan: list[tuple[str, str, str]], artifacts: Path) -> dict[str, str]:
    """Build every executable of the plan on the host; returns cargo args →
    executable path relative to the repo. Lines sharing all arguments but
    `--test <name>` build in one cargo invocation."""
    groups: dict[str, list[str]] = {}
    for _, cargo, _ in plan:
        args = shlex.split(cargo)
        tests = [args[i + 1] for i, arg in enumerate(args) if arg == "--test"]
        rest = [arg for i, arg in enumerate(args)
                if arg != "--test" and (i == 0 or args[i - 1] != "--test")]
        groups.setdefault(shlex.join(rest), [])
        groups[shlex.join(rest)].extend(test for test in tests if test not in groups[shlex.join(rest)])
    tool = "mbx" if subprocess.run(["which", "mbx"], capture_output=True).returncode == 0 else "cargo"
    env = build_env()
    executables: dict[tuple[str, str], str] = {}
    for rest, tests in groups.items():
        command = [tool, "test", *shlex.split(rest), *[part for t in tests for part in ("--test", t)],
                   "--no-run", "--message-format=json"]
        print("build:", " ".join(command), flush=True)
        result = subprocess.run(command, cwd=REPO, env=env, capture_output=True, text=True, timeout=3600)
        (artifacts / "build.log").open("a").write(result.stderr)
        if result.returncode:
            raise tart_vm.RunnerError(f"host build failed; see {artifacts / 'build.log'}")
        for line in result.stdout.splitlines():
            try:
                message = json.loads(line)
            except json.JSONDecodeError:
                continue
            if message.get("reason") == "compiler-artifact" and message.get("executable") \
                    and message["profile"]["test"]:
                executables[(rest, message["target"]["name"])] = message["executable"]
    resolved = {}
    for _, cargo, _ in plan:
        args = shlex.split(cargo)
        rest = shlex.join([arg for i, arg in enumerate(args)
                           if arg != "--test" and (i == 0 or args[i - 1] != "--test")])
        target = args[args.index("--test") + 1] if "--test" in args else args[args.index("--bin") + 1]
        exe = executables.get((rest, target)) or executables.get((rest, target.replace("-", "_")))
        if not exe:
            raise tart_vm.RunnerError(f"no executable built for {cargo}")
        resolved[cargo] = str(Path(exe).relative_to(REPO))
    return resolved


def nix_dependencies(paths: list[Path]) -> list[str]:
    """The /nix/store paths the executables link against (realised in the
    guest before the run)."""
    store = set()
    for path in paths:
        listing = tart_vm.run(["otool", "-L", str(path)], check=False).stdout
        store.update(re.findall(r"(/nix/store/[^/\s]+)/", listing))
    return sorted(store)


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("recipes", nargs="+", help="just e2e recipes, or `e2e` for all")
    parser.add_argument("--filter", help="test name filter passed to every executable")
    parser.add_argument("--source-vm", default="macos-27-golden-gate")
    parser.add_argument("--display", default="3200x1800", help="e2e_windows needs two windows side by side")
    parser.add_argument("--keep-vm", action="store_true")
    parser.add_argument("--env", action="append", default=[], metavar="KEY=VALUE",
                        help="exported in the guest before every executable (guest paths)")
    parser.add_argument("--collect", metavar="GUEST_DIR",
                        help="guest directory copied into the run's artifacts after the plan")
    args = parser.parse_args(argv[1:])
    exports = guest_exports(args.env)

    plan = plan_for(args.recipes)
    stamp = time.strftime("%Y%m%d-%H%M%S")
    artifacts = WORK / f"run-{stamp}-{os.getpid()}"
    artifacts.mkdir(parents=True)
    exes = build(plan, artifacts)
    target_dir = (REPO / "target").resolve()
    files = sorted({str(Path(e).relative_to("target")) for e in exes.values()}
                   | {p for p in ("debug/bbr", "release/bbr") if (target_dir / p).exists()})
    deps = nix_dependencies([target_dir / f for f in files])

    clone, ssh, banner = tart_vm.boot(
        args.source_vm, f"bbr-e2e-vm-{stamp}-{os.getpid()}", artifacts, args.keep_vm, args.display,
        [f"bbr:{REPO}:ro,tag=bbr-src", f"bbrtarget:{target_dir}:ro,tag=bbr-target"],
    )
    atexit.register(clone.cleanup)
    print(banner, flush=True)

    prepare = f'''set -euo pipefail
sudo pmset -a displaysleep 0 sleep 0 disksleep 0
sudo defaults write /Library/Preferences/com.apple.screensaver askForPassword -int 0
defaults -currentHost write com.apple.screensaver idleTime -int 0
nohup caffeinate -dimsu >"$HOME/bbr-caffeinate.log" 2>&1 &
session=$(ioreg -n Root -d1 | grep IOConsoleUsers)
printf '%s\\n' "$session" | grep -q 'kCGSSessionOnConsoleKey"=Yes'
printf '%s\\n' "$session" | grep -q 'kCGSSessionUserNameKey"="admin"'
if printf '%s\\n' "$session" | grep -q 'CGSSessionScreenIsLocked"=Yes'; then echo locked >&2; exit 76; fi
pgrep -x WindowServer >/dev/null
/usr/bin/sw_vers
for dep in {" ".join(deps)}; do /nix/var/nix/profiles/default/bin/nix-store --realise "$dep" >/dev/null; done
mkdir -p "$HOME/runner-mounts/bbr" "$HOME/runner-mounts/target"
mount | grep -q 'on /Users/admin/runner-mounts/bbr ' || sudo mount_virtiofs bbr-src "$HOME/runner-mounts/bbr"
mount | grep -q 'on /Users/admin/runner-mounts/target ' || sudo mount_virtiofs bbr-target "$HOME/runner-mounts/target"
sudo mkdir -p {REPO}/target
sudo chown -R admin:staff {Path(*REPO.parts[:3])}
cd "$HOME/runner-mounts/bbr/bbr"
tar cf - --no-xattrs --no-mac-metadata --exclude ./target --exclude ./.git --exclude ./.direnv . | (cd {REPO} && tar xf -)
cd "$HOME/runner-mounts/target/bbrtarget"
tar cf - --no-xattrs --no-mac-metadata {" ".join(files)} | (cd {REPO}/target && tar xf -)
'''
    # sshd can drop the first sessions right after boot on a loaded host
    # (rc 255): retry those only.
    for _ in range(4):
        prepared = tart_vm.guest_script(ssh, prepare, timeout=1800)
        if prepared.returncode != 255:
            break
        time.sleep(15)
    (artifacts / "prepare.log").write_text(prepared.stdout + prepared.stderr)
    if prepared.returncode:
        raise tart_vm.RunnerError(f"guest prepare failed ({prepared.returncode}); see {artifacts / 'prepare.log'}")

    failed = 0
    for recipe, cargo, test in plan:
        exe = exes[cargo]
        package = "crates/bbr-gallery" if "bbr-gallery" in cargo else "crates/bbr-app"
        test_args = test + (f" {shlex.quote(args.filter)}" if args.filter else "")
        command = (f"{exports}cd {REPO}/{package} && CARGO_MANIFEST_DIR={REPO}/{package} "
                   f"RUST_BACKTRACE=1 {REPO}/{exe} {test_args}")
        started = time.monotonic()
        try:
            result = tart_vm.run([*ssh, command], check=False, timeout=1800)
            code, output = result.returncode, result.stdout + result.stderr
        except tart_vm.RunnerError as error:
            code, output = 124, str(error)
            tart_vm.run([*ssh, f"pkill -f {Path(exe).name}; pkill -x bbr"], check=False)
        name = Path(exe).name.rsplit("-", 1)[0]
        (artifacts / f"{recipe}--{name}.log").write_text(output)
        passed = re.findall(r"test result: ok\. (\d+) passed", output)
        line = f"{recipe} {name} rc={code} passed={sum(map(int, passed))} {int(time.monotonic() - started)}s"
        failed += code != 0
        with (artifacts / "summary.log").open("a") as handle:
            handle.write(line + "\n")
        print(line, flush=True)
    if args.collect:
        count = collect(ssh, args.collect, artifacts / "collected")
        print(f"collected {count} files from {args.collect} into {artifacts / 'collected'}", flush=True)
    print(f"{'FAILED' if failed else 'ok'}: {failed} of {len(plan)} executables failed; logs in {artifacts}")
    return 1 if failed else 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv))
    except tart_vm.RunnerError as exc:
        print(f"e2e-vm error: {exc}", file=sys.stderr)
        raise SystemExit(2)
