# Linux-to-macOS build tooling

Opt-in cross-build of the complete ARM64 macOS app on x86_64 Linux. This is not
a Linux browser port. All graph edits occur in fresh committed-source snapshots;
the native checkout, root flake/justfile and shared VM runner stay unchanged.
Full acceptance belongs to the
[cross-build packet](../../docs/roadmap/tasks/linux-macos-cross-build.md).

## Full build route

Python 3.11+, Git and existing mbx are required. SDK unpacking requires Python's
`tarfile` data filter. Compiler builds use the pinned Nix closure without system
activation. Resolve applicable SDK terms before exporting; no Apple content,
credentials or distribution signing identity is included here.

1. On macOS, export only the selected installed Xcode SDK:

   ```sh
   python3 /Users/patwoz/dev/Personal/macos/bbr-linux-macos-cross/scripts/cross_macos/apple_sdk.py export \
     --developer-dir /Applications/Xcode-27.0.0-Beta.6.app/Contents/Developer \
     --output /private/tmp/bbr-sdk-export
   ```

   The archive and adjacent manifest record Xcode version/build, SDK settings,
   all regular-file hashes and bounded relative links. Hardlinked SDK headers
   are materialized as separate regular entries. Existing outputs are refused.

2. Export an explicit **committed** BBR revision and complete pinned GPUI tree:

   ```sh
   python3 /Users/patwoz/dev/Personal/macos/bbr-linux-macos-cross/scripts/cross_macos/source.py export \
     --repo /Users/patwoz/dev/Personal/macos/bbr-linux-macos-cross \
     --upstream /absolute/pinned/gpui-checkout \
     --revision FULL_COMMIT_SHA --output /private/tmp/bbr-source.tar.gz
   ```

   Dirty/staged work is deliberately not exported. Retain the printed archive
   hash independently. Transfer only these explicit artifacts and the tooling
   through the established SSH route, never an entire home/worktree or SDK from
   an unverified third-party download.

3. On Linux, realize the task-local tools, unpack the SDK, and prepare a fresh
   source snapshot. Use a `/tmp/bbr-cross-<unique-id>` root so compiled absolute
   fixture/executable paths can be reproduced inside Tart:

   ```sh
   timeout --kill-after=15s 900s nix build --impure --file /absolute/tooling/toolchain.nix --no-link --print-out-paths
   python3 /absolute/tooling/apple_sdk.py unpack --archive /absolute/MacOSX.sdk.tar.gz \
     --manifest /absolute/MacOSX.sdk.manifest.json --destination /absolute/sdks
   python3 /absolute/tooling/source.py unpack --archive /absolute/bbr-source.tar.gz \
     --sha256 INDEPENDENT_ARCHIVE_SHA --root /tmp/bbr-cross-example
   ```

4. Build the production release bundle with the resulting `/nix/store/...` tool
   closure and an existing mbx executable. No testkit feature is permitted in
   this production output:

   ```sh
   timeout --kill-after=30s 2100s python3 /absolute/tooling/build.py \
     --root /tmp/bbr-cross-example --output /absolute/new-release \
     --tools /nix/store/TOOL_CLOSURE --sdk /absolute/sdks/MacOSX.sdk \
     --mbx /absolute/existing/mbx
   ```

   The snapshot redirects every GPUI Git package and the crates.io override to
   one patched export. Offline metadata validates one GPUI type world and exact
   registry versions/checksums, then compilation is locked/offline. A fresh Linux
   Cargo cache must first fetch the locked sources explicitly with `cargo fetch
   --locked --target aarch64-apple-darwin --manifest-path
   /tmp/bbr-cross-example/bbr/Cargo.toml`; this downloads dependencies, not an SDK.
   Target is
   `aarch64-apple-darwin`, deployment target 13.0 (existing package minimum),
   runtime shaders enabled. Source and SDK hashes are checked before/after.
   LLVM inspects ARM64/macOS load commands and system-only dependencies.
   Existing packaging creates the bundle; credential-free `rcodesign` ad-hoc
   signing disables default config and network timestamps. Apple signature
   validation happens separately on macOS, never by rebuilding/resigning.

5. Compile E2E executables with the same command and a **new output directory**,
   adding repeated `--test e2e_smoke --test e2e_profiles` selectors; tests are
   compiled with testkit but never run on Linux. `--gallery` separately compiles
   the existing Gallery harness. `--release` optimizes test artifacts only.
   Capture each artifact before an incompatible feature build overwrites it;
   staging refuses stale/mixed byte identities. No managed target override.

6. Stage source/executable/bundle bytes from explicit build receipts:

   ```sh
   python3 /absolute/tooling/payload.py --root /tmp/bbr-cross-example \
     --build /absolute/test-build --build /absolute/release-build \
     --output /absolute/new-payload
   ```

   Retain the printed manifest hash independently and transfer the three payload
   files to a fresh canonical Mac directory. Never include an SDK, build cache,
   SSH key or unrelated data in the VM share.

7. Validate on macOS through the existing hardened disposable Tart lifecycle:

   ```sh
   timeout --kill-after=120s 900s python3 /Users/patwoz/dev/Personal/macos/bbr-linux-macos-cross/scripts/cross_macos/verify_vm.py \
     --payload /private/tmp/bbr-cross-payload --sha256 INDEPENDENT_MANIFEST_SHA \
     --artifacts /private/tmp/bbr-cross-evidence e2e-smoke e2e-profiles
   ```

   For a payload containing every functional executable plus Gallery, use
   `--functional` instead of recipes with a 2100s outer limit. This derives the
   canonical sequence plus Peek/Gallery, explicitly excluding deferred perf.
   It is not full performance acceptance. The guest rehashes files before/after,
   verifies Apple signatures, exercises the optional production bundle's
   liveness/clean shutdown, then executes each unchanged recipe's test arguments
   with bounded deadlines. Zero/ignored/failed tests cannot pass. Run records are
   published only after clone/key/process/disk/SSH cleanup is observed. No host
   GUI, Xcode installation, baseline adoption or automatic networking fallback.

Logs and `build-receipt.json` distinguish source, toolchain, SDK, linker, binary,
bundle and runtime evidence. `verified.json` is functional evidence, not a
byte-reproducibility, distribution signing, performance or review certificate.
Normal native builds retain their original graph and static shaders.

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

Do not apply this patch to Cargo's shared dependency cache. The opt-in snapshot
route preserves a single GPUI source/type world and validates it before compiling.
Sibling source discovery is intentionally for this pinned Git workspace, not a claim about independently published crates.

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

## Verification and remaining acceptance

Run portable contracts without compiler, SDK or VM operations:

```sh
cd /Users/patwoz/dev/Personal/macos/bbr-linux-macos-cross
timeout --kill-after=3s 60s python3 -m unittest discover -s scripts/cross_macos -p 'test_*.py' -v
```

The [evidence report](../../docs/delivery/linux-macos-cross-evidence.md) records
actual full Linux build, ad-hoc production launch and 59-program functional
Tart execution. A retained intermittent Hover readiness failure is not erased
by successful focused/full rechecks. Runtime-shader startup/rendering budgets,
canonical performance evidence, independent review, supported-minimum-OS testing
and repeat-byte release reproducibility remain separate, unpassed claims.
SDK provenance and technical success are not legal advice or a licensing grant.
