#!/usr/bin/env python3
"""Stage exact Linux build receipts, sources and executables for read-only Tart transfer."""
from __future__ import annotations

import argparse
import hashlib
import io
import json
from pathlib import Path
import tarfile

try:
    from . import apple_sdk, build
except ImportError:
    import apple_sdk
    import build


def stage(root, outputs, output):
    prepared = build.verify_inputs(root)
    if output.exists() or output.is_symlink() or output.absolute() != output.resolve():
        raise ValueError("fresh canonical payload output required")
    files = {}
    for relative, sha in prepared["inputs"].items():
        if relative.startswith("bbr/"):
            path = root / relative
            files[relative.removeprefix("bbr/")] = (path, sha)
    tests, bundle, records = {}, False, []
    for directory in outputs:
        receipt_path = directory / "build-receipt.json"
        receipt = json.loads(receipt_path.read_text(), object_pairs_hook=apple_sdk.unique_object)
        if (receipt["prepared_sha256"] != apple_sdk.digest(root / "prepared.json")
                or receipt["target"] != build.TARGET):
            raise ValueError("build receipt belongs to another snapshot/target")
        records.append({"path": str(directory), "sha256": apple_sdk.digest(receipt_path)})
        for relative, data in receipt["executables"].items():
            apple_sdk.relative_path(relative)
            if not relative.startswith(f"target/{build.TARGET}/"):
                raise ValueError("unsupported executable destination")
            candidate = (root / "bbr" / relative, data["sha256"])
            if relative in files and files[relative][1] != data["sha256"]:
                raise ValueError("different executable variants share a path")
            files[relative] = candidate
            if data["test"]:
                if data["target"] in tests:
                    raise ValueError("duplicate test target")
                tests[data["target"]] = relative
        for relative, data in receipt["bundle"].items():
            apple_sdk.relative_path(relative)
            if not relative.startswith("BBR.app/") or relative in files:
                raise ValueError("duplicate/unsupported bundle destination")
            bundle = True
            files[relative] = (directory / relative, data["sha256"])
    # Verify every input before creating output. No raw target-tree or home share.
    manifest = {}
    for relative, (path, sha) in sorted(files.items()):
        if path.is_symlink() or not path.is_file() or apple_sdk.digest(path) != sha:
            raise ValueError(f"stale build/source bytes: {path}")
        manifest[relative] = {"sha256": sha, "bytes": path.stat().st_size,
                              "mode": 0o755 if path.stat().st_mode & 0o111 else 0o644}
    output.mkdir(mode=0o700)
    archive = output / "payload.tar.gz"
    with tarfile.open(archive, "w:gz") as tar:
        for relative, (path, sha) in sorted(files.items()):
            data = path.read_bytes()
            if hashlib.sha256(data).hexdigest() != sha:
                raise ValueError("input changed while staging")
            info = tarfile.TarInfo(relative)
            info.size, info.mode, info.mtime = len(data), manifest[relative]["mode"], 1577836800
            tar.addfile(info, io.BytesIO(data))
    receipt = {"schema": 1, "repo": str(root / "bbr"), "source": prepared["source"]["bbr_revision"],
               "prepared_sha256": apple_sdk.digest(root / "prepared.json"), "builds": records,
               "archive_sha256": apple_sdk.digest(archive), "files": manifest,
               "tests": tests, "production_bundle": bundle}
    (output / "SHA256SUMS").write_text("".join(f'{r["sha256"]}  {p}\n' for p, r in manifest.items()))
    (output / "payload.json").write_text(json.dumps(receipt, indent=2) + "\n")
    return {"payload_sha256": apple_sdk.digest(output / "payload.json"),
            "archive_sha256": receipt["archive_sha256"], "files": len(files)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--build", type=Path, action="append", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(stage(args.root, args.build, args.output), indent=2))


if __name__ == "__main__":
    main()
