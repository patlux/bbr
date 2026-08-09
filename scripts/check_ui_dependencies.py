#!/usr/bin/env python3
"""The G1a bbr-ui dependency-boundary audit gate (the proposed `just ui-deps`).

Audits, from `cargo metadata` and the crate source:

1.  **Exact direct set** — `bbr-ui`'s direct *normal* dependencies are exactly
    `gpui` and `bbr-domain` (the standard library is implicit).
2.  **Exact first-party closure** — every first-party package reachable from
    `bbr-ui` through normal dependency edges is exactly `bbr-domain` plus the
    existing `bbr-domain -> bbr-web` identity edge (`bbr-web`); no other
    first-party package (`bbr-browser`, `bbr-app`, `bbr-testkit`,
    `bbr-fixtures`, `bbr-persistence`, `bbr-split`, `bbr-command`,
    `bbr-web-macos`, `bbr-gallery`) may appear anywhere in the closure.
3.  **Every path to `bbr-web`** is exactly `bbr-ui -> bbr-domain -> bbr-web` —
    the approved portable identity path documented in ARCHITECTURE.md. No
    other route (direct, or through another crate) is permitted.
4.  **Forbidden source imports/API use** — no `bbr-ui` source file imports or
    names a forbidden first-party crate (`use bbr_*` / `bbr_*::` tokens).
5.  **Diagnostic evidence** — `cargo tree -p bbr-ui --edges normal` is printed
    for the reviewer.

The gate deliberately does **not** flat-blacklist third-party transitive
packages: the explicitly allowed GPUI-CE source legitimately brings platform
packages such as `raw-window-handle`, the objc2/AppKit support crates, and
`serde`. GPUI's own audited platform closure is not `bbr-ui` product
authority.

Exit status: 0 when every check passes, 1 otherwise (each failure names the
exact contract violation).
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

ALLOWED_DIRECT = {"gpui", "bbr-domain"}
# First-party packages allowed anywhere in bbr-ui's normal closure.
ALLOWED_FIRST_PARTY = {"bbr-domain", "bbr-web"}
# Every other first-party package is forbidden in the closure and in source.
FORBIDDEN_FIRST_PARTY = {
    "bbr-browser",
    "bbr-app",
    "bbr-testkit",
    "bbr-fixtures",
    "bbr-persistence",
    "bbr-split",
    "bbr-command",
    "bbr-web-macos",
    "bbr-gallery",
}
# Rust source spelling of the crates `bbr-ui` may never import directly.
# `bbr-web` is included even though it is allowed in the closure via
# `bbr-domain`: the source must never import it itself (the approved path
# is exactly `bbr-ui -> bbr-domain -> bbr-web`).
FORBIDDEN_SOURCE_TOKENS = {
    name.replace("-", "_")
    for name in FORBIDDEN_FIRST_PARTY | {"bbr-web"}
}

USE_IMPORT_RE = re.compile(r"\buse\s+([a-zA-Z0-9_]+)")
PATH_IMPORT_RE = re.compile(r"\b([a-zA-Z0-9_]+)::")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Audit the bbr-ui dependency boundary (direct set, "
        "first-party closure, bbr-web paths, forbidden source imports)."
    )
    parser.add_argument(
        "--manifest-path",
        default="crates/bbr-ui/Cargo.toml",
        help="bbr-ui manifest path (default: crates/bbr-ui/Cargo.toml)",
    )
    parser.add_argument(
        "--src-dir",
        default="crates/bbr-ui/src",
        help="bbr-ui source directory (default: crates/bbr-ui/src)",
    )
    parser.add_argument(
        "--cargo",
        default="cargo",
        help="cargo binary (default: cargo)",
    )
    return parser.parse_args()


def cargo_metadata(manifest_path: str, cargo: str) -> dict:
    out = subprocess.run(
        [cargo, "metadata", "--manifest-path", manifest_path, "--format-version", "1"],
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(out.stdout)


def _is_normal(entry: dict) -> bool:
    """A dependency edge is normal when its (possibly absent) kind is the
    default: cargo emits `kind: None` for normal deps on this toolchain."""
    return entry.get("kind") in (None, "normal")


def direct_dependency_names(metadata: dict, package_name: str) -> set[str]:
    """The direct *normal* dependency names of `package_name`, per metadata."""
    package = next(p for p in metadata["packages"] if p["name"] == package_name)
    names = set()
    for dep in package["dependencies"]:
        kinds = dep.get("dep_kinds") or [{"kind": "normal"}]
        if any(_is_normal(entry) for entry in kinds):
            names.add(dep["name"])
    return names


def normal_edges(metadata: dict) -> dict[str, list[str]]:
    """Adjacency of normal dependency edges: pkg id -> [dep pkg ids]."""
    adjacency: dict[str, list[str]] = {}
    for node in metadata["resolve"]["nodes"]:
        ids = []
        for dep in node.get("deps", []):
            kinds = dep.get("dep_kinds") or [{"kind": "normal"}]
            if any(_is_normal(entry) for entry in kinds):
                ids.append(dep["pkg"])
        adjacency[node["id"]] = ids
    return adjacency


def package_by_id(metadata: dict) -> dict[str, dict]:
    return {p["id"]: p for p in metadata["packages"]}


def first_party_closure(
    adjacency: dict[str, list[str]], packages: dict[str, dict], root_id: str
) -> set[str]:
    """All first-party package names reachable from `root_id` via normal edges."""
    seen: set[str] = set()
    stack = [root_id]
    closure: set[str] = set()
    root_name = packages[root_id]["name"]
    while stack:
        node_id = stack.pop()
        if node_id in seen:
            continue
        seen.add(node_id)
        name = packages[node_id]["name"]
        if name.startswith("bbr-") and name != root_name:
            closure.add(name)
        for dep_id in adjacency.get(node_id, []):
            if dep_id not in seen:
                stack.append(dep_id)
    return closure


def paths_to(
    adjacency: dict[str, list[str]], packages: dict[str, dict], root_id: str, target: str
) -> list[list[str]]:
    """Every simple path from `root_id` to the package named `target`."""
    results: list[list[str]] = []

    def dfs(node_id: str, trail: list[str]) -> None:
        name = packages[node_id]["name"]
        if name == target:
            results.append(trail + [name])
            return
        for dep_id in adjacency.get(node_id, []):
            if dep_id not in trail_ids:
                trail_ids.add(dep_id)
                dfs(dep_id, trail + [name])
                trail_ids.remove(dep_id)

    trail_ids = {root_id}
    dfs(root_id, [])
    return results


def forbidden_source_imports(src_dir: Path) -> list[str]:
    """Every forbidden crate token referenced by a `use` or path in the source.
    One entry per (file, token) — a `use bbr_web::…` line is one violation,
    not two."""
    offending: list[tuple[Path, str]] = []
    for path in sorted(src_dir.rglob("*.rs")):
        text = path.read_text()
        for token in sorted(FORBIDDEN_SOURCE_TOKENS):
            for pattern in (
                rf"\buse\s+{re.escape(token)}\b",
                rf"\b{re.escape(token)}::",
            ):
                if re.search(pattern, text):
                    offending.append((path, token))
                    break
    return [f"{path}: forbidden source reference of `{token}`" for path, token in offending]


def direct_manifest_keys(manifest: Path) -> set[str]:
    text = manifest.read_text()
    keys = set()
    in_deps = False
    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            in_deps = stripped == "[dependencies]"
            continue
        if not in_deps:
            continue
        if not stripped or stripped.startswith("#"):
            continue
        key = stripped.split("=")[0].strip()
        if key:
            keys.add(key)
    return keys


def run_checks(args: argparse.Namespace) -> int:
    failures: list[str] = []
    manifest = Path(args.manifest_path)
    src_dir = Path(args.src_dir)

    # --- 0. Manifest direct-key check (fast, before metadata) ---
    keys = direct_manifest_keys(manifest)
    if keys != ALLOWED_DIRECT:
        failures.append(
            f"direct manifest keys {sorted(keys)} != allowed {sorted(ALLOWED_DIRECT)}"
        )

    # --- cargo metadata + cargo tree evidence ---
    metadata = cargo_metadata(args.manifest_path, args.cargo)
    packages = package_by_id(metadata)
    root = next(p for p in metadata["packages"] if p["name"] == "bbr-ui")
    adjacency = normal_edges(metadata)

    # --- 1. Exact direct normal dependency set ---
    direct = direct_dependency_names(metadata, "bbr-ui")
    if direct != ALLOWED_DIRECT:
        failures.append(
            f"direct normal dependencies {sorted(direct)} != allowed "
            f"{sorted(ALLOWED_DIRECT)}"
        )

    # --- 2. Exact first-party normal closure ---
    closure = first_party_closure(adjacency, packages, root["id"])
    if closure != ALLOWED_FIRST_PARTY:
        failures.append(
            f"first-party normal closure {sorted(closure)} != allowed "
            f"{sorted(ALLOWED_FIRST_PARTY)}"
        )

    # --- 3. Every path to bbr-web is exactly bbr-ui -> bbr-domain -> bbr-web ---
    web_paths = paths_to(adjacency, packages, root["id"], "bbr-web")
    approved = ["bbr-ui", "bbr-domain", "bbr-web"]
    if not web_paths:
        failures.append("no path to bbr-web found (the approved identity edge is missing)")
    for path in web_paths:
        if path != approved:
            failures.append(
                f"path to bbr-web {path} != approved {approved} (no direct "
                "bbr-ui -> bbr-web or alternate-route edge is permitted)"
            )

    # --- 4. Forbidden source imports/API use ---
    offending = forbidden_source_imports(src_dir)
    if offending:
        failures.append("forbidden source imports/API use:\n  " + "\n  ".join(offending))

    # --- 5. Diagnostic evidence ---
    print("== cargo tree -p bbr-ui --edges normal ==")
    tree = subprocess.run(
        [
            args.cargo,
            "tree",
            "--manifest-path",
            args.manifest_path,
            "--edges",
            "normal",
            "-p",
            "bbr-ui",
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    print(tree.stdout, end="")
    if tree.stderr:
        print(tree.stderr, file=sys.stderr, end="")

    if failures:
        print("\nUI-DEPS FAIL:\n  " + "\n  ".join(failures), file=sys.stderr)
        return 1
    print("\nUI-DEPS PASS: direct set, first-party closure, bbr-web paths, "
          "and source imports all match the approved boundary.")
    return 0


def main() -> int:
    args = parse_args()
    try:
        return run_checks(args)
    except subprocess.CalledProcessError as exc:
        print(f"UI-DEPS ERROR: cargo invocation failed: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
