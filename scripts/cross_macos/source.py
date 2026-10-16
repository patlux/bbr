#!/usr/bin/env python3
"""Export immutable BBR/GPUI commits; unpack and patch only a fresh build snapshot."""
from __future__ import annotations

import argparse
import hashlib
import io
import json
from pathlib import Path
import subprocess
import tarfile
import tomllib

try:
    from . import apple_sdk, prepare
except ImportError:
    import apple_sdk
    import prepare

HERE = Path(__file__).resolve().parent
URL = "https://github.com/gpui-ce/gpui-ce"
GIT_SOURCE = f"git+{URL}?rev={prepare.REVISION}#{prepare.REVISION}"


def git(repo, *args):
    return subprocess.check_output(["git", "-C", str(repo), *args], timeout=60)


def committed_files(repo, revision):
    names = git(repo, "ls-tree", "-rz", "--name-only", revision).decode().split("\0")
    for name in filter(None, names):
        apple_sdk.relative_path(name)
        if any(p == ".env" or p.startswith(".env.") and p != ".env.example"
               or p in ("secrets", ".ssh") for p in Path(name).parts):
            raise ValueError("refusing secret-like source path")
    raw = git(repo, "archive", "--format=tar", revision)
    result = {}
    with tarfile.open(fileobj=io.BytesIO(raw)) as tar:
        for member in tar:
            apple_sdk.relative_path(member.name.rstrip("/"))
            if member.isdir():
                continue
            # No build input follows repository links. GPUI's unused perf license
            # alias is excluded; its root license is retained as a regular file.
            if member.issym() and member.name == "tooling/perf/LICENSE-APACHE":
                continue
            if not member.isfile():
                raise ValueError(f"unsupported source member: {member.name}")
            result[member.name] = (tar.extractfile(member).read(),
                                   0o755 if member.mode & 0o111 else 0o644)
    return result


def export(repo, upstream, revision, output):
    if output.exists() or output.is_symlink() or output.absolute() != output.resolve():
        raise ValueError("new canonical output archive required")
    revision = git(repo, "rev-parse", f"{revision}^{{commit}}").decode().strip()
    if git(upstream, "rev-parse", "HEAD").decode().strip() != prepare.REVISION:
        raise ValueError("GPUI checkout must be at the exact pinned revision")
    sources = {"bbr": committed_files(repo, revision),
               "gpui": committed_files(upstream, prepare.REVISION)}
    upstream_manifest = json.loads((HERE / "upstream.json").read_text())
    for path, sha in upstream_manifest["files"].items():
        if prepare.digest(sources["gpui"][path][0]) != sha:
            raise ValueError(f"GPUI committed input differs: {path}")
    lock = tomllib.loads(sources["bbr"]["Cargo.lock"][0].decode())
    if {p["source"] for p in lock["package"] if URL in p.get("source", "")} != {GIT_SOURCE}:
        raise ValueError("BBR lock does not use the supported GPUI pin")
    files = {f"{root}/{path}": value for root, tree in sources.items()
             for path, value in tree.items()}
    files["gpui-generators.patch"] = ((HERE / "gpui-generators.patch").read_bytes(), 0o644)
    receipt = {"schema": 1, "bbr_revision": revision, "gpui_revision": prepare.REVISION,
               "files": {p: {"sha256": prepare.digest(data), "bytes": len(data), "mode": mode}
                         for p, (data, mode) in sorted(files.items())}}
    with output.open("xb") as handle, tarfile.open(fileobj=handle, mode="w:gz") as tar:
        for path, (data, mode) in sorted(files.items()):
            info = tarfile.TarInfo(path)
            info.size, info.mode, info.mtime = len(data), mode, 1577836800
            tar.addfile(info, io.BytesIO(data))
        data = (json.dumps(receipt, sort_keys=True, indent=2) + "\n").encode()
        info = tarfile.TarInfo("source.json")
        info.size, info.mode, info.mtime = len(data), 0o644, 1577836800
        tar.addfile(info, io.BytesIO(data))
    return {"archive_sha256": apple_sdk.digest(output), "bbr_revision": revision,
            "gpui_revision": prepare.REVISION, "files": len(files)}


def registry(lock):
    return sorted((p["name"], p["version"], p["source"], p.get("checksum"))
                  for p in lock["package"] if p.get("source", "").startswith("registry+"))


def patch_snapshot(root):
    manifest = root / "bbr/Cargo.toml"
    original = manifest.read_text()
    parsed = tomllib.loads(original)
    if URL in parsed.get("patch", {}):
        raise ValueError("existing GPUI Git override requires review")
    needle = f'gpui = {{ git = "{URL}", rev = "{prepare.REVISION}" }}'
    if original.count(needle) != 1:
        raise ValueError("expected one crates.io GPUI override")
    local = root / "gpui/crates/gpui"
    text = original.replace(needle, f'gpui = {{ path = {json.dumps(str(local))} }}')
    text += f'\n[patch."{URL}"]\n'
    packages = {}
    for path in (root / "gpui/crates").glob("*/Cargo.toml"):
        package = tomllib.loads(path.read_text()).get("package", {})
        if isinstance(package.get("name"), str):
            packages[package["name"]] = path.parent
    lockpath = root / "bbr/Cargo.lock"
    lock = tomllib.loads(lockpath.read_text())
    expected = {p["name"] for p in lock["package"] if p.get("source") == GIT_SOURCE}
    if not expected or not expected <= packages.keys():
        raise ValueError("GPUI export lacks locked packages")
    for name in sorted(expected):
        text += f'{name} = {{ path = {json.dumps(str(packages[name]))} }}\n'
    patch = root / "gpui-generators.patch"
    for args in (["--check"], []):
        subprocess.run(["git", "apply", *args, str(patch)], cwd=root / "gpui",
                       check=True, capture_output=True, timeout=30)
    (root / "original-Cargo.lock").write_bytes(lockpath.read_bytes())
    manifest.write_text(text)
    lockpath.write_text(lockpath.read_text().replace(f'source = "{GIT_SOURCE}"\n', ""))
    return {"gpui_packages": sorted(expected), "registry": registry(lock)}


def unpack(archive, expected_sha, root):
    if not apple_sdk.sha256_value(expected_sha) or apple_sdk.digest(archive) != expected_sha:
        raise ValueError("source archive SHA-256 mismatch")
    if root.exists() or root.is_symlink() or root.absolute() != root.resolve():
        raise ValueError("fresh canonical build root required")
    with tarfile.open(archive) as tar:
        members = tar.getmembers()
        if len({m.name for m in members}) != len(members):
            raise ValueError("duplicate source archive member")
        for m in members:
            apple_sdk.relative_path(m.name)
            if not m.isfile() or m.mode not in (0o644, 0o755):
                raise ValueError("non-regular or unsafe source entry")
        manifest = next(m for m in members if m.name == "source.json")
        receipt = json.loads(tar.extractfile(manifest).read(), object_pairs_hook=apple_sdk.unique_object)
        if (receipt.get("schema") != 1 or receipt.get("gpui_revision") != prepare.REVISION
                or {m.name for m in members} != set(receipt["files"]) | {"source.json"}):
            raise ValueError("source receipt membership differs")
        for member in members:
            if member.name == "source.json":
                continue
            data = tar.extractfile(member).read()
            expected = receipt["files"][member.name]
            if {"sha256": prepare.digest(data), "bytes": len(data), "mode": member.mode} != expected:
                raise ValueError("source member differs from receipt")
        root.mkdir(mode=0o700)
        tar.extractall(root, filter="data")
    graph = patch_snapshot(root)
    inputs = {p.relative_to(root).as_posix(): apple_sdk.digest(p)
              for tree in (root / "bbr", root / "gpui") for p in sorted(tree.rglob("*"))
              if p.is_file()}
    (root / "prepared.json").write_text(json.dumps({"archive_sha256": expected_sha,
        "source": receipt, "graph": graph, "inputs": inputs}, indent=2) + "\n")
    return root


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    exp = commands.add_parser("export")
    exp.add_argument("--repo", type=Path, required=True)
    exp.add_argument("--upstream", type=Path, required=True)
    exp.add_argument("--revision", required=True)
    exp.add_argument("--output", type=Path, required=True)
    extract = commands.add_parser("unpack")
    extract.add_argument("--archive", type=Path, required=True)
    extract.add_argument("--sha256", required=True)
    extract.add_argument("--root", type=Path, required=True)
    args = parser.parse_args()
    if args.command == "export":
        print(json.dumps(export(args.repo, args.upstream, args.revision, args.output), indent=2))
    else:
        print(unpack(args.archive, args.sha256, args.root))


if __name__ == "__main__":
    main()
