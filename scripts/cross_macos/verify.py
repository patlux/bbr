#!/usr/bin/env python3
"""Exercise compiled upstream generator scripts, without compiling BBR or an SDK."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile


def artifact_paths(receipt, required=None):
    if required is None:
        required = {"shader-generator", "media-generator"}
    found = {}
    for line in receipt.read_text().splitlines():
        if not line.startswith("{"):
            continue
        event = json.loads(line)
        if event.get("reason") == "compiler-artifact" and event.get("executable"):
            found[event["target"]["name"]] = Path(event["executable"])
    if set(found) != required:
        raise ValueError(f"receipt must identify exactly these generator executables: {sorted(required)}")
    return found


def verify_sources(root):
    receipt = json.loads((root / "source-receipt.json").read_text())
    for relative, expected in receipt["probe_inputs"].items():
        path = root / relative
        if path.is_symlink() or hashlib.sha256(path.read_bytes()).hexdigest() != expected:
            raise ValueError(f"probe input changed after preparation: {relative}")
    return receipt


def exercise(root, binaries, static=False):
    source = verify_sources(root)
    results = []
    with tempfile.TemporaryDirectory(prefix="bbr-generator-check.") as tmp:
        tmp = Path(tmp)
        tripwire = tmp / "bin"
        tripwire.mkdir()
        xcrun = tripwire / "xcrun"
        xcrun.write_text("#!/bin/sh\necho UNEXPECTED_XCRUN >&2\nexit 91\n")
        xcrun.chmod(0o700)

        def run(name, target_os="macos", sdk=None, host="x86_64-unknown-linux-gnu"):
            out = tmp / str(len(results))
            out.mkdir()
            env = {k: v for k, v in os.environ.items()
                   if k not in {"SDKROOT", "CARGO_ENCODED_RUSTFLAGS"}
                   and not k.startswith("BINDGEN_EXTRA_CLANG_ARGS")}
            crate = root / "crates" / ("gpui_macos" if name == "shader" else "gpui_media")
            env.update(CARGO_CFG_TARGET_OS=target_os, HOST=host,
                       TARGET="aarch64-apple-darwin" if target_os == "macos"
                       else "x86_64-unknown-linux-gnu", CARGO_MANIFEST_DIR=str(crate),
                       OUT_DIR=str(out), PATH=str(tripwire))
            if sdk is not None:
                env["SDKROOT"] = sdk
            result = subprocess.run([str(binaries[f"{name}-generator"])],
                                    cwd=crate, env=env, capture_output=True,
                                    text=True, timeout=30)
            results.append({"generator": name, "target_os": target_os,
                            "host": host, "sdk": sdk, "exit": result.returncode})
            return result, out

        for name in ("shader", "media"):
            result, out = run(name, target_os="linux", sdk="/invalid/unused")
            assert result.returncode == 0, result.stderr
            assert not list(out.iterdir()), "non-macOS target produced artifacts"
        result, out = run("shader")
        if static:
            assert result.returncode != 0 and "requires the runtime_shaders feature" in result.stderr, result.stderr
            assert not list(out.iterdir()), "failed prerequisite still produced artifacts"
        else:
            assert result.returncode == 0, result.stderr
            header = out / "scene.h"
            stitched = out / "stitched_shaders.metal"
            assert header.is_file() and stitched.is_file(), "macOS target did not generate shader artifacts"
            h = header.read_bytes()
            shader = (root / "crates/gpui_macos/src/shaders.metal").read_bytes()
            assert stitched.read_bytes() == h + b"\n" + shader
            for symbol in (b"struct Quad", b"struct Shadow", b"struct PathRasterizationVertex", b"struct PolychromeSprite"):
                assert symbol in h, f"missing generated shader type: {symbol!r}"
            results[-1]["sha256"] = {p.name: hashlib.sha256(p.read_bytes()).hexdigest()
                                      for p in (header, stitched)}
        for sdk, message in ((None, "requires explicit SDKROOT"),
                             ("", "requires explicit SDKROOT"),
                             ("relative", "absolute macOS SDK path"),
                             (str(tmp), "absolute macOS SDK path")):
            result, out = run("media", sdk=sdk)
            assert result.returncode != 0 and message in result.stderr, result.stderr
            assert "UNEXPECTED_XCRUN" not in result.stderr, "Linux host invoked xcrun"
            assert not list(out.iterdir()), "failed media prerequisite produced bindings"
        result, out = run("media", host="aarch64-apple-darwin")
        assert result.returncode != 0 and "macOS SDK discovery failed" in result.stderr, result.stderr
        assert not list(out.iterdir())
    return {"source": source["variant"], "revision": source["revision"],
            "mode": "static" if static else "runtime", "checks": results,
            "claim": "generator contracts only; no app compilation, SDK parsing or native runtime"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--probe", type=Path, required=True)
    parser.add_argument("--artifacts", type=Path, required=True)
    parser.add_argument("--static", action="store_true")
    args = parser.parse_args()
    try:
        result = exercise(args.probe.resolve(), artifact_paths(args.artifacts), args.static)
    except (AssertionError, ValueError, OSError, subprocess.SubprocessError) as exc:
        parser.exit(1, f"generator contract failed: {exc}\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
