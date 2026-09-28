#!/usr/bin/env python3
"""Run the pinned portable CLI; never install, download, serve or auto-accept."""
import sys

from support import ROOT, capture_environment, install_signal_handlers, run, tool_binary


def main() -> int:
    if len(sys.argv) < 2 or sys.argv[1] not in {
        "check", "compare", "accept", "doctor", "validate", "sources"
    }:
        raise ValueError("Expected check, compare, accept, doctor, validate or sources")
    binary = tool_binary()
    return run([binary, *sys.argv[1:]], cwd=ROOT, seconds=3600,
               env=capture_environment(), check=False, cleanup_seconds=8).returncode


if __name__ == "__main__":
    install_signal_handlers()
    sys.exit(main())
