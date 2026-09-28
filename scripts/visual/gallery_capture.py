#!/usr/bin/env python3
"""Raw Gallery adapter for the unchanged comparator's source/environment seal."""
import json
import os
import re
import sys
from pathlib import Path

from gallery import build_producer, capture, inventory, validate
from support import ROOT, install_signal_handlers, run

PAGE_ID = re.compile(r"(.+)--(light|dark)-page-(\d+)")


def output_directory(raw: str) -> Path:
    path = Path(raw)
    roots = (ROOT / "reports/visual/gallery-runs", ROOT / "reports/visual/batches")
    if not path.is_absolute() or ".." in path.parts:
        raise ValueError("Gallery output must be absolute without traversal")
    if any(p.is_symlink() for p in (path, *path.parents)):
        raise ValueError("Gallery output cannot contain symlinks")
    if path.exists() or not any(path != root and path.is_relative_to(root) for root in roots):
        raise ValueError("Gallery output must be fresh beneath its configured runs")
    path.parent.mkdir(parents=True, exist_ok=True)
    return path


def comparator_id(page: str) -> str:
    match = PAGE_ID.fullmatch(page)
    if not match:
        raise ValueError("Invalid Gallery page identity")
    base, mode, index = match.groups()
    return f"{base}-page-{index}--{mode}"


def prepare_inventory(output: Path) -> list[str]:
    # validate() has already proven registry/range completeness. The unchanged
    # comparator requires appearance LAST; rename files only, never pixels.
    mappings = [(p, comparator_id(p.stem)) for p in sorted(output.glob("*.png"))]
    ids = {case for _, case in mappings}
    if len(ids) != len(mappings) or not ids:
        raise ValueError("Empty or duplicate Gallery page inventory")
    for case in ids:
        base, mode = case.rsplit("--", 1)
        counterpart = f"{base}--{'dark' if mode == 'light' else 'light'}"
        if counterpart not in ids:
            raise ValueError("Gallery pages have no complete Light/Dark pairing")
    for path, case in mappings:
        destination = output / f"{case}.png"
        if destination.exists() or destination.is_symlink():
            raise ValueError("Gallery canonical page already exists")
        path.rename(destination)
    result = sorted(ids)
    (output / "inventory.json").write_text(json.dumps(result, indent=2) + "\n")
    return result


def main() -> int:
    output = output_directory(os.environ["GPUI_VISUAL_OUTPUT"])
    binary, env = build_producer()
    cases = inventory(run([binary, "--capture-inventory"], seconds=30,
                          capture=True, env=env).stdout, None)
    capture(binary, output, None, env)
    validate(output, cases)
    prepare_inventory(output)
    return 0


if __name__ == "__main__":
    install_signal_handlers()
    sys.exit(main())
