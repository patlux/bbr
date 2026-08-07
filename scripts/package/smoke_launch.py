"""Bounded launch smoke for a packaged BBR.app.

Proves the packaged app starts by launching the bundle executable directly
(never via `open`/LaunchServices) with --bbr-hidden, requiring it to stay
alive for a short observation window, then terminating its process group.
Every step is bounded and the process group is always reaped (SIGTERM, then
SIGKILL after a grace period) on every path, success or failure.
"""

from __future__ import annotations

import argparse
import os
import plistlib
import signal
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

DEFAULT_OBSERVE = 5.0
DEFAULT_LAUNCH_TIMEOUT = 30.0
TERMINATE_GRACE = 5.0
POLL_INTERVAL = 0.2


@dataclass
class ProbeResult:
    ok: bool
    message: str
    exit_code: int | None = None
    stderr_tail: str = ""
    observation_seconds: float = 0.0
    termination: str = "none"  # "signal" | "killed" | "none"


def _codesign_verify(app: Path) -> tuple[bool, str]:
    result = subprocess.run(["codesign", "--verify", "--strict", str(app)],
                            capture_output=True, text=True)
    return result.returncode == 0, (result.stderr or result.stdout or "")


def _stderr_tail(stderr_path: Path, limit: int = 2000) -> str:
    try:
        text = stderr_path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""
    return text[-limit:] if len(text) > limit else text


def _kill_group(proc: subprocess.Popen, grace: float = TERMINATE_GRACE) -> str:
    """Terminate the process group; SIGKILL after the grace period."""
    if proc.poll() is not None:
        return "none"
    try:
        os.killpg(proc.pid, signal.SIGTERM)
    except ProcessLookupError:
        return "none"
    deadline = time.monotonic() + grace
    while time.monotonic() < deadline and proc.poll() is None:
        time.sleep(POLL_INTERVAL)
    if proc.poll() is None:
        try:
            os.killpg(proc.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            pass
        return "killed"
    return "signal"


def probe_app(app: Path, *, observe: float = DEFAULT_OBSERVE,
              launch_timeout: float = DEFAULT_LAUNCH_TIMEOUT,
              verify_signed: bool = True, stderr_path: Path | None = None,
              extra_args: list[str] | None = None,
              env: dict[str, str] | None = None) -> ProbeResult:
    """Launch `app`'s bundle executable and require it to stay alive."""
    app = Path(app)
    if not app.is_dir():
        return ProbeResult(False, f"app bundle missing: {app}")
    plist_path = app / "Contents" / "Info.plist"
    if not plist_path.is_file():
        return ProbeResult(False, f"Contents/Info.plist missing in {app}")
    with plist_path.open("rb") as handle:
        plist = plistlib.load(handle)
    executable = plist.get("CFBundleExecutable")
    if not executable:
        return ProbeResult(False, f"CFBundleExecutable missing in {plist_path}")
    exe = app / "Contents" / "MacOS" / executable
    if not exe.is_file():
        return ProbeResult(False, f"bundle executable missing: {exe}")
    if not os.access(exe, os.X_OK):
        return ProbeResult(False, f"bundle executable not runnable: {exe}")
    if verify_signed:
        ok, output = _codesign_verify(app)
        if not ok:
            return ProbeResult(False, f"codesign --verify failed: {output.strip()}")

    args = [str(exe), *(extra_args if extra_args is not None else ["--bbr-hidden"])]

    def run_probe() -> ProbeResult:
        with tempfile.TemporaryDirectory(prefix="bbr-smoke-") as tmpdir:
            capture = Path(tmpdir) / "app-stderr.txt"
            with capture.open("wb") as stderr_file:
                start = time.monotonic()
                try:
                    proc = subprocess.Popen(
                        args, stdout=subprocess.DEVNULL, stderr=stderr_file,
                        start_new_session=True, env=env)
                except OSError as exc:
                    return ProbeResult(False, f"launch failed: {exc}")

                observed = 0.0
                deadline = start + launch_timeout
                try:
                    while True:
                        now = time.monotonic()
                        if proc.poll() is not None:
                            code = proc.returncode
                            tail = _stderr_tail(capture)
                            if code == 0:
                                return ProbeResult(
                                    False,
                                    f"app exited cleanly during observation "
                                    f"after {observed:.1f}s",
                                    code, tail, observed)
                            return ProbeResult(
                                False,
                                f"app exited during observation after "
                                f"{observed:.1f}s (exit {code}); stderr:\n{tail}",
                                code, tail, observed)
                        if now - start >= observe:
                            break
                        if now > deadline:
                            termination = _kill_group(proc)
                            return ProbeResult(
                                False,
                                f"observation did not complete within "
                                f"{launch_timeout:.0f}s",
                                proc.returncode, "", observed, termination)
                        time.sleep(POLL_INTERVAL)
                        observed = now - start

                    observed = time.monotonic() - start
                    termination = _kill_group(proc)
                    return ProbeResult(
                        True,
                        f"app stayed alive for {observed:.1f}s observation; "
                        f"process group terminated ({termination}), "
                        f"exit {proc.returncode}",
                        proc.returncode, "", observed, termination)
                finally:
                    if proc.poll() is None:
                        _kill_group(proc)  # never leak a live app process

    result = run_probe()
    if stderr_path is not None and result.stderr_tail:
        stderr_path.parent.mkdir(parents=True, exist_ok=True)
        stderr_path.write_text(result.stderr_tail, encoding="utf-8", errors="replace")
    return result


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Bounded launch smoke for a packaged BBR.app")
    parser.add_argument("--app", required=True, type=Path, help="path to the .app bundle")
    parser.add_argument("--observe", type=float, default=DEFAULT_OBSERVE,
                        help=f"seconds the app must stay alive (default {DEFAULT_OBSERVE})")
    parser.add_argument("--launch-timeout", type=float, default=DEFAULT_LAUNCH_TIMEOUT,
                        help="hard overall bound in seconds (default 30)")
    parser.add_argument("--no-verify", action="store_true",
                        help="skip codesign --verify before launch (fake binaries in tests)")
    parser.add_argument("--stderr", type=Path, default=None,
                        help="save captured app stderr to this file")
    args = parser.parse_args(argv[1:])

    result = probe_app(args.app, observe=args.observe,
                       launch_timeout=args.launch_timeout,
                       verify_signed=not args.no_verify, stderr_path=args.stderr)
    print(result.message)
    return 0 if result.ok else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
