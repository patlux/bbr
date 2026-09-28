#!/usr/bin/env python3
"""Explicit reviewed selections only; never capture, accept-all or choose latest."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import tomllib

from batch import SUITES, batch_path, safe_path, suite_args
from support import ROOT, capture_environment, install_signal_handlers, run, tool_binary

CASE = re.compile(r"[a-z0-9][a-z0-9-]*--(?:light|dark)")


def selection(args: list[str]) -> tuple[Path, dict[str, list[str]]]:
    if len(args) < 2:
        raise ValueError("usage: just visual-update <batch> <gallery:case|browser:case>...")
    batch = batch_path(args[0])
    selected: dict[str, list[str]] = {}
    for item in args[1:]:
        suite, separator, case = item.partition(":")
        if not separator or suite not in SUITES or not CASE.fullmatch(case):
            raise ValueError("Name exact suite:case ids; no wildcards, latest or accept-all")
        if case in selected.setdefault(suite, []):
            raise ValueError("Duplicate selected snapshot")
        selected[suite].append(case)
    return batch, selected


def digest(path: Path) -> str:
    safe_path(path)
    return hashlib.sha256(path.read_bytes()).hexdigest()


def preflight(binary: Path, batch: Path, suite: str, cases: list[str]) -> None:
    # The authoritative validator proves source/image seals. Check review binding
    # for EVERY selected suite before invoking any writes, not only the first.
    config = tomllib.loads((ROOT / SUITES[suite]).read_text())
    baselines = ROOT / config["baselines"]
    safe_path(baselines)
    capture = batch / suite / "run/capture.json"
    review_path = batch / suite / "report/review.json"
    safe_path(capture)
    safe_path(review_path)
    manifest = json.loads(capture.read_text())
    review = json.loads(review_path.read_text())
    if review.get("schema") != 1 or review.get("capture_hash") != digest(capture):
        raise ValueError(f"{suite}: capture changed since review; run just visual again")
    state = {path.name: digest(path) for path in baselines.iterdir()
             if path.suffix == ".png" or path.name == "baseline.json"} if baselines.exists() else {}
    if review.get("baseline_state") != state:
        raise ValueError(f"{suite}: baselines changed since review; run just visual again")
    if review.get("errors") or review.get("environment") != manifest.get("environment"):
        raise ValueError(f"{suite}: invalid comparison/environment")
    if review.get("environment_mismatch") is not None:
        raise ValueError(f"{suite}: environment drift needs explicit upstream --adopt-environment review")
    results = {item["case"]: item["status"] for item in review["results"]}
    if len(results) != len(review["results"]) or set(results) != set(manifest["cases"]):
        raise ValueError(f"{suite}: review inventory mismatch")
    if any(results.get(case) not in ("changed", "missing-baseline") for case in cases):
        raise ValueError(f"{suite}: select only reviewed changed/missing snapshots")
    run([binary, "validate", "--run", capture.parent], seconds=600, env=capture_environment())


def update(binary: Path, batch: Path, selected: dict[str, list[str]]) -> int:
    for suite, cases in selected.items():
        preflight(binary, batch, suite, cases)
    for suite, cases in selected.items():
        args = suite_args(binary, "accept", batch, suite)
        args.extend(arg for case in cases for arg in ("--case", case))
        result = run(args, seconds=600, env=capture_environment(), check=False)
        if result.returncode:
            print(f"Stopped at {suite}; earlier suites may have been updated. Inspect git diff; "
                  "do not retry or roll back blindly.", file=sys.stderr)
            return result.returncode
    print("Updated only the named reviewed snapshots. Inspect git diff, then run just visual.")
    return 0


if __name__ == "__main__":
    install_signal_handlers()
    try:
        batch, selected = selection(sys.argv[1:])
        sys.exit(update(tool_binary(), batch, selected))
    except (ValueError, OSError, KeyError, subprocess.SubprocessError) as error:
        print(f"INVALID: {error}", file=sys.stderr)
        sys.exit(2)
