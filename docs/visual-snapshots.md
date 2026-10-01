# Production-chrome visual snapshots

BBR reuses the portable **cargo-gpui-visual 0.1.1** comparator/report from the
standalone GPUI Visual authority. The complete unchanged 83-file export lives in
`vendor/cargo-gpui-visual`, pinned to commit
`c2723cac924b98b8a2c8c170d580bd2e434b674a`. Its file hashes and previous SDK
import provenance are in `scripts/visual/tool-source.json`; every invocation
verifies them and rejects
unexpected vendor files (excluding mbx's generated target directory). The CLI is
Apache-2.0 with its original LICENSE/NOTICE retained. Its independent Cargo
root never links GPUI; BBR does not depend on a developer's SDK checkout or
on the SDK renderer helper. Nothing is installed or published.

## Migration checkpoint and execution boundary

The standalone import is portable-tool evidence only. The existing Python
entrypoints below remain during migration; they have **not** become Tart-aware.
Do not execute their Gallery/browser producers on the host. The Rust consumer
must first prove its VM pilot and full capture parity before replacing them.
Read-only inspection and portable fixture contracts need no native session.
Existing eight PNG references remain untouched; Gallery baselines and combined-
product review remain open. A new tool/source/environment seal does not authorize
reference adoption. The current task is owned by the
[visual tooling packet](roadmap/tasks/visual-snapshot-tooling.md).

The separate Rust consumer foundation is restored under
`/Users/patwoz/dev/Personal/macos/bbr/scripts/visual`. It calls the immutable
library for Collection comparison, explicit acceptance, process cleanup and
leases. Its `produce` and `check` commands deliberately fail before building or
creating output until the Tart adapter is implemented; there is no host override.
`doctor` is portable/read-only. The recovered `environment` command describes
only the host and must not be used as guest rendering evidence. `just visual-test`
includes its portable acceptance, page validation, cancellation and refusal tests.
The recovered Gallery plan/catalog is a historical contract fixture, not a fresh
registry, applicability, asset or nested-framing audit. No capture parity or
Python cutover is claimed by this checkpoint.

### Read-only Rust diagnostics

The already-built consumer provides versioned upstream JSON envelopes (including
structured invalid-input errors) without running capture/build/environment commands:

```sh
/Users/patwoz/dev/Personal/macos/bbr/scripts/visual/target/debug/bbr-visual doctor --json
/Users/patwoz/dev/Personal/macos/bbr/scripts/visual/target/debug/bbr-visual check --dry-run --suite gallery \
  --run /Users/patwoz/dev/Personal/macos/bbr/reports/visual/gallery-runs/example-preview \
  --report /Users/patwoz/dev/Personal/macos/bbr/reports/visual/example-preview-review --json
/Users/patwoz/dev/Personal/macos/bbr/scripts/visual/target/debug/bbr-visual inspect \
  --report /absolute/path/to/explicit/review --limit 12 --json
```

Build the consumer through bounded `mbx` when needed; a missing/GC-evicted binary
is not permission to use a native producer instead. `doctor` validates sources,
export and authored coverage metadata, explicitly not the current Gallery
registry or framing. Dry-run requires both run and report paths, never writes
them, and labels the configured legacy argv as blocked pending Tart. It does not
probe/acquire locks or authorize execution. `inspect` delegates to the portable
stored-evidence reader; use `--case suite:case` for Collection detail. It never
recaptures, compares or validates current source/baseline/environment integrity.

`list --suite gallery --json` and dry-run `--story <exact-id>` use the immutable
selection API. Until independently audited story catalogs and producer support
are connected, real configs correctly return `selection_unsupported`, rather
than advertising historical requirements as executable coverage. Disposable
fixtures prove complete Light/Dark pairs/pages, authored selection order and
unknown/duplicate/unsealed-catalog rejection. They do not prove producer-side
selection before real rendering. The historical 49-story plan is already known
to lag the current 53-story Gallery registry; fresh plan/framing evidence is
required before full Gallery capture can pass.

### VM payload foundation (not yet executable capture)

The Rust payload module stages only the configured, hashed source inventory plus
exact Cargo-reported release executables. It snapshots sources before a future
build, rejects drift before/after copying, preserves guest compile-time paths and
resolves mbx's target-root symlink only for those executables. It never shares an
entire working tree or target directory. A private fresh external directory,
streamed bounded copies, per-file SHA-256/size/mode checks, exact inventories and
a completion manifest make transfer mutations fail closed. A partial payload is
not resumable or complete. No capture command uses this foundation yet.

Portable fixtures prove stale sources, missing/duplicate/out-of-tree executables,
symlinks, unexpected files, wrong permissions, changed manifests, forged guest
mappings and already-cancelled operations are rejected. This does **not** prove
actual VM transfer, guest execution/environment, post-capture source identity,
artifact return or Tart cleanup. Those require the pending adapter and approved
VM pilot; native producer entrypoints remain blocked.

### Tart lifecycle and guest-environment prerequisites

The shared Tart helper now attempts all bounded cleanup actions even after a
failed detach/stop, reports deletion/key/log failures, and preserves successful
cleanup evidence across repeated calls. Optional offline-clone hooks leave the
normal E2E boot/key policy unchanged. The visual-only policy replaces inherited
clone SSH settings and keys before boot, disables known unrelated service jobs,
and requires observed effective key-only SSH, no forwarding, a TCP listener audit
and host reachability checks before yielding a session. The lifecycle wrapper
uses only the fixed golden image and a read-only payload share, restores signal
handlers, and propagates cleanup failure. There is no runnable capture CLI yet.

Host compiler/SDK identity and guest OS/model/display/font/session identity are
separate typed records. Guest evidence must verify every staged source/executable,
bind the exact payload hash, and match before/after observations. Binary hashes
belong to the per-run receipt, not the rendering-environment equality key: ordinary
code changes must not masquerade as environment adoption. Guest observations
never fall back to host values, user fonts or a locked/missing guest session.
All tests so far use synthetic observations and mocked Tart/SSH; no actual guest
hardening, environment, transport, timeout or cleanup pass is claimed. The
consumer host-capture guard remains closed pending adapter integration and the
explicitly approved pilot.

### Raw artifact return prerequisite

The transport helper packages only flat raw PNGs and explicitly named JSON/TSV
evidence, including a guest receipt bound to the expected payload hash. The host
hashes every bounded archive member before creating a fresh staging directory;
it never calls a generic archive extraction API. Missing/extra/duplicate files,
unsafe paths, links, extended/sparse members, oversized/truncated content and
changed bytes fail. Extraction rechecks bytes and publishes its completion
manifest last; interrupted staging is retained without a usable completion marker
and cannot be overwritten. PNG bytes are not decoded, transformed or accepted by
this transport. The existing Rust validators must still prove format, dimensions,
coverage and guest provenance after return and before upstream capture sealing.
Fixture round trips and hostile archive tests are portable evidence only: real
SSH transfer, native images and capture integration remain unverified.

### Guest installation prerequisite

`vm_install.py` consumes the Rust payload schema and an independently supplied
manifest hash, compiled repository path and suite. It checks every shared file
before writing, then copies only exact sources/artifacts into a fresh tree at
the compiled paths. Before/after checks reject changed bytes, executable modes,
missing/extra entries, symlinks/hardlinks, path collisions and incomplete copies.
Copying/hashing has byte/count/deadline limits. It returns observed installed
hashes for the later guest receipt, never a capture seal. There is no installer
CLI, executable launch, destination remapping, reuse or overwrite. The adapter
must prepare the parent directory only inside its approved disposable guest.

Current evidence is synthetic installation plus real-payload read-only parsing;
actual guest installation/execution and cleanup remain unverified. The real
capture guard stays closed. Python transport helpers own neither comparison nor
coverage/acceptance policy; those remain Rust/upstream responsibilities.

### Host-only producer preparation

`just visual-prepare gallery /absolute/fresh/external/payload` (or `browser`)
uses the repository's existing Xcode/SDK environment and bounded `mbx build`,
resolves the exact Cargo artifacts, checks host environment and source identity
around the build, then stages the verified payload. It prints typed host-build
JSON; redirect that explicitly to diagnostics outside the payload when needed.
No inventory executable, app, renderer, VM, SSH or service is started. Output
must be fresh, absolute and outside the checkout, under an existing directory.
An existing payload is never reused or overwritten; source changes require
fresh preparation. This build-only command does not bypass `produce`/`check`'s
closed native execution guard and does not make Python capture entrypoints safe.

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

## One command for all current snapshots

Use the existing macOS dev environment and Xcode/Metal toolchain:

```sh
just visual
```

Historical implementation: runs the adapter/upstream contracts, then all
registered Gallery selector/theme pages and the existing eight browser-chrome
snapshots sequentially. Gallery is windowless; **browser capture creates native
BBR windows** and needs a guest macOS session. Until the Tart-aware migration
lands, this legacy producer command is not authorized for host execution. This does not run unrelated functional E2E/performance.
Both suites run even when the first comparison needs review or fails. Exit 0
means both pass, 1 means an upstream non-passing comparison (inspect findings and
validation errors), 2 means invalid input or failed execution. Contract failure
stops before capture.

The command prints one batch ID and an HTML entrypoint at
`reports/visual/batches/<batch>/index.html`, linking both unchanged upstream
reports, command logs and per-batch candidate directories. It never opens a
browser/server. The Gallery config (`visual-gallery.toml`) has its own environment
and baseline namespace, keeping the existing browser renderer/environment intact.
Both configs seal current source/environment before and after capture. Gallery
output names move `page-NN` before final `--light`/`--dark`; PNG content is untouched.

### Update reviewed snapshots

After reviewing the report, use one command for selections from either suite:

```sh
just visual-update <batch-id> \
  gallery:gallery-22-state-00-standard-normal-page-00--light \
  gallery:gallery-22-state-00-standard-normal-page-00--dark \
  browser:ordinary-full--light
```

Copy exact IDs from the suite report and prefix them with `gallery:` or
`browser:`. Only named changed/missing cases are updated, using the unchanged
upstream accept operation. The explicit batch is required: no implicit latest,
accept-all, wildcards, new captures or automatic baseline writes. Every selected
suite is preflighted before writes; upstream revalidates its own acceptance.
Multiple suites are not a single filesystem transaction: on interruption inspect
the reported suite and Git diff before retrying. Re-run `just visual` after an
update or source edit before accepting another selection. Environment drift
still requires separately reviewed upstream `--adopt-environment`; this shortcut
never opts in automatically. Adding the command does not accept any Gallery
baseline or complete its supported-state/framing audit.

## Individual commands

The existing browser-only operations remain available:

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
source/environment sealing and HTML review for the current matrix are available
through `just visual`; complete coverage review/acceptance remains in the
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
