#!/usr/bin/env python3
"""One bounded entrypoint; comparison/sealing/reporting remain upstream."""
from __future__ import annotations

import html
import json
import os
from pathlib import Path
import re
import sys
import time

from support import ROOT, capture_environment, install_signal_handlers, run, tool_binary

SUITES = {"gallery": "visual-gallery.toml", "browser": "visual.toml"}
BATCHES = ROOT / "reports/visual/batches"


def safe_path(path: Path) -> None:
    if ".." in path.parts or any(p.is_symlink() for p in (path, *path.parents)):
        raise ValueError("Traversal/symlink paths are not permitted")


def batch_path(raw: str) -> Path:
    path = Path(raw)
    if len(path.parts) == 1:
        path = BATCHES / path
    elif not path.is_absolute():
        path = ROOT / path
    safe_path(path)
    if path.parent != BATCHES or not re.fullmatch(r"\d+-\d+", path.name):
        raise ValueError("Name an explicit visual batch id or its exact directory")
    return path


def suite_args(binary: Path, command: str, batch: Path, suite: str) -> list:
    return [binary, command, "--config", SUITES[suite],
            "--run", batch / suite / "run", "--report", batch / suite / "report"]


def write_summary(batch: Path, outcomes: dict) -> None:
    document = {"schema": 1, "batch": batch.name, "suites": outcomes}
    (batch / "batch.json").write_text(json.dumps(document, indent=2) + "\n")
    sections = []
    for name, outcome in outcomes.items():
        status = {0: "PASS", 1: "REVIEW REQUIRED", 2: "INVALID / FAILED"}[outcome["exit_code"]]
        report = batch / name / "report/index.html"
        link = f'<a href="{name}/report/index.html">Open exact {name} report</a>' if report.is_file() else "No valid report produced"
        sections.append(f'<li><h2>{name}: {status}</h2><p>{link}</p>'
                        f'<p>{html.escape(outcome.get("detail", ""))}</p>'
                        f'<a href="{name}/command.log">Command log</a></li>')
    (batch / "index.html").write_text(
        '<!doctype html><html lang="en"><meta charset="utf-8">'
        '<meta name="viewport" content="width=device-width,initial-scale=1">'
        '<meta http-equiv="Content-Security-Policy" content="default-src \'none\'; style-src \'unsafe-inline\'">'
        '<title>BBR visual snapshots</title><style>body{font:16px system-ui;max-width:900px;'
        'margin:48px auto;padding:0 24px}li{margin-bottom:32px}code{overflow-wrap:anywhere}</style>'
        f'<h1>BBR visual snapshots</h1><p>Batch <code>{batch.name}</code></p>'
        '<p>Gallery is offscreen; browser captures use real windows. '
        'Functional E2E and complete supported-state coverage are separate.</p><ul>'
        + ''.join(sections) + '</ul><p>Review images, then explicitly select references:</p>'
        f'<pre>just visual-update {batch.name} gallery:CASE-ID browser:CASE-ID</pre>'
        '<p>No baseline is updated by this check. No browser or server is opened.</p></html>')


def check(binary: Path) -> int:
    safe_path(BATCHES)
    BATCHES.mkdir(parents=True, exist_ok=True)
    batch = BATCHES / f"{time.time_ns()}-{os.getpid()}"
    batch.mkdir()
    print(f"Visual batch: {batch.name}\nReport: {batch / 'index.html'}", flush=True)
    outcomes = {}
    for suite in SUITES:
        (batch / suite).mkdir()
        started = time.monotonic()
        print(f"\n=== {suite} snapshots ===", flush=True)
        try:
            # mbx can evict the independent CLI target while building a producer.
            # Resolve a fresh artifact when needed; never override managed storage.
            if not binary.is_file():
                binary = tool_binary()
            result = run(suite_args(binary, "check", batch, suite), seconds=3600,
                         env=capture_environment(), check=False, capture=True, cleanup_seconds=8)
            (batch / suite / "command.log").write_text(result.stdout or "")
            code = result.returncode if result.returncode in (0, 1) else 2
            # Preserve upstream verdict/report, including invalid comparisons.
            detail = ""
            review = batch / suite / "report/review.json"
            if review.is_file():
                value = json.loads(review.read_text())
                same = sum(r["status"] == "same" for r in value["results"])
                detail = f"{same}/{len(value['results'])} exact; {len(value['errors'])} errors"
                if value.get("environment_mismatch") is not None:
                    detail += "; environment mismatch"
            elif code != 2:
                raise ValueError("Comparator returned without a review report")
        except Exception as error:
            code, detail = 2, str(error)
            with (batch / suite / "command.log").open("a") as log:
                log.write(f"\nFAILED: {detail}\n")
        outcomes[suite] = {"exit_code": code, "seconds": time.monotonic() - started, "detail": detail}
        write_summary(batch, outcomes)
        print(f"{suite}: exit {code} · {detail}", flush=True)
    print(f"\nReport: {batch / 'index.html'}\nBatch: {batch.name}", flush=True)
    return max(item["exit_code"] for item in outcomes.values())


if __name__ == "__main__":
    install_signal_handlers()
    if len(sys.argv) != 1:
        raise SystemExit("usage: batch.py (no implicit update/accept mode)")
    sys.exit(check(tool_binary()))
