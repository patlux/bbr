#!/usr/bin/env python3
"""Compare native media bindings from unpatched and prepared GPUI scripts.

Runs generators only, on macOS. Does not build/launch BBR or transfer an SDK.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import time

from verify import artifact_paths, verify_sources


def compare(baseline, patched, artifacts, output, developer):
    if platform.system() != "Darwin" or platform.machine() != "arm64":
        raise ValueError("native parity check requires an ARM64 Mac")
    before, after = verify_sources(baseline), verify_sources(patched)
    if before["variant"] != "baseline" or after["variant"] != "patched":
        raise ValueError("expected baseline and patched source receipts")
    if before["upstream"] != after["upstream"]:
        raise ValueError("parity requires identical upstream source inputs")
    if output.exists() or output.absolute() != output.resolve():
        raise ValueError("output must be new and have no symlink components")
    env = {k: v for k, v in os.environ.items()
           if k != "SDKROOT" and not k.startswith("BINDGEN_EXTRA_CLANG_ARGS")}
    env.update(DEVELOPER_DIR=str(developer), PATH="/usr/bin:/bin:" + env["PATH"])
    sdk = subprocess.check_output(
        ["/usr/bin/xcrun", "--sdk", "macosx", "--show-sdk-path"],
        env=env, text=True, timeout=15,
    ).strip()
    output.mkdir()
    results = {}
    for variant, root, binary in (
        ("baseline", baseline, artifacts[0]["media-generator"]),
        ("explicit", patched, artifacts[1]["media-generator"]),
        ("discovery", patched, artifacts[1]["media-generator"]),
    ):
        out = output / variant
        out.mkdir()
        call_env = dict(env, CARGO_CFG_TARGET_OS="macos", TARGET="aarch64-apple-darwin",
                        HOST="aarch64-apple-darwin", OUT_DIR=str(out),
                        CARGO_MANIFEST_DIR=str(root / "crates/gpui_media"))
        if variant == "explicit":
            call_env["SDKROOT"] = sdk
        start = time.monotonic()
        proc = subprocess.run([str(binary)], cwd=root / "crates/gpui_media",
                              env=call_env, capture_output=True, text=True, timeout=90)
        (out / "stdout.log").write_text(proc.stdout)
        (out / "stderr.log").write_text(proc.stderr)
        if proc.returncode:
            raise ValueError(f"{variant} generation failed; inspect {out / 'stderr.log'}")
        results[variant] = {
            "seconds": time.monotonic() - start,
            "sha256": hashlib.sha256((out / "bindings.rs").read_bytes()).hexdigest(),
        }
    if len({r["sha256"] for r in results.values()}) != 1:
        raise ValueError("generated native bindings differ")
    report = {"sdk": sdk, "revision": after["revision"], "results": results,
              "claim": "native media generation parity only; not app/runtime acceptance"}
    (output / "summary.json").write_text(json.dumps(report, indent=2) + "\n")
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline", type=Path, required=True)
    parser.add_argument("--patched", type=Path, required=True)
    parser.add_argument("--baseline-artifacts", type=Path, required=True)
    parser.add_argument("--patched-artifacts", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--developer-dir", type=Path, required=True)
    args = parser.parse_args()
    try:
        report = compare(args.baseline.resolve(), args.patched.resolve(),
                         (artifact_paths(args.baseline_artifacts, {"media-generator"}),
                          artifact_paths(args.patched_artifacts)),
                         args.output.absolute(), args.developer_dir.resolve())
    except (OSError, ValueError, subprocess.SubprocessError) as exc:
        parser.exit(1, f"native parity failed: {exc}\n")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
