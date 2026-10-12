# Linux-to-macOS preparation evidence

Contract: [cross-build packet](../roadmap/tasks/linux-macos-cross-build.md).
Live state and ownership: [Current delivery](current.md).

## Scope and source

This is inactive generator preparation, not a complete application build. No
BBR Cargo manifest/lock, flake, justfile, browser source, baselines, packager or
Tart runner changed. GPUI-CE remains pinned to
`568271c01c5a9d640d7b091e5bfd136b04035f28`. Exact filtered macOS Cargo metadata
reports one GPUI package/source world. The candidate patch is stored, not applied
to the live dependency cache. Exported inputs and probe Cargo lock are hash-bound.

Evidence gathered 2026-10-12. Linux source/log root:
`/home/patwoz/dev/bbr-macos-generators.4dclFo` on `ms-a2-dev`.
Mac source: `/private/tmp/bbr-cross-probe-final`; Linux log copy:
`/private/tmp/bbr-cross-linux-evidence`.

## Linux generator regression

Linux x86_64, rustc 1.98.1, mbx 1.21.0. Existing tools:

- `/nix/store/lb1rig38w2q50rr7h2qh4abkab6hmkf5-rust-minimal-1.98.1/bin`
- `/nix/store/nnfy96600wip8y7fz1n9n85ms46hxzal-mbx-1.21.0/bin`

The final source archive hash was
`2bb030a43b37a157bcb6169a8912e2132e80dfe30640338edca5245a4d81218a`, checked
before extraction. No SDK, credentials, app data or unrelated source was sent.
Before commit, blank patch-context lines were normalized for Git whitespace
checks. A fresh export at `/private/tmp/bbr-cross-probe-commit` applies the
normalized patch and has exactly the same `probe_inputs` hashes as the tested
export. Final patch SHA-256:
`14b88acc50be7502c08782b910b66b9a2bf57990794d7a30b8e867bf9ad377a4`.
The final verifier was transferred separately; SHA-256:
`77a2b19b84f462c9536dbe8a1e601c3f1b9b824f6507a3b8bdfedbf0fcf339a5`.

Exact bounded commands in the final export:

```sh
timeout --kill-after=10s 120s mbx build --locked --offline --message-format=json
python3 ../verify.py --probe . --artifacts ../final-build.jsonl
timeout --kill-after=10s 120s mbx clippy --locked --offline --all-targets -- -D warnings
timeout --kill-after=10s 120s mbx build --locked --offline --no-default-features --message-format=json
python3 ../verify.py --probe . --artifacts ../final-static-build.jsonl --static
timeout --kill-after=10s 120s mbx clippy --locked --offline --all-targets --no-default-features -- -D warnings
cargo fmt --all -- --check
```

All pass. Actual build-script executables (not reimplementations) prove:
non-macOS no-op; macOS runtime shader generation; missing runtime feature
refusal on Linux; missing/empty/relative/invalid SDK refusal without xcrun;
checked native SDK discovery failure. Strict Clippy passes both feature modes.
The unmodified upstream scripts compile on Linux but the same verifier fails:
`macOS target did not generate shader artifacts`. This is the red regression.

Both Linux and Mac generated identical bytes:

| Artifact | SHA-256 |
|---|---|
| scene.h | `44fce294d926d8510501a6beb9b9ec8ba9c7b2dd6a92630e2e054f759dd1b87d` |
| stitched_shaders.metal | `00eadcf1a274e75e18a2225be511424e8b85ad7571ee302f2dfea77d6ae8d635` |

Linux logs/reports: `final-build.jsonl`, `final-verification.json`,
`final-static-verification.json`, `final-clippy-runtime.log`,
`final-clippy-static.log`, `final-format.log`, `baseline-verification.log` under
the evidence root above. The cached final build/check sequence ran from
20:30:01 to 20:30:03 +02:00; this is generator tooling time, not product
performance. Planning/implementation/review wall time was not instrumented.

## Native media parity

On the ARM64 Mac, rustc 1.97.1 and mbx build the same probe without GUI startup.
Baseline is compiled with `--bin media-generator` because the unmodified native
shader script still expects its omitted GPUI host dependency. Patched build and
strict Clippy pass in both runtime/static feature modes.

`verify_native.py` uses local Xcode 27.0 Beta 6, macOS 27.0 SDK and libclang
21.1.8. Baseline, patched explicit SDKROOT and patched native xcrun discovery all
emit SHA-256 `6a43c9f285ebe1d2364155b1135375eb749090bd20e6f5151b9cbd810e10ed22`.
Measured generation times: 5.864s / 4.684s / 4.543s. These are not launch timings.

Report/logs: `/private/tmp/bbr-cross-native-final-parity2/summary.json` and the
three sibling output directories, `/private/tmp/bbr-cross-native-shader-contracts.json`,
`/private/tmp/bbr-cross-native-clippy.log`, and
`/private/tmp/bbr-cross-native-clippy-static.log`.

## Header-free linker feasibility, not app proof

Existing Linux Clang/LLD 21.1.8 compiled `int main(void) { return 0; }` with
`--target=arm64-apple-macosx11.0 -ffreestanding -c` and linked with
`ld64.lld -arch arm64 -platform_version macos 11.0 11.0 -e _main`.
No framework, libSystem, SDK stub or Rust standard library was used. No control
binary was launched. Checksum-verified transfer and native `otool -hv -l` confirm
ARM64 `MH_EXECUTE`, `LC_BUILD_VERSION` platform macos, LLD 21.1.8. The SDK 11.0
load-command field is a supplied linker value, **not SDK provenance**.
Control SHA-256:
`a7fe95edeba4af8b9d8c17319fddfc3786f890176fb6865096fff1eb76ce6dfa`.

The real exported GPUI media header, parsed with the same Linux Clang target,
fails at `CoreMedia/CMFormatDescription.h` not found. Evidence:
`/private/tmp/bbr-cross-linux-evidence/linker-control` and the matching guest
directory. A Darwin linker exists; a usable SDK and Rust Darwin standard
libraries remain absent.

## Portable tooling and failed attempts

- 14 new integrity/receipt tests pass via
  `python3 -m unittest scripts/cross_macos/test_prepare.py -v`.
- Existing `just script-tests`: 49/49 pass (16.242s).
- `just roadmap` and `python3 scripts/check_roadmap.py --source index` pass
  after staging (168 Markdown files). Git whitespace checks pass. The first
  roadmap run correctly rejected links to then-untracked new files.
- Initial offline Linux build lacked cached cfg-if 1.0.5. A locked online build
  downloaded the missing crates; final builds/checks pass offline.
- A probe initially asserted an obsolete `Uniforms` name from upstream's export
  list. The pinned source has no such declaration or shader use; the check now
  asserts the real `PathRasterizationVertex` instead, alongside actual scene types.
- Strict Clippy found an upstream redundant formatting borrow. The narrow patch
  removes it; no lint suppression was added.
- Early export tests had the wrong fixture directory depth, making the output
  guard reject them. Corrected the fixture, not the production boundary.
- One native rerun found mbx had reclaimed idle probe executables under disk
  pressure. Rebuilt from identical verified sources through mbx and reran parity;
  no target-directory override or cache-policy change was made.

## Review and remaining acceptance

Focused self-review checked exact pin/input hashes, patch scope, host/target
selection, early refusals, source receipts, no native product-graph changes,
byte-identical native bindings and shader source, bounded execution, and claim
limits. This is **not independent review**; none was delegated.

The goal remains blocked after this preparation on an established SDK/sysroot
source and permission to use it on this Linux host. No SDK was transferred.
Do not mistake this rights/input question for a proof of technical impossibility.
Once actionable, provision Darwin Rust standard libraries declaratively, prove
a framework/libSystem link, then wire and test the complete app graph/bundle.

No complete BBR compile/link, bundle, signing, native app launch, Tart run,
rendering/startup performance, full `just verify` or canonical E2E gate was run
for this preparation. The active application is unchanged. Full app and
independent-review acceptance stays open; no merge or publication is authorized.
