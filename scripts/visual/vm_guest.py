"""Guest-only composition of installation, fixed Rust producer, receipt and return.

No comparator, inventory authority or image rewriting. The installed independent
Rust consumer reuses upstream process cleanup for native producers. This wrapper
owns only that direct child; on interruption it first gives Rust time to reap its
producer group. Any failed step prevents receipt/return success. Host transport
must still prove VM teardown before it can expose candidates for Rust validation.
"""
from __future__ import annotations

import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

import vm_install
import vm_observe
import vm_transfer
import vm_selection

LOG_LIMIT = 16 * 1024 * 1024
CONSUMER_GRACE = 15


class GuestError(RuntimeError):
    pass


def _producer(consumer: Path, suite: str, action: str, repo: Path, share: Path,
              payload_hash: str, work: Path, seconds: float, selected_sha256: str | None = None) -> None:
    """Wait on the owned Rust consumer, never kill a numeric saved group/PID."""
    env = {"HOME": "/Users/admin", "USER": "admin", "LOGNAME": "admin",
           "PATH": "/usr/bin:/bin:/usr/sbin:/sbin", "TMPDIR": "/private/tmp",
           "TZ": "UTC", "LANG": "C", "LC_ALL": "C", "SEED": "0"}
    if suite == "gallery-semantic" and (action != "semantic" or selected_sha256 is not None):
        raise GuestError("semantic purpose requires fixed semantic action without filter")
    if action == "semantic" and suite != "gallery-semantic":
        raise GuestError("semantic action requires semantic purpose")
    if (action == "capture-selected") != (selected_sha256 is not None):
        raise GuestError("selected action requires independent request hash")
    argv = [str(consumer), "guest-execute", suite, action, str(repo), str(share), payload_hash, str(work)]
    if selected_sha256 is not None:
        vm_selection.read(work, selected_sha256, payload_hash, suite)
        argv.append(selected_sha256)
    with (work / f"{action}.log").open("xb") as log:
        child = subprocess.Popen(argv,
                                 cwd=repo, env=env, stdin=subprocess.DEVNULL,
                                 stdout=log, stderr=subprocess.STDOUT)
        try:
            until = time.monotonic() + seconds
            while child.poll() is None:
                if time.monotonic() >= until:
                    raise GuestError("guest consumer deadline")
                if os.fstat(log.fileno()).st_size > LOG_LIMIT:
                    raise GuestError("guest consumer log limit")
                time.sleep(0.01)
            if child.returncode != 0 or os.fstat(log.fileno()).st_size > LOG_LIMIT:
                # Preserve bounded test-only diagnostics before the disposable
                # guest is destroyed. Failure output is never a returned receipt.
                log.flush()
                handle, _ = vm_install._open(work / f"{action}.log", LOG_LIMIT)
                with handle:
                    diagnostic = handle.read(LOG_LIMIT + 1)
                print(diagnostic.decode("utf-8", errors="replace"), file=sys.stderr)
                raise GuestError(f"guest consumer failed: {action} rc={child.returncode}")
        finally:
            if child.poll() is None:
                child.terminate()  # Rust handler cancels upstream group and reaps.
                try:
                    child.wait(timeout=CONSUMER_GRACE)
                except subprocess.TimeoutExpired:
                    child.kill()
                    child.wait(timeout=5)
                    raise GuestError("guest consumer hard terminated; producer cleanup unproven")


def _check_observation(value: dict) -> None:
    if (value.get("origin") != "tart-guest" or value.get("console_user") != "admin"
            or value.get("on_console") is not True or value.get("screen_locked") is not False
            or value.get("window_server") is not True or value.get("timezone") != "UTC"
            or value.get("locale") != "C" or not str(value.get("model", "")).startswith("VirtualMac")):
        raise GuestError("actual unlocked disposable guest observation required; host capture forbidden")


def run(share: Path, payload_hash: str, repo: Path, suite: str, work: Path,
        selected: str | None = None, selected_sha256: str | None = None) -> dict:
    if suite not in {"gallery", "browser", "gallery-semantic"}:
        raise GuestError("unsupported guest suite")
    # Observe before writes/installation. A fabricated environment variable is
    # insufficient; the observer probes the guest console, hardware and fonts.
    before = vm_observe.observe()
    _check_observation(before)
    vm_selection.decode(selected, selected_sha256, payload_hash, suite)
    for path in (share, repo, work):
        vm_install._absolute(path)
    for left, right in ((share, repo), (share, work), (repo, work)):
        if left == right or left in right.parents or right in left.parents:
            raise GuestError("guest source/share/output must be disjoint")
    if work.exists() or not work.parent.is_dir():
        raise GuestError("fresh guest work directory required")
    payload = vm_install.inspect(share, payload_hash, repo, suite)
    installed = vm_install.install(share, payload_hash, repo, suite)
    work.mkdir(mode=0o700)
    consumer = repo / "scripts/visual/target/release/bbr-visual"
    if selected is not None:
        vm_selection.write(work, selected, selected_sha256, payload_hash, suite)
        _producer(consumer, suite, "capture-selected", repo, share, payload_hash, work, 660, selected_sha256)
        if vm_selection.read(work, selected_sha256, payload_hash, suite) != selected:
            raise GuestError("selected request changed during execution")
    elif suite == "gallery-semantic":
        _producer(consumer, suite, "semantic", repo, share, payload_hash, work, 660)
    else:
        if suite == "gallery":
            _producer(consumer, suite, "inventory", repo, share, payload_hash, work, 90)
        _producer(consumer, suite, "capture", repo, share, payload_hash, work,
                  660 if suite == "gallery" else 240)
    after_installed = vm_install.verify_installed(share, payload_hash, repo, suite)
    after = vm_observe.observe()
    _check_observation(after)
    if installed != after_installed or before != after:
        raise GuestError("guest bytes or rendering environment changed during capture")
    raw = work / "raw"
    vm_install._absolute(raw)
    if not raw.is_dir():
        raise GuestError("producer did not create raw output")
    if suite == "gallery-semantic":
        expected = {"semantic-results.json", "semantic-every-story-renders.json",
                    "semantic-mode-state-switching.json"}
        # Transport inventory only. Rust owns test/result/diagnostic validation.
        found = set()
        for path in raw.iterdir():
            if path.name not in expected or len(found) >= len(expected):
                raise GuestError("semantic raw inventory differs")
            handle, meta = vm_install._open(path, 4 * 1024 * 1024)
            handle.close()
            if not meta.st_size:
                raise GuestError("empty semantic raw evidence")
            found.add(path.name)
        if found != expected:
            raise GuestError("semantic raw inventory incomplete")
    if suite == "gallery":
        inventory = work / "live-inventory.tsv"
        handle, meta = vm_install._open(inventory, LOG_LIMIT)
        with handle:
            if not meta.st_size:
                raise GuestError("missing live inventory")
            with (raw / "live-inventory.tsv").open("xb") as output:
                output.write(handle.read(LOG_LIMIT + 1))
    receipt = {"schema": 1, "payload_sha256": payload_hash, "sources": payload["sources"],
               "installed": after_installed, "environment_before": before, "environment_after": after}
    with (raw / "guest-receipt.json").open("x") as output:
        json.dump(receipt, output, sort_keys=True)
    # Return transport validates exact bytes; independent coverage/image/receipt
    # checks on the host are still mandatory. This is not a GPUI Visual seal.
    return vm_transfer.pack(raw, work / "return.tar", payload_hash)


def main(argv: list[str]) -> None:
    if len(argv) not in (6, 8):
        raise GuestError("usage: vm_guest.py <share> <payload-hash> <repo> <suite> <fresh-work> [selected-json selected-hash]")
    previous = {}
    def interrupt(number, _frame):
        for sig in previous:
            signal.signal(sig, signal.SIG_IGN)
        raise GuestError(f"guest wrapper cancelled by signal {number}")
    try:
        for sig in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
            previous[sig] = signal.signal(sig, interrupt)
        run(Path(argv[1]), argv[2], Path(argv[3]), argv[4], Path(argv[5]), *argv[6:])
    finally:
        for sig, handler in previous.items(): signal.signal(sig, handler)


if __name__ == "__main__":
    import sys
    try:
        main(sys.argv)
    except (GuestError, vm_install.InstallError, vm_observe.ObservationError, vm_transfer.TransferError) as error:
        print(f"guest capture invalid: {error}", file=sys.stderr)
        raise SystemExit(2)
