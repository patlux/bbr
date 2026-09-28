#!/usr/bin/env python3
"""Produce PNGs for cargo-gpui-visual using BBR's real optimized app."""
import fcntl
import json
import os
from pathlib import Path
import sys

from support import (
    CAPTURE_SECONDS, CASES, ROOT, build, capture_environment,
    executable, install_signal_handlers, run,
)

sys.path.insert(0, str(ROOT / "scripts"))
import with_gui_lock


def output_directory(raw: str) -> Path:
    path = Path(raw)
    # The CLI creates a unique path but not the directory. Limit this adapter's
    # writes to its own repository's ignored runs; refuse reuse and symlinks.
    if not path.is_absolute() or ".." in path.parts:
        raise ValueError("Capture output must be an absolute path without traversal")
    if any(parent.is_symlink() for parent in (path, *path.parents)):
        raise ValueError("Capture output cannot contain a symlink")
    roots = (ROOT / "reports/visual/runs", ROOT / "reports/visual/batches")
    if path.exists() or not any(path != root and path.is_relative_to(root) for root in roots):
        raise ValueError("Capture needs a fresh directory beneath visual runs/batches")
    return path


def write_inventory(output: Path) -> None:
    actual = {p.name for p in output.glob("*.png")}
    expected = {f"{case}.png" for case in CASES}
    if actual != expected:
        raise ValueError(f"Incomplete capture: missing={expected - actual}, extra={actual - expected}")
    if any((output / name).is_symlink() or not (output / name).is_file()
           or (output / name).stat().st_size == 0 for name in expected):
        raise ValueError("Non-regular or empty capture PNG")
    (output / "inventory.json").write_text(json.dumps(CASES, indent=2) + "\n")


def run_suite(test: Path, env: dict[str, str]) -> None:
    # Own the existing repository lock in this process, not a child wrapper.
    # run() reaps the test/app group before this finally releases GUI capacity,
    # including on an outer recipe timeout or interruption.
    path = with_gui_lock.lock_path()
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a+", encoding="utf-8") as lock:
        with_gui_lock.acquire(lock, path, with_gui_lock.timeout_seconds())
        print(f"BBR GUI gate acquired: {path}", file=sys.stderr)
        try:
            run([test, "--ignored", "--nocapture", "--test-threads=1"],
                seconds=CAPTURE_SECONDS, env=env)
        finally:
            fcntl.flock(lock.fileno(), fcntl.LOCK_UN)


def main() -> int:
    output = output_directory(os.environ["GPUI_VISUAL_OUTPUT"])
    artifacts = build(ROOT / "Cargo.toml", "-p", "bbr", "--features", "visual-capture",
                      "--bin", "bbr", "--test", "e2e_visual_shell_capture")
    binary = executable(artifacts, "bbr")
    test = executable(artifacts, "e2e_visual_shell_capture", test=True)
    output.mkdir(parents=True, exist_ok=False)
    env = capture_environment()
    env.update(BBR_VISUAL_CAPTURE_BIN=str(binary), BBR_VISUAL_CAPTURE_DIR=str(output))
    # Build before taking the GUI lock; only the actual suite owns GUI capacity.
    run_suite(test, env)
    write_inventory(output)
    return 0


if __name__ == "__main__":
    install_signal_handlers()
    sys.exit(main())
