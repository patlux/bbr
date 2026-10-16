#!/usr/bin/env python3
"""Opt-in Linux -> ARM64 macOS build from a verified, isolated source snapshot."""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import platform
import re
import shlex
import subprocess
import sys
import time
import tomllib

try:
    from . import apple_sdk, prepare, source
except ImportError:
    import apple_sdk
    import prepare
    import source

TARGET = "aarch64-apple-darwin"
MINIMUM = "13.0"  # Existing package Info.plist contract, not a new OS promise.


def verify_inputs(root):
    receipt = json.loads((root / "prepared.json").read_text())
    actual = {p.relative_to(root).as_posix()
              for tree in (root / "bbr", root / "gpui")
              for p in tree.rglob("*") if p.is_file() and not p.is_relative_to(root / "bbr/target")}
    if actual != set(receipt["inputs"]):
        raise ValueError("prepared source inventory drift")
    for relative, sha in receipt["inputs"].items():
        path = root / relative
        if path.is_symlink() or not path.is_file() or apple_sdk.digest(path) != sha:
            raise ValueError(f"prepared input drift: {path}")
    return receipt


def environment(root, tools, sdk, version, mbx, output):
    if any(not p.is_absolute() or not p.exists() for p in (root, tools, sdk, mbx)):
        raise ValueError("existing absolute source/toolchain/SDK/mbx paths required")
    for name in ("rustc", "cargo", "clang", "clang++", "ld64.lld", "llvm-ar", "llvm-readobj", "rcodesign"):
        if not (tools / "bin" / name).is_file():
            raise ValueError(f"toolchain missing {name}")
    if not re.fullmatch(r"\d+\.\d+(?:\.\d+)?", version):
        raise ValueError("invalid SDK version")
    wrapper = output / "link-macos.sh"
    command = [str(tools / "bin/clang"), "-target", f"arm64-apple-macosx{MINIMUM}",
               f"--ld-path={tools / 'bin/ld64.lld'}", "-isysroot", str(sdk),
               f"-Wl,-platform_version,macos,{MINIMUM},{version}"]
    wrapper.write_text("#!/bin/sh\nexec " + shlex.join(command) + ' "$@"\n')
    wrapper.chmod(0o700)
    # Deliberately discard Cargo overrides, credential helpers, cache wrappers,
    # injected Rust flags and user SDK/toolchain selection. Keep host tool lookup.
    env = {key: os.environ[key] for key in ("HOME", "USER", "TMPDIR", "PATH") if key in os.environ}
    env.update({"PATH": f"{tools}/bin:{mbx.parent}:" + env.get("PATH", "/usr/bin:/bin"),
                "SDKROOT": str(sdk), "MACOSX_DEPLOYMENT_TARGET": MINIMUM,
                "LIBCLANG_PATH": str(tools / "lib"), "CARGO_TERM_COLOR": "never",
                "PYTHONDONTWRITEBYTECODE": "1",
                "CC_aarch64_apple_darwin": str(tools / "bin/clang"),
                "CXX_aarch64_apple_darwin": str(tools / "bin/clang++"),
                "AR_aarch64_apple_darwin": str(tools / "bin/llvm-ar"),
                "CFLAGS_aarch64_apple_darwin": shlex.join(["-isysroot", str(sdk)]),
                "CXXFLAGS_aarch64_apple_darwin": shlex.join(["-isysroot", str(sdk)]),
                "CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER": str(wrapper)})
    return env


def run(command, cwd, env, output, label, seconds=1800):
    # timeout owns its directly started process group, including compiler children;
    # no PID search or global stop is involved. Every command is bounded.
    bounded = ["timeout", "--kill-after=15s", f"{seconds}s", *map(str, command)]
    started = time.monotonic()
    with (output / f"{label}.stdout").open("x") as stdout, (output / f"{label}.stderr").open("x") as stderr:
        result = subprocess.run(bounded, cwd=cwd, env=env, stdout=stdout, stderr=stderr)
    info = {"command": bounded, "exit": result.returncode, "seconds": time.monotonic() - started}
    (output / f"{label}.json").write_text(json.dumps(info, indent=2) + "\n")
    if result.returncode:
        raise RuntimeError(f"{label} failed ({result.returncode}); see {output / (label + '.stderr')}")
    return info


def check_graph(root, metadata):
    gpui = [p for p in metadata["packages"] if p["name"] == "gpui"]
    expected = root / "gpui/crates/gpui/Cargo.toml"
    if len(gpui) != 1 or gpui[0]["source"] is not None or Path(gpui[0]["manifest_path"]) != expected:
        raise ValueError("cross graph does not have exactly one patched GPUI source")
    if any(source.URL in p.get("source", "") for p in metadata["packages"] if p.get("source")):
        raise ValueError("unpatched Git GPUI package remains")
    original = tomllib.loads((root / "original-Cargo.lock").read_text())
    actual = tomllib.loads((root / "bbr/Cargo.lock").read_text())
    if source.registry(original) != source.registry(actual):
        raise ValueError("registry versions/checksums drifted")


def validate_macho(text):
    if not all(value in text for value in ("Format: Mach-O arm64", "FileType: Executable",
                                          "Platform: macos", f"Version: {MINIMUM}")):
        raise ValueError("unexpected Mach-O architecture/platform/deployment target")
    libraries = re.search(r"NeededLibraries \[\n(.*?)\n\]", text, re.S)
    if not libraries:
        raise ValueError("Mach-O dependencies missing")
    for library in libraries.group(1).splitlines():
        if not library.strip().startswith(("/System/Library/Frameworks/", "/usr/lib/")):
            raise ValueError("non-system Mach-O dependency")


def build(args):
    if sys.platform != "linux" or platform.machine() != "x86_64":
        raise ValueError("build requires x86_64 Linux; never runs macOS executables")
    root, output = args.root.resolve(strict=True), args.output.absolute()
    if output.exists() or output.is_symlink() or output != output.resolve():
        raise ValueError("fresh canonical evidence/output directory required")
    if output.is_relative_to(root / "bbr") or output.is_relative_to(root / "gpui"):
        raise ValueError("output must be outside source inputs")
    receipt = verify_inputs(root)
    sdk = apple_sdk.verify(args.sdk)
    output.mkdir(mode=0o700)
    env = environment(root, args.tools, args.sdk, sdk["version"], args.mbx, output)
    repo = root / "bbr"
    if args.gallery and args.test:
        raise ValueError("Gallery and app tests need separate builds")
    testing = bool(args.test or args.gallery)
    features = "gpui_platform/runtime_shaders" + (",testkit" if args.test else "")
    common = ["--locked", "--offline", "--target", TARGET, "-p",
              "bbr-gallery" if args.gallery else "bbr", "--features", features]
    # Resolve the snapshot lock once, offline; then prove no registry drift and
    # freeze it for actual compilation. Only path/source redirection may change.
    run(["cargo", "metadata", "--offline", "--format-version=1", "--filter-platform", TARGET,
         "--features", features], repo, env, output, "metadata", 120)
    metadata = json.loads((output / "metadata.stdout").read_text())
    check_graph(root, metadata)
    lock_sha = apple_sdk.digest(repo / "Cargo.lock")
    tool_versions = {}
    for tool in (args.tools / "bin/rustc", args.tools / "bin/clang",
                 args.tools / "bin/ld64.lld", args.tools / "bin/rcodesign", args.mbx):
        tool_versions[str(tool)] = subprocess.check_output(
            [str(tool), "--version"], env=env, text=True, timeout=15).strip()
    command = [str(args.mbx), "test" if testing else "build", *common]
    if args.release or not testing:
        command.append("--release")
    if args.test:
        for test in args.test:
            if not re.fullmatch(r"e2e_[a-z0-9_]+", test):
                raise ValueError("explicit E2E test target required")
            command.extend(["--test", test])
        command.append("--no-run")
    elif args.gallery:
        command.extend(["--bin", "bbr-gallery", "--no-run"])
    else:
        command.extend(["--bin", "bbr"])
    command.append("--message-format=json")
    compile_record = run(command, repo, env, output, "build")
    executables = {}
    for line in (output / "build.stdout").read_text().splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        if message.get("reason") == "compiler-artifact" and message.get("executable"):
            path = Path(message["executable"])
            relative = path.relative_to(repo).as_posix()
            if not relative.startswith(f"target/{TARGET}/"):
                raise ValueError("unexpected artifact path")
            executables[relative] = {"sha256": apple_sdk.digest(path), "bytes": path.stat().st_size,
                "target": message["target"]["name"], "features": message["features"],
                "test": message["profile"]["test"]}
    if not executables:
        raise ValueError("Cargo emitted no macOS executable")
    for index, relative in enumerate(executables):
        label = f"macho-{index}"
        run([args.tools / "bin/llvm-readobj", "--file-headers", "--macho-version-min",
             "--needed-libs", repo / relative], repo, env, output, label, 30)
        inspection = (output / f"{label}.stdout").read_text()
        validate_macho(inspection)
    if not testing:
        artifact = f"target/{TARGET}/release/bbr"
        if set(executables) != {artifact} or "testkit" in executables[artifact]["features"]:
            raise ValueError("production artifact contains unexpected targets/testkit")
        run([sys.executable, repo / "scripts/package/build_app.py", "--binary", repo / artifact,
             "--output", output / "BBR.app", "--skip-codesign"], repo, env, output, "package", 60)
        run([args.tools / "bin/rcodesign", "sign", "--config-file", "/dev/null",
             "--timestamp-url", "none", output / "BBR.app"], repo, env, output, "adhoc-sign", 60)
    # Cargo metadata may normalize the prepared lock, but must not alter sources.
    for relative, sha in receipt["inputs"].items():
        if relative != "bbr/Cargo.lock" and apple_sdk.digest(root / relative) != sha:
            raise ValueError(f"source changed during build: {relative}")
    if apple_sdk.digest(repo / "Cargo.lock") != lock_sha:
        raise ValueError("lock changed during locked compilation")
    verify_inputs(root)
    apple_sdk.verify(args.sdk)
    bundle = {p.relative_to(output).as_posix(): {"sha256": apple_sdk.digest(p), "bytes": p.stat().st_size}
              for p in (output / "BBR.app").rglob("*") if p.is_file()}
    result = {"schema": 1, "host": platform.platform(), "target": TARGET, "minimum": MINIMUM,
        "prepared_sha256": apple_sdk.digest(root / "prepared.json"), "source": receipt["source"]["bbr_revision"],
        "gpui_revision": prepare.REVISION, "sdk": sdk, "tools": str(args.tools),
        "mbx": str(args.mbx), "tool_versions": tool_versions,
        "driver_sha256": apple_sdk.digest(Path(__file__)),
        "lock_sha256": lock_sha, "build": compile_record,
        "executables": executables, "bundle": bundle, "runtime_verified": False}
    (output / "build-receipt.json").write_text(json.dumps(result, indent=2) + "\n")
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("root", "output", "tools", "sdk", "mbx"):
        parser.add_argument(f"--{name}", type=Path, required=True)
    parser.add_argument("--test", action="append", help="compile a named E2E without running it; repeatable")
    parser.add_argument("--gallery", action="store_true", help="compile the existing Gallery test harness")
    parser.add_argument("--release", action="store_true", help="optimized E2E; production always optimized")
    print(json.dumps(build(parser.parse_args()), indent=2))


if __name__ == "__main__":
    main()
