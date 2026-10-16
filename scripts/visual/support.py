"""BBR-only process/build adapter; comparison and acceptance stay upstream."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
TOOL = ROOT / "vendor/cargo-gpui-visual"
BUILD_SECONDS = 900
CAPTURE_SECONDS = 180
CASES = tuple(
    f"{fixture}-{region}--{appearance}"
    for fixture in ("ordinary", "representative")
    for region in ("full", "rail")
    for appearance in ("light", "dark")
) + tuple(
    f"{surface}--{appearance}"
    for surface in ("history", "history-clear-data", "library-empty", "library-populated",
                    "settings-general", "settings-profiles", "settings-search",
                    "settings-privacy", "settings-downloads", "settings-appearance",
                    "settings-shortcuts", "library-collections", "library-selection-single",
                    "library-selection-multiple", "library-move-sheet", "library-remove-sheet",
                    "library-collection-create", "history-search", "history-search-empty",
                    "history-clear-range-menu", "history-recently-closed",
                    "settings-general-menu", "settings-search-menu", "settings-motion-menu",
                    "settings-profile-create", "settings-profile-edit",
                    "settings-shortcuts-scroll-1", "settings-shortcuts-scroll-2",
                    "settings-shortcuts-scroll-3", "command-palette-open",
        "command-palette-actions-match", "command-palette-actions-empty",
        "find-open", "find-matches", "find-no-matches", "downloads-empty",
        "downloads-history", "downloads-missing-file", "footer-spaces",
        "footer-hover-downloads", "footer-hover-chip", "footer-plus-open",
        "footer-strip-focus", "footer-downloads-running", "footer-drag-target",
        "footer-overflow", "library-search", "library-search-empty", "library-sort-title", "library-row-rename",
        "library-collection-rename", "library-collection-remove", "library-long-list", "history-dated",
        "history-range-today", "history-selected", "history-cleared", "settings-contrast-menu",
        "settings-profile-create-named", "settings-profiles-multiple", "settings-profile-delete",
        "welcome", "welcome-import", "welcome-default-browser", "welcome-first-space",
        "about", "find-next", "command-palette-url", "command-palette-search",
        "peek-bar", "permission-prompt", "permission-site-settings", "client-certificate-sheet",
        "settings-privacy-populated", "open-app-no-app", "open-app-sheet", "auth-sheet")
    for appearance in ("light", "dark")
)


def terminate_group(process: subprocess.Popen, grace: float) -> None:
    """Kill descendants even when the group leader has already exited."""
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        pass
    until = time.monotonic() + grace
    while time.monotonic() < until:
        process.poll()  # Reap the leader; descendants may still be cleaning up.
        try:
            os.killpg(process.pid, 0)
        except ProcessLookupError:
            break
        except PermissionError:
            # Darwin may report EPERM while a terminated group contains only
            # zombies. This is not absence. Reap our direct child and observe
            # until ESRCH within the original deadline; never signal this now
            # uncertain numeric group again or accept persistent EPERM.
            while time.monotonic() < until:
                process.poll()
                try:
                    os.killpg(process.pid, 0)
                except ProcessLookupError:
                    process.wait(timeout=max(0, until - time.monotonic()))
                    return
                except PermissionError:
                    pass
                time.sleep(min(0.05, max(0, until - time.monotonic())))
            raise RuntimeError("process group cleanup remains unproven after permission ambiguity")
        time.sleep(min(0.05, max(0, until - time.monotonic())))
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    process.wait()


def run(argv, *, cwd=ROOT, seconds=60, capture=False, env=None, check=True,
        cleanup_seconds=3):
    process = subprocess.Popen(
        [str(arg) for arg in argv], cwd=cwd, env=env,
        start_new_session=True, text=True,
        stdout=subprocess.PIPE if capture else None,
    )
    try:
        stdout, _ = process.communicate(timeout=seconds)
    except BaseException:
        terminate_group(process, cleanup_seconds)
        raise
    result = subprocess.CompletedProcess(argv, process.returncode, stdout)
    if check:
        result.check_returncode()
    return result


def install_signal_handlers() -> None:
    # Unwind run() on an outer recipe timeout; its isolated process group must
    # not outlive us. SIGINT already raises KeyboardInterrupt in Python.
    def interrupted(signum, _frame):
        raise SystemExit(128 + signum)
    signal.signal(signal.SIGTERM, interrupted)


def verify_tool() -> dict:
    provenance = json.loads((ROOT / "scripts/visual/tool-source.json").read_text())
    for name, expected in provenance["files"].items():
        path = TOOL / name
        if path.is_symlink() or not path.is_file():
            raise ValueError(f"Missing/non-regular pinned tool source: {name}")
        if hashlib.sha256(path.read_bytes()).hexdigest() != expected:
            raise ValueError(f"Pinned tool source changed: {name}")
    # Cargo discovers build.rs/modules automatically. Hashing only the listed
    # files would miss additions. Never follow mbx's generated target symlink.
    for directory, folders, files in os.walk(TOOL, followlinks=False):
        if Path(directory) == TOOL:
            folders[:] = [name for name in folders if name != "target"]
        for name in folders:
            if (Path(directory) / name).is_symlink():
                raise ValueError(f"Non-regular pinned tool directory: {name}")
        for name in files:
            # os.walk classifies mbx's dangling target link as a file after GC.
            # Exclude the same generated link, never an unexpected regular file.
            if Path(directory) == TOOL and name == "target" and (TOOL / name).is_symlink():
                continue
            relative = str((Path(directory) / name).relative_to(TOOL))
            if relative not in provenance["files"]:
                raise ValueError(f"Unexpected pinned tool source: {relative}")
    return provenance


def build(manifest: Path, *args: str) -> list[dict]:
    result = run(
        ["mbx", "build", "--manifest-path", manifest, "--locked", "--release",
         # Match BBR's host-only build override without modifying the independent
         # upstream manifest. Xcode 27 beta strip corrupts proc-macro LINKEDIT.
         "--config", "profile.release.build-override.strip=false",
         "--message-format=json-render-diagnostics", *args],
        cwd=manifest.parent, seconds=BUILD_SECONDS, capture=True,
    )
    artifacts = []
    for line in result.stdout.splitlines():
        try:
            value = json.loads(line)
        except json.JSONDecodeError:
            continue  # mbx may print cache diagnostics before Cargo's JSON.
        if isinstance(value, dict) and value.get("reason") == "compiler-artifact":
            artifacts.append(value)
    return artifacts


def executable(artifacts: list[dict], name: str, *, test: bool = False) -> Path:
    matches = [Path(a["executable"]) for a in artifacts
               if a.get("executable") and a["target"]["name"] == name
               and a["profile"]["test"] is test]
    if len(matches) != 1 or not matches[0].is_file():
        raise ValueError(f"Expected exactly one built executable: {name} (test={test})")
    return matches[0]


def tool_binary() -> Path:
    verify_tool()
    return executable(build(TOOL / "Cargo.toml", "--bin", "cargo-gpui-visual"),
                      "cargo-gpui-visual")


def capture_environment() -> dict[str, str]:
    env = os.environ.copy()
    env.update(TZ="UTC", LC_ALL="C", LANG="C")
    return env
