#!/usr/bin/env python3
"""Translate explicit just recipe case arguments to upstream --case flags."""
import sys

from support import CASES, ROOT, install_signal_handlers, run, tool_binary


def acceptance_args(args: list[str]) -> list[str]:
    if len(args) < 2:
        raise ValueError("Expected an explicit run and at least one reviewed case")
    run_path, *cases = args
    if len(set(cases)) != len(cases) or any(case not in CASES for case in cases):
        raise ValueError("Cases must be unique exact BBR inventory ids (no wildcards)")
    return ["accept", "--run", run_path, *[arg for case in cases for arg in ("--case", case)]]


if __name__ == "__main__":
    install_signal_handlers()
    args = acceptance_args(sys.argv[1:])
    sys.exit(run([tool_binary(), *args], cwd=ROOT, seconds=120, check=False).returncode)
