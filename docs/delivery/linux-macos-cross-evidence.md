# Linux-to-macOS build evidence

Contract: [cross-build packet](../roadmap/tasks/linux-macos-cross-build.md).
Live state and ownership: [Current delivery](current.md).

## Initial SDK-free preparation (historical)

This first checkpoint was inactive generator preparation, not an application build. No
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

At that checkpoint the agent paused on SDK provenance/use uncertainty; no SDK
had been transferred. Patrick then directed review of successful GPUI-Mobile
sessions and said “lets go” to the proposed macOS export/framework-link path.
The next checkpoint supersedes that technical/input stop. It does not supply a
legal opinion or silently resolve licensing terms.

No complete BBR compile/link, bundle, signing, native app launch, Tart run,
rendering/startup performance, full `just verify` or canonical E2E gate was run
for this preparation. The active application is unchanged. Full app and
independent-review acceptance stays open; no merge or publication is authorized.

## SDK-backed full application checkpoint — 2026-10-12

One local writer, isolated branch/worktree from `70f6026a`, preparation committed
as `73dd8ecfdfc7de49ac3481ea93ecc043299c44af`. Main and every foreign writer's
work remain untouched. No active Cargo manifest/lock, root flake/justfile,
product/test source, shared Tart runner, baseline or upstream cache was modified.
Only fresh source snapshots apply the pinned GPUI generator patch and source
redirects. No agent/workflow, merge, publication or machine activation occurred.

### Inputs and build route

The GPUI-Mobile reference is commit `985815c7da74cc9145727c6219d28360487a5f6e`.
Its exporter is adapted with Apache-2.0 attribution, explicit installed Xcode
selection, hardlink materialization and stricter archive/manifest validation.
Selected Xcode: `/Applications/Xcode-27.0.0-Beta.6.app/Contents/Developer`, version
27.0/build27A5252f. macOS SDK27.0: **41,543 entries**, fully rehashed on Linux.

- SDK archive SHA256:
  `0038c8b81ea0a8ed15c0eed3d431fa7ad200fa08a6a5453af9f34d4140a7380c`.
- SDK manifest SHA256:
  `42ab900a2cd3c1041bd5d0c90bb434d2648c24c49694d93946556316943111f3`.
- Exact BBR/GPUI source export: **2,226 files**, BBR `73dd8ecf`, same GPUI pin;
  archive `51ff28ac708e25f2a2b2a2a7d7478e347496bcd2409322ad94d0c53812e6cfd7`.
- Linux build root `/tmp/bbr-cross-repro-5wmwvM`, host `ms-a2-dev`, x86_64
  Linux6.18.55. Durable experiment/evidence root:
  `/home/patwoz/dev/bbr-macos-build.5wmwvM`.
- Pinned Nix inputs: nixpkgs `89570f24e97e614aa34aa9ab1c927b6578a43775`,
  rust-overlay `ab777f900c3de943e117eb4814ddff6f645b28ad`. Rust1.98.1 with
  Darwin std, LLVM/Clang/LLD21.1.8, rcodesign0.29.0. Tool closure:
  `/nix/store/r2k2ix0igyw0aiwp2ng4crrcmsxxaw5q-bbr-linux-macos-tools`.
  Existing mbx1.21.0 is used; no cache wrapper or target-directory override.

Actual Foundation/WebKit/CoreMedia Objective-C Linux link succeeded before the
app build; Mac inspection confirms ARM64 and system framework dependencies.
Probe SHA256 `d86bc0ea3722d622fd65ffa0ff6f5f69824e577ce26d2585456d275bc6af3d9d`.
An initial root-manifest-only path override exposed a second GPUI Git source
through bbr-ui. The final exporter patches every locked GPUI Git package plus
the crates.io override; metadata proves one GPUI type world and unchanged registry
versions/checksums. No dependency upgrade or product compatibility shim.

The opt-in build uses runtime shaders and deployment target13.0, matching existing
package metadata (earlier feasibility binaries used12.0). LLVM records ARM64
MH_EXECUTE, macOS13.0/SDK27.0 and only system frameworks and `/usr/lib` dependencies.
Source/SDK inputs are verified before/after; compiler calls are locked/offline and
bounded through mbx. Production builds reject testkit. Logs and machine-readable
commands/source/toolchain/SDK/artifact receipts remain in `repro-release-final`,
`repro-all-tests` and `repro-gallery` under the Linux evidence root.

### Production release bundle

Existing packager runs on Linux with codesign skipped only during assembly;
rcodesign then signs ad hoc on Linux with empty config, no credentials and no
network timestamp. The five-file bundle retains the production icon/plist.
Bundle executable SHA256:
`33a5ff2155814d75e8bc4649aa6ffe4b25a8776f046142b159dd97ba1adcc6ca`
(**21,373,728 bytes**). Native `codesign --verify --deep --strict` passes without
resigning. Production process liveness and clean SIGINT shutdown pass inside Tart
with an isolated test profile. This proves package startup/shutdown, not a full
interactive release acceptance or notarization. Linux compilation took46.768s
measured by the final release wrapper; no clean-cache or product timing claim.

### Exact-artifact functional evidence

The payload contains **1,920 files**, including all required source/fixtures,
Linux-built testkit app, 59 test programs and the production bundle. It contains
no SDK, credentials, Cargo cache, host data or unrelated work. Final payload
manifest SHA256 `227da6e063f80849ceb3d8314591a504f3c1af2794ec1720b6a0036d28760e69`;
archive SHA256 `f5142ea84b6fc1ef16cfc97011ba50d5279a349b82281076cfad5ef7e0461b3d`.
All bytes and executable modes are checked before installation/execution; source
and binary hashes are checked again after tests. No Mac compilation or resigning.

Mac evidence root: `/private/tmp/bbr-cross-active`.

- First exact Linux-built smoke: `vm-smoke-20261012-211002`, full native WKWebView
  navigation/back/forward/reload, resize, focus and shutdown passed.
- First production launch: `vm-release-20261012-211936`, Apple signature and
  liveness/clean shutdown passed; bytes rehashed afterward.
- Fresh reproducible-build targeted run: `vm-repro-20261012-213341`, seven suites
  (Smoke, Shutdown, Tabs, Profiles, Portal, Real Input, Web Data) passed.
- First full source-derived functional order plus Peek/Gallery:
  `vm-all-20261012-214444`, **59/59 programs, 348 tests**, zero failures/ignored;
  summed executable time479.320s.
- Final repository-driver complete recheck: `final-vm-recheck/verified.json`,
  **59/59 programs, 348 tests**, production bundle smoke, before/after hashes
  and verified cleanup all pass. Summed stages472.923s, including preparation,
  production observation and posthashes, excluding VM boot/cleanup.

The runner reuses existing exclusive-key/private-NAT Tart policy, source golden
image, no-graphics boot, readonly narrow share, listener/authentication/access
audits, signal-safe bounded cleanup and independent resource observation.
Each executed run records clone/key/process/disk absence and former SSH
unreachability. No host GUI, new ingress, forwarding, Golden Image mutation,
Xcode installation, baseline acceptance or fallback. Timing is tooling only.

### Retained failures and regressions

- SDK hardlinked headers originally archived as tar hardlinks; materialization
  preserves the manifest's regular-file model. Regression roundtrip proves it.
- Three pre-app smoke failures: wrong ioreg location, pipefail/SIGPIPE from
  grep-q, guest otool requiring absent Developer Tools. Corrected exact paths/
  shell observation and kept dependency inspection on the build/Mac host; no
  guest installation or weakened isolation.
- Source exporter first retained Git archive group-write modes; normalization
  to0644/0755 is covered by a contract. Unpack initially refused before writing.
- Packager Python generated cache files inside the snapshot; the next inventory
  preflight correctly refused. Generated cache was moved to task-owned diagnostics,
  Python bytecode generation disabled, and post-build complete inventory rechecked.
- First final-driver aggregate `final-vm` failed unchanged
  `dark_hover_press_wash_states_are_unchanged`: fresh about:blank tab `Loading`
  instead of `Selected`. Source helper flushes a frame but does not wait for this
  tab's load completion. Earlier full run passed. One focused same-artifact
  `final-vm-hover` run and the final complete recheck passed. No test/product
  modification, sleep/readiness workaround or assertion relaxation; intermittent
  readiness failure remains an observed risk, not proven fixed.

### Verification, self-review and remaining acceptance

- **32 cross-tooling contracts** pass: export pin/hash/source guards, SDK
  hardlinks/links/path/type/duplicate/hash drift, clean environment, one-GPUI
  graph, registry preservation, Mach-O platform/dependencies, immutable payload,
  fixed compiled paths, recipe order/deferral, zero/ignored-test refusal.
  `/private/tmp/bbr-cross-active/all-tooling-tests-final.log`.
- Native complete mbx-equivalent `just verify` passes, including49 shared script
  tests; strict `mbx clippy -p bbr --features testkit --all-targets -- -D warnings`
  passes. Linux cross-target testkit all-target strict Clippy also passes
  (17.250s, Linux `cross-clippy/clippy.json`). Final production driver with
  integrated Mach-O inspection passes in `repro-release-inspected`; all five
  bundle files match the executed `repro-release-final` byte-for-byte. This
  incremental repeat is not independent clean-build reproducibility evidence.
  Logs `native-verify.log`, `native-testkit-clippy.log`. Only compiler
  command names were substituted in a temporary recipe; no assertions/budgets
  changed. Native default graph/static shader path remains unchanged.
- Source, toolchain, SDK, package signature, runtime, complete functional result
  and actual cleanup are separate evidence surfaces. Self-review checked these
  boundaries, exact source/receipt retention, no secret transfer, canonical fresh
  outputs, one GPUI world, target-vs-host generation and no hidden Mac rebuilding.

Performance execution remains explicitly deferred and unpassed. The functional
aggregate intentionally omits e2e-perf under that existing decision; runtime shader
startup/rendering budgets and canonical evidence remain open. Independent review
was not authorized/performed. No byte-identical two-build reproducibility,
minimum-macOS runtime, distribution signing/notarization, release readiness,
phase completion, main integration or publication is claimed. This coherent
intermediate tooling slice can be committed without completing the whole task.
