# Linux-to-macOS generator preparation

This directory is **inactive preparation**, not a BBR cross-build command.
It does not modify the application Cargo graph, native defaults, flake, package
recipe or VM runner. Full acceptance belongs to the
[cross-build packet](../../docs/roadmap/tasks/linux-macos-cross-build.md).

## What the patch changes

`gpui-generators.patch` applies only to GPUI-CE revision
`568271c01c5a9d640d7b091e5bfd136b04035f28`:

- Both build scripts select `CARGO_CFG_TARGET_OS`, not host `cfg(target_os)`.
- The macOS shader generator uses sibling GPUI source files in this exact Git
  workspace instead of compiling host GPUI just to read `GPUI_MANIFEST_DIR`.
  Its cbindgen dependency is available on every build host. The runtime-shader
  feature still chooses source stitching; native non-runtime builds retain
  Apple's `metal`/`metallib` commands. Cross-host non-runtime builds fail early.
- Media generation accepts explicit `SDKROOT`; native hosts retain xcrun
  discovery when it is absent. A non-Mac host cannot silently invoke xcrun.
  The SDK path must be absolute and contain CoreMedia headers. Bindgen receives
  the Cargo target and sysroot separately, including paths with spaces.
- One redundant upstream formatting borrow is removed for strict Rust 1.98
  Clippy. No renderer, shader source, type declaration or binding allowlist changes.

Do not apply this patch to Cargo's shared dependency cache. A later active
integration must preserve a single GPUI source/type world and test the complete
native and cross-target graphs. Sibling source discovery is intentionally for
this pinned Git workspace, not a claim about independently published crates.

## Reproduce the isolated probe

Python 3.11+, Git, Rust, mbx and the pinned upstream checkout are required.
Use absolute, canonical paths. On macOS `/tmp` is a symlink: use `/private/tmp`
for the output argument. The output must not exist or be inside either checkout.

From `/Users/patwoz/dev/Personal/macos/bbr-linux-macos-cross`:

```sh
python3 scripts/cross_macos/prepare.py \
  --upstream /absolute/path/to/gpui-ce-at-the-pin \
  --output /absolute/new/probe
```

The exporter checks Git HEAD and SHA-256 for each allowlisted upstream input
before writing. It retains upstream licensing, checks/applies the patch, and
writes an isolated Cargo manifest, `probe.lock` and a source receipt. The probe
compiles the actual build scripts **as host command-line executables**, with
cbindgen 0.28.0 and bindgen 0.71.1. It does not resolve the BBR dependency graph
or compile a Darwin Rust crate. The probe lock is separate from BBR's lock.

On Linux, in `/absolute/new/probe`:

```sh
timeout --kill-after=10s 300s mbx build --locked --message-format=json > /absolute/new/runtime-artifacts.jsonl
python3 /absolute/path/to/verify.py --probe /absolute/new/probe --artifacts /absolute/new/runtime-artifacts.jsonl
timeout --kill-after=10s 150s mbx clippy --locked --offline --all-targets -- -D warnings
timeout --kill-after=10s 150s mbx build --locked --offline --no-default-features --message-format=json > /absolute/new/static-artifacts.jsonl
python3 /absolute/path/to/verify.py --probe /absolute/new/probe --artifacts /absolute/new/static-artifacts.jsonl --static
timeout --kill-after=10s 150s mbx clippy --locked --offline --all-targets --no-default-features -- -D warnings
cargo fmt --all -- --check
```

Run verification immediately after each build: both variants share executable
names, and mbx may reclaim idle managed targets. A missing executable is a failed
check, not a pass. Build again from the verified inputs rather than bypassing mbx.
Downloads on the first locked build are normal; subsequent runs can use offline.
No compiler-cache wrapper or target-directory override is needed.

`verify.py` checks receipt hashes, drives actual generator executables with
Cargo's target/host variables, and traps unexpected xcrun use. It requires real
shader declarations and exact header-plus-source bytes, checks both non-macOS
no-op paths, missing/empty/relative/invalid SDK refusals, native discovery failure
and the non-runtime cross-host refusal. No framework headers are fabricated.

For the red regression control, export with `--baseline`, build on Linux and
run the same default verification. It must fail at
`macOS target did not generate shader artifacts`. Upstream returned success
without generating anything because its compiler cfg described the Linux host.
The baseline probe is intentionally not a native shader build: upstream's
native script requires the omitted GPUI host dependency.

## Native media parity (Mac only, no GUI)

Build the baseline **media binary only**, and both patched probe executables:

```sh
# In the baseline export:
timeout --kill-after=10s 180s mbx build --locked --offline --bin media-generator --message-format=json > /absolute/baseline-artifacts.jsonl
# In the patched export:
timeout --kill-after=10s 180s mbx build --locked --offline --message-format=json > /absolute/patched-artifacts.jsonl
```

Then run `/Users/patwoz/dev/Personal/macos/bbr-linux-macos-cross/scripts/cross_macos/verify_native.py`
with `--baseline`, `--patched`, `--baseline-artifacts`, `--patched-artifacts`,
`--output` (new directory) and `--developer-dir` (installed Xcode Developer
folder). Each argument is an absolute path. The command compares baseline,
patched explicit-SDK and patched xcrun-discovery bindings byte-for-byte. It uses
only the locally installed SDK and libclang; it never transfers an SDK or
launches BBR. Whole-app native compatibility still requires its own gates.

Portable export tests:

```sh
cd /Users/patwoz/dev/Personal/macos/bbr-linux-macos-cross
timeout --kill-after=3s 30s python3 -m unittest scripts/cross_macos/test_prepare.py -v
```

## Remaining prerequisite and acceptance gap

`ms-a2-dev` has working Linux Clang/LLD capable of emitting and linking a
header-free ARM64 macOS Mach-O control. That control has no framework/libSystem
links and was not launched; it is not an application or runtime proof. The real
GPUI media header fails at missing `CoreMedia/CMFormatDescription.h`.

The next input is a macOS SDK/sysroot with established origin and permitted use
on this Linux host. The installed Xcode agreement raised a use-rights question;
this is not proof that cross-compilation is technically impossible. No SDK was
copied or unofficial archive fetched. The existing Rust toolchain also lacks
`aarch64-apple-darwin` standard libraries; provision those declaratively only
when the SDK path is actionable. LLD is already available, so it is not an
installation blocker.

Then prove a minimal framework/libSystem link, wire the pinned patch and opt-in
runtime shaders into the application, build the real bundle on Linux, inspect
its architecture/dependencies, and exercise the checksum-bound Linux artifact
through disposable Tart. Ordinary Mac-rebuilding VM preparation cannot prove
that artifact. Startup/rendering measurements, existing functional gates and
independent review remain required. Neither the preparation tests nor native
binding parity close these requirements.
