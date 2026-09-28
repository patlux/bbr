#!/usr/bin/env python3
"""Unsealed, windowless Gallery probes; never compare or accept baselines."""
from __future__ import annotations

import argparse
import fcntl
import json
import os
from pathlib import Path
import re
import struct
import sys
import time

from support import ROOT, build, capture_environment, executable, install_signal_handlers, run

sys.path.insert(0, str(ROOT / "scripts"))
import with_gui_lock

CAPTURE_SECONDS = 600
CASE_ID = re.compile(r"gallery-(\d+)-state-\d+-(?:standard|increased)-(?:small|normal|extralarge)--(?:light|dark)")


def inventory(text: str, story: int | None) -> dict[str, tuple[str, str]]:
    cases = {}
    for line in text.splitlines():
        fields = line.split("\t")
        if len(fields) != 3 or not (match := CASE_ID.fullmatch(fields[0])):
            raise ValueError("Malformed Gallery inventory")
        if fields[0] in cases:
            raise ValueError("Duplicate Gallery inventory case")
        cases[fields[0]] = (fields[1], fields[2])
    if story is not None:
        cases = {key: value for key, value in cases.items()
                 if int(CASE_ID.fullmatch(key).group(1)) == story}
    if not cases:
        raise ValueError("No Gallery cases for requested story")
    return cases


def validate(output: Path, cases: dict[str, tuple[str, str]]) -> dict:
    coverage = output / "coverage.tsv"
    if coverage.is_symlink() or not coverage.is_file():
        raise ValueError("Missing regular Gallery completion marker")
    files = {"coverage.tsv"}
    ranges: dict[str, tuple[int, int, int]] = {}
    for line in coverage.read_text().splitlines():
        fields = line.split("\t")
        if len(fields) != 7:
            raise ValueError("Malformed Gallery coverage row")
        page, case, label, state, start, end, total = fields
        if case not in cases or (label, state) != cases[case]:
            raise ValueError("Unknown case or mismatched fixture labels")
        start, end, total = int(start), int(end), int(total)
        previous, known_total, index = ranges.get(case, (0, total, 0))
        if not (start == previous < end <= total == known_total):
            raise ValueError("Incomplete, overlapping or inconsistent child ranges")
        if page != f"{case}-page-{index:02}":
            raise ValueError("Non-canonical or duplicated page identity")
        name = f"{page}.png"
        png = output / name
        if png.is_symlink() or not png.is_file():
            raise ValueError("Missing or non-regular Gallery PNG")
        with png.open("rb") as stream:
            header = stream.read(24)
        if (len(header) != 24 or header[:16] != b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR"
                or struct.unpack(">II", header[16:24]) != (2400, 2000)):
            raise ValueError("Gallery PNG must be native 2400x2000, never rescaled")
        files.add(name)
        ranges[case] = (end, total, index + 1)
    if set(ranges) != set(cases) or any(end != total for end, total, _ in ranges.values()):
        raise ValueError("Gallery coverage omits cases or children")
    if {path.name for path in output.iterdir()} != files:
        raise ValueError("Unexpected Gallery output entries")
    return {"cases": len(cases), "pages": len(files) - 1}


def capture(binary: Path, output: Path, story: int | None, env: dict[str, str]) -> None:
    # Retain the common host lock for GPU serialization. Own it here so timeout
    # and signal cleanup reap the producer group BEFORE releasing capacity.
    path = with_gui_lock.lock_path()
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a+", encoding="utf-8") as lock:
        with_gui_lock.acquire(lock, path, with_gui_lock.timeout_seconds())
        try:
            args = [binary, "--capture-gallery", output]
            if story is not None:
                args += ["--story", str(story)]
            run(args, seconds=CAPTURE_SECONDS, env=env)
        finally:
            fcntl.flock(lock.fileno(), fcntl.LOCK_UN)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--story", default="all", help="registry story id, or all")
    args = parser.parse_args()
    story = None if args.story == "all" else int(args.story)
    if story is not None and story < 0:
        parser.error("story id must be nonnegative")
    binary = executable(build(ROOT / "Cargo.toml", "-p", "bbr-gallery",
                              "--features", "visual-capture"), "bbr-gallery")
    env = capture_environment()
    env["SEED"] = "0"
    cases = inventory(run([binary, "--capture-inventory"], seconds=30,
                          capture=True, env=env).stdout, story)
    parent = ROOT / "reports/visual/gallery-probes"
    if any(path.is_symlink() for path in (parent, *parent.parents)):
        raise ValueError("Gallery output ancestors cannot be symlinks")
    parent.mkdir(parents=True, exist_ok=True)
    output = parent / f"{time.time_ns()}-{os.getpid()}"
    started = time.monotonic()
    capture(binary, output, story, env)
    summary = validate(output, cases)
    print(json.dumps({"output": str(output), **summary,
                      "capture_and_validation_seconds": time.monotonic() - started,
                      "renderer": "macos-offscreen-metal-real-text",
                      "sealed": False, "baseline_acceptance": False}, indent=2))
    return 0


if __name__ == "__main__":
    install_signal_handlers()
    sys.exit(main())
