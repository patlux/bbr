#!/usr/bin/env python3
"""Validate local Markdown links in BBR's roadmap and architecture docs."""

from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parent.parent
DOCUMENTS = [
    ROOT / "AGENTS.md",
    ROOT / "ROADMAP.md",
    ROOT / "PLAN.md",
    ROOT / "ARCHITECTURE.md",
    ROOT / "E2E_ARCHITECTURE.md",
    *sorted((ROOT / "docs" / "roadmap").rglob("*.md")),
]
LINK_PATTERN = re.compile(r"(?<!!)\[[^\]]+\]\(([^)]+)\)")


def main() -> int:
    failures: list[str] = []
    for source in DOCUMENTS:
        if not source.exists():
            failures.append(f"missing roadmap document: {source.relative_to(ROOT)}")
            continue
        for raw_target in LINK_PATTERN.findall(source.read_text()):
            target = raw_target.split("#", 1)[0]
            if not target or "://" in target or target.startswith("mailto:"):
                continue
            destination = (source.parent / target).resolve()
            if not destination.exists():
                failures.append(
                    f"{source.relative_to(ROOT)}: broken link {raw_target!r}"
                )

    if failures:
        print("Roadmap validation failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print(f"Roadmap links valid ({len(DOCUMENTS)} Markdown files checked).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
