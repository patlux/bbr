# Production-chrome visual snapshots

BBR reuses the portable **cargo-gpui-visual 0.1.1** comparator/report from GPUI
Mobile Kit. The unchanged source lives in `vendor/cargo-gpui-visual`, pinned to
SDK commit `208e24eb10ff62df8ef0d78ab041512448776af2`. Its file hashes are in
`scripts/visual/tool-source.json`; every invocation verifies them and rejects
unexpected vendor files (excluding mbx's generated target directory). The CLI is
Apache-2.0 with its original LICENSE/NOTICE retained. Its independent Cargo
root never links GPUI; BBR does not depend on a developer's SDK checkout or
on the SDK renderer helper. Nothing is installed or published.

## Capture contract

`visual.toml` delegates PNG production to `scripts/visual/capture.py`. It builds
both the actual `bbr` binary and its capture test in release mode through mbx,
selects their exact paths from Cargo's artifact messages (no assumed target
path), applies BBR's existing unstripped host-artifact override to the independent
CLI root (Xcode 27 beta otherwise corrupts proc-macro dylibs), and executes the
suite under the common-directory GUI lock. The adapter holds that lock until
its test/app process group has been reaped, including on timeout or interruption.
Build, lock-wait and execution deadlines are bounded.

The existing authenticated, feature-gated `CaptureChrome` request reads the
running production GPUI window's Metal scene. The capture suite drives real
WKWebViews and normal testkit mutations in fresh, non-persistent profiles:

| Fixture | Contents | Images in each appearance |
|---|---|---|
| `ordinary` | One loaded fixture tab, no Favorites or folders | `ordinary-full`, `ordinary-rail` |
| `representative` | Four loaded tabs, four Favorites, one pinned selected tab, one empty collapsed folder | `representative-full`, `representative-rail` |

Each case has `--light` and `--dark`, for eight PNGs. The contract is exactly
1080×1407 points at 2× (2160×2814 pixels); the left rail crop is 304 points
(608 pixels), including its boundary. Wrong dimensions fail; no resizing,
masking or tolerance is applied. Appearance, normal contrast and Reduce Motion
are forced through existing typed overrides. Capture waits for settled native
geometry, no loading/crash/unresponsive tabs and no transient motion moments.
Semantic state is checked again after readback; pixels never synchronize state.

**This is chrome-only evidence.** Native WKWebView content, AppKit traffic
lights and the separate portal child are not in these PNGs. We do not paint
substitutes for them. The accompanying typed snapshots preserve native/session
and geometry diagnostics. Functional E2E, native-boundary tests, performance
and animation tests remain separate mandatory evidence.

## Commands

Use the existing macOS dev environment and Xcode/Metal toolchain:

```sh
just visual-test               # adapter + unchanged upstream contracts
just visual-doctor             # build CLI, then read-only source/config preflight
just visual-check              # fresh unique run; capture + compare + report
just visual-compare reports/visual/runs/<explicit-run>
just visual-accept reports/visual/runs/<explicit-run> \
  ordinary-full--light ordinary-full--dark
```

`visual-check` prints the exact run and `reports/visual/latest/index.html`.
Open that static HTML file manually when desired; the tool never opens a
browser or listener. Compare before acceptance: the report must be bound to
that exact run and unchanged baselines. Name each reviewed changed/missing
case explicitly. There is no accept-all or automatic update on failure.
Accepting another batch requires regenerating the comparison after the first
batch changed the baseline state.

Upstream CLI exit codes: 0 = exact pass/success, 1 = non-passing comparison
(including first missing baselines; inspect the report's validation errors),
2 = invalid capture/configuration. Build/adapter failures can also fail before
the CLI produces a report and are never visual passes. A first run should fail for missing baselines; inspect
all eight images before explicitly accepting them, then run a second fresh
capture and require eight exact matches.

For a consciously reviewed OS/compiler/renderer environment change, use the
underlying `python3 scripts/visual/tool.py accept --run <run> --case <id> ...
--adopt-environment`. The CLI refuses unreviewed reference drift. Never edit
baseline metadata by hand. Use `just`'s Xcode environment for capture, not a
bare invocation with a different SDK selected.

## Source and environment integrity

Before and after capture the CLI hashes configured first-party source,
fixtures/assets, vendored UI/CLI source, Cargo pins and the capture/build
scripts. New relevant non-ignored source files participate. Baselines and
reports are excluded; secret-name paths and symlinks are refused/excluded.
The environment probe records macOS version/build, hardware model/arch,
compiler, SDK/Xcode, exact GPUI source, geometry, scale, fonts/OS binding and
appearance/readiness policy. Capture assertions prove the stated 2× geometry.

Runs and reports are ignored under `reports/visual`; only reviewed baseline
PNGs plus `baseline.json` are versioned under `tests/visual/baselines`.
Source changes after capture invalidate comparison/acceptance; take a fresh
run instead of resealing old images. A hash seal detects stale inputs; it is
not authentication of a human review.

## Windowless Gallery probes

```sh
just gallery-capture       # all registered selector/theme cases
just gallery-capture 22    # one registry story: Tab Row
```

This separate dev-only producer uses pinned GPUI `HeadlessAppContext` with the
real macOS font-kit text system and `current_headless_renderer()` (Metal). It
creates no NSWindow, activates no application, uses no DisplayLink and never
falls back to a hidden native window. The ordinary interactive `just gallery`
remains unchanged. Browser/WebKit/native-window E2Es are **not** made headless.
macOS and a working Metal device are required; a Linux/server substitute is not
claimed. Locked-session operation still needs direct runtime evidence.

The optimized binary renders the same Gallery entities, `gpui_component::Root`
and production story children. Pages are 1200×1000 points at fixed 2×. Direct
children are measured and paginated without rescaling or splitting a child;
oversized content fails. Finite Reduced Motion opacity endpoints use production
durations and layout timestamps, not sleeps, image polling or a simulated clock.
This proves static endpoints, not animation or native interaction behavior.

The recipe resolves the executable from mbx artifact output and retains the
common host lock for conservative GPU serialization through timeout/process-group
cleanup. Build: 900s; producer: 600s; lock wait: existing repository policy;
outer recipe: `BBR_VISUAL_TIMEOUT` (60m default). Output goes to a fresh ignored
`reports/visual/gallery-probes/<run>` directory. `coverage.tsv` is written only
after all pages and app shutdown succeed. The adapter checks case completeness,
contiguous child ranges, exact page names/dimensions and no extra output.

**These are unsealed probes, not accepted baselines or all-state coverage.** The
registry selector/appearance matrix does not yet audit interactive states,
family overrides/redundancy or nested/shadow framing. Gallery comparator config,
source/environment sealing, HTML review and explicit acceptance remain in the
[coverage packet](roadmap/tasks/gallery-visual-coverage.md). Do not feed these
probe directories to the existing eight-case browser acceptance command.

## Updating the shared tool

Import only a reviewed immutable CLI revision, stage it outside the repository
first, preserve licensing, update the provenance hashes, and rerun unchanged
upstream contracts plus BBR adapter/capture checks. Do not patch the vendor
copy to relax image comparisons or invent a second comparator in BBR.
The upstream package is still `publish = false`; vendoring this small tool is
not a registry-release or distribution claim for the complete mobile SDK.

## Integration boundary

This tooling lane is isolated from the concurrent Paper-alignment product
lane. Any baseline reviewed here describes this lane's source/environment,
not the eventual combined product. Integrate product fixes first, then this
tooling, and review fresh captures on the combined tree. Do not silently
accept visual changes introduced by another lane.
