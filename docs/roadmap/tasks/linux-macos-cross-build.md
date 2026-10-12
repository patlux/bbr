# Task packet — Build the macOS application on Linux

Parents: [Phase 9](../phases/09-quality-release.md),
[Performance/release](../tracks/performance-release.md),
[Product](../../../PRODUCT.md).

## Outcome

Build the complete ARM64 macOS BBR application on Linux and validate those exact
artifact bytes on macOS. This is not a Linux browser port. The first bounded
increment prepares and tests the pinned GPUI generators without an Apple SDK;
it does not activate a new application dependency graph or claim an app build.

## Scope

- Prepare a small patch against the existing GPUI-CE revision. Select the Cargo
  target rather than the build host for shader and media generation.
- Prove SDK-free runtime-shader source generation on Linux using the actual
  patched build script and original shader/type sources.
- Make media generation require explicit SDK input on a non-Mac host and pass
  the Cargo target to libclang. Preserve native xcrun discovery.
- After SDK feasibility is established, connect an opt-in build environment,
  application dependency patch, unsigned bundling and exact-artifact VM tests.

## Non-goals

Linux web-engine support; GPUI upgrades; renderer replacement; changing native
Mac defaults; distribution signing/notarization; publication; visual baseline
acceptance; taking over the separate Visual migration.

## Parallel-lane contract

```text
Base condition:          existing pinned GPUI-CE application graph
Depends on:              usable macOS SDK/sysroot and Linux Darwin linker
Provides/API:            inactive pinned generator patch and portable proof
Exclusive paths:         scripts/cross_macos; this task packet
Shared integration points: roadmap links and Current delivery; later Cargo/flake/runner changes
Integration order:       Visual owner settles main before any serial integration
```

Only one local writer; no delegated work. Active ownership and the exact base
are recorded in [Current delivery](../../delivery/current.md).

## Constraints

Keep one GPUI source/type world. Preserve GPUI controls, native WKWebView,
profile isolation, normal Mac builds and every existing test/budget. Use mbx
for compilation, bounded commands, the existing SSH route and task-owned
outputs. Do not transfer an SDK before its provenance and usability on this
host are established. Do not install/activate machine configuration, add
network exposure, or use the host GUI. Keep preparation separate from active
product code until native and cross-build verification are possible.

## Evidence

### Unit and contract

- [x] Exact-revision export and patch application reject wrong or modified inputs.
- [x] Linux execution of the actual shader build script emits the real header
      and stitched Metal source for a macOS target, not for a Linux target.
- [x] Non-runtime cross builds and missing/invalid SDK inputs fail explicitly.
- [x] Strict portable Clippy, formatting, script tests and roadmap checks pass.

Preparation evidence: [generator and native parity report](../../delivery/linux-macos-cross-evidence.md).

- [ ] Complete Linux build records source, toolchain, SDK provenance, commands,
      checksums, ARM64 Mach-O platform and dynamic library dependencies.

### Real-app E2E

- [ ] The checksum-verified Linux-built artifact presents GPUI and navigates
      real WKWebView fixtures in a disposable headless Tart guest.
- [ ] Affected native regressions and the canonical sequential suite pass;
      rebuilding on the Mac does not count as Linux-artifact evidence.

### Performance

- [ ] Opt-in runtime shaders preserve startup/rendering budgets. VM results
      remain report-only; canonical evidence is separate. Existing execution
      deferrals remain unpassed, not accepted exceptions to budgets.

## Completion gates

Preparation uses the standalone probe, not the BBR application graph. Its
commands and evidence live in [the preparation guide](../../../scripts/cross_macos/README.md).
The full slice additionally requires `just verify`, mbx all-target strict
Clippy with testkit, affected Tart suites and `just e2e-vm e2e`. Runtime shader
startup/rendering requires optimized performance evidence, subject to the
recorded execution deferral. No acceptance checkbox substitutes for runtime.

## Delivery sequence

1. One writer proves the smallest SDK-free generator path.
2. Resolve SDK provenance/usability; prove a minimal Darwin link before broad
   application/toolchain changes.
3. Wire the real app build and validate exact Linux artifact bytes in Tart.
4. Obtain independent read-only review before final acceptance. Delegation is
   not authorized by this task; local self-review is not independent review.

## Iteration policy

Inspect each failure and change only its demonstrated cause. Preserve the
unmodified upstream pin, active native graph and unrelated work. Do not create
speculative linker/signing/VM scaffolding before the prerequisite can run.
Commit verified preparation separately without declaring full completion.

## Blocked stop condition

After independently useful SDK-free preparation, stop if no defensible next
action remains. Report evidence, preserved branch, failed paths, exact missing
SDK/toolchain or runtime prerequisite, and minimal input needed to resume.

## Delivery ledger

Detailed evidence: [generator and native parity report](../../delivery/linux-macos-cross-evidence.md).
Runtime state is linked from [Current delivery](../../delivery/current.md).
Preparation is not compilation, linking, bundling, signing, launch or performance
acceptance of BBR. Timings are measured only for instrumented commands.
