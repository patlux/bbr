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

### Explicit Rust stored-run comparison

The already-built consumer can review sealed runs without enabling capture:

```sh
timeout --kill-after=10s 5m /Users/patwoz/dev/Personal/macos/bbr/scripts/visual/target/debug/bbr-visual compare \
  --gallery-run /absolute/path/to/gallery-run \
  --browser-run /absolute/path/to/browser-run \
  --report /absolute/path/to/fresh-review --json
```

Supply at least one explicit suite run. Paths must be absolute, canonical,
symlink-free and disjoint from each other and the configured baseline stores.
The report must not exist; inside the checkout it may only be a child of
`/Users/patwoz/dev/Personal/macos/bbr/reports/visual`. There is no latest lookup,
build, environment probe, native process, recapture, resealing or baseline write.
Current config/source-policy binding is checked before review; unchanged upstream
validates sources, image integrity, exact pixels, environment and independent
canonical coverage. Invalid full-suite captures remain visible beside valid
siblings in the Collection rather than disappearing from the report.

A single selected Gallery run produces a separate `TEILLAUF` development report,
not a Collection; its configured story catalog and independent page plan must
still match. Selected/full mixtures, selected browser runs and sequence inputs
are refused. Full still comparisons return `scope=collection`; selected reviews
return `scope=selected-development`, never full-suite success. Exit codes are
0 for a passing scope, 1 for review required/invalid suite evidence, and 2 for
invalid request/configuration or operation failure. JSON explicitly reports
`producer_executed=false`, `environment_probed=false`, `baseline_writes=false`.

Keep the outer hard timeout: upstream comparison is synchronous and cancellation
is checked at operation boundaries, not per image. Interruption may retain partial
diagnostics or a completed report; it never authorizes acceptance or recovery of
an interrupted report lock. Do not reuse the output path or remove recovery evidence
automatically. Baseline/environment adoption still requires Patrick's concrete
approval and explicit reviewed IDs. Python recipes are intentionally unchanged
until real Tart capture and end-to-end parity are proven.

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

### Cooperative transport supervision prerequisite

The Rust VM supervisor retains upstream process/gate authority but requests
cooperative cancellation through a private, request-hash-bound control directory
before the hard deadline. The Python Tart owner installs signal/finally handling
before watching requests; a successful concrete cleanup verifier is required
before atomic terminal acknowledgement. The host independently validates its
resource evidence before finishing the gate. Timeout or cancellation can release
a proven-clean lease, but never succeed as a capture. Missing/stale acknowledgements,
resource-check failures, output overflow or hard termination preserve unfinished
gate evidence and require explicit investigation; there is no automatic recovery.

Real local fixture processes prove the handshake, detached-process cleanup and
both leases' lifetime, including INT/TERM/HUP. These are not VM cleanup evidence.
The visual lifecycle now invokes a concrete read-only resource observer after
cleanup: its own Popen must be reaped, key/clone path absent, clone absent from
Tart inventory, disk image detached and guest SSH unreachable. Unknown or failed
observations cannot publish cleanup evidence. Optional bound lifecycle records
now link the independently expected clone, original supervision request and
payload before boot and actual IP/Popen before producer access. A concrete Rust
host verifier checks these records and repeats clone/key/disk/SSH/process absence
under both leases. Existing/reused PIDs and ambiguous observations fail closed;
only existence signal 0 is allowed, never numeric-PID termination. Record-only,
stale, swapped or partial-boot evidence cannot release the gate. Request identity
comes from supervisor memory, not a child-writable final file.

Local contracts exercise this verifier wiring with mocked resource observations,
including both leases' lifetime. They are not actual VM teardown evidence. Real
VM verification and end-to-end bootstrap/SSH/return adapter wiring remain required.
The normal capture entrypoints are still unconditionally blocked.

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

### Fixed guest execution prerequisite

Both suites include the independent release consumer at
`/Users/patwoz/dev/Personal/macos/bbr/scripts/visual/target/release/bbr-visual`.
Its internal `guest-execute` entrypoint requires actual macOS admin/VirtualMac
identity, exact compiled paths and hash-bound shared/installed bytes. Only fixed
Gallery inventory/capture or the existing browser capture test may run, with a
cleared deterministic environment and immutable upstream process cleanup. No
host override, arbitrary command, build, baseline write or capture seal exists
in this entrypoint. The normal `produce` and `check` commands remain blocked.

The guest transport composition observes the real environment, installs into a
fresh compiled tree, invokes that consumer, rechecks sources/artifacts/environment
and packages a bound receipt with unchanged raw output. On interruption it gives
the Rust child bounded grace to reap its producer group; hard termination is
unproven, never success. Host receipt validation, exact coverage checks, source
revalidation and observed VM cleanup are still required before upstream sealing.

Current evidence comprises synthetic composition/negative tests, host-refusal
contracts and actual host-only release preparation with all staged hashes checked.
No actual guest producer or VM was launched. Old payloads lacking the consumer
role cannot satisfy the current inventory; preserve them as diagnostics only.
Host bootstrap, SSH, supervision/cleanup integration and approved real capture
remain open. This does not establish current Gallery applicability/page coverage.

### Host-only producer preparation

`just visual-prepare gallery /absolute/fresh/external/payload` (or `browser`)
uses the repository's existing Xcode/SDK environment and bounded `mbx build`,
builds the separate release consumer, resolves the exact Cargo artifacts, checks
host environment and source identity around both builds, then stages the verified
payload. An offline runtime preflight is required before typed host-build JSON is
printed; redirect that explicitly to diagnostics outside the payload when needed.
No inventory executable, app, renderer, VM, SSH or service is started. Output
must be fresh, absolute and outside the checkout, under an existing directory.
An existing payload is never reused or overwritten; source changes require
fresh preparation. This build-only command does not bypass `produce`/`check`'s
closed native execution guard and does not make Python capture entrypoints safe.

### Pinned guest-runtime prerequisite

`/Users/patwoz/dev/Personal/macos/bbr/scripts/visual/guest-runtime.json` pins the
existing Darwin arm64 Python 3.13.11 interpreter, executable hash and complete
Nix closure (51 store paths; 1,492,264,592 NAR bytes), including current producers'
libiconv dependency. These are reproducibility inputs, not a host package install
or a claim that the golden image has the runtime. The full closure includes
Python's toolchain references; no guessed smaller closure is substituted.

Preparation checks every staged executable with `lipo`/`otool`, refuses unresolved
rpath/Homebrew/unknown dylibs, compares offline Nix closure metadata with the
source-bound lock, verifies actual store bytes with `nix-store --verify-path`,
and checks Python's hash/version/architecture and required standard-library
imports using isolated `-I -B`. It then repeats metadata and payload/source checks.
Nix's optimized hardlinks are valid runtime storage; payload single-link checks
remain unchanged. Rust bounds the complete probe group and validates the returned
original payload/lock/executable identities. No Nix realization, repair,
substitution, profile change or network operation is performed. Failure leaves
staging diagnostics, never a successful preparation or capture seal.

The host observations are explicitly `host-runtime-prerequisite`, not guest
rendering evidence. Guest runtime availability/transfer, source-bound bootstrap,
SSH execution/return and approved VM pilot remain unproven. Baseline/environment
adoption and the native capture guard are unchanged.

### Source-bound guest bootstrap (not yet launched)

The bootstrap planner generates one fixed zsh input from a verified staged
payload. It refuses different compiled destinations and helper bytes that do not
match current source. The generated command clears the environment, suppresses
interactive zsh configuration, requires actual UID 501/admin/VirtualMac before
mutation, hashes every pinned NAR offline, checks the interpreter executable and
mounts only the existing Tart payload share with `mount_virtiofs -r`. No runtime
realization, repair, download or Xcode Python fallback is generated.

Before any helper code runs, isolated Python requires an observed read-only mount
and validates the independently supplied payload/helper hashes with bounded
single-link regular-file reads. All helpers are compiled in memory first and
loaded from those frozen bytes, not by adding shared directories to `sys.path`.
The verified guest tail repeats actual guest identity, preflights the payload,
then creates only a **fresh** `/Users/patwoz` ancestor and its compiled repository
parents. It never recursively changes ownership or reuses an existing tree.
The existing guest runner remains the only installation/receipt composition;
its Rust consumer retains native producer process ownership.

Local tests prove generation, shell syntax, host refusal, source/mount/path
rejection, import ordering and frozen-byte execution using inert fixtures.
Actual guest NAR availability, VirtioFS layout/mounting and compiled-parent setup
remain unproven. The internal supervised transport now calls this planner, but
normal capture entrypoints remain blocked and no actual SSH/Tart run is claimed.
Network approval, runtime availability and real pilot evidence are still required.

### Supervised transport and return (local-fixture evidence)

The internal Rust transport validates current sources, runtime prerequisite and
host-built executable identity before acquiring the upstream supervisor's leases.
Python composes the fixed bootstrap with the existing hardened lifecycle and
exact isolated SSH command. Only `/usr/bin/ssh` executes, with a cleared local
environment, no agent/proxy/forwarding/multiplexing inheritance, bounded files for
stdout/stderr and no detached transport child. The guest's fixed `return.tar`
path is streamed unchanged; there is no scp path selection, automatic retry or
new native producer runner.

Archive and payload checks precede teardown. Raw candidates are unpacked only
after lifecycle cleanup and independent resource observations. The control result
is published after another cleanup recheck; Rust repeats its original-bound host
verification before either lease finishes. Then Rust verifies the exact returned
file set/bytes, completion marker and hash-bound guest receipt, binding separate
host-build and actual VirtualMac rendering identities. Operation failure with
proven cleanup may release leases but cannot succeed; host rejection of cleanup
retains the unfinished gate. A swapped receipt cannot become capture evidence.

Tests execute the real supervisor/control/transport/return/binding chain with
private locks and synthetic file bytes, substituting only runtime and resource/
SSH boundaries. They prove protocol ordering, not actual network, guest teardown,
PNG validity or full coverage. This route has no normal CLI caller yet; the
unconditional `produce`/`check` guard remains. Source/current-catalog validation,
approved real pilot, two exact complete captures and upstream sealing/review
integration are still required. No baseline is changed by transport success.

### Selected Gallery plan bridge (portable only)

`/Users/patwoz/dev/Personal/macos/bbr/scripts/visual/src/gallery_selection.rs`
resolves an upstream `Selection` against both its hash-bound authored catalog and
an independently hash-bound Gallery page plan. Its immutable schedule exposes
numeric story IDs in authored order, retaining every planned state/page and both
appearances (including different Light/Dark child ranges). Missing/extra,
reordered, cross-story or sequence requirements fail before a schedule is usable.
Rechecking cannot adopt a changed plan merely because a caller supplies new hashes.

Selected raw validation reuses the existing Gallery validator, with regular
single-link output checks and an exact upstream `selection.json` receipt only
after complete page/image/source validation. Upstream retains schema-2 sealing,
partial report labels and unconditional selected-run acceptance/Collection refusal,
including when all catalog stories were explicitly selected. Portable fixtures
exercise those APIs and the actual consumer update router with synthetic PNGs.
They prove scheduling data and protocol boundaries, not native pre-render behavior.

No production story catalog is enabled by this bridge and no current Gallery
page counts are invented. The internal guest route described below now consumes
it; the internal host transport is connected as described below, but the real
approved pilot remains required. Ordinary capture remains blocked; these tests
do not satisfy visual or framing acceptance.

### Selected guest execution (ordinary capture still blocked)

The existing guest-only entrypoint accepts `capture-selected` only for Gallery
and only with an independently supplied hash for the fixed
`/Users/admin/bbr-visual-work/selected-request.json`. The request binds the payload
and upstream Selection; the installed, source-hashed config must explicitly name
its story catalog. Normal configs still do not enable selection. Actual guest
identity, installed executable/source checks and the host refusal remain intact.

Before rendering, the existing non-rendering inventory command must match every
selected state/appearance from the independent plan; extra selected-story states
are rejected. Then one fixed `--capture-gallery … --story <numeric-id>` command
runs per selected story in authored order, with the cleared guest environment,
upstream process ownership and one overall 600s deadline. This reuses the Gallery
filter which precedes renderer-context creation, without changing Gallery code.
Per-story and aggregated raw outputs pass the same existing image/range validator.
No images are filtered after rendering, decoded into replacements or transformed;
raw files move unchanged and the exact selection receipt is published last.

Portable command-boundary tests inject synthetic outputs, never native rendering.
They prove ordering, invalid-request/pre-cancelled no-invocation behavior, registry
mismatch and output/source/cancellation failures. No end-to-end VM/native selection,
complete coverage or acceptance evidence is claimed.

### Selected request transport and return binding

The internal Rust transport retains an immutable request with the original
Selection, payload digest and source-bound independent schedule. The same request
validator is reused by the guest action. Exact UTF-8 request bytes (at most 64KiB)
and their independent SHA-256 pass through the fixed, source-frozen bootstrap;
no arbitrary request path or guest command is accepted. The bootstrap remains
bounded at 256KiB and rejects oversized quoted input before boot. Normal configs
do not activate real story selection, and normal capture still has no caller.

The guest writes only a fresh fixed request file, invokes `capture-selected` with
the expected hash, and rechecks it before return. Original source/install/guest
observations, native process ownership, cancellation and cleanup remain unchanged.
After independent host cleanup and byte verification, Rust requires the exact
serialized selection receipt retained by the host, not a new expectation inferred
from returned files. Missing/swapped selection, an unexpected receipt on a full
run, request drift or source drift cannot produce a successful typed capture.
The typed result explicitly retains selected scope; this transport does not seal,
rename images, prove coverage or authorize full-suite Collection/acceptance use.

Seven new Python contracts, two Rust request contracts and extended real local
supervisor/transport-chain fixtures exercise these paths. Known-clean failure or
cancellation releases both leases; rejected host cleanup retains active gate
evidence. Returned fixture bytes are explicitly not native PNG evidence. Host
raw diagnostics may remain after a receipt rejection, but no seal is published.
Actual guest runtime/SSH/capture/teardown, current catalog activation and the
approved real pilot remain required. The internal raw-to-seal bridge below does
not enable the ordinary CLI or establish runtime parity.

### Fresh transport to upstream sealing

The internal `vm_capture` route calls immutable upstream `capture_local`; its
source snapshot precedes one fresh supervised transport invocation. The first
local environment callback performs that transport, which obtains actual guest
before/after observations and proves cleanup. Both callbacks revalidate the
original return evidence and source/payload identity. They never execute the
configured legacy host environment or producer command. There is no import-old-
raw/reseal CLI; fresh workspace and run paths are mandatory.

Returned PNGs are bounded, hash-checked copies into a fresh run, never decoded
and re-encoded. Gallery preparation reuses the independent full plan or the exact
selected schedule, including asymmetric Light/Dark child ranges and name-only
canonicalization. Browser preparation shares the retained eight-image validator;
no second browser inventory authority. Extra files, injected seal/inventory/
sequence metadata, missing or extra pages, wrong dimensions/precision, duplicate
registry rows, source/payload/receipt drift and cancellation prevent sealing.
Prepared bytes and inventory are checked again before the final environment
callback returns. Upstream alone publishes `capture.json` and owns comparisons,
partial labels and acceptance refusal.

Original return diagnostics/marker remain unchanged outside the run. The guest
receipt and live registry are preserved as unchanged ancillary files alongside
the prepared PNGs, after producer validation. These sidecars are not additional
upstream-sealed manifest fields or an authentication boundary. The original
transport evidence remains necessary to audit build identity and cleanup.

Four portable contracts exercise fresh synthetic PNGs through this route and
actual upstream seal/report APIs (selected and full Gallery fixtures, all eight
browser fixtures), byte preservation, explicit partial HTML/nonacceptance and
negative cases. They are not actual image review, native capture, current full
Gallery coverage or VM teardown proof. Normal `produce`/`check` remain blocked;
real pilot, current catalog/state/framing audit and functional Tart gates remain
open. Existing baselines and retained Python entrypoints are unchanged.

### Current Gallery source-audit boundary

The [bounded source audit](visual/gallery-source-audit.md) checks the pilot and
four additions against actual story/production sources. The retained 49-story
plan still requires 936 page pairs; it is not current 53-story coverage. Explicit
withheld catalog entries now name Delete Space, Peek, Toast/zoom and Media card,
without fabricated page counts. Read-only `doctor --json` exposes those blockers
alongside the four historical interaction/framing/locked-session blockers.

In particular, outer-child pagination does not prove nested/shadow bounds,
canvas density is not always effective component density, Checkbox capture does
not drive pointer transitions, and Peek's stand-in pages are not native content.
Toast retains its entrance fade under Reduced Motion but lacks a nonzero current
settling policy. These are source findings, not observed guest pixels. Selection,
full applicability/framing, runtime readiness and two exact captures remain open.

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
