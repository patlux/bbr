#!/usr/bin/env python3
"""Manifest-bound installed macOS SDK export, unpack and full-byte verification.

Adapted from GPUI-Mobile 985815c7 (Apache-2.0); see GPUI-MOBILE-NOTICE.md.
Export requires an explicit installed Xcode Developer directory. This command
records provenance, not permission: the owner must resolve applicable SDK terms.
No upload, download, credential access or machine activation is performed.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import plistlib
import re
import subprocess
import sys
import tarfile

SCHEMA = 1
PLATFORMS = {"macosx": "MacOSX"}


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def relative_path(value):
    if (not isinstance(value, str) or not value or "\\" in value
            or any(ord(c) < 32 for c in value)
            or PurePosixPath(value).is_absolute()
            or any(p in ("", ".", "..") for p in value.split("/"))):
        raise ValueError("non-canonical relative SDK path")
    return value


def safe_link(path, link):
    if (not isinstance(link, str) or not link or "\\" in link
            or any(ord(c) < 32 for c in link) or link.startswith("/")):
        raise ValueError("invalid SDK link")
    joined = os.path.normpath(str(PurePosixPath(path).parent / link))
    if joined == ".." or joined.startswith("../"):
        raise ValueError("SDK link escapes root")


def sha256_value(value):
    return isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value)


def digest(path: Path) -> str:
    with path.open("rb") as file:
        return hashlib.file_digest(file, "sha256").hexdigest()


def sdk_settings(sdk: Path) -> dict:
    settings = plistlib.loads((sdk / "SDKSettings.plist").read_bytes())
    version = settings.get("Version")
    canonical = settings.get("CanonicalName", "")
    if not isinstance(version, str) or not re.fullmatch(r"\d+\.\d+(?:\.\d+)?", version):
        raise ValueError(f"{sdk}: SDKSettings.plist has no usable Version")
    platform = next((key for key, name in PLATFORMS.items() if canonical.startswith(key)), None)
    if platform is None:
        raise ValueError(f"{sdk}: unsupported SDK {canonical!r}; expected {sorted(PLATFORMS)}")
    return {"platform": platform, "version": version, "canonical": canonical}


def installed_sdk(platform: str, developer_dir: str | None = None) -> Path:
    """The selected Xcode's SDK directory; never a symlink target outside Xcode."""
    if sys.platform != "darwin":
        raise ValueError("apple-sdk export needs macOS with the selected Xcode")
    if not developer_dir or not Path(developer_dir).is_absolute():
        raise ValueError("explicit absolute Developer directory required")
    developer = Path(developer_dir).resolve(strict=True)
    if not developer.is_dir():
        raise ValueError("Developer directory required")
    env = dict(os.environ, DEVELOPER_DIR=str(developer))
    path = Path(subprocess.check_output(["xcrun", "--sdk", platform, "--show-sdk-path"],
                                        text=True, env=env, timeout=60).strip()).resolve()
    if not path.is_relative_to(developer) or not (path / "SDKSettings.plist").is_file():
        raise ValueError(f"unexpected SDK path for {platform}: {path}")
    return path


def entries(sdk: Path) -> list[dict]:
    """Every file and relative symlink below the SDK, sorted; nothing else is allowed."""
    result = []
    for path in sorted(sdk.rglob("*")):
        relative = relative_path(path.relative_to(sdk).as_posix())
        if path.is_symlink():
            link = os.readlink(path)
            if Path(link).is_absolute() or not path.resolve().is_relative_to(sdk):
                raise ValueError(f"{path}: symlink leaves the SDK")
            # Xcode ships links like `../../..//System/...`; tar extraction
            # normalizes them, so record the canonical relative form.
            result.append({"path": relative, "link": os.path.normpath(link)})
        elif path.is_file():
            result.append({"path": relative, "sha256": digest(path), "bytes": path.stat().st_size})
        elif not path.is_dir():
            raise ValueError(f"{path}: unsupported entry type")
    return result


def export(platform: str, output: Path, developer_dir: str | None = None) -> dict:
    sdk = installed_sdk(platform, developer_dir)
    settings = sdk_settings(sdk)
    if settings["platform"] != platform:
        raise ValueError(f"{sdk}: SDKSettings platform {settings['platform']} != {platform}")
    output.mkdir(parents=True, exist_ok=True)
    name = f"{PLATFORMS[platform]}.sdk"
    archive = output / f"{name}.tar.gz"
    manifest_path = output / f"{name}.manifest.json"
    if archive.exists() or manifest_path.exists():
        raise ValueError(f"refusing to overwrite {archive} / {manifest_path}")
    records = entries(sdk)
    with tarfile.open(archive, "w:gz") as tar:
        for record in records:
            # macOS SDKs contain hardlinked header aliases. Materialize those
            # as separate regular files to retain the manifest's file model.
            tar.inodes.clear()
            tar.add(sdk / record["path"], arcname=f"{name}/{record['path']}", recursive=False)
    xcode = subprocess.check_output(["/usr/bin/xcodebuild", "-version"], env={**os.environ, "DEVELOPER_DIR": developer_dir}, text=True, timeout=60).strip()
    manifest = {"schema": SCHEMA, "name": name, "source": str(sdk), "xcode": xcode,
                **settings, "entries": records, "archive_sha256": digest(archive)}
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
    return manifest


def load_manifest(path: Path) -> dict:
    manifest = json.loads(path.read_text(), object_pairs_hook=unique_object)
    if (not isinstance(manifest, dict) or manifest.get("schema") != SCHEMA
            or manifest.get("platform") != "macosx"
            or manifest.get("name") != "MacOSX.sdk"
            or not sha256_value(manifest.get("archive_sha256"))
            or not isinstance(manifest.get("entries"), list)):
        raise ValueError(f"{path}: unsupported SDK manifest")
    paths = set()
    links = set()
    for record in manifest["entries"]:
        if not isinstance(record, dict):
            raise ValueError("invalid SDK entry")
        relative = relative_path(record.get("path"))
        if relative in paths:
            raise ValueError("duplicate SDK entry")
        paths.add(relative)
        if set(record) == {"path", "link"}:
            safe_link(relative, record["link"])
            links.add(relative)
        elif (set(record) != {"path", "sha256", "bytes"}
              or not sha256_value(record["sha256"])
              or type(record["bytes"]) is not int or record["bytes"] < 0):
            raise ValueError("invalid SDK file record")
    for relative in paths:
        if any(str(p) in paths for p in PurePosixPath(relative).parents):
            raise ValueError("SDK file/link is ancestor of another entry")
    return manifest


def verify(sdk: Path, manifest_path: Path | None = None) -> dict:
    """Rehash the whole SDK directory against its manifest; extra files fail too."""
    sdk = sdk.resolve()
    manifest_path = manifest_path or sdk.parent / f"{sdk.name}.manifest.json"
    manifest = load_manifest(manifest_path)
    if sdk.name != manifest["name"]:
        raise ValueError(f"{sdk}: manifest describes {manifest['name']}")
    expected = {record["path"]: record for record in manifest["entries"]}
    actual = {record["path"]: record for record in entries(sdk)}
    if actual != expected:
        changed = sorted(set(actual) ^ set(expected)
                         | {p for p in actual if p in expected and actual[p] != expected[p]})
        raise ValueError(f"{sdk}: {len(changed)} entries differ from manifest, e.g. {changed[:5]}")
    settings = sdk_settings(sdk)
    if settings != {k: manifest[k] for k in ("platform", "version", "canonical")}:
        raise ValueError(f"{sdk}: SDKSettings differ from manifest")
    return {**settings, "manifest_sha256": digest(manifest_path), "entries": len(expected),
            "xcode": manifest.get("xcode")}


def unpack(archive: Path, manifest_path: Path, destination: Path) -> Path:
    """Extract only manifest-listed regular files/relative links, then verify."""
    manifest = load_manifest(manifest_path)
    if digest(archive) != manifest["archive_sha256"]:
        raise ValueError(f"{archive}: archive hash differs from manifest")
    name = manifest["name"]
    destination = destination.absolute()
    if destination != destination.resolve():
        raise ValueError("SDK destination must have no symlink components")
    target = destination / name
    if target.exists() or target.is_symlink():
        raise ValueError(f"{target}: already exists; verify it instead of unpacking over it")
    allowed = {f"{name}/{record['path']}": record for record in manifest["entries"]}
    manifest_copy = destination / f"{name}.manifest.json"
    if manifest_copy.exists() and manifest_copy.resolve() != manifest_path.resolve():
        raise ValueError("refusing to replace existing SDK manifest")
    with tarfile.open(archive, "r:gz") as tar:
        members = tar.getmembers()
        if len(members) != len(allowed) or {m.name for m in members} != set(allowed):
            raise ValueError("archive member count differs from manifest")
        for member in members:
            record = allowed.get(member.name)
            if record is None or ".." in Path(member.name).parts:
                raise ValueError(f"archive member not in manifest: {member.name}")
            if member.issym():
                if "link" not in record or os.path.normpath(member.linkname) != record["link"]:
                    raise ValueError(f"unexpected link: {member.name}")
            elif (not member.isfile() or "sha256" not in record
                  or member.size != record["bytes"]):
                raise ValueError(f"unexpected member type/size: {member.name}")
        destination.mkdir(parents=True, exist_ok=True)
        tar.extractall(destination, members=members, filter="data")
    if manifest_copy.resolve() != manifest_path.resolve():
        manifest_copy.write_bytes(manifest_path.read_bytes())
    verify(target, manifest_copy)
    return target


def add_arguments(parser: argparse.ArgumentParser) -> None:
    commands = parser.add_subparsers(dest="sdk_command", required=True)
    exporter = commands.add_parser("export", help="archive the selected Xcode's SDK with a hash manifest (macOS)")
    exporter.add_argument("--platform", choices=sorted(PLATFORMS), default="macosx")
    exporter.add_argument("--output", type=Path, required=True)
    exporter.add_argument("--developer-dir", required=True,
                          help="explicit installed Xcode Developer directory")
    verifier = commands.add_parser("verify", help="rehash an SDK directory against its manifest")
    verifier.add_argument("--sdk", type=Path, required=True)
    verifier.add_argument("--manifest", type=Path)
    unpacker = commands.add_parser("unpack", help="extract a manifest-bound SDK archive, then verify it")
    unpacker.add_argument("--archive", type=Path, required=True)
    unpacker.add_argument("--manifest", type=Path, required=True)
    unpacker.add_argument("--destination", type=Path, required=True)


def main_args(args: argparse.Namespace) -> int:
    if args.sdk_command == "export":
        manifest = export(args.platform, args.output.resolve(), args.developer_dir)
        print(json.dumps({k: manifest[k] for k in ("name", "platform", "version", "xcode", "archive_sha256")}))
    elif args.sdk_command == "verify":
        print(json.dumps(verify(args.sdk, args.manifest)))
    elif args.sdk_command == "unpack":
        print(unpack(args.archive.resolve(), args.manifest.resolve(), args.destination.resolve()))
    return 0


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    add_arguments(parser)
    sys.exit(main_args(parser.parse_args()))
