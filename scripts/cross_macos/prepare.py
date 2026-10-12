#!/usr/bin/env python3
"""Export exact pinned inputs for an SDK-free, inactive GPUI generator probe."""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
REVISION = "568271c01c5a9d640d7b091e5bfd136b04035f28"


def digest(data):
    return hashlib.sha256(data).hexdigest()


def checked_inputs(source):
    manifest = json.loads((HERE / "upstream.json").read_text())
    if manifest["revision"] != REVISION:
        raise ValueError("unexpected upstream revision in manifest")
    head = subprocess.run(
        ["git", "-C", str(source), "rev-parse", "HEAD"],
        check=True, capture_output=True, text=True, timeout=10,
    ).stdout.strip()
    if head != REVISION:
        raise ValueError(f"expected GPUI-CE {REVISION}, got {head}")
    inputs = {}
    for relative, expected in manifest["files"].items():
        path = source / relative
        if path.resolve() != path or not path.is_file():
            raise ValueError(f"upstream input is not a regular non-symlink file: {relative}")
        data = path.read_bytes()
        if digest(data) != expected:
            raise ValueError(f"upstream input differs from pin: {relative}")
        inputs[relative] = data
    return inputs


def prepare(source, output, baseline=False):
    source = source.resolve(strict=True)
    output = output.absolute()
    if output != output.resolve() or output.exists():
        raise ValueError("output must be a new path without symlink components")
    if output.is_relative_to(source) or output.is_relative_to(HERE.parents[1]):
        raise ValueError("output must be outside upstream and BBR checkouts")
    inputs = checked_inputs(source)  # Validate everything before writing anything.
    output.mkdir(parents=False)
    for relative, data in inputs.items():
        path = output / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
    patch = (HERE / "gpui-generators.patch").read_bytes()
    if not baseline:
        for flags in (["--check"], []):
            subprocess.run(
                ["git", "apply", *flags, "-"], input=patch,
                cwd=output, check=True, capture_output=True, timeout=10,
            )
    (output / "Cargo.toml").write_text('''[package]
name = "bbr-gpui-generator-probe"
version = "0.0.0"
edition = "2021"
publish = false
autobins = false

[workspace]

[features]
default = ["runtime_shaders"]
runtime_shaders = []

[dependencies]
cbindgen = { version = "=0.28.0", default-features = false }
bindgen = "=0.71.1"

[[bin]]
name = "shader-generator"
path = "crates/gpui_macos/build.rs"

[[bin]]
name = "media-generator"
path = "crates/gpui_media/build.rs"
''')
    (output / "Cargo.lock").write_bytes((HERE / "probe.lock").read_bytes())
    receipt = {
        "revision": REVISION,
        "variant": "baseline" if baseline else "patched",
        "upstream": {p: digest(data) for p, data in inputs.items()},
        "patch_sha256": digest(patch) if not baseline else None,
        "probe_inputs": {
            p.relative_to(output).as_posix(): digest(p.read_bytes())
            for p in sorted(output.rglob("*")) if p.is_file()
        },
        "claim": "generator probe only; not an application build or SDK validation",
    }
    (output / "source-receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    return receipt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--upstream", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--baseline", action="store_true",
                        help="export unpatched scripts for the Linux regression control")
    args = parser.parse_args()
    try:
        receipt = prepare(args.upstream, args.output, args.baseline)
    except (ValueError, OSError, subprocess.SubprocessError) as exc:
        parser.exit(1, f"preparation refused: {exc}\n")
    print(json.dumps(receipt, indent=2))


if __name__ == "__main__":
    main()
