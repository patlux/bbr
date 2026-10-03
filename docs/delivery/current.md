<!-- bbr:current-delivery -->
# Current delivery

Start at [ROADMAP.md](../../ROADMAP.md), then read this file completely before choosing or scheduling work. This is the sole owner of volatile execution state. Durable acceptance lives in the linked packets; detailed chronology remains in Git and the historical delivery ledgers.

## Control metadata

| Field | Value |
|---|---|
| Generation | `authority-reconcile-2026-09-02` |
| Cutover commit | `d2b507f56b9da068e7df0aa08c3f8b12cdec506c`; reconciles the durable contracts and replaces the stale live queue |
| Verified product base | `main@e464ca3282e969b192a30906915445a24dd704e6` |
| Integrated product checkpoint | product tree `c39201948409017aa4f95bdfd4713928a78aad0f` at `main@e464ca3282e969b192a30906915445a24dd704e6`; the following commits change documentation/control only |
| Previous canonical pointer | `docs/delivery/2026-08-08-post-mvp-delivery.md` |
| Rollback pointer | `docs/delivery/2026-08-08-post-mvp-delivery.md@0dc22fee16d737a077aa1b097e359e2f1d08a764` |
| Target Orchestrator session | `01a0e454-d11c-7743-8e0f-53fd4138ff08` (v2 program, since 2026-09-28; the RA-MEDIA session `01a0df39` has no live process); reverify pane/session activity before any ownership transfer |
| Preserved design checkpoint | `design/browser-shell-paper@414c97c7a5139cf6070a54138969a3bb34403bc2`; design intent only, not runtime acceptance |

UI-CONFORMANCE, FOLDER-PIN-INVARIANT, DESIGN-CARD, DESIGN-ADDRESS and DESIGN-FAVORITES are integrated on `main` (2026-09-27, through `d51a4b52`), each independently reviewed with the review findings remediated. Their performance evidence is deferred with the rest of performance execution until the end of the queue (Patrick, 2026-09-27). RA-MEDIA integrated its portable `bbr-web` contract as `ec5e0bad` and is paused there (macOS adapter not started, no owner). Since 2026-09-28 the serial work on `main` is the V2 visual refresh and motion program (DESIGN §18), preflighted clean at `main@ec5e0bad368179b669983a60a22a7b47b152ebfb` with no other BBR writer process. Historical lane worktrees retain evidence or abandoned experiments but own no current queue state. The stale Split worktree remains explicitly untouched.

## GPUI Visual migration activation (2026-10-01)

Patrick activated the current plan as goal `1790884867215-ea6dba478fa758`.
Session `01a0f8f4-1c46-71bf-9fae-3c394de53395` owns only this bounded visual
migration on `/Users/patwoz/dev/Personal/macos/bbr`, branch `main`, clean
activation base `28cea1ce03f131bcfa41e4ff1820c2521d996c1c`. The preceding Portal
owner explicitly finished its integration; cleanup is awaiting separate worktree
permission, and internal-pages continues only in its isolated lane. Process cwd
alone is not treated as active write ownership. Main index/worktree were clean;
no Git lock or holder of the repository/GPUI Visual capture locks was found.

| ID | Slice | Status | Worktree | Owner | Contract | Next gate |
|---|---|---|---|---|---|---|
| VISUAL-RUST | Standalone GPUI Visual consumer migration | `running` | main, activation base above | session `01a0f8f4` | [visual tooling](../roadmap/tasks/visual-snapshot-tooling.md) | Exactly six Command Palette images explicitly approved and adopted; both stored captures now compare72/72 exact. Prior66 references, rendering environment and historical evidence preserved. Reference-only verification passes; capture implementation remains uncommitted, and the unchanged50ms Command maximum still fails its recorded aggregate/isolated checks. Seven withheld families, complete coverage, public cutover and stable aggregate remain open; latest checkpoint below |

Historical reuse evidence is read-only Git/session metadata, not a takeover of
`/Users/patwoz/dev/Personal/macos/bbr-visual-snapshots`: its five migration commits
end at `74b94eb07120ddb62e6dda5c7a935b2deaee21b1`, while the stopped owner
`01a0e635` explicitly preserved six uncommitted files on 2026-09-28. The old
session's final BBR state is stopped, not finished; its later work was upstream
story selection. No isolated working files have been inspected or modified here.
Reuse committed implementations only after inspecting their contracts; any dirty
handoff requires separate permission. Current main differs by 79/5 commits.

Bounded scope: immutable tool import, separate Rust consumer, existing real
Gallery/browser producers, story selection, independent coverage, Review
Collections, read-only agent JSON and explicit-only acceptance. Shared integration
points are the visual recipes/configs, Gallery capture support and roadmap/E2E
contracts. No product graph/GPUI pin change, browser behavior rewrite or work in
other lanes. Old committed work and all baselines remain preserved. Later
internal-pages integration must retain both authorities and recapture on the
combined tree; no stale lane pixels become combined-main acceptance.

Parallelism audit: no workflow/subagent/new worktree is authorized by this goal.
The portable import is runnable locally; VM capture waits for the current network
exposure acknowledgement, and the actual Checkbox pilot needs the narrow real
Gallery asset-source correction (currently `Arc::new(())`, no Check SVG provider).
Neither is replaced by fake pixels or weakened checks. Gallery full state/framing,
missing Gallery references and changed browser references remain open. Existing
Tart golden image is present/stopped, observed read-only; no guest was started.
All actual GUI/Gallery/browser gates use Tart, superseding historical host-GUI
instructions below. Performance execution remains deferred/unpassed. Capture
throughput is not product performance. Next actions are portable tests and a
bounded VM adapter/pilot; no main/foreign-app launch, push or auto-acceptance.

### Standalone import checkpoint

The complete 83-file immutable `c2723cac924b98b8a2c8c170d580bd2e434b674a`
export was staged and inspected at `/tmp/bbr-gpui-visual-import.vgaTth`, then
imported without modification. The existing LICENSE is byte-identical; source
revision/file hashes and prior SDK provenance are recorded. Normal BBR Cargo
manifest/lock, all crates and the eight existing PNG/baseline metadata files
have no diff. This changes only the portable tool, not a renderer or capture.

Fresh bounded evidence under the existing BBR Rust 1.97.1 environment:
- `mbx test --locked --offline --manifest-path vendor/cargo-gpui-visual/Cargo.toml`:
  175 top-level passing tests plus 3 child-process fixture executions, no failures.
- Same root, `mbx clippy --locked --offline --all-targets -- -D warnings` and
  `cargo fmt -- --check`: pass.
- `just visual-test`: 35 Python adapter tests pass (real-CLI class skipped in
  discovery), upstream tests pass, separately executed real-CLI two-suite
  acceptance/stale-source regression passes using disposable stores only.
- Exact 83-file provenance check, `doctor --json` (1,413 source inputs; producer
  and environment probe not executed), roadmap and whitespace checks pass.
- Logs: `/tmp/bbr-visual-import-upstream-tests.log`,
  `/tmp/bbr-visual-import-clippy.log`, `/tmp/bbr-visual-import-adapter-tests.log`,
  `/tmp/bbr-visual-import-update-test.log`, `/tmp/bbr-visual-import-visual-test.log`.

No native/GPU capture, VM boot, baseline/environment update or product gate is
claimed. Existing Python producer commands still target host execution and must
not be run; the documentation explicitly marks them pending Tart-aware cutover.
The Rust-consumer/VM pilot, coverage/selection and full repeat evidence remain
open. Self-review checked import hashes, immutable upstream authority, feature
and dependency boundaries, old CLI adapter contracts, and no protected-file diff.

### Recovered portable consumer checkpoint

Recovered only committed Rust consumer files from `74b94eb07120ddb62e6dda5c7a935b2deaee21b1`
through `/tmp/bbr-visual-consumer-source.6aogir`, with the current standalone
manifest API and an independent lockfile. No stopped-lane working file was read.
The consumer calls upstream Collection/acceptance, gate and process APIs; its
page validator rejects missing/extra/duplicate/reordered pages and checks RGBA8
and dimensions before identity-only renames. Historical catalogs remain an
unaudited fixture contract, not current complete Gallery coverage.

Both `produce` and `check` now fail closed before native builds/output while
Tart is unwired. No environment-variable host bypass exists. Portable `doctor`
remains available; the recovered environment probe is host-only and is not guest
rendering evidence. Existing Python capture commands stay unexecuted and retain
the documented no-host boundary; their removal waits for real VM parity.

Fresh evidence: consumer 17/17 portable tests, strict all-target Clippy and fmt
pass. Tests include real CLI explicit-only multi-suite acceptance, no writes on
stale report/source or invalid selection, exact page-plan checks, cancellation
on INT/TERM/HUP, descendant reaping before both leases release, and unwired
native-entrypoint refusal. `just visual-test` now includes this consumer and
passes alongside the unchanged upstream/Python fixture gates. Logs:
`/tmp/bbr-visual-consumer-recovered-tests2.log`,
`/tmp/bbr-visual-consumer-clippy.log`,
`/tmp/bbr-visual-consumer-visual-test.log`. Clippy measured 24.90s; implementation
and review wall time were not instrumented. Self-review inspected source export
integrity, library authority, output guards, acceptance journals, no product graph
or baseline changes and host refusal. No GUI, VM, performance or fresh pixel
coverage evidence is claimed.

Parallelism audit: only this local writer is authorized. Read-only discovery,
selection/coverage contracts and VM adapter development are runnable; VM boot and
the real Checkbox asset-source correction still await Patrick's two requested
approvals. No workflow/subagent/new lane, foreign worktree change or push.

### Read-only consumer diagnostics checkpoint

The consumer now exposes `doctor --json`, explicit-path `check --dry-run`,
`list --suite` and `inspect --report` through immutable upstream schema-1
response, selection, planning and inspection APIs. Strict option parsing never
turns a malformed dry-run into execution; no implicit latest, native command,
environment probe, gate acquisition, report creation or baseline writes. Stored
inspection is labeled unvalidated, not a fresh seal or acceptance permission.
The configured legacy argv remains diagnostic only and blocked pending Tart.

Four new direct/actual-CLI tests pass (consumer total **21/21**), including
compiler/native/probe tripwires, no filesystem output, structured failures,
explicit reports, interrupted-report refusal and unchanged stored bytes. An
allowlisted disposable fixture proves authored story order, complete pair/page
requirements and unknown/duplicate/unsealed-catalog rejection. Initial fixture
runs correctly rejected macOS's `/var` symlink; fixture paths were canonicalized,
not the production safety check weakened. Strict consumer Clippy/fmt,
`just visual-test`, roadmap and whitespace checks pass. Logs:
`/tmp/bbr-visual-discovery-tests.log` (initial red fixture),
`/tmp/bbr-visual-discovery-tests2.log`, `/tmp/bbr-visual-discovery-clippy.log`,
`/tmp/bbr-visual-discovery-visual-test.log`,
`/tmp/bbr-visual-discovery-doctor.json`. No actual PNG/report inspection or GUI
execution is claimed; fixture tests are portable protocol evidence only.

Read-only source audit found current Registry IDs 0–52 (53 stories) versus the
historical plan's 0–48 (49): Delete Space sheet, Peek presentation, Toast/zoom and
Media card were added. Existing selector/framing changes also need audit.
Real `list`/selected previews therefore remain correctly unsupported until the
independent story catalogs and real producer support are connected; no generated
inventory or old page ranges are promoted to current coverage authority.

Self-review checked library reuse, source-bound catalog loading, read-only output
semantics, exact explicit paths, strict arguments and protected-file diffs.
Parallelism audit: still one local writer, no agents/lanes; VM adapter development
and independent coverage audit remain runnable. Boot/network and Checkbox asset
approval remain pending; no VM, host GPU, baseline update, product gate or perf
claim. All changes are confined to consumer diagnostics/tests and documentation.

### VM payload foundation checkpoint

A consumer-only transfer module now snapshots the narrow configured source
inventory and stages source bytes plus exact suite Cargo artifacts in a fresh
private external directory. Source identity must match before/after copying;
byte/size/executable checks and complete source/guest-role mappings are verified
before publishing the transfer manifest. The mbx root link is resolved only for
named release executables, preserving their compiled guest paths. No whole-tree,
Git metadata, report, baseline or target-directory share. Copies/hashes are
bounded, cancellation-aware, and interrupted staging is not resumed or sealed.

Six new portable tests pass (consumer **27/27**): current source policies,
allowlisted Gallery/browser artifacts, byte identity, stale build source,
missing/duplicate/foreign artifacts, path overlap, symlinks, mutation, unexpected
files, permissions, changed manifests, forged source/guest/suite mappings and
pre-cancelled no-write behavior. `mbx test --locked --offline` and strict
all-target Clippy at the consumer root, fmt, `just visual-test`, roadmap and
whitespace gates pass. Logs: `/tmp/bbr-visual-payload-tests3.log`,
`/tmp/bbr-visual-payload-clippy2.log`,
`/tmp/bbr-visual-payload-visual-test.log`. Self-review added exact manifest/source
mapping, role checks and completion-after-validation. Timing was not instrumented.

This is a portable transfer prerequisite, deliberately not connected to capture.
No native producer or guest was started; actual build linkage, guest integrity,
environment separation, PNG return and Tart interruption/cleanup remain unproven.
Host capture refusal stays unconditional. Existing E2E/Tart infrastructure and
product/baseline/upstream sources are unchanged. Read-only inspection of the
existing Tart helper found its key injection does not itself prove password
login disabled or unrelated listeners absent; the visual adapter must enforce
and verify clone-only hardening before any capture, not trust SSH success alone.

Parallelism audit: no additional lane/agent authorized. Next runnable local work
is the bounded adapter using existing Tart clone/boot/cleanup infrastructure and
portable fault-injection tests, plus current coverage audit. Actual VM boot and
Checkbox asset change still need Patrick's requested permissions. No GUI, full
coverage, baseline acceptance, performance or stable-product completion claim.

### Tart lifecycle / environment contract checkpoint

Continuation preflight: clean `main@e1fc345`, no index lock or active visual/Tart
build/capture process. Only the existing visual owner writes locally; no lane,
subagent or foreign worktree access. Current queue still permits portable VM
adapter work but not an actual boot or Checkbox asset modification.

Direct fault injection exposed shared Tart cleanup stopping at the first failed
detach/stop and reporting delete/log failures as success. Seven new regression
contracts were red before the bounded repair. Cleanup now attempts every owned
resource, preserves errors/key on failed deletion, reports missing durable
evidence, and is idempotent after success. Optional offline-clone preparation and
exclusive-key/isolated-SSH flags preserve the default E2E boot policy. The visual
policy replaces clone-only SSH config before boot, disables known unrelated
launch jobs, then requires observed effective authentication, listener and host
port checks. Signal unwinding and cleanup propagation are exercised with mocks;
no actual Tart, SSH, listener, key generation or mounted-volume operation ran.

A separate guest observer and Rust receipt binding distinguish host build
compiler/SDK from guest OS/model/display/font/session facts. Receipt checks bind
all staged bytes and reject missing/swapped sources/artifacts or environment
drift. Build hashes remain run provenance, not environment-equality keys, so
ordinary source changes do not force environment adoption. Both visual source
policies now include the shared Tart helper. The observer refuses a host home,
user-font additions, symlinks, incomplete observations or a locked guest. No
current VM/environment provenance claim follows from synthetic fixtures.

Fresh gates: `just visual-test` passes 60 Python unit tests (one additional
real-CLI class skipped in discovery then run separately), 175 upstream tests
plus child fixtures, and **31 Rust consumer tests**. All 49 shared script tests,
strict consumer all-target Clippy, fmt, roadmap and whitespace checks pass.
Logs: `/tmp/bbr-visual-tart-cleanup-red.log`,
`/tmp/bbr-visual-tart-cleanup-green.log`,
`/tmp/bbr-visual-vm-foundation-final-visual-test.log`,
`/tmp/bbr-visual-vm-foundation-final-script-tests.log`,
`/tmp/bbr-visual-vm-foundation-final-clippy.log`. Test timing is in those logs;
implementation/review wall time was not measured. Self-review checked cleanup
failure propagation, no inherited SSH config/agent/proxy, first-boot hardening,
private-IP validation before initial SSH, exact environment binding and no
protected product/upstream/baseline diff. Guest policy compatibility and real
cleanup remain pending runtime evidence; the unconditional host guard stays.

Next local step is installed-payload verification / bounded artifact return and
transport wiring, followed by current Gallery coverage selection. These remain
runnable without VM permission; no completion, GUI gate, baseline acceptance or
performance pass is claimed. Boot/network and Checkbox approvals remain pending.

### Raw artifact return checkpoint

Added the bounded flat-file transport prerequisite, not a comparator or capture
seal. Raw PNG and explicit JSON/TSV evidence are bound to the payload hash;
all members are preflighted and hashed before host staging. No generic archive
extraction, symlink/hardlink, path rewriting, sparse/extended member, resize or
image conversion. Copied bytes are rechecked and the completion marker is last.
An interrupted copy leaves unusable partial evidence and refuses overwrite.

Six portable tests pass: exact synthetic round trip, payload mismatch,
missing/extra/duplicate files/JSON keys, traversal/absolute paths, links,
nonregular members, invalid sizes/hashes, truncation, byte limits, expired
deadlines and interruption without completion. `just visual-test` now passes
66 Python unit tests (one extra integration class skipped then run separately),
the unchanged upstream suite and 31 Rust consumer tests. Roadmap/whitespace
checks pass. Log: `/tmp/bbr-visual-vm-transfer-visual-test.log`.
Self-review added cumulative-size limits and second-pass extended-member checks.
No actual PNG, guest, SSH stream or baseline was produced/accepted; format,
coverage and receipt validation remain Rust/upstream responsibilities.

Parallelism audit: sole local writer remains; transport installation/wiring and
current Gallery selection/coverage audit are next non-network actions. No
subagent, new lane, host GUI, push or expanded product scope. The requested VM
and Checkbox permissions remain pending and do not justify a completion claim.

### Actual host-build / payload preparation checkpoint

Added a bounded host-only `visual-prepare` command: validate fresh external
paths first, snapshot sources and host toolchain, invoke the existing exact
release mbx build, recheck environment/sources and stage only suite artifacts.
No producer/inventory executable is run. Typed host build evidence is printed
outside the read-only payload. One new negative/pre-cancelled path contract
passes; consumer total **32/32**. `just visual-test` (66 Python unit contracts,
separate real CLI integration, upstream contracts and consumer), strict Clippy,
fmt, roadmap and diff checks pass. Logs: `/tmp/bbr-visual-prepare-visual-test.log`,
`/tmp/bbr-visual-prepare-final-clippy.log`.

First actual host-only preparations:
- `/private/tmp/bbr-visual-prepare.DW3Hlm/gallery-payload`: 1,449 sealed sources,
  one Gallery executable, 1,450 files / 39,454,744 bytes. Release build 1m29s.
  Gallery SHA-256 `568ce5a76d5846a0b72de8d58220162fd43504326ea03eef6fbefbd8f40f6c52`.
- `/private/tmp/bbr-visual-prepare.DW3Hlm/browser-payload`: 1,449 sealed sources,
  app + capture-test, 1,451 files / 57,229,032 bytes. Release build 1m40s.
  App SHA-256 `01da59651365c823764b86caeb457df6ba1fd7ffdfe35a8efd195d85cb36c072`;
  test SHA-256 `7ee611dcb964f785bf7eb071b97fbd113d8ef814b5a7fb207656a5c8db3e4788`.
- All staged hashes independently rechecked after preparation. Host macOS 27.0
  build 26A428, arm64, Rust 1.97.1, Xcode 27A5252f / SDK 27.0; GPUI exact pinned
  revision unchanged. `host-build.json`, `browser-host-build.json`, `prepare.log`
  and `browser-prepare.log` are under the run directory. Existing `block` future
  incompatibility warning remains, not suppressed.

These are build/staging artifacts only, not a guest receipt, image or capture
seal. Documentation changed afterward; capture must prepare fresh current-source
payloads, never re-seal these. No VM boot, producer execution, baseline or perf
claim. Self-review checked host-only call graph, artifact identities, source and
probe stability, mbx/Xcode route and fresh output guards. Parallelism remains
single-writer; VM transport wiring and independent coverage are next runnable
work, while actual boot and Checkbox asset permission remain pending.

### Guest installation contract checkpoint

Continuation preflight: clean `main@716a3fd`, no index lock or active visual/Tart
build process. Entire delivery queue reviewed; this owner may continue transport
and coverage work locally, not activate other lanes. No subagent is authorized.

Added `scripts/visual/vm_install.py`: bounded strict parsing of the Rust manifest,
independently supplied hash/repo/suite, exact shared-file preflight and fresh
compiled-path installation. Shared/installed bytes are reverified; no hardlinks,
symlinks, traversal, unknown executables, source omission, extra directories,
case/normalization aliases, file/directory collisions, mode drift or overwrite.
Failures leave incomplete diagnostic trees and never emit a capture seal. No
producer execution, sudo or VM boot is built into this transport module.

Evidence: **8 new Python contracts** plus **1 Rust→Python contract** exercising
both real Rust-staged suite formats with synthetic non-executed artifacts.
`just visual-test` passes 74 Python units + separate CLI integration, immutable
upstream tests and **33 consumer tests**; strict consumer Clippy/fmt pass.
Logs: `/tmp/bbr-visual-vm-install-tests.log`,
`/tmp/bbr-visual-vm-install-cross-language2.log`,
`/tmp/bbr-visual-vm-install-visual-test.log`,
`/tmp/bbr-visual-vm-install-clippy.log`. Initial cross-language compile caught an
incorrect test-only assumption about upstream Output.stderr (not exposed); the
fixture now uses the actual stdout/status API, without upstream modification.
Read-only inspector also validates all 1,450/1,451 files of the earlier real
Gallery/browser payloads; these remain stale diagnostic artifacts, not current
capture evidence, and were **not installed** into the host checkout.

Self-review checked strict manifest binding, APFS path collisions, nonblocking
regular-file checks, exact post-copy inventory and failure/no-resume behavior.
Real guest parent preparation, Python bootstrap, producer/receipt execution,
SSH return and supervision/cleanup wiring remain open. Native guard stays closed;
no baseline, product/performance or actual guest claim. Next runnable work is
transport supervision and current Gallery coverage; VM/network and Checkbox
asset approvals still remain external boundaries, not reasons to stop local work.

### Cooperative transport-supervision checkpoint

Preflight: clean `main@efe470f`, no index lock or active visual/Tart/build process;
full delivery queue read. Only this local visual owner is authorized. Portable
transport supervision and coverage remain runnable; other product lanes and
performance are not activated, and actual boot still needs network approval.

`vm_supervision.rs` reuses upstream Gate/execute rather than changing its SIGKILL
policy. It sends an atomic request-bound cancellation marker, allows bounded
cleanup grace, and finishes both leases only after terminal acknowledgement and
the caller's independent resource verifier. Hard kill, output overflow, stale or
missing acknowledgement, signal exit or rejected cleanup leaves active unfinished
gate evidence. `vm_control.py` installs the existing lifecycle signal guard before
watching requests; it publishes no terminal result until lifecycle unwinding and
concrete cleanup verification return. No automatic recovery, native entrypoint,
VM boot, host GUI or seal/baseline write is introduced.

Verification: **5 new Python contracts**, **6 new Rust top-level contracts**
(including the inert subprocess fixture), plus 3 isolated supervisor executions
for actual INT/TERM/HUP. Real Python processes own/reap a detached `/bin/sleep`
stand-in while checking both private leases remain held. Normal/failure exits,
cooperative timeout/cancellation, hard termination, output flooding, malformed
cancel requests, stale/missing acknowledgements and both host/child cleanup
rejection are covered. Detached stand-ins have their own 8s maximum lifetime;
no Tart/network/production binary is invoked. `just visual-test` passes **79
Python units** + separate CLI integration, immutable upstream and **39 consumer
tests** plus signal fixtures; strict Clippy/fmt pass. Logs:
`/tmp/bbr-visual-vm-supervision-final-visual-test.log`,
`/tmp/bbr-visual-vm-supervision-final-clippy.log`,
`/tmp/bbr-visual-vm-control-tests.log`. Self-review added atomic publication,
panic-path monitor shutdown, strict signal-exit rejection and test cleanup even
when lock assertions fail. No upstream/product graph/baseline diff.

This proves the protocol and local process lifetime, not real clone/key/disk/SSH
absence. The adapter still needs concrete resource evidence, bootstrap/guest
execution/receipt/return wiring, and outer recipe budgets that preserve cleanup
grace. Current produce/check remain blocked. Queue audit: no new agent/lane;
those integration steps and current coverage are next defensible local work.

### Concrete cleanup observation prerequisite

After the supervision checkpoint, added the narrow read-only `vm_cleanup.py`
observation and connected it after successful visual lifecycle cleanup. It
requires the owned Popen reaped, run key absent, clone directory absent, exact
clone absent from local Tart JSON inventory, exact disk image absent from
hdiutil's mounted-image plist and removed guest SSH unreachable. A stopped but
undeleted clone, unknown inventory, failed observation or missing validated IP
cannot emit `cleanup-observed.json`. No automatic stop/delete/retry, key read,
foreign PID signaling or other VM mutation is added. Failed/partial boot remains
unacknowledged and requires explicit investigation rather than invented absence.

Seven added contracts cover retained resources, malformed observations, unsafe IP
and observer failure after otherwise successful commands. `just visual-test`:
**86 Python units** + separate CLI integration, immutable upstream and 39 consumer
tests plus the 3 signal fixtures pass. Strict Clippy/fmt, roadmap and whitespace
pass. Logs: `/tmp/bbr-visual-vm-cleanup-visual-test.log`,
`/tmp/bbr-visual-vm-cleanup-clippy.log`,
`/tmp/bbr-visual-vm-cleanup-roadmap.log`. Self-review checked only exact-resource
absence claims, bounded read-only commands, process ownership and no success
marker after any failed observation.

Read-only `tart list --help`, local JSON inventory and hdiutil plist structure
were checked on the host; the golden image is stopped and no visual clone is
present. No port probe, key operation, disk mount or VM boot was performed.
These parser/fault-injection passes are not actual guest teardown evidence.
Remaining runnable work: adapter bootstrap/producer/receipt/return wiring,
independent host resource recheck, source-bound selection and coverage. No new
lane or subagent; pending VM/network and Checkbox asset approvals are unchanged.

### Fixed guest-execution prerequisite

Preflight: `main@1501246`, only this session's pending guest slice, no index or
capture-lock holder and no visual/Tart build or capture process. An ordinary
user-owned BBR app and an unrelated mbx process remain untouched. Entire delivery
queue re-read; only this local visual owner is authorized, no subagent or lane.

Both payload suites now require the separately built release consumer at its
compiled path. The internal Rust guest entrypoint checks actual admin/VirtualMac
identity, fixed suite/action, bounded hash-bound manifest, compiled paths and
installed source/executable bytes/modes before invoking the existing native
producer through immutable upstream process cleanup. Its cleared deterministic
environment has no arbitrary argv or host bypass. Postchecks precede inventory
publication. Python composes observation, fresh installation, fixed Rust calls,
rechecks, receipt and unchanged raw-byte return; it gives the owned Rust child
bounded termination grace before declaring hard-kill cleanup unproven. No seal,
comparison, baseline or environment acceptance authority moves into Python.

Evidence:
- **10 new Python guest contracts**, using synthetic non-executed artifacts and
  mocked producer/environment calls, plus **3 Rust pure preflight/command tests**.
  Both suites, receipt/return linkage, source/shared-manifest/environment drift,
  host refusal, fresh/disjoint output and TERM-before-KILL are covered. Rust
  adds exact flags, bounded manifest, hardlink/symlink/special-mode rejection and
  cancellation-aware installed hashing. Real host CLI rejects guest execution.
- `just visual-test`: **96 Python units** (one additional CLI class skipped in
  discovery, then separately passing), unchanged upstream contracts and **42
  consumer tests** plus existing signal fixtures pass. Strict consumer all-target
  Clippy, fmt, roadmap and whitespace pass. Logs:
  `/tmp/bbr-visual-guest-final-visual-test.log`,
  `/tmp/bbr-visual-guest-final-clippy.log`,
  `/tmp/bbr-visual-guest-final-roadmap.log`.
- Actual host-only preparation at `/private/tmp/bbr-visual-guest-prepare.usejuP`:
  Gallery 1,462 sources + 2 executables, 1,464 files / 43,714,894 bytes; browser
  1,462 sources + 3 executables, 1,465 files / 61,489,182 bytes. Both complete
  manifests independently pass the Python read-only inspector, and observed
  artifact hashes match host-build JSON. No payload was installed on the host.
  Consumer SHA-256: `04b8d26140a2042842dc856822edd03ac392d17f7735a058eb9b7231e9d608f7`.
  Product artifact hashes match the preceding host-build checkpoint. Gallery
  cached release build 1.04s, first consumer release build 9.55s; browser 0.38s,
  cached consumer 0.05s (Cargo-reported, not overall elapsed or product timing).
  Host remains macOS 27.0/26A428. Existing `block` warning is not suppressed.

Self-review tightened fresh inventory guards, bounded manifest reads, special
mode/hardlink checks, cancellation-aware hashing and failure diagnostics. These
contracts prove local composition and build/staging only, not actual guest
execution, producer pixels, full coverage or VM teardown. Old preparation
payloads without the consumer role are superseded diagnostics, never current
capture inputs. The new payloads are also build evidence, not capture seals.

Next runnable local work: host bootstrap/SSH/return wiring with independent
cleanup observation and supervision, then current Gallery coverage/selection.
The normal `produce`/`check` guard remains unconditional. VM/network and legitimate
Checkbox asset approval are still pending; no native execution, actual PNG/HTML
review, baseline acceptance, product/performance gate or migration completion
is claimed. Implementation/review wall time was not measured.

### Bound independent host-cleanup checkpoint

Preflight: clean `main@ade418b`, no Git/capture lock holder or visual/Tart process;
ordinary development app and unrelated mbx processes remain untouched. Full active
delivery document re-read. Only this local visual writer is authorized; no new
lane, subagent or product scope. Portable transport integration remains runnable.

Before connecting bootstrap, closed the supervisor's concrete cleanup-verifier
gap. Optional bound lifecycle records now name the independently expected clone,
request hash and payload hash before boot, then actual IP/Popen identity before
producer access. Completion is atomic/no-replace and follows owned cleanup and
resource observations. The Rust host verifier validates that binding and repeats
clone/key/disk/SSH absence plus read-only PID existence checks before either lease
finishes. A present/reused PID, permission ambiguity, stale/missing/changed record
or rejected observation keeps the gate unfinished; no numeric-PID termination,
resource repair, automatic retry or key-content read. Partial boot remains
unproven rather than manufacturing cleanup evidence.

Self-review found that a callback could rediscover its expected request identity
from a child-writable record. The supervisor now passes its original in-memory
digest and checks the final request against it; a post-acknowledgement mutation
regression preserves the unfinished gate. Shared Tart execution/cleanup defaults
and immutable upstream remain unchanged. These records establish integrity and
ordering, not authentication against malicious same-user processes.

Fresh evidence:
- Seven added Python contracts cover bound lifecycle ordering, separate resource
  observations, stale/swapped/missing/changed records, partial boot, exact fields,
  links/size bounds, no overwrite, PID reuse/permission ambiguity and no signals
  other than existence-only signal 0 (mocked, never actual Tart/socket activity).
- Three new Rust contracts validate every identity/absence field, actual verifier
  no-record refusal before resource commands, and end-to-end supervisor/record/
  verifier wiring with isolated observation stubs. Both private leases are asserted
  held in child and host verifiers; successful/failed capture with proven cleanup
  releases them, host rejection retains active evidence. Existing real local
  cancellation fixtures still pass; no VM or native producer is involved.
- `just visual-test`: **103 Python units**, separate real-CLI integration,
  unchanged upstream contracts and **45 consumer tests** plus signal fixtures
  pass. Strict consumer all-target Clippy/fmt, roadmap and whitespace pass.
  Logs: `/tmp/bbr-visual-host-cleanup-final-visual-test.log`,
  `/tmp/bbr-visual-host-cleanup-final-clippy.log`,
  `/tmp/bbr-visual-host-cleanup-final-roadmap.log`.
  An initial expanded negative fixture expected the wrong exception type for an
  invalid IP; the evidence boundary now wraps policy errors consistently. The
  rejection itself was never removed. Earlier diagnostic log:
  `/tmp/bbr-visual-host-cleanup-python2.log`.

No actual network probe, key operation, VM boot/teardown, image transfer, baseline
acceptance, GUI gate or performance claim. Source changes supersede previous
payloads for future captures. Next local work remains source-bound bootstrap,
SSH/return integration and Gallery selection/independent coverage. Real execution
stays guarded pending the approved route/pilot and network/Checkbox permissions.
Parallelism audit: no other lane is authorized; those portable actions remain
runnable. Implementation/review elapsed time was not measured; test durations
are present in logs, not product-performance evidence.

### Source-bound guest-runtime prerequisite

Preflight: clean `main@b037b54`, no Git lock or visual/Tart process; full active
delivery contract read. This session remains the only authorized local visual
writer; foreign worktrees, ordinary development app and unrelated builds are
untouched. No workflow/subagent/new lane is authorized. Bootstrap integration and
coverage are runnable; actual boot and Checkbox asset approval remain pending.

Host inspection found the copied consumer/Gallery/browser link to Nix libiconv,
and neither system nor per-user profile exports Python. Pinned the already
installed Darwin arm64 Python 3.13.11 and its entire observed Nix closure, including
libiconv, in a source-bound JSON lock: **51 paths / 1,492,264,592 NAR bytes**. The
closure includes upstream Python toolchain references; no reduced/guessed runtime
or package install. Nix query/verification are offline/read-only, not realization,
repair, substitution, profile changes or guest availability evidence.

`visual-prepare` now requires staged thin-arm64 Mach-O dependency validation,
exact lock/closure metadata, actual NAR-byte verification, isolated interpreter
hash/version/architecture/standard-library checks, and postchecks before it
returns build evidence. Unknown dylib origins/store paths, incomplete or drifted
closure, changed payload/lock and failed probes cannot pass. Rust owns the bounded
probe group through upstream; Python read-only children do not detach. No host
producer, native capture, seal or baseline path is enabled.

Evidence:
- **9 new Python contracts** and **2 Rust contracts** cover strict closure graph,
  hashes/size/types, bad Mach-O records/origins, architecture and byte failures,
  pre/post metadata drift, source-bound lock/payload/executable identity, output/
  deadline bounds and no detached probe child. Initial red was the missing module;
  focused fixtures pass without native/Nix/VM operations. Real direct `/bin/sleep`
  timeout and inert `/usr/bin/printf`/`false` probes are local-only test fixtures.
- `just visual-test`: **112 Python units** plus the separately run real-CLI
  integration, unchanged upstream contracts and **47 consumer tests** plus signal
  fixtures pass. Strict consumer all-target Clippy and fmt pass. Logs:
  `/tmp/bbr-visual-runtime-focused-final.log`, `/tmp/bbr-visual-runtime-rust.log`,
  `/tmp/bbr-visual-runtime-final-visual-test.log`,
  `/tmp/bbr-visual-runtime-final-clippy.log`. Roadmap, whitespace and the 49 shared
  script contracts also pass (`/tmp/bbr-visual-runtime-final-roadmap.log`,
  `/tmp/bbr-visual-runtime-final-script-tests.log`).
- Actual host-only integrated preparations at
  `/private/tmp/bbr-visual-runtime-integrated.fVYUtf`: Gallery 1,471 sources + 2
  executables (1,473 files / 43,859,980 bytes), browser 1,471 + 3 (1,474 files /
  61,634,268 bytes). Both preparation commands pass the new mandatory runtime
  check. Gallery manifest `09e43dddc13584c9eeee795809bcbe80e26cba854267d9122ea0a7e001b5ee75`;
  browser `5c56bd151d8b7f7fe570b7133efac2d16ff7ceb8582d1f5176bde4130215d64b`.
  Host-build JSON and prepare logs are alongside the payloads. No payload installed
  on the host and no producer executed. Existing `block` warning remains.
- Earlier read-only full runtime observations on predecessor payloads:
  `/tmp/bbr-visual-runtime-actual.log`, 5.661s Gallery / 2.578s browser measured.
  Those timings are prerequisite tooling, not capture/product performance.
  Stale staging at `/private/tmp/bbr-visual-runtime-prepare.yvMS7d` is diagnostic.

Self-review corrected inherited Python test-class discovery, strict metadata type
validation, and ensured probe children stay in the upstream-owned group rather
than escape on cancellation. Full runtime checks succeeded on actual host bytes;
no VM, key, network probe, image return, visual review or product gate occurred.
Documentation changes do not make these payloads capture seals; all subsequent
source changes require fresh preparation. Next remains source-bound bootstrap/
SSH/return wiring and independent Gallery coverage/selection. No migration or
baseline acceptance claim; performance remains deferred. Implementation/review
wall time was not instrumented. Parallelism audit: no additional lane authorized.

### Source-bound guest bootstrap checkpoint

Preflight: clean `main@55f13a9`, no Git lock or visual/Tart process; ROADMAP and
complete delivery document re-read. Only this session's bounded visual work is
runnable here. Other product lanes, worktrees, ordinary app and unrelated builds
remain untouched; no workflow/subagent/new lane authorized. Actual VM/network
and Checkbox asset approvals remain pending, not bypassed.

Implemented pure/read-only bootstrap generation from the verified staged payload
and current helper bytes. Generated zsh uses fixed destinations/commands, a cleared
UTC/C environment and actual UID/admin/VirtualMac checks before mutation. It hashes
the complete pinned NAR closure offline and the interpreter executable, then
mounts the existing share read-only. No download/realization/repair/fallback. The
inline isolated loader validates all helper hashes before executing any helper,
compiles them in memory and intercepts only the exact helper module names. It
never imports mutable source filenames or adds the share to `sys.path`.

Verified guest tail repeats actual guest identity, preflights the payload and
creates only a fresh compiled-path ancestor with nonrecursive bounded commands.
Existing paths fail without takeover; partial preparation is never resumed. The
existing guest installer/receipt chain and Rust native-process owner remain
unchanged. This is still a prerequisite, not connected SSH/return execution.

Evidence:
- **16 new Python contracts**: fixed generation and shell syntax; actual local
  host refusal before runtime/mount/mutations; stale/missing/linked helpers and
  payload rejection; writable mount refusal; verify-all-before-execute; real
  helper imports followed by host refusal; frozen-byte imports after shared file
  mutation; strict parent ordering/no recursive ownership changes; shell heredoc
  delimiter collision rejection. Mocks are limited to local fixtures/resource
  calls; no native producer, Nix install, key, mount, SSH or VM ran.
- `just visual-test`: **128 Python units** plus separate real-CLI integration,
  unchanged upstream and **47 consumer tests** plus signal fixtures pass. Strict
  all-target consumer Clippy/fmt pass. Logs:
  `/tmp/bbr-visual-bootstrap-focused-final.log`,
  `/tmp/bbr-visual-bootstrap-final-visual-test.log`,
  `/tmp/bbr-visual-bootstrap-final-clippy.log`. Roadmap, whitespace and all 49
  shared script contracts also pass (`/tmp/bbr-visual-bootstrap-final-roadmap.log`,
  `/tmp/bbr-visual-bootstrap-final-script-tests.log`). Initial test red was the
  missing generator module (`/tmp/bbr-visual-bootstrap-red.log`).
- Actual host-only two-suite preparation and read-only plan generation at
  `/private/tmp/bbr-visual-bootstrap-prepare.h97eML`: 1,475 sources, Gallery 1,477
  files / 43,884,287 bytes, browser 1,478 / 61,658,575 bytes. Runtime prerequisite
  checks pass. Generated 17,631-byte per-suite scripts pass actual `zsh -n`; they
  were not executed or sent over SSH. Exact manifests/script hashes and counts:
  `/tmp/bbr-visual-bootstrap-actual-plan.log`. Additional test and delimiter-guard
  edits afterward supersede these payloads; they remain diagnostic, not seals.

Self-review added observed read-only mount enforcement, verification-before-any-
import, source-freezing, current planner/runtime-source binding, disjoint work
preflight and shell delimiter rejection. Required next steps remain supervised
SSH/return integration (including unavailable pinned guest runtime handling),
Gallery independent coverage/selection and approved real pilot. No guest/NAR
availability, mounting, pixels, cleanup, visual review, baseline or product gate
is proven by these local tests. Performance remains deferred; implementation/
review wall time was not instrumented. Parallelism audit: no other lane authorized;
portable transport and coverage work remain runnable.

### Supervised transport/return integration checkpoint

Preflight: clean `main@b8ec8d54`, no Git lock or visual/Tart process. Current
activation/queue and bootstrap checkpoint reviewed; only this local visual owner
is authorized. No subagent/workflow/new lane or foreign worktree activity. Normal
capture remains closed and actual network/Checkbox permissions remain pending.

The internal Rust transport now composes current-source/runtime/host-build checks,
upstream supervision, fixed Python bootstrap/lifecycle/SSH exchange, bounded raw
archive return, request-bound cleanup and independent host recheck, then returned
file/guest-receipt validation. Native process, comparison and seal authority stay
upstream/Rust. Python never detaches its direct SSH child or infers remote cleanup
from SSH exit; the owned clone must be destroyed and observed absent. SSH argv is
the exact isolated lifecycle route with absolute system executable and cleared
local environment; returned archive path is fixed. No retry or runtime install.

Archive/payload preflight precedes teardown; resource verification precedes raw
unpacking and terminal publication. Rust repeats the original request-bound
cleanup observation before finishing leases, checks exact returned bytes/marker,
and parses the same bounded hash-verified receipt bytes. Source changes, swapped
receipt/build identity, retained resources and uncertain termination prevent
success. Receipt binding now also requires an actual VirtualMac model rather
than accepting a host model tagged as guest. No CLI caller activates this route;
`produce`/`check` remain unconditionally blocked.

Fresh evidence:
- **13 new Python contracts** exercise success/failure/cancellation ordering,
  truncated/swapped archive, payload drift, rejected resource observations,
  isolated/fixed SSH argv, direct-child TERM-before-KILL, no detached group,
  fresh diagnostics and bounded actual local cat/printf/false/sleep fixtures.
- **3 Rust contracts**: returned inventory/bytes/receipt rejection plus real
  supervisor → Python transport → raw transfer → host verifier → receipt binding
  composition. Fixture modules replace only hardware/runtime/lifecycle/SSH
  boundaries. Both private leases are asserted held during child and host
  observations. Operation failure with proven cleanup releases gates; host
  rejection leaves the gate active; swapped guest receipt fails after cleanup.
  Synthetic bytes are explicitly NOT PNG and cannot prove pixel/coverage parity.
- `just visual-test`: **141 Python units** (one integration skip rerun separately),
  unchanged upstream and **50 consumer tests** plus signal fixtures pass. Strict
  all-target consumer Clippy, fmt, whitespace and 49 shared script tests pass.
  Logs: `/tmp/bbr-visual-transport-final-visual-test.log`,
  `/tmp/bbr-visual-transport-final-clippy.log`,
  `/tmp/bbr-visual-transport-final-script-tests.log`,
  `/tmp/bbr-visual-transport-chain.log`. Initial Clippy rejected an eight-argument
  entrypoint; it now takes a typed input bundle without lint suppression.

Self-review tightened independent build matching, bounded receipt hash-before-
parse, VirtualMac validation, SSH environment inheritance and fixed return path.
No real key/SSH/Tart/network/native producer, runtime install, returned PNG, visual
review, baseline or product-performance gate ran. Bootstrap runtime availability
in the golden image and actual transport/teardown remain unproven; sources require
fresh preparation before any approved capture. Next runnable work is current
Gallery story selection/independent coverage and guarded producer integration;
real pilot still needs the recorded network and asset approvals. Performance
remains deferred. Test durations are logged; implementation/review timing was not
instrumented. Parallelism audit: no additional lane authorized.

### Current Gallery source-audit checkpoint

Preflight: transport slice committed clean as `b9228dce`; no active visual/Tart
process or other authorized main writer. ROADMAP/current queue were re-read at
continuation; only this local owner may work on the visual migration. No additional
lane, workflow, subagent or foreign worktree inspection is authorized. Current
source audit/coverage diagnostics are runnable while real VM/network and Checkbox
asset approvals remain pending. Stable functional/performance gates are not
claimed by this non-rendering prerequisite.

The [bounded source audit](../visual/gallery-source-audit.md) records the pilot
and four added stories directly against current story and production sources.
The historical 49-story plan/936 required page pairs stay unchanged. Four explicit
withheld entries now name IDs 49–52 (Delete Space, Peek, Toast/zoom, Media card),
without guessing their page counts or calling missing work inapplicable.
Read-only `doctor --json` now exposes all eight pending rows. This does not
complete the remaining stories' applicability audit or enable selected capture.

Source findings: one numeric producer filter already precedes context creation,
but upstream multi-story selection/request binding is not connected. Canvas
settings do not necessarily equal family density/contrast. Checkbox captures no
pointer transition and its checked asset approval is still pending. Direct-child
framing is not nested/shadow proof; Sidebar's app slots are labeled placeholders
and its dense model is windowed. Peek paints stand-in pages/empty portal bar,
never native page/portal proof. Toast entrance fades survive Reduced Motion but
story 51 currently has zero settling duration. Media cards have six static
fragments/no-op callbacks. None of these findings invents runtime/pixel evidence
or authorizes production changes.

Evidence: new missing-blocker contract fails on the unchanged catalog, then passes
with the four authored entries. Existing 936 required pairs and original blockers
are preserved. Actual read-only CLI/tripwire tests prove the added blockers remain
visible without builds/producers/environment probes/gates/baseline writes.
`just visual-test`: 141 Python tests (one integration skip rerun separately),
unchanged upstream and **51 consumer tests** plus signal fixtures pass. Strict
all-target consumer Clippy, fmt and roadmap pass. Logs:
`/tmp/bbr-visual-gallery-audit-red.log`,
`/tmp/bbr-visual-gallery-audit-contracts.log`,
`/tmp/bbr-visual-gallery-audit-clippy.log`,
`/tmp/bbr-visual-gallery-audit-visual-test.log`,
`/tmp/bbr-visual-gallery-audit-doctor.json`,
`/tmp/bbr-visual-gallery-audit-roadmap2.log`.
Initial roadmap validation rejected an evidence note under the task-packet path
and an absolute Markdown link; the note moved to the evidence directory and uses
a durable repository link. No validator relaxation. Self-review checked direct
source support, catalog-only additions, unchanged page requirements and truthful
static/native exclusions. No Gallery/product/upstream/baseline edits, VM/native
execution, runtime installation or visual review. Test timings are logged; audit
elapsed time was not measured. Next local work: remaining applicability audit
and pre-render selection contracts; real pilot needs the recorded approvals.

### Retained adapter cleanup regression checkpoint

Preflight: selection contract work is owned locally on `main@dcd05f6c`; no Git
lock, active visual/Tart process or other main writer. Full delivery queue read.
No subagent, new lane, foreign worktree or native operation is authorized. The
portable selection bridge remains a separate pending slice; native capture is
still unconditionally guarded.

The selected-plan full gate exposed the retained Python adapter failing during
`test_timeout_kills_descendants`: after successful TERM, signal 0 returned Darwin
EPERM. Three instrumented isolated repetitions reproduced it. A bounded owned
process-group observation then showed only the zombie leader at EPERM, followed
by ESRCH after reaping. No surviving live fixture process was found. The immutable
Rust executor already documents this Darwin state; it is unchanged. Failure and
diagnostic logs remain `/tmp/bbr-visual-selection-plan-visual-test-failed-cleanup.log`,
`/tmp/bbr-visual-selection-plan-cleanup-diagnostic.log` and
`/tmp/bbr-visual-selection-plan-group-observation.log`.

The narrow Python repair treats EPERM as uncertainty, never absence: poll/reap the
owned direct child and observe signal 0 within the original cleanup deadline.
Success requires ESRCH and a reaped leader; persistent permission ambiguity fails
cleanup, without sending another termination signal to the uncertain group. Four
injected contracts cover transient/persistent EPERM, a still-present group and
unreaped leader. Initial red contracts and the real failure are preserved. One
first test correction fixed an exhausted mocked clock, not a cleanup predicate.
Three real timeout/descendant repetitions plus the existing TERM-resistant
lock-lifetime fixture pass (eight focused tests, 7.240s measured).

Evidence: `/tmp/bbr-visual-legacy-cleanup-red.log`,
`/tmp/bbr-visual-legacy-cleanup-focused-final.log` and
`/tmp/bbr-visual-selection-plan-visual-test-repaired.log`. The combined candidate's
`just visual-test` passes 144 Python discovery tests (one further integration
skipped, then run separately and passed), unchanged upstream and 58 consumer
tests plus signal fixtures. Strict consumer Clippy/fmt, 49 shared script tests,
roadmap and whitespace pass. These combined-tree counts include the pending
seven selected-plan contracts; they are not a claim that this cleanup-only commit
contains the bridge. Additional logs: `/tmp/bbr-visual-selection-plan-clippy-repaired.log`,
`/tmp/bbr-visual-selection-plan-script-tests.log`,
`/tmp/bbr-visual-selection-plan-roadmap-repaired.log`.

Self-review checked that only observation uncertainty is retried, with no deadline
extension, swallowed persistent error or change to the descendant/lock assertions.
No VM, SSH, native producer, product E2E/performance or baseline action occurred.
The retained host producer remains prohibited. Parallelism audit: only the local
selection/request integration and remaining coverage audit are runnable here;
network and Checkbox approvals remain pending. Test timings are measured tooling
evidence, not product performance; implementation/review timing was not recorded.

### Gallery selected-plan bridge checkpoint

The preceding cleanup repair is committed separately as `a9caed00`. This local
slice adds an immutable portable `Schedule` connecting upstream story Selection
to the independent Gallery page plan, not a producer or another coverage source.
Both files are bounded, single-link, no-follow, exact-byte hash-bound inputs;
caller-refreshed hashes cannot silently adopt a changed plan after scheduling.
Canonical story/state identities, authored story order, complete planned state/
contrast/density/page requirements and asymmetric Light/Dark child ranges survive
selection. Unknown/partial/reordered/cross-story/sequence requirements fail.

Before receipt publication, existing Gallery validation checks the narrowed plan
against complete raw inventory/coverage and actual RGBA8 dimensions, with no
pixel transformations. New output checks reject links. Source checks bracket
validation; only the exact upstream selection receipt is added. Upstream retains
schema-2 sealing, partial HTML labels and nonacceptance, even when every story
was explicitly selected. These are synthetic portable PNG fixtures, not native
captures or actual PNG/HTML review. The seven added contracts exercise schedule
order, source/plan drift and rebinding, valid-byte linked requirements, incomplete/
extra/ignored output, no-write failures and unchanged pixel bytes. Self-review
added positive-hash hardlink rejection and noncontiguous/metadata-mismatch plan
cases. Direct upstream acceptance rejects with `development-only` with or without
adoption; Collection comparison records `not full-suite collection evidence`.
The actual consumer update router rejects the unusable Collection without writing
baselines (it has no suite review directory); its generic exit alone is not a
selection-specific rejection claim.

Fresh focused seven-contract test and strict Clippy pass after those review
additions: `/tmp/bbr-visual-selection-plan-focused-review.log`,
`/tmp/bbr-visual-selection-plan-clippy-review.log`. Full final combined gate logs
are `/tmp/bbr-visual-selection-plan-final-visual-test.log` and
`/tmp/bbr-visual-selection-plan-final-roadmap.log`: 144 Python discovery tests
pass (one additional integration skip, then separately passed), unchanged
upstream and **58 consumer tests** plus signal fixtures pass. Consumer formatting,
strict all-target Clippy, whitespace and 49 shared script tests pass. Initial
red was the missing module; the first upstream integration fixture lacked the
required explicit `layer=gpui` environment and was corrected without changing
production validation. The cleanup failure/reproduction/repair is preserved in
the preceding checkpoint. Focused test time 0.92s; implementation/review elapsed
time was not measured. No product-performance evidence.

Full queue and ownership audit: same sole local visual writer, no subagent/lane
or foreign worktree access. No real story catalog enabled, no guest/producer
caller connected, no current page-count invention, and no normal capture guard
removed. Gallery/product/upstream/baselines are unchanged. Next runnable local
step is source-bound guest request → existing numeric producer selection before
rendering, plus the remaining applicability/framing audit. Actual runtime and
network/Checkbox approvals, real pilot, two complete independent captures,
image/report review and current functional Tart gates remain required. These
prerequisites do not complete any migration milestone or authorize acceptance.

### Selected guest execution checkpoint

Preflight: clean `main@9182cef4`, no Git lock, visual/Tart process or capture-lock
holder. ROADMAP and the complete delivery queue re-read. This session remains
the only authorized local visual writer; no agents, new lanes or foreign worktree
access. Request/producer integration is runnable locally, while VM/network and
Checkbox asset approvals still remain external boundaries.

The internal guest entrypoint now accepts Gallery-only `capture-selected` with
an explicit independent hash for fixed `selected-request.json`. It binds that
request to the verified payload and source-hashed config's explicit story catalog,
then resolves the independent page schedule before invoking a producer. Existing
actual admin/VirtualMac identity and installed/shared bytes checks remain before
this path; no host bypass or real config activation. The non-rendering inventory
must match all selected-story states/appearances before the first capture. One
fixed numeric `--story` command follows per selected story, in authored order,
with the same cleared environment and immutable upstream executor. The existing
Gallery filter already precedes renderer-context creation and is unchanged.

The overall 600s deadline spans all calls; preflight time reduces the remaining
native timeout. Installed/shared bytes are checked around each call and again
before returning. Per-story and aggregate validation reuse the extracted,
behavior-preserving raw phase of the existing Gallery validator. Complete raw
PNG bytes move without rewriting/resampling; selected receipt follows validation
and source checks, never a capture seal. Partial output remains diagnostics only.
Host transport does not yet convey selected requests, nor validate the returned
selection receipt against an independently retained host request. Those are the
next integration requirements, not capabilities claimed by this checkpoint.

Evidence: five new injected-command contracts (eight guest unit contracts total)
cover ordered explicit argv before synthetic output, no unselected rendering call,
unchanged PNG hashes, payload/config/request identity, unsupported/oversized/linked
requests, pre-cancelled/stale/preexisting-output no-invocation, extra selected state,
duplicate inventory, ignored selection, missing/extra/hardlinked/wrong-size output,
nonzero producer status and cancellation/source drift before and after capture.
Actual host CLI refusal is extended to the selected action. Initial red was the
missing module, `/tmp/bbr-visual-selected-guest-red.log`. Final focused and strict
Clippy logs: `/tmp/bbr-visual-selected-guest-focused-final.log`,
`/tmp/bbr-visual-selected-guest-clippy-final.log`. `just visual-test` passes 144
Python discovery tests (one further integration skipped, then separately passed),
unchanged upstream and **63 consumer tests** plus signal fixtures;
`/tmp/bbr-visual-selected-guest-final-visual-test.log`. Formatting, roadmap and
whitespace checks pass (`/tmp/bbr-visual-selected-guest-final-roadmap.log`).
Focused guest tests run in 0.76s in the full gate; timing is tooling evidence only.

Self-review tightened extra-state rejection (not merely matching known rows),
post-capture source/cancellation boundaries, no direct selected command without a
validated schedule, hardlink/extra output rejection and deadline subtraction for
source preflight. No Gallery/product/upstream/baseline edits, VM/SSH/native process,
actual image/report review, functional GUI gate or product-performance run. The
host capture guard stays unconditional and source edits require fresh future
payloads/captures. Parallelism audit: only local selected-request transport/return
integration and remaining coverage audit are runnable here; no additional lane
is authorized. Migration acceptance, real pilot/full repeat and approvals remain
open; implementation/review wall time was not instrumented.

### Selected host-request transport checkpoint

Preflight: clean `main@d2988015`, no Git index lock, visual/Tart capture process
or observed capture-lock holder. ROADMAP and all of this delivery document were
re-read. Sole visual writer and scope remain unchanged; no workflow/subagent,
foreign worktree access or extra product lane is authorized. Local request/return
integration is runnable; VM/network and Checkbox asset approvals remain pending.

The internal host transport now retains an immutable source/config/plan-bound
selected request and its exact bytes/hash. Guest and host reuse one Rust request
validator. Only the fixed request path travels through the existing source-frozen
bootstrap; Python verifies bounded bytes/hash/payload but never owns coverage or
selection resolution. The guest invokes only the existing selected action with
the independent hash. The host rechecks its original request after transport and
requires the exact serialized selection receipt from bounded, transport-hashed
bytes. Missing/swapped receipts and unexpected selection on a full run fail;
the typed capture retains selected scope instead of implying full-suite evidence.
No seal, image rewrite, normal CLI caller or real catalog is enabled.

Evidence: two new Rust request contracts, seven Python transport/bootstrap/guest
contracts and extended cross-language supervisor/return fixtures. They cover
source/config/payload/hash binding, request bounds/links, exact quoted forwarding,
frozen-loader arguments, no separate full capture, request mutation, missing or
swapped selection, full-run injection, cancellation and cleanup rejection. Both
leases remain held during child/host cleanup observations. Proven-clean failures
release them; rejected cleanup retains active evidence. Actual changed-request
fixture permissions are deliberately made writable before mutation so the test
proves hash rejection rather than only a permission error. The frozen loader also
rejects a null selected binding rather than silently treating it as full capture.

Initial red was the missing Rust module (`/tmp/bbr-visual-selected-transport-red.log`).
Initial focused fixtures then exposed a missing mandatory config source policy
and an incorrect Python test module reference; the fixtures were corrected without
relaxing production checks. Focused logs:
`/tmp/bbr-visual-selected-transport-focused3.log`,
`/tmp/bbr-visual-selected-transport-python2.log` and
`/tmp/bbr-visual-selected-transport-python-all.log` (105 VM-helper tests, 1.220s).
Full candidate `just visual-test` passes **151 Python discovery tests** (one extra
integration skipped and then separately passed), unchanged upstream and **65 Rust
consumer tests** plus signal fixtures. Full gate:
`/tmp/bbr-visual-selected-transport-visual-test.log`; final post-review recheck:
`/tmp/bbr-visual-selected-transport-final-visual-test.log`. Strict consumer
all-target Clippy/fmt, roadmap and whitespace are checked before commit. The
cross-language 12-mode chain took 5.53s in the candidate full gate; these timings
are tooling evidence, not product performance. Implementation/review elapsed
time was not instrumented.

Self-review checked unchanged guest/native identity boundaries, source-frozen
helper membership, bounded shell input, original host expectation retention,
exact return receipt checks and unchanged cleanup semantics. Product/renderers,
immutable upstream, configs, baseline files and GPUI pin are untouched. No key,
SSH, Tart, native producer, actual image/report review, GUI gate or performance
run occurred. Diagnostic raw staging may remain after receipt rejection, never a
capture seal. All source edits require fresh future preparation/captures.

Parallelism audit: only local raw-return preparation into upstream selected/full
sealing and the remaining applicability/framing audit are next runnable work;
no other lane is authorized. Real runtime availability, network/Checkbox approval,
current catalog activation, approved pilot, independent complete repeats, actual
PNG/HTML review and current functional Tart gates remain open. Migration and
baseline acceptance are not complete; retained Python entrypoints stay preserved.

### Fresh raw-return to upstream-seal checkpoint

Preflight: `main@1b0abd5b`, only this session's pending raw-to-seal slice, no Git
index lock or visual/Tart process. ROADMAP and the entire delivery document were
re-read. Ordinary user BBR and unrelated mbx processes remain untouched. Sole
local visual writer; no agents, workflows, foreign worktree access or extra lane.
The same VM/network and Checkbox permissions remain pending.

The internal `vm_capture` route invokes immutable upstream `capture_local` so its
source snapshot precedes one fresh supervised transport. Both local environment
callbacks use validated guest before/after receipt facts, not host probes. The
original return bytes and selected request remain host expectations throughout
copying, structural preparation and the final callback. Gallery reuses the full
independent plan or exact selected schedule; browser shares the retained eight-
image validator. PNGs are copied and renamed, never decoded/re-encoded for output.
Original guest receipt/live registry bytes remain ancillary evidence; the raw
transport marker is not rewritten to pretend it describes canonicalized names.
Upstream alone writes `capture.json`, compares and enforces partial nonacceptance.

Fresh run/workspace guards prevent old-directory resealing. Bounded single-link,
no-follow copies check exact sizes/hashes and cancellation/deadlines. Original
return and prepared output are rechecked before sealing; unexpected metadata,
injected seals/inventory/sequences, invalid images/coverage, changed receipts,
source/payload/environment/scope and cancellation cannot seal. Failed diagnostics
remain without automatic retry, report generation or baseline writes. Normal
capture/produce remains blocked, and no real catalog is activated.

Evidence:
- Four new parameterized portable contracts generate fresh synthetic RGBA8 PNGs
  for selected/full Gallery and all eight browser images, then use actual upstream
  seal/compare/HTML/accept APIs. They prove unchanged image/receipt hashes, exact
  guest/source identity, schema-2 `TEILLAUF` and selected acceptance refusal.
  They also exercise 24 invalid/drifted-return variants with specific rejection
  messages, pre-cancelled/stale no-invocation, no transport retry, copy expiry/hash
  rejection and mutation of already prepared PNG/inventory/receipt/selection.
- Initial red: missing module (`/tmp/bbr-visual-return-seal-red.log`). Focused four
  contracts pass in 6.21s (`/tmp/bbr-visual-return-seal-focused-final.log`); rejection
  reason recheck passes (`/tmp/bbr-visual-return-seal-reason-check.log`). Full gate
  `/tmp/bbr-visual-return-seal-final-visual-test.log` passes **151 Python discovery
  tests** (one extra integration skip, separately passed), unchanged upstream and
  **69 consumer tests** plus signal fixtures. Consumer unit group: 6.34s.
- Post-review full gate and strict consumer all-target Clippy pass:
  `/tmp/bbr-visual-return-seal-reviewed-visual-test.log`,
  `/tmp/bbr-visual-return-seal-reviewed-clippy.log`. Formatting, roadmap, all
  49 shared script contracts and whitespace pass; logs:
  `/tmp/bbr-visual-return-seal-roadmap.log`,
  `/tmp/bbr-visual-return-seal-script-tests.log` (17.845s).
  Timings measure tooling tests only; implementation/review elapsed time was not
  instrumented. No product-performance or native runtime evidence is inferred.

Self-review checked upstream callback/seal ordering, original source/config/plan
and byte retention, exact canonical inventory, freshness, refusal boundaries and
unchanged product/upstream/baselines. Negative tests assert their intended failure
rather than incidental rejection. Ancillary receipt files are not new upstream-
sealed manifest fields and must not be advertised as same-user authentication.
No key, SSH, Tart, native producer, actual PNG/HTML visual review, GUI gate or
baseline/environment adoption occurred. Retained Python entrypoints stay intact;
source edits require fresh future payloads and captures.

Parallelism audit: remaining applicability/state/framing source audit is locally
runnable; no other lane is authorized. Actual guest runtime, network and Checkbox
approval, current catalog, real pilot, two independent complete captures, actual
image/report review and current functional Tart gates remain open. This internal
bridge is prerequisite evidence, not migration completion or parity acceptance.

### Foundations / Primitives / Core Controls source-audit checkpoint

Preflight: clean `main@4a5fedf9` after the raw-to-seal commit, same sole local
visual owner and no new lane/subagent authorization. Current full queue and
ownership were reviewed in the preceding checkpoint; only bounded visual source
work is runnable here. No native/VM/network/asset operation is activated.

The [source continuation](../visual/gallery-foundations-audit.md) inspects IDs
0–20 against actual story, fixture, token/component and vendored-control symbols.
It separates represented samples from narrow non-applicability and still-open
states/framing. Findings: States' eleven names collapse to monogram/Loading/Error
favicon branches; Iconography lists only 25 samples and omits newer production
variants; Core Controls expose initial recorder/entities with separate vendored
Light/Dark but no explicit BBR contrast/density propagation. The initially checked
Radio already requires the same missing `icons/check.svg` as the Checkbox pilot.
The checked-icon transition also owns a 250ms animation/timer; no source-only
claim of a settled real endpoint follows. Direct-child bounds do not prove nested
text, glyph, focus-ring or painted shadow extents.

Four additive withheld catalog entries expose those concrete gaps through the
existing read-only doctor path. Every one of the prior catalog rows is byte-value
preserved, including all 936 required page pairs and eight prior blockers; no
page/moment invention, required-row deletion or unsupported-state relabeling.
The earlier source note now distinguishes the implemented internal selected path
from still-unproven guest/native parity. Renderer, asset, dependency, upstream,
normal configs and baseline sources are unchanged.

Evidence: the added missing-blocker contract fails before catalog edits
(`/tmp/bbr-visual-foundation-audit-red.log`), then all nine catalog/validator and
four discovery contracts pass (`/tmp/bbr-visual-foundation-audit-focused.log`,
0.79s/0.81s). Actual CLI tripwires still prohibit builds, producers, probes and
writes. The first roadmap check correctly rejected the new untracked linked
report; staging the owned report makes it visible, without validator changes.
Full `just visual-test` passes 151 Python discovery tests plus separate CLI
integration, unchanged upstream and **70 consumer tests** plus signal fixtures:
`/tmp/bbr-visual-foundation-audit-visual-test.log`. Strict consumer all-target
Clippy/fmt, roadmap and whitespace pass:
`/tmp/bbr-visual-foundation-audit-clippy.log`,
`/tmp/bbr-visual-foundation-audit-roadmap2.log`. An independent structured diff
confirms all old catalog rows unchanged and exactly four additive withheld rows.
No actual image/HTML review, guest or functional gate was performed. These are
source/contract observations, not product-performance or visual acceptance.

Self-review traces each finding to actual call sites, checks default recorder and
entity values, compares the literal icon list with the production enum, and
preserves original catalog entries. Parallelism audit: IDs 21–48 and remaining
matrix/composition applicability are locally runnable, no other lane authorized.
Actual network/asset permissions, runtime availability, legitimate endpoint and
nested framing, fresh pilot/full repeats and current functional Tart gates remain
required. The goal and Gallery acceptance stay open.

### Status / Tab Row / Library source-audit checkpoint

Preflight: clean `main@7cc720cc`, no index lock, visual/Tart process or observed
capture-lock holder. ROADMAP and this complete delivery contract were re-read.
Ordinary user BBR and unrelated mbx processes remain untouched. Sole local visual
writer; no workflow/subagent, foreign worktree or extra product lane authorized.
IDs 21–48 were audited as runnable source work; the concrete 21–30 findings form
this smaller reviewable slice. Preliminary reads of 31–34 are not a completed
remaining-family audit. VM/network and asset approvals remain pending.

The [source continuation](../visual/gallery-library-audit.md) traces IDs 21–30
through actual builders, production branches, recorder and semantic-test seams.
Tab Row still rejects bitmap favicons despite the current production image/fallback
path; every fixture omits the split mark, and the focused label has no dedicated
ring/input path. This records gaps only, without activating deferred Split work.
Library selection/multi set row flags but leave the separate selection set empty,
so the production selection toolbar is absent. Move sheet and collection stages
are never supplied. Recorder dispatch does not mutate the fixture models. The
large-1k model supplies only rows 1–24 inside a 520pt nested scroll viewport;
outer pagination cannot establish complete descendant visibility or full-list
coverage. No runtime clipping defect or native pixels are inferred from source.

Three additive withheld rows expose these concrete gaps; all prior rows, including
936 historical required page pairs and twelve existing blockers, are preserved
exactly as JSON values. No page/sequence/moment invention, old exclusion promotion,
renderer/fixture/product/vendor/asset/config/baseline change or catalog activation.
The canonical Gallery packet records the bounded scope before its implementation.

Evidence: the new missing-blocker contract fails before catalog edits
(`/tmp/bbr-visual-library-audit-red.log`); all ten catalog/validator and four
read-only discovery contracts then pass (0.79s each test binary,
`/tmp/bbr-visual-library-audit-focused.log`). Full `just visual-test` passes 151
Python discovery tests (one additional integration skip, separately passed),
unchanged upstream and **71 consumer tests** plus existing signal fixtures:
`/tmp/bbr-visual-library-audit-visual-test.log`. Strict consumer all-target Clippy
passes (`/tmp/bbr-visual-library-audit-clippy.log`). Formatting, roadmap (160
tracked first-party Markdown files; `/tmp/bbr-visual-library-audit-roadmap.log`)
and whitespace checks pass. Structured comparison confirms every prior
catalog row unchanged, exactly three new withheld rows and all new source paths
present. These timings are tooling evidence only; audit/review wall time was not
instrumented and no product-performance claim is made.

Self-review checks actual render predicates rather than comments, separates
selection-row fill from toolbar state, traces recorder-only callbacks, preserves
Library's legitimate monogram-only boundary, and distinguishes potential nested
paint concerns from measured clipping. No native/VM run, actual PNG/HTML review,
functional GUI gate or baseline/environment adoption. Parallelism audit: IDs
31–48 remain runnable locally; all other lane authorizations stay unchanged.
Required approvals, guest runtime, legitimate input/endpoints and framing, fresh
pilot/full repeats, current functional Tart gates and final requirement-to-artifact
completion audit remain open. No goal completion or pause is claimed.

### Address / Sidebar components source-audit checkpoint

Preflight: clean `main@83109086`, no Git index lock, visual/Tart process or
observed capture-lock holder. ROADMAP and this complete delivery contract were
re-read without truncation. Ordinary user BBR and unrelated builds remain
untouched. Sole local visual writer, no workflow/subagent/new lane or foreign
worktree access. Audit of runnable queue work selected IDs 31–37 as the next
bounded slice; IDs 38–48 remain explicitly uninspected by this checkpoint.
VM/network and asset permissions remain pending and no actual capture is enabled.

The [source continuation](../visual/gallery-sidebar-components-audit.md) traces
seven stories through current matrices, production render predicates and recorder
seams. Address omits zoom/progress/device branches; staging always supplies end-of-
text with no selection/IME range. Favorites omit images/pop and compute tile widths
for a 300pt rail without constraining the story grid to it. Folder counts are
always transparent, including the count-labeled family; drop/selected-rename/image
branches are absent. Footer never supplies the Downloads focus ring and embeds a
fixture strip, not production space chips. Plus-menu shares item data but draws
its own accent-rail rows, unlike current production rounded cursor/elevation.
Profile chip has a legitimate static, noninteractive boundary. Potential nested
geometry/shadow concerns are not reported as measured clipping defects.

Six additive withheld catalog rows expose these concrete gaps without inventing
pages or moments. Structured comparison confirms every prior JSON row unchanged:
936 required page pairs and fifteen old blockers preserved, now 21 withheld rows.
The canonical packet was extended before implementation. No product, Gallery
fixture/renderer, vendor, config, independent page plan or baseline change.

Evidence: the new missing-blocker regression fails before catalog edits (exit 101,
`/tmp/bbr-visual-sidebar-components-audit-red.log`). Eleven catalog/validator and
four actual read-only discovery contracts then pass (0.76s / 0.81s measured,
`/tmp/bbr-visual-sidebar-components-audit-focused.log`). Full `just visual-test`
passes 151 Python discovery tests (one additional integration skip, then separately
passed), unchanged upstream and **72 consumer tests** plus signal fixtures:
`/tmp/bbr-visual-sidebar-components-audit-visual-test.log`. Strict consumer
all-target Clippy passes (`/tmp/bbr-visual-sidebar-components-audit-clippy.log`).
Consumer fmt, roadmap (161 tracked Markdown files;
`/tmp/bbr-visual-sidebar-components-audit-roadmap.log`), whitespace and all new
source-reference paths pass. Test timings are tooling evidence; audit/review wall
time was not instrumented. No product-performance evidence is claimed.

Self-review checked stale explanatory prose against render branches, real hover
styling despite absent DTO fields, invisible counts despite positive geometry,
fixture versus production strip/menu anatomy, unchanged old catalog rows and
narrow static exclusions. The new tests prove blocker visibility, not fixed
renderers or complete coverage. No native/VM execution, actual PNG/HTML review,
functional GUI gate or baseline/environment adoption occurred. Final queue audit:
IDs 38–48 remain runnable locally, no other lane authorized. Real guest runtime,
network/asset approvals, legitimate input/endpoints/framing, fresh pilot/full
repeats, current functional Tart gates and the final requirement-to-artifact
completion audit remain open. Goal completion and global pause are not justified.

### Downloads / Find / Error / Auth / Context-menu source-audit checkpoint

Preflight: clean `main@f9b80c9c`, no Git index lock, visual/Tart process or
observed capture-lock holder. ROADMAP and this complete delivery document were
re-read. Sole local visual writer; no workflow/subagent, foreign worktree access
or additional lane. Ordinary user BBR and unrelated builds remain untouched.
Queue audit selected IDs 38–43 as the next bounded runnable source slice; actual
VM/network and asset approvals remain pending, not bypassed.

The [source continuation](../visual/gallery-surfaces-audit.md) traces six stories
through matrices, production render predicates, recorder state and semantic-test
seams. Downloads shares one repeatedly reset ScrollHandle across all families,
so its 8..22 window does not establish independent scroll/viewport evidence;
completion/fade remain static. Find aliases seven production debug IDs to whole-
bar wrappers rather than actual control bounds. All Offline error fixtures choose
Lifering instead of the app's illustration branch. Crash/Empty omits the existing
production unresponsive pane. Context-menu fixtures fix height at 148pt instead
of the app's row-count/label/padding formula. Actual clipping is not inferred from
source arithmetic. Auth remains a legitimate narrow redacted card; no new blocker
or fake Portal/credential case is invented for its app-owned composition boundary.

Five additive withheld entries expose these concrete gaps. Structured comparison
proves every preceding catalog JSON row unchanged: 936 required page pairs and
21 old blockers preserved, now 26 withheld. The canonical packet was extended
before catalog implementation. Product/Gallery fixtures/renderers, upstream,
assets, configs, independent page plan and baselines have no diff.

Evidence: missing-blocker regression is red before catalog changes (exit 101,
`/tmp/bbr-visual-surfaces-audit-red.log`); twelve catalog/validator and four
actual read-only discovery contracts then pass (0.74s / 0.73s measured,
`/tmp/bbr-visual-surfaces-audit-focused.log`). Full `just visual-test` passes
151 Python discovery tests (one additional integration skip, then separately
passed), unchanged upstream and **73 consumer tests** plus signal fixtures:
`/tmp/bbr-visual-surfaces-audit-visual-test.log`. Strict consumer all-target
Clippy passes (`/tmp/bbr-visual-surfaces-audit-clippy.log`). Consumer fmt,
source-reference existence, whitespace and roadmap checks pass (162 tracked
Markdown files, grandfathered warnings only;
`/tmp/bbr-visual-surfaces-audit-roadmap.log`). Timings measure tooling tests;
audit/review elapsed time was not instrumented, no product-performance claim.

Self-review checks actual render predicates and call sites, shared mutable handle
ordering, wrapper versus control IDs, fixed bullets versus claimed masked length,
rest versus moment branches, app-derived geometry versus measured paint, and
narrow native/credential exclusions. Portable contracts prove diagnostic visibility,
not repaired coverage. No native/VM execution, actual PNG/HTML review, functional
GUI gate, baseline/environment adoption or push occurred. Final queue audit:
IDs 44–48 remain runnable locally, no other lane authorized. Guest runtime,
network/asset approvals, legitimate input/endpoints/framing, pilot/full repeats,
current functional Tart gates and final requirement-to-artifact audit remain
open; neither goal completion nor global pause is justified.

### Sidebar / Command / Space configuration / Menu source-audit checkpoint

Preflight: clean `main@eca54f31`, no Git index lock, visual/Tart process or
observed capture-lock holder. ROADMAP and the complete 1,978-line delivery ledger
were read without truncation. Sole local visual writer; no workflow/subagent,
foreign worktree access, new lane, asset or VM/network operation. Ordinary user
BBR and unrelated builds remain untouched. Queue audit selected the remaining
44–48 source slice; the registry, not the continuation shorthand, identifies
46 as Space configuration and 47 as the combined Tab / Favorite menu.

The [composition audit](../visual/gallery-compositions-audit.md) traces the actual
production branches and records five concrete withheld gaps. Sidebar never shows
closed rows/tiles/members, status, floating/grain/tint or drag-proxy/destination
branches; its dense model supplies only the first 64 of 500 regular rows inside
a 700pt wrapper. Command uses end-of-text without selection/IME, no URL fields,
no Saved Page/History or actions-chip/shortcut sample, and no entrance/glide model.
Space configuration omits the actual empty-name placeholder. Tab / Favorite menu
omits unfocused, Unpin, Remove Bookmark and disabled Close representations. Page
menu inherits the shared 148pt fixed-card versus app geometry mismatch. Source
arithmetic is not measured clipping, recorder dispatch is not model mutation,
and app-owned Portal/controller behavior remains a narrow exclusion.

Self-review preserves stronger existing mechanisms: Sidebar actually supplies
zoom, focused folder counts, production Plus-menu and strip; its Reduced Motion
skips the Plus fade. Command already waits Motion::fine for its overlay. Tab-menu
fixture height is calculated and tested against natural production layout, not
left at the old 240pt constant. The new audit does not erase those facts or claim
native verification from them. The canonical packet was extended before catalog
implementation. All 936 required page pairs and 26 old withheld rows are unchanged
as JSON values; five additive rows bring the withheld count to 31, with no invented
pages/sequences/moments. No product, fixture, renderer, vendor, asset, config,
independent plan or baseline diff.

Evidence: the new regression fails for the intended missing blocker before
catalog changes (exit 101, `/tmp/bbr-visual-compositions-audit-red.log`). Thirteen
catalog/validator and four read-only discovery contracts then pass (0.74s / 0.62s,
`/tmp/bbr-visual-compositions-audit-focused.log`). Full `just visual-test` passes
151 Python discovery tests (one additional integration skip, separately passed),
unchanged upstream and **74 consumer tests** plus signal fixtures:
`/tmp/bbr-visual-compositions-audit-visual-test.log`. Strict consumer all-target
Clippy and fmt pass (`/tmp/bbr-visual-compositions-audit-clippy.log`), as do source-
reference existence, structured prior-row preservation and whitespace checks.
Roadmap verification is recorded in `/tmp/bbr-visual-compositions-audit-roadmap.log`.
Timings are tooling measurements; audit/review elapsed time was not instrumented.
No product-performance, native/VM run, actual PNG/HTML review, functional GUI gate,
baseline/environment adoption or push is claimed.

Final queue audit: source continuations now touch all registered IDs 0–52, but the
supported-state contract is not complete. Local work remains on translating the
explicit gaps into bounded effective-state/fixture contracts and checking the
remaining consumer cutover requirements without enabling native execution. No
other lane is authorized. Required VM/network and asset approvals, actual guest
runtime, legitimate input/endpoints/framing, fresh pilot/full repeats, current
functional Tart gates and final requirement-to-artifact audit remain open. This
source checkpoint is neither migration completion nor a global blocked stop.

### Explicit stored-run review checkpoint

Preflight: `main@8c769fe1`, only this session's packet/test changes followed by
its bounded consumer implementation; no index lock or matching visual/Tart
process. ROADMAP and all 2,034 lines of this delivery document were read in
bounded chunks. Sole local visual writer; no workflow/subagent, foreign worktree
or other queue activation. Ordinary app/build processes remain untouched.

The Rust consumer now exposes explicit `compare --gallery-run … --browser-run …
--report … [--json]`, with at least one named run and fresh, canonical, disjoint
output. It uses current configured baseline paths and canonical coverage catalogs;
config/source-policy drift fails before publication. Full still runs use unchanged
upstream Collections with invalid sibling visibility. A single selected Gallery
run instead gets the upstream non-accepting `TEILLAUF` report, bound to its current
story catalog and independent page plan. Selected/full mixtures, selected browser
runs and valid sequence inputs are refused. No latest lookup, build/probe/native
call, resealing, baseline write, real catalog activation or Python cutover.

Evidence: seven synthetic stored-run contracts plus one actual CLI tripwire test
prove independent coverage, invalid sibling visibility, exact one-pixel difference,
environment drift, current config/policy/source binding, image/report integrity,
freshness/overlap/symlink refusal, pre-cancellation and partial acceptance refusal
with/without adoption. Disposable test-only acceptance demonstrates a subsequent
exact pass, not real baseline approval. Initial module red is retained at
`/tmp/bbr-visual-review-red.log`. Adversarial tests exposed a real canonical-path
guard bug: Rust Path equality normalizes `./` and duplicate separators. Comparing
raw OsStr spelling fixes it; the negative-first regression is retained at
`/tmp/bbr-visual-review-adversarial-red.log`. Selected output is atomically reserved
because the upstream single-suite API otherwise permits report replacement.

`just visual-test` passes 152 discovered Python tests (151 pass, one integration
skip separately passed), unchanged upstream and **82 consumer tests** plus signal
fixtures: `/tmp/bbr-visual-review-visual-test.log`. Final focused review tests pass
(`/tmp/bbr-visual-review-focused-reviewed.log`); strict all-target consumer Clippy,
formatting, roadmap and whitespace checks pass. Clippy evidence:
`/tmp/bbr-visual-review-clippy-reviewed.log`; roadmap:
`/tmp/bbr-visual-review-roadmap.log`. Focused pre-review times were 1.14s plus
0.29s CLI; these are tooling timings, not product performance. Implementation and
self-review wall time were not measured.

Self-review checked delegation to immutable upstream, no-write failure paths,
explicit selection scope, actual source/image revalidation, fresh report ownership
and unchanged capture guard/product/config/catalog/baseline files. Cancellation
is checked at operation boundaries: upstream comparison is synchronous, so the
CLI documentation requires an outer hard timeout and does not claim per-image
cooperative cancellation. Interruption can retain partial diagnostics or a finished
report; no automatic output reuse or lock recovery. No native/VM operation, actual
migration PNG/HTML review, current functional GUI gate or acceptance occurred.

Queue audit: remaining local work includes current portable product verification
and a requirement-to-artifact audit to identify exact uncovered integration steps;
source audit findings do not authorize product/fixture changes. Real network/asset
approval, guest runtime, legitimate input/framing, pilot/full repeats, current
functional Tart gates and concrete baseline review remain open. No other lane is
authorized; no migration completion or global pause is claimed.

### Current portable verification and requirement audit

On clean `main@36eea50e6d54f041127ba641b4dedfbecc10955d`, the full current
non-GUI `just verify` sequence passes, with only compiler-invoking Cargo calls
substituted by mbx in `/tmp/bbr-visual-verify-hza9sd_c.just`. No assertion,
recipe order, budget, target directory or GUI behavior changed. Evidence:
`/tmp/bbr-visual-migration-verify.log`, including 49 shared script tests (14.606s).
Strict testkit all-target, browser visual-capture test and Gallery capture-feature
all-target Clippy also pass. Gallery feature units: **159 passed, 48 ignored**;
those ignored GUI tests were not executed on the host. Evidence:
`/tmp/bbr-visual-migration-feature-gates.log`. Existing `block` warning remains.
No VM/network/native producer was launched; no current functional GUI pass follows.

The [requirement-to-artifact audit](../visual/migration-evidence-audit.md) maps the
entire goal to source/contracts and explicit missing proof. It confirms unchanged
product Cargo graph/pin/crates/gpui-component/baselines since activation, separates
portable protocols from actual guest availability/teardown/pixels, and records
that public capture, real catalog/pilot, complete framing, exact full repeats,
visual approval and Python cutover remain incomplete. Additional compositions and
sequences remain sequenced after stable migration. This is not a completion audit
pass and no milestone boxes are checked.

Scheduling audit: same sole local owner, clean base above, no matching visual/Tart
process or new lane authorization. Locally runnable next work is the bounded
pilot/effective-state/framing contract derived from concrete source findings,
not repeating the completed source audits. Network and legitimate Check asset
approvals remain external boundaries; actual guest closure presence is unknown.
No workflow, foreign worktree, host GUI, push or performance execution. Tooling
check durations are measured where recorded, not product timings; requirement-
audit elapsed time was not instrumented. Goal remains active and incomplete.

### Bounded pilot contract checkpoint

Preflight: clean `main@509e565b94d3ec2e29346f26731e46ceb1d47a86`, same visual
session/sole writer, no Git index lock, matching visual/Tart process or observed
capture-lock holder. ROADMAP and all 2,121 delivery lines were read in bounded
chunks. No other queue item is authorized here; no subagent/workflow, foreign
worktree, host GUI, VM/network, asset change or publication.

The [pilot packet](../roadmap/tasks/gallery-visual-pilot.md) now owns concrete
Tokens, Checkbox and Sidebar requirements, linked through roadmap/phase/track
and the parent packets. Source inspection at the unchanged GPUI pin confirms:
headless input must use public Window dispatch, not the VisualTestContext-only
bounds helper; the existing hug-child wrapper can provide capture-only bounds;
Checkbox has a 250ms scheduler timer plus a wall-clock animation; selecting a
capture case does not reset its recorder. The contract requires fresh journey
state, false/true/false snapshots around real GPUI down/up events, both clock
boundaries, distinct still identities and legitimate Check SVG pixels. It does
not claim OS input or an animation timeline.

Sidebar outer pagination remains separate from nested/paint framing. The default
family requires its actual descendants/clip ancestry; dense-500 remains a bounded
window diagnostic, never full-list coverage. Existing complete selected-story
scheduling, all 936 historical pairs and 31 withheld rows are unchanged. No
catalog/config/producer is activated and no native result is claimed.

Self-review traced the public input/clock seams against the exact GPUI checkout
and current component/fixture sources. Verification for this documentation-only
slice: `just roadmap` and index roadmap validation, source-reference existence,
whitespace and unchanged protected inputs; command logs are
`/tmp/bbr-visual-pilot-contract-roadmap.log` and
`/tmp/bbr-visual-pilot-contract-index.log`. No Rust/Python behavior changed; prior
portable test results are not advertised as fresh runtime evidence. Planning
and review time were not instrumented; product performance remains unproven.

Scheduling audit: the next useful local implementation is the bounded source-bound
endpoint identity/rejection contract; it must preserve upstream authority and
selected scope before any producer extension. Actual pilot still requires the
specific VM/network and asset approvals, guest runtime availability and measured
input/paint/framing evidence. No acceptance boxes are checked, and neither goal
completion nor a global blocked stop follows from publishing this contract.

### Portable Checkbox endpoint requirements checkpoint

Preflight base: clean `main@5359741d75fc8bed2dcea0ef0c37c3a76ce0382c`, with
only this writer's pilot-contract/test changes afterward. ROADMAP and all 2,162
lines of the prior delivery document were read in bounded chunks. No Git index
lock or matching Tart/visual process was observed; ordinary app/build processes
and foreign worktrees remain untouched. Sole local visual writer, no workflow,
subagent, new lane, network/VM/asset operation or publication.

The consumer now has one shared independent `Plan::validate` authority for static
and optional typed schema-2 endpoint requirements. A Checkbox story-16/State-00
static case is preserved, followed by the exact ordered initial-unchecked,
checked-after-click and unchecked-after-second-click triple. Endpoint metadata
and per-appearance child ranges must match its static case. Unknown/orphan/
missing/extra/duplicate/reordered identities, invalid schemas/metadata/ranges,
noncanonical numbers and incomplete selected pages fail. Upstream selection and
original source hashes still bind the entire authored story schedule. No State
selector is invented, and still identities do not imply a timeline or input proof.

`require_static_capture` blocks endpoint-bearing request creation, raw validation,
stored review, fresh full capture, direct transport and guest command construction.
Self-review found and guarded the latter two lower-level entrypoints rather than
relying only on the normal CLI's existing host refusal. Correctly named complete
synthetic RGBA8 output and valid synthetic upstream image seals cannot substitute
for unwired semantic evidence. No sidecar or new seal/process authority is added.
Static-only schema-2 selection remains usable and inherits unchanged upstream
partial/nonacceptance behavior. Real plan/catalog/config, product crates, GPUI pin,
upstream export, assets and baselines remain unchanged: 630 static cases / 936
historical pairs and 31 withheld requirements are preserved.

Evidence:
- Initial focused red is the absent endpoint field/API, exit 101:
  `/tmp/bbr-visual-endpoints-red.log`. Seven added endpoint integration contracts
  plus one fresh-capture tripwire pass. Existing direct guest and cross-language
  transport tests now include endpoint refusal; the latter exercises 14 modes.
- The first expanded consumer run exposed an old command-only fixture lacking
  the newly mandatory independent plan. The fixture was source-bound, not the
  guard weakened: `/tmp/bbr-visual-endpoints-consumer.log`, then passing
  `/tmp/bbr-visual-endpoints-consumer2.log`.
- Final `just visual-test` passes 152 Python discovery tests (151 pass, one
  integration skip separately passed), unchanged upstream tests and **90 consumer
  tests** plus signal fixtures: `/tmp/bbr-visual-endpoints-visual-test.log`.
  Endpoint integration group 0.34s, consumer unit group 6.33s, transport chain
  6.19s measured. These are tooling timings, not product performance.
- Strict consumer all-target Clippy and fmt pass:
  `/tmp/bbr-visual-endpoints-final-clippy.log`. Roadmap, index-roadmap and whitespace
  checks are recorded in `/tmp/bbr-visual-endpoints-roadmap.log` and
  `/tmp/bbr-visual-endpoints-index.log`. Protected-input Git diff is empty.

No native producer, VM, key/SSH operation, actual migration image/report review,
current functional GUI gate or baseline/environment adoption occurred. This is
an intermediate portable tooling slice, not stable-migration handoff or completion
of the pilot acceptance boxes. Product `just verify` and native gates from earlier
checkpoints are historical evidence, not rerun claims for this increment.
Implementation/review elapsed time was not instrumented.

Scheduling audit: only the bounded pilot's capture-only target/input/readiness
seams and portable rejection contracts are next locally runnable; no other product
queue item or parallel lane is authorized. Specific network/asset approvals,
actual guest runtime, legitimate input/paint/framing, fresh repeated captures,
visual review and stable Tart functional gates remain open. Source edits require
fresh future preparation/captures. No goal completion or global pause is justified.

### Checkbox target-observation candidate — guest verification pending

Base remains `main@635bb3652886a7176b4b8570141f8d87843641f2`. This writer's
uncommitted candidate is confined to Gallery capture support, its tests, the
feature-enabled Gallery recipe and this documentation. At restoration, HEAD and
ownership match; no index lock or matching visual/Tart/build process was observed.
ROADMAP and all 2,226 lines of the preceding delivery document were read in bounded
chunks. Ordinary app/build processes and isolated worktrees remain untouched.
No workflow, subagent, new lane, VM/network/asset operation or publication.

The pilot packet now defines the bounded observer before its implementation.
`capture_target::Target` records exactly one current-draw sole-child rectangle
and active rectangular content-mask intersection. Missing/stale/unarmed/duplicate,
multiple/empty-child, nonfinite/nonpositive, clipped or out-of-frame observations
fail without clamping; a later good callback cannot repair that draw. Page reset
preserves/increments the generation so an old callback cannot become current.
The existing Checkbox container gains only a capture-feature prepaint observer;
its production child, layout, input callbacks, recorder and assets are unchanged.
The existing runner arms after draining pending work and validates each explicit
page draw, requiring absence on other pages and stories. No endpoint guard is
lifted, no event is dispatched, and no output schema/catalog/baseline changes.

The ignored feature scenario compiles for all twelve Checkbox appearance cases.
It checks repeated draw tokens, full/control-only pages, equality to the existing
semantic wrapper, pagination/switch-away absence and unchanged semantic snapshot.
It is **not executed**. A new `e2e-gallery-capture` recipe declares the feature-on,
serial ignored-suite plan; a host-only parser test proves the existing VM planner
selects those exact args. This does not validate or authorize that runner's boot:
its default preparation realizes guest Nix dependencies and does not use the
visual clone-hardening policy. Do not run it unchanged under this goal. A bounded
no-install, hardened guest execution path remains to be wired/verified before
these scenarios can run; no generic-runner rewrite or runtime fallback is inferred.

Evidence:
- Missing implementation red: `/tmp/bbr-visual-target-red.log` (exit 101, absent
  Target/PageState field). Guest recipe regression also fails for the intended
  absent plan before the recipe: `/tmp/bbr-visual-target-recipe-red.log`.
- Feature-on tests **164 passed / 49 ignored**, including four pure target tests
  and the plan-parser contract; `/tmp/bbr-visual-target-gallery-final.log`, 0.44s.
  Feature-off tests **153 passed / 48 ignored**;
  `/tmp/bbr-visual-target-normal-tests.log`, 0.48s. None of the ignored GUI
  scenarios ran on the host or in a guest.
- Strict Gallery all-target Clippy passes with and without the feature:
  `/tmp/bbr-visual-target-feature-clippy-final.log` and
  `/tmp/bbr-visual-target-normal-clippy-final.log`. Workspace fmt/whitespace pass;
  existing `block v0.1.6` future-incompatibility warning remains unsuppressed.
- `/tmp/bbr-visual-target-visual-test.log`: full `just visual-test` passes 152
  Python discoveries (151 pass / one integration skip subsequently run and passed),
  unchanged upstream and **90 consumer tests** plus signal fixtures. Consumer
  endpoint execution/review refusal remains exercised and unchanged.
- `/tmp/bbr-visual-target-verify.log`: full non-GUI `just verify` order passes,
  including 49 shared script tests (15.141s). The inspected temporary recipe
  `/tmp/bbr-visual-target-verify-8qpnwopf.just` substitutes only compiler-invoking
  Cargo calls with mbx; `/tmp/bbr-visual-target-verify-plan.log` records the plan.
  No GUI recipe, assertion, deadline or target-directory override was introduced.

Self-review checked callback scope against pinned GPUI Div prepaint/content-mask
and window-layout APIs, fresh-token capture after notify/refresh, and screenshot
readback of the existing rendered scene (not an implicit draw). Review expanded
absence checking to all stories and added the explicit feature recipe, so normal
Gallery tests cannot silently stand in for the new scenario. Bounds are not the
inner Checkbox hitbox, painted glyph/shadow extent, ancestor identities or input
proof. No synthetic fixture is advertised as callback/runtime/pixel evidence.

The candidate stays **uncommitted**, as required by the pilot packet, until its
affected guest gate passes. No PNG/HTML review, actual guest runtime/cleanup,
functional GUI aggregate, input/two-clock readiness, baseline adoption or product
performance is proven. Source changes require fresh future preparation/captures;
all GPUI/vendor/product-Cargo/baseline/config/catalog inputs remain unchanged.
Tooling timings above are measured; implementation/self-review wall time was not.

Scheduling audit: only this local pilot writer is authorized. The next defensible
local prerequisite is a narrowly bounded guest-semantic execution plan using the
existing hardened/no-install transport rather than layering more unverified input
wiring on this observer. Actual boot still needs Patrick's acknowledgement of
private NAT, dynamic guest TCP 22 from host/allowed NAT peers, run-specific
key-only SSH, no public/bridge/forwarded listeners, and listener/auth/access plus
clone/key cleanup audits. The legitimate Check SVG asset-source change separately
needs permission; report-specific baseline adoption remains later. Other product
lanes and post-migration compositions/sequences remain outside current activation.
Goal completion and global pause are not claimed; one concrete local execution
prerequisite remains, while the affected runtime path is blocked by approval.

### Gallery semantic payload preparation checkpoint

Continuation base remains `main@635bb3652886a7176b4b8570141f8d87843641f2`;
the observer candidate above is still owned locally and uncommitted. ROADMAP and
all 2,308 delivery lines were read in bounded chunks. No staged changes/index lock
or observed capture-lock holder; an unrelated mbx process belonged to another
project and was left untouched. No subagent, workflow, foreign worktree, asset,
VM/network operation or publication was authorized or performed.

The pilot packet was extended before this bounded prerequisite. The existing
host-only prepare path accepts `gallery-semantic`, separate from the two image
suites. It uses fixed release `mbx test --no-run` with `visual-capture` and the
Gallery binary selector, verifies Cargo's test/bin/features metadata, then stages
exactly that hashed test executable plus the standalone consumer. Source snapshots,
compiled paths, copy/hash/mode limits, host evidence and offline runtime verification
reuse existing authorities. The Rust/Python installers agree on the distinct
`gallery-test` role; wrong/missing/duplicate/feature-off artifacts and capture-role
substitution fail. No harness execution, including listing, is part of preparation.

Semantic payloads explicitly fail bootstrap and guest composition before runtime
probes, installation or producer calls. Existing `Suite` parsing, capture guards,
receipts and endpoint refusal remain unchanged. This increment does not pretend
that source/build preparation implements the semantic guest action or a successful
test result. The subsequent fixed-action/result-validation bridge must reuse the
existing source-frozen supervised transport and cleanup, not the generic E2E runner.

Evidence:
- Negative-first missing-purpose/build-command regressions:
  `/tmp/bbr-visual-semantic-payload-red.log` and
  `/tmp/bbr-visual-semantic-build-red.log` (exit 101). Explicit unwired-route
  regressions fail before guards in `/tmp/bbr-visual-semantic-bootstrap-red.log`
  and `/tmp/bbr-visual-semantic-guest-red.log`.
- Full `just visual-test`: `/tmp/bbr-visual-semantic-visual-test.log`, **155 Python
  discoveries** (154 pass / one integration skip, separately passed), unchanged
  upstream and **92 consumer tests** plus signal fixtures. Python discovery 6.315s,
  consumer unit group 6.31s, cross-language transport chain 6.19s measured.
  The complete gate passes again after self-review in
  `/tmp/bbr-visual-semantic-visual-test-final.log`. Consumer/workspace fmt,
  whitespace, protected-input diff and roadmap pass; roadmap log:
  `/tmp/bbr-visual-semantic-roadmap.log`.
- Strict consumer all-target Clippy passes after moving the test module below
  production items (no lint suppression):
  `/tmp/bbr-visual-semantic-clippy-final.log`. Post-review focused build contract
  and nine integration tests pass in `/tmp/bbr-visual-semantic-focused-final.log`
  and `/tmp/bbr-visual-semantic-integrations-final.log`; the first command's name
  filter selected only the unit contract, so the integrations were rerun unfiltered.
- Actual host-only `just visual-prepare gallery-semantic` succeeds at
  `/private/tmp/bbr-visual-semantic-prepare.br7ZPY`: **1,502 sources / 1,504 files /
  41,201,765 bytes**; manifest SHA-256
  `4fa7db98899ebeb381805c4348a950067dd6c8c5cef295df74dee15336e008f3`.
  Source inventory includes all four currently untracked implementation files.
  Gallery test SHA-256
  `3c0ef7ad9813e3128eeac9f473d530e792aaf3783d9f4b85d100fe3e8ed935c8`;
  consumer `192cbdd826eef110037e4ebadd7038fc0731ccf9ca9d9f5d361e5d32e8876a1f`.
  Exact compiled paths/host observations are in that directory's `host-build.json`;
  `prepare.log` records release Gallery build 52.64s and consumer 3.84s. Independent
  Python read-only inspection rechecks all payload bytes against those host hashes;
  actual bootstrap generation refuses the semantic purpose. No installation or
  executable launch follows. Documentation edits afterward make this preparation
  historical diagnostics, not current guest inputs or any capture seal.
- Self-review also guarded direct bootstrap-tail entry before parent creation;
  `/tmp/bbr-visual-semantic-tail-red.log` retains the regression. Full contracts
  pass afterward in `/tmp/bbr-visual-semantic-reviewed-visual-test.log`.
  A fresh host-only preparation at `/private/tmp/bbr-visual-semantic-final.xCwtYP`
  passes with the same two executable hashes, 1,502 sources / 1,504 files /
  41,202,603 bytes and manifest
  `059057fee9798675f7292b6b627e7245c40d322e1a911de216a98ea4311753cf`.
  Its `inspection.json` records independent byte/hash checks and bootstrap refusal;
  cached build times are 1.11s Gallery / 0.08s consumer. The following ledger update
  again requires fresh source staging for future guest execution, not resealing.

Self-review checked disjoint purpose/role mapping, exact pre-copy artifact roles,
feature/test/bin metadata, same bounded upstream build executor, both interpreter
validators and source-inventory inclusion. Existing `block v0.1.6` warning remains
unsuppressed. No product Cargo/vendor/GPUI/baseline/config/catalog changes. Earlier
Gallery/verify evidence belongs to the observer checkpoint; no new functional GUI
or product-performance claim. Measured times are tooling only; implementation and
review elapsed time were not instrumented.

Scheduling audit: only this local visual writer is authorized. Fixed semantic
guest execution, exact successful-suite evidence and returned-result validation
remain concrete local work; no new input wiring or other product lane is started.
Actual callback validation still requires the specific network approval and an
already available guest runtime, with no realization/repair fallback. Asset and
baseline approvals remain separate. The observer stays uncommitted pending its
affected guest gate. No completion or global blocked-stop claim is made.

### Fixed Gallery semantic action checkpoint

Base is `main@b4c0795379f88d8193b5cf9c33da9ecd15a81ba6`; the prior Gallery
observer/recipe remains the only inherited dirty candidate and is not committed
by this tooling increment. ROADMAP and the changed delivery checkpoint/queue were
rechecked against the preceding full read. No staged changes/index lock; unrelated
mbx process cwd was outside BBR and remained untouched. A transient holder of the
existing GUI lock was observed during final checks; no signal, lock takeover or
GUI operation was attempted. Only this local visual writer is authorized, with no
new lane/workflow/subagent, foreign working-tree inspection or publication.

The pilot packet now bounds one guest-only fixed `gallery-semantic semantic`
action. An independently authored 49-name requirement includes the pending target
scenario; listing must match before the first test runs. Fixed ignored/serial/
pretty/no-color commands use the exact staged harness. Actual successful exit
and exact ordered per-test `ok` lines plus one zero-failed/ignored/measured summary
are required. Unknown output, missing/extra/duplicate names/results, zero tests,
failed/ignored cases, overflow, cancellation and source drift fail closed.
The typed result retains raw listing/run stdout and original payload, requirement
and executable identities. Validation repeats completeness; no image seal,
Collection input, baseline or native-paint proof is invented.

Dispatch retains actual admin/VirtualMac, compiled-path, manifest/shared/installed
byte guards. The existing upstream executor owns every child group. One 600s
budget starts before guest identity/preflight, reduces command time by repeated
byte checks, and is rechecked after final identity verification. No caller filter,
arbitrary argv, host bypass, new dependency or runtime installation is added.
Python bootstrap and guest-composition guards remain closed pending the next
return integration. A result file alone is not terminal transport success.

Evidence:
- Missing-module/action reds: `/tmp/bbr-visual-semantic-action-red.log` and
  `/tmp/bbr-visual-semantic-invoke-red.log`. The first source-inventory check
  caught a mistyped Library test name; the authored name was corrected rather
  than relaxing the check (`/tmp/bbr-visual-semantic-action-protocol.log`).
- Five protocol/source contracts pass, including actual stable libtest listing
  and serial execution of two inert consumer-only fixtures; no Gallery executable
  is launched. Four injected guest-action tests cover exact argv/environment,
  ordering, fresh output, cancelled/expired budgets, symlink/source drift,
  failed status/cleanup and incomplete/invalid output. A further dispatcher test
  rejects capture actions/filters on this purpose. Actual host CLI refusal is
  extended to the full semantic argv.
- Full `just visual-test` passes after self-review:
  `/tmp/bbr-visual-semantic-action-reviewed.log`: **155 Python discoveries**
  (154 pass / one integration skip separately passed), unchanged upstream and
  **102 consumer tests** plus signal fixtures. Two inert fixture tests are ignored
  by normal discovery and explicitly executed by the protocol contract.
  Consumer unit group 6.75s; Python discovery 6.433s. These are tooling times.
- Strict consumer all-target Clippy passes after a test-only `and_then`→`map`
  lint correction: `/tmp/bbr-visual-semantic-action-clippy-reviewed.log`.
  Workspace/consumer fmt, whitespace and protected-input checks pass. Roadmap:
  `/tmp/bbr-visual-semantic-action-roadmap.log`. No assertion or deadline weakened.

Self-review found an integration detail for the next slice: existing Gallery sweep
scenarios write exactly two report-only files under the compiled
`target/bbr-gallery-perf` directory. The current Python installed-tree verifier
rightly rejects extra files. Before semantic transport activation, account for
those specific outputs without changing tests or broadening the installed-file
allowlist; preserve them as diagnostics, never product-performance acceptance.
The 49-name requirement is not image coverage. It intentionally requires the
currently uncommitted observer and cannot pass on a harness lacking that scenario.

No real Gallery/VM/SSH/network, guest runtime proof, current GUI gate, visual review
or baseline adoption occurred. The previous host preparations are stale; subsequent
execution requires fresh source/build staging. Product Cargo/pin/vendor/baselines,
image catalogs and endpoint execution guards are unchanged. Implementation/review
elapsed time was not measured. The observer remains uncommitted pending its real
guest gate; this portable consumer increment is not a stable-migration handoff.
Final commit review found that its exact source-completeness regression depends
on that observer candidate. Therefore this action/catalog increment also stays
uncommitted: splitting it now would create a revision that fails its own source
check. No assertion is weakened or dependency hidden to obtain a separate commit.
Only this writer's staged paths are unstaged, preserving all working files.

Scheduling audit: receipt/return validation and the exact report-output handling
are the next concrete local prerequisite using existing transport. No more input
wiring or other product queue item is activated. Actual boot still requires the
specific private-NAT/key-only SSH approval; legitimate Check SVG and report-specific
adoption require separate approval. Goal remains active and incomplete, not globally
blocked while that bounded local integration remains defensible.

### Semantic diagnostics / guest composition / host return checkpoint

Continuation reverified `main@b4c0795379f88d8193b5cf9c33da9ecd15a81ba6`,
empty index, no index lock, and this writer's preserved observer/action candidates.
ROADMAP and all 2,476 then-current delivery lines were read in bounded chunks.
No matching Tart/build process or holder of the two historical capture-lock files
was observed; the ordinary user BBR process was left untouched. One local writer,
no workflow, subagent, new lane or foreign worktree access. The pilot packet was
extended before implementation; no other product queue item is authorized here.

The fixed semantic action now requires a fresh sweep-output directory before
listing and again before execution. After exact test success it preflights exactly
the two existing report-only JSON files, rejects links/nonregular/oversized/invalid
or extra output, and preserves their bytes in `semantic-every-story-renders.json`
and `semantic-mode-state-switching.json`. Both destinations are fresh; copying,
byte rechecks and removal stay within the action's cancellation/deadline checkpoints.
Only the two known source files and their now-empty generated directory are removed.
No installed-tree allowlist exception or test/budget change was made. Missing output
is failure; diagnostics are neither image snapshots nor product-performance proof.

Python guest composition invokes only the fixed semantic action for this purpose,
then performs the unchanged exact installed-tree/shared-byte checks and repeated
guest observation before receipt/archive creation. It requires the exact three raw
semantic artifacts, and the existing bounded transfer metadata allowlist names
only those new files. Filters and capture actions are rejected. No Python test
coverage parser, second transport or capture authority was introduced.

The new pure host validator reuses extracted unchanged returned-file/marker checks
and host/guest receipt binding from the capture path. It requires the exact result,
both diagnostics and receipt, checks hash-bound semantic stdout/requirements/build
identity and report-only diagnostics, and returns a separate `Verified` type rather
than `Captured`, an image suite or a rendering policy. Rechecks retain the original
host payload/build/digest and transport evidence; rewriting the output marker cannot
adopt changed bytes. Capture receipt binding rejects a semantic payload. This is
pure integrity/completeness evidence, not proof that a guest process or VM ran.
Bootstrap and the host execution route remain closed to this purpose pending the
next supervised-chain integration; no public VM command is enabled.

Evidence:
- Missing diagnostic function red: `/tmp/bbr-visual-semantic-diagnostics-red.log`.
  Behavioral reds show missing reports wrongly allowed success and absent diagnostic
  return: `/tmp/bbr-visual-semantic-diagnostic-action-red.log`.
- Guest-composition red: `/tmp/bbr-visual-semantic-composition-red.log`.
  Green VM-helper group: `/tmp/bbr-visual-semantic-composition-green.log`,
  111 tests / 1.345s. All native/environment/process calls are synthetic boundaries;
  fixtures still use real strict installation/archive checks in disposable folders.
- Host return missing-module red: `/tmp/bbr-visual-semantic-return-red.log`.
  Three return contracts plus the six existing receipt/transport contracts pass:
  `/tmp/bbr-visual-semantic-return-green.log`. Negative cases include missing/extra/
  linked/changed output, rehashed invalid result/diagnostic data, source/build/guest
  drift, cross-purpose image substitution, cancellation and rewritten manifests.
- Final full `just visual-test`: `/tmp/bbr-visual-semantic-return-visual-test.log`:
  **158 Python discoveries** (157 pass / one integration skip separately passed),
  unchanged upstream and **108 top-level consumer tests** plus existing signal and
  inert protocol fixtures. The count excludes echoed injected libtest transcripts.
  Python discovery 6.445s; separate CLI fixture 0.601s, no real baseline mutation.
- Strict consumer all-target Clippy: `/tmp/bbr-visual-semantic-return-clippy.log`,
  1.65s cached. Earlier complete diagnostic/composition pass:
  `/tmp/bbr-visual-semantic-composition-visual-test.log`; Clippy25.05s at
  `/tmp/bbr-visual-semantic-composition-clippy.log`. Workspace/consumer formatting,
  whitespace and protected-input checks pass; roadmap rechecked after documentation.

Self-review kept the existing capture environment JSON unchanged while separating
receipt validation from image-policy construction, and reused rather than duplicated
the return-byte checker. Dedicated names avoid mislabeling timing reports as image
snapshots. No actual Gallery, VM, SSH, network, guest runtime, current GUI gate,
PNG/HTML review, cleanup or visual acceptance is proven. Prior preparations remain
stale; fresh source/build staging is required. Tooling times are not product metrics;
implementation/review elapsed time was not measured. All candidates remain
uncommitted because the exact semantic inventory depends on the observer awaiting
its actual guest gate. No staged changes or acceptance checkboxes are introduced.

Scheduling audit: the next useful local step is source-frozen bootstrap and shared
supervisor-to-semantic-return integration, retaining original identities and
cleanup-before-publication ordering. Specific private-NAT/key-only SSH approval,
available guest runtime without installation, Check SVG approval and later concrete
baseline adoption remain required. No repeated source audit or extra product/input
work is activated. The goal remains active and incomplete while this local step is
still defensible.

### Shared supervised semantic route checkpoint

The same local candidate now connects semantic payloads through the existing host
transport rather than a second pipeline. A private shared transfer input carries
`Purpose`, while image callers retain their unchanged `Suite`/`Captured` API and
semantic callers expose only `Verified`, with no selection field. Exact payload
purpose/repository is checked before runtime/workspace creation. Only source/runtime
policy uses the Gallery source suite. Supervision, original payload digest, gates,
source rechecks and independent resource cleanup remain the same implementation.
Semantic record/receipt validation follows successful supervisor return.

Source-frozen bootstrap generation/tail now accept semantic purpose unchanged;
actual admin/VirtualMac guards still precede runtime checks/mounts/parent writes.
The pinned runtime must already exist and verify offline. No installation, repair,
new command-line entrypoint, automatic boot or public exposure was added. The
preparation recipe remains host-only and never even lists the Gallery harness.
Historical checkpoints describing explicit semantic refusal remain provenance;
this checkpoint supersedes only that internal capability, not approval boundaries.

Evidence:
- Missing internal route red: `/tmp/bbr-visual-semantic-chain-red.log`.
  The existing cross-language supervisor test now exercises 21 modes, including
  semantic success, operation failure, cancellation, host cleanup rejection,
  swapped receipt, invalid exact result and current-source mutation. Native/SSH/
  runtime/resource boundaries are inert fixtures; real supervisor/archive/receipt
  logic is used. Both leases are checked inside child and independent host resource
  observations. Failure with proven cleanup releases leases; rejected host cleanup
  retains active evidence. Return-validation failures never become capture seals.
  Focused pass: `/tmp/bbr-visual-semantic-chain-green.log`.
- Bootstrap red: `/tmp/bbr-visual-semantic-bootstrap-integration-red.log`.
  Semantic source-bound generated zsh passes syntax validation and an actual host
  invocation refuses before runtime/mount/writes; direct guest-tail host refusal,
  fixed tail ordering and no-install flags are checked. VM helper group passes in
  `/tmp/bbr-visual-semantic-bootstrap-integration-green.log` (111 tests).
- Full `just visual-test`: `/tmp/bbr-visual-semantic-chain-visual-test.log`:
  **158 Python discoveries** (157 pass / one separately passed integration),
  unchanged upstream and **109 top-level consumer tests** plus existing signal/
  inert protocol fixtures. Python discovery6.559s, separate integration0.607s.
  Strict all-target consumer Clippy passes in
  `/tmp/bbr-visual-semantic-chain-clippy.log` (1.34s cached).
- Self-review adds stale/wrong-purpose/pre-cancelled no-workspace/no-lease tripwires,
  checks specific semantic/source/receipt rejection reasons and retains existing
  full/selected image and endpoint-refusal coverage. No old assertion is removed.
  Final follow-up also checks semantic frozen-loader host refusal and filter
  rejection; its focused/full rechecks are recorded in the next verification log.

No actual Gallery, VM, key/SSH/network or guest-runtime operation occurred, and no
current callback, pixel, baseline, actual cleanup or product-performance claim
follows. The ordinary user app, foreign worktrees and protected product/vendor/
image inputs remain untouched. Candidates stay uncommitted with the unverified
observer dependency; index is empty. This is not a stable-slice handoff or parity
claim. Implementation/review elapsed time was not measured.

Scheduling audit: same sole visual owner; no other lane or workflow authorized.
Next useful local action is a fresh actual host-only preparation plus independent
byte/feature/hash/bootstrap-plan inspection on these changed sources, without
running any Gallery executable or starting a VM. Actual semantic execution then
needs the specific recorded private-NAT/key-only SSH approval and a preavailable
guest runtime; no host/installation fallback. Check SVG approval and later
report-specific baseline adoption remain separate. No goal completion is claimed.

### Semantic route preparation and approval-bound stop

The fresh actual host-only preparation succeeds at
`/private/tmp/bbr-visual-semantic-route-prepare.Mw6A35`. It builds but never lists
or executes the feature-enabled Gallery harness. `host-build.json`, `prepare.log`,
`inspection.json` and syntax-only `bootstrap.zsh` are retained there:
- 1,511 sources / 1,513 payload files / 41,175,047 bytes;
  manifest SHA-256 `8d35f10724def69bbe9af9539a9d402de3d40776bc59f1fcb0d26d9520493a03`.
- Gallery harness `target/release/deps/bbr_gallery-cce0640bfb5ec5ef`:
  `c1f8127b66a1c28422617bd19e7db211027eb94fb3ae4945a249fb77417b8d05`.
  Standalone consumer: `560956264865694670c5f477ec912788d2905332737d618b875cb36f9269c7d5`.
- The independent Python inspector rechecks every staged byte/role and equality
  with host build hashes; the source-bound semantic plan contains49 cases.
  Generated bootstrap18,076 bytes, SHA-256
  `c484e9764d3f90b8e5a67c66c70c0ae5343baa568a4f4c10d753faabed0df50f`, passes `zsh -n`
  and no-realization/repair/fallback checks. It was neither executed nor sent over
  SSH. Release builds take2m16s Gallery /23.36s consumer (Cargo-reported).
- Host macOS27.0/26A428, arm64, Rust1.97.1, Xcode27.0/27A5252f, SDK27.0.
  These remain host observations, not guest runtime/rendering evidence. A following
  Rust doc-comment correction and this ledger update make the payload historical;
  prepare fresh again before any approved execution, never reseal it.

Final verification:
- `/tmp/bbr-visual-semantic-chain-reviewed.log`: complete `just visual-test` passes,
  including the final frozen-loader/filter regressions:158 Python discoveries
  (157 pass / one integration skip separately passed), unchanged upstream and109
  top-level consumer tests plus signal/inert fixtures. Python6.661s; separate CLI
  integration0.488s. Synthetic acceptance changes only disposable fixture stores.
- `/tmp/bbr-visual-semantic-verify.log`: full non-GUI `just verify` sequence passes,
  including49 shared script tests17.791s. The inspected
  `/tmp/bbr-visual-semantic-verify-i2wm8g6k.just` substitutes only compiler Cargo
  calls with mbx; its plan is `/tmp/bbr-visual-semantic-verify-plan.log`. No ignored
  GUI tests, weakened checks, target override or host native capture is introduced.
- `/tmp/bbr-visual-semantic-feature-gates.log`: strict testkit all-target Clippy,
  browser visual-capture-target Clippy and Gallery capture-feature all-target
  Clippy pass. Gallery feature units164 pass /49 ignored,0.50s. The ignored
  semantic scenarios remain unexecuted. Existing `block v0.1.6` warning remains.
- Consumer strict Clippy, workspace/consumer fmt and whitespace pass; separate
  script suite49/49 in `/tmp/bbr-visual-semantic-route-script-tests.log`19.248s.
  Roadmap: `/tmp/bbr-visual-semantic-route-roadmap.log`; rechecked after this ledger.
  Root Cargo/GPUI/vendor/baseline/image-config/actual image catalog and plan diff is
  empty. HEAD stays `b4c0795379f88d8193b5cf9c33da9ecd15a81ba6`; index stays empty.

Current bounded-step audit: fixed-action requirements, exact stdout/status evidence,
known diagnostic handling, strict installed-tree recheck, source-frozen bootstrap,
shared supervision/cleanup protocol, host receipt/result validation and actual
host-only preparation now have portable evidence. No actual guest test, callback,
Metal/input, current PNG/HTML review, normal/timeout/cancelled VM teardown, full
repeat or baseline adoption has occurred. The observer plus dependent49-case
catalog/action/route therefore remain deliberately uncommitted. Local green gates
cannot satisfy their required guest gate, and old native/Portal evidence is not
reused as current proof.

Scheduling/blocked-stop audit: this sole owner has no authorized parallel/product
lane. Repeating source audits or pure builds would add no missing runtime proof.
More input/readiness/framing code before the observer gate is explicitly deferred
by the pilot packet. Full matrices/browser repeat/review and Python parity removal
all depend on approved fresh guest execution. Later compositions/sequences are
sequenced after stable migration; product performance remains deferred. No safe
nonredundant local implementation step remains before that runtime boundary.

Minimum next input: Patrick's explicit approval for the disposable headless Tart
clone's private NAT and dynamic guest TCP22, reachable from the host/allowed NAT
peers, authenticated only by a run-specific SSH key, no public/bridge/forwarded
listener; clone-only listener/auth hardening, allowed/forbidden access audits and
clone/key cleanup are part of the run. Pinned guest dependencies must already be
available; actual absence or authentication failure stops without installation or
host fallback. Legitimate Check SVG source wiring is a separate approval for the
subsequent input/pixel pilot; concrete report/case baseline approval is later still.
Preserve all current files and foreign worktrees. No push, commit, network/VM action
or migration-completion claim accompanies this approval-bound stop.

### Authorized semantic guest run (2026-10-02)

Patrick explicitly instructed execution after the bounded private-NAT/key-only
SSH route was explained. This supersedes the preceding network-approval stop for
this semantic gate only: one disposable headless clone, run-specific SSH key,
clone-only hardening and access audits, existing supervised transport and verified
cleanup. No host GUI, guest dependency installation/repair, baseline adoption,
Check SVG change, other lane, subagent or publication is authorized by this run.

Continuation confirms the same session `01a0f8f4-1c46-71bf-9fae-3c394de53395`,
`main@b4c0795379f88d8193b5cf9c33da9ecd15a81ba6`, empty index and preserved
observer/semantic candidates. No index lock, Tart/build process or observed
capture-lock holder; the shared host gate is idle. The ordinary BBR process and
foreign worktrees remain untouched. The entire current delivery was reread in
bounded chunks. Queue audit selects the actual 49-case semantic gate; no additional
lane is authorized. Sources and binaries will be freshly prepared, then a small
external caller will invoke the existing Rust `vm_semantic::run` API with normal
host/compatibility gates. It adds no capture CLI, runner or acceptance authority.
Actual runtime or authentication failure stops without a fallback; all attempts
and resource-cleanup evidence will be recorded before any commit decision.

### Actual semantic VM attempts and runtime blocker (2026-10-02)

The network approval is now consumed by two real bounded attempts, not pending.
Fresh host payload and external API-only caller evidence is retained under
`/private/tmp/bbr-visual-semantic-approved.09DN2I`. The caller invokes only existing
`vm_semantic::run` plus original-identity recheck, normal host/compatibility locks
and signal cancellation; it adds no repository CLI or second execution pipeline.

1. First payload: 1,511 sources / 1,513 files / 41,175,051 bytes, manifest
   `456d43d1170a0606b7d309d51cd7a15c2eb90afa1f6acc1ed5da5b7c3fc33ec3`.
   Clone `bbr-visual-96867-a9c1986687c0`, guest `192.168.66.23`. Offline clone-only
   hardening and key-only SSH succeeded, but the authentication-output parser
   rejected normal repeated `listenaddress` and `hostkey` fields before bootstrap.
   The actual output is in `transport/vm/sshd-audit.log`; this is not a failed
   authentication or permission bypass. The regression reproduces the parser error
   in `sshd-parser-red.log` before the narrow repair.
2. Repair permits distinct values only for those two OpenSSH list directives;
   policy fields, unknown repeated fields, identical duplicates and malformed
   lines remain rejected. Authentication config, listener/access requirements,
   runtime policy and cleanup predicates are unchanged. The pilot packet records
   this bounded remediation. Policy/lifecycle 18/18 pass (`sshd-parser-green.log`),
   and the original recorded output now validates. Full `just visual-test` passes
   (`sshd-parser-visual-test.log`):159 Python discoveries (158 pass, one separately
   passing integration), unchanged upstream and109 consumer tests plus fixtures.
3. Fresh retry under `retry-sshd`: 1,511 sources / 1,513 files / 41,176,366 bytes,
   manifest `34c42f73f11358c5fd80817553bdc34ccc8f71396c617fc8b71ad5bf43f666f4`.
   Clone `bbr-visual-64490-aa0dae8dae38`, guest `192.168.66.24`. Effective
   authentication and listener audits pass; host probes reach TCP22 and reject
   TCP88/5900. Fixed source-bound bootstrap then fails its offline pinned-runtime
   check because `/nix/store/jspv3c5l2zx4kiwzhq0zgxcwp34cqifz-libiconv-115.100.1`
   does not exist in the guest (`retry-sshd/transport/bootstrap.stderr`).
   No dependency realization, download, repair, share mount, installation,
   Gallery listing/test execution, image production or baseline change followed.

Both attempts exited2 and completed owned cleanup: stop/delete exit0, owned Tart
process reaped, key deleted, clone/disk absent and former SSH unreachable. Each
supervised failure had a request-bound failed terminal and passed independent host
resource recheck before the shared host/compatibility gates returned idle. Exact
records are each run's `transport/vm/cleanup.log` and `cleanup-observed.json`.
Golden Image and ordinary host BBR remain untouched. This proves two real
failure-path teardowns and the retry's network policy, not successful semantic
execution, normal capture, timeout/cancellation teardown, Metal or visual parity.

The parser fix is committed independently as `5992f5af` (only the policy parser
and its regression), runtime-verified through the second authentication/listener
audit. Final roadmap/whitespace checks pass, and both cleanup records were
independently rechecked again in `final-cleanup-recheck.json` under the run root.
The observer and dependent semantic changes
still lack their actual49-case gate and remain uncommitted. No acceptance box is
checked. Scheduling audit: no other lane is authorized; further input/framing
work still depends on that observer gate. The remaining blocker is concrete guest
runtime absence, not further network permission. Do not retry unchanged or install
against the no-install contract; a separately authorized compatible golden/runtime
preparation is required before another fresh run. Test durations in logs are
tooling measurements; total implementation/review time was not instrumented.

### Offline runtime remediation resumed

Patrick requested “fix alles” and then “continue” for the explained offline,
byte-bound runtime transfer followed by the49-scenario guest gate. The earlier
preinstalled-only precaution is superseded for this prerequisite. The bounded
[pilot contract](../roadmap/tasks/gallery-visual-pilot.md#authorized-offline-runtime-provisioning)
permits only already verified local closure export and a single offline import
into the hardened disposable clone. No host/Golden Image write, download,
substitution, repair, extra network route or baseline acceptance is involved.

Continuation preserves `main@5992f5af`, the empty index and all observer/semantic
candidates. Entire delivery document reread; no other lane or agent is authorized.
Ordinary host BBR remains untouched. Queue audit selects the smallest real
runtime-transfer → existing bootstrap → semantic execution path; downstream
input/framing and full image coverage still depend on its evidence. Existing
request-bound supervisor, authentication/listener audits and cleanup remain the
only transport authority. The new transfer tests first failed for absent module/
transport support, then passed after implementation; one intermediate test edit
had a missing Python continuation slash and was corrected. Logs are under
`/private/tmp/bbr-visual-runtime-fix`. Fresh staging and a real clone will now test
the dependency boundary; no successful import or semantic test is yet claimed.

### Verified semantic runtime checkpoint (2026-10-02)

The bounded semantic prerequisite now passes actual guest execution. Evidence is
under `/private/tmp/bbr-visual-runtime-fix`; this checkpoint supersedes the earlier
runtime-absence and unexecuted-observer stops, not the overall migration gates.
One local writer, preserved `main@5992f5af` candidates, empty pre-commit index;
no agent, foreign worktree, baseline, product graph, vendor or host GUI change.

Runtime remediation and measured attempts:
- `attempt1`: exported51 already verified locked paths,1,492,282,464 archive bytes,
  then imported once offline into the hardened clone. All original NAR/Python
  checks passed. Console parser rejected macOS27's direct-root plist. Its exact
  shape was reproduced with a read-only host `ioreg` query and a negative-first
  test. One-element-array support and unlocked-admin requirements are preserved.
- `attempt2`–`attempt4`: diagnostic-only progression identified an empty user
  `HomeManager` directory plus the exact hidden version-marker link, not extra
  fonts. A negative-first contract permits only those inert entries without
  following the marker; populated directories and other links stay rejected.
- `attempt5`/`attempt7`: system-font diagnostics identified only the stock Arial
  Unicode alias. Exact destination and directly inventoried identical bytes are
  required; arbitrary links/chains are still rejected. No font installed/deleted.
- `attempt6`: enclosing command reported137 after the25m outer timeout. The
  supervisor request was created late in the invocation; no runtime export or
  bootstrap began. The recorded cancellation was cooperative: terminal cancelled,
  clone/key/process/disk removed and SSH unreachable, independently rechecked.
  No hard-kill teardown claim. The external API caller now bounds gate waiting
  explicitly to30s; production gates and operation/cleanup budgets are unchanged.
- `attempt8`:49/49 actual tests passed, including the new observer, and the exact
  archive returned after cleanup. Host receipt validation correctly refused the
  empty `system_profiler` display array. No weakened empty-array acceptance.
- `attempt9`: a bounded pinned-Python CoreGraphics/Metal probe observes the actual
  virtual display and device without AppKit/window/capture creation. Complete
  original-payload/source/build binding, semantic result, report-only diagnostics,
  before/after environment, installed bytes and cleanup now pass. API status is
  `verified`, exit0,96.22s complete caller /5.08s tests (tooling timings only).

First fully verified run identity:
- Payload `2e3458ab9e9ce373851bce04333ce432e0a9f5243b7437cd13af26c2b740a6cc`;
  Gallery harness `3c0ef7ad9813e3128eeac9f473d530e792aaf3783d9f4b85d100fe3e8ed935c8`;
  consumer `79f00227aee6e2b247886d9f2d00e50cda64cfc347ae04efb442d066ebabae1b`.
- Clone `bbr-visual-14773-b1df688e3ce8`, former IP192.168.66.33. Guest macOS27.0 /
  26A5353q, VirtualMac2,1,3200×1800 logical /6400×3600 physical, Apple Paravirtual
  Metal device,444 font hashes, unlocked admin session, UTC/C. Host compiler/SDK
  facts remain separately recorded. The observed scale is not fabricated from
  the Tart display argument. This is device availability, not native image proof.
- Offline export SHA256 `7ac6ea3058ebb31f87b1e87733b8bd9a2da1ebb591f2e3d632f146608c4be9e4`;
  exact51 import paths and original lock/payload hashes in runtime-transfer.json.
  No substitution/builders/downloads, host store or Golden Image modification.
- Authentication/listener/host-port audits passed on each executed transport;
  request-bound terminal and independent cleanup preceded lease release.
  `cleanup-recheck-1-9.json` repeats all nine resource-absence observations.
  There are no retained visual clones/keys or active visual gate. Ordinary host
  BBR and unrelated workloads remain untouched.

Final portable evidence:
- `final-visual-test.log`: full `just visual-test`,171 Python discoveries (170
  pass / one integration skip separately passed), unchanged upstream and109
  consumer tests plus existing signal/inert fixtures. Includes export/hash/type/
  bound/host-refusal, exact SSH import path inventory, failure/cancel cleanup,
  console/font/display negatives and21-mode supervised chain. No runtime flag
  bypass or acceptance guard was added.
- `final-consumer-clippy.log` and `final-feature-gates.log`: strict consumer,
  testkit, browser-capture and Gallery capture-feature Clippy pass. Gallery
  feature units164 pass /49 ignored on host; their49 actual executions are in
  the guest result, never host GUI. Format/roadmap/whitespace pass; unchanged
  `block v0.1.6` future-incompatibility warning remains.
- Full mbx-equivalent `just verify` is **not green**. Two actual attempts fail
  only at the unchanged Command benchmark with p95=67.988ms /93.215ms against
  50ms. Logs: `final-verify.log`, `final-verify-recheck.log`. Observed host load
  averages were286 then109; concurrent unrelated compiler/simulator workloads
  were not stopped. The isolated unchanged benchmark passes p95=19.686ms,
  max=21.208ms (`benchmark-isolated.log`); this does not erase aggregate failures.
- Remaining non-GUI checks pass in `final-remaining-verify.log`. Supplementary
  workspace tests excluding the already separately tested Command crate pass in
  `remaining-units-configured.log`; this is not advertised as full verify. A first
  direct invocation lacked the recipe's Xcode selection and failed compiling Metal;
  rerunning through the existing configured recipe environment resolved it without
  any installation or setup change. No assertion/budget was relaxed.

Self-review rechecked original-source/hash retention, fixed-purpose invocation,
full49-name inventory, monotonic draw tokens/sole-child callback, feature-off
behavior, exact diagnostic relocation and strict installed-tree verification,
source-frozen bootstrap, offline import argv, guest-only native observations and
success/failure/cancellation cleanup ordering. No independent agent was requested.
The now guest-verified observer and dependent semantic/runtime group can be
committed as an intermediate coherent slice; full stable-migration handoff is not
claimed with the aggregate verify failure or absent image gates.

Queue audit: runtime absence is resolved; the next migration work is the bounded
Checkbox input/two-clock readiness and Tokens/Sidebar framing pilot. No unrelated
product lane is activated. Check SVG wiring and concrete baseline adoption remain
separate from this semantic evidence. Native pixels, PNG/HTML inspection, two
fresh complete exact captures, public capture cutover, stable functional aggregate
and Python parity removal remain unproven. No completion checkbox, performance
pass, push or publication. Test/transport durations above are measured; total
implementation/self-review time was not instrumented. Subsequent edits require
fresh payloads; no old image or semantic result is resealed as current capture.

Final source-bound recheck: `attempt10` freshly prepares and verifies all1,513
current source inputs after the final transport-inventory regression and Rust
observation comment. No source changed after staging; independent post-run
hash checks are in `/private/tmp/bbr-visual-runtime-fix/attempt10/final-inspection.json`.
Payload `893cb26302d245ec92fd4f704d6a4869ebc01193c96f8e36a8865f142242c8d7`,
consumer `560956264865694670c5f477ec912788d2905332737d618b875cb36f9269c7d5`,
unchanged Gallery executable above. Exact49/49 result passes in5.38s; full caller
119.54s, API exit0/status verified and original-identity recheck pass. Fresh clone
`bbr-visual-7443-b3929582bda0` /192.168.66.34 repeats offline runtime import,
source/installed-byte and before/after environment checks. Independent cleanup
recheck again proves clone/key/disk/process absent and SSH unreachable; both gates
are unheld. These are two successful semantic runs, **not two image captures**.
Only documentation changes after this final source check. The aggregate benchmark
failures above remain unresolved; intermediate commit does not claim full verify,
stable-migration handoff or visual acceptance. No further unchanged aggregate retry
is useful while host contention persists.

### First static Tokens image report (2026-10-02)

Patrick's “lets go” activated the proposed first partial Tokens Light/Dark
report. Base is `299dc64408c7fdc1b486b64bb9f68cb995c50a53`, the committed
semantic/runtime checkpoint above. Only this local writer is authorized; the
complete delivery queue was reread, with no additional lane, agent, foreign
worktree, product change or publication activated.

The independent Tokens-only story catalog now maps story 0 to all six existing
standard/increased × small/normal/extralarge cases, both appearances and ten
ordered children per case. No persistent selection/default filter, public capture
activation, endpoint guard removal or change to the historical 936 required pairs
and 31 withheld requirements. A fixed external caller reuses `vm_capture::run`
and upstream sealing/comparison through the existing hardened Tart transport.

Actual first run:
- Report: `/Users/patwoz/dev/Personal/macos/bbr/reports/visual/tokens-pilot-20261002-a/index.html`.
  Sealed run: `/Users/patwoz/dev/Personal/macos/bbr/reports/visual/gallery-runs/tokens-pilot-20261002-a`.
  Preparation, transport and verification evidence: `/private/tmp/bbr-visual-tokens-pilot/attempt1`.
- Twelve actual guest Metal PNGs, each 2400×2000 RGBA8. Schema 2,
  `selected-development`, six complete pairs; upstream review has exactly twelve
  `missing-baseline` results, no validation errors and no environment mismatch.
  Review remains false, with no baseline writes; partial runs cannot be accepted.
- Payload SHA256 `af480208076a7e2d5e65ff2cc845cea6d80b4748e3fcf376843052eaa28dd787`;
  capture SHA256 `d50c8114f1dea8e66fb628aaef1f939f8d67b23fbcc74ce6388a87140f906301`.
  Caller exit 0, 286.428s outer measured duration; this is tooling, not product
  performance. `final-inspection.json` independently rechecks all 1,515 source
  hashes, twelve sealed PNG hashes and twelve unchanged report copies.
- Clone `bbr-visual-45196-b56cb7c3b3e8`, former IP 192.168.66.35: request-bound
  teardown and a later independent recheck prove clone/key/process/disk absent
  and SSH unreachable. Both capture gates are unheld. Offline import and original
  before/after guest/source/build checks passed; host and Golden Image unchanged.

All twelve original PNGs were opened and visually inspected: five labeled material
swatches and three elevation-description rows are legible in Light/Dark, with no
observed truncation; increased contrast strengthens borders/text and the three
text densities remain visible. Elevation is descriptive text, not actual shadow
samples, so no shadow/paint-framing proof is inferred. The original HTML was opened
through existing managed Chrome-patwoz. Its rendered banner says TEILLAUF, twelve
missing references are visible and adoption is explicitly blocked. All twelve
unique local images load at their original dimensions. A broad DOM check initially
counted empty/lazy hidden image elements as broken; explicit loading of the twelve
nonempty sources found zero failures. Screenshot evidence is
`/private/tmp/bbr-visual-tokens-pilot/report-viewport.png`. No report bytes changed.

Verification under `/private/tmp/bbr-visual-tokens-pilot`:
- `catalog-red.log` fails for the intended missing configured story catalog;
  `catalog-green.log` passes the focused selection/discovery contracts.
- Ordinary `just visual-test` stopped in two unchanged upstream 250ms process
  tests because their expected child PID files were absent. Host load was 365.66;
  this is observed contention, not a proven root cause. `final-visual-test.log`
  preserves both failures. One serial recheck, `RUST_TEST_THREADS=1 just visual-test`,
  passes every suite with unchanged assertions/deadlines: 171 Python discoveries
  (170 pass, one integration skip separately passed), immutable upstream and 111
  top-level consumer tests plus existing fixtures (`final-visual-test-serial.log`).
- Strict consumer all-target Clippy and formatting pass (`final-consumer-clippy.log`,
  `final-fmt.log`); roadmap and whitespace are checked before the local checkpoint.
  No source-bound file changed after capture; following edits are documentation.

Self-review checked authored selection completeness, unknown/duplicate story
refusal, current-source binding, partial nonacceptance and unchanged renderer,
upstream, plan, guards and baselines. This closes only the first-static-report
prerequisite, not a pilot/migration milestone. One image run is not an exact repeat.
Checkbox assets/input/two-clock readiness, Sidebar framing, complete Gallery/browser
captures, stable functional aggregate and Python parity removal remain open. The
preceding aggregate Command benchmark failures remain unresolved; no full verify,
performance or baseline approval is claimed. Implementation/review wall time was
not measured. Next migration work remains the bounded interactive/framing pilot;
this report does not activate unrelated queue items or additional writers.

### Real application stills checkpoint (2026-10-02)

Patrick requested Sidebar and the actual application, including History and
Settings, instead of Gallery alone. This bounded follow-up starts from clean
`main@89623860314ab189fb792fc6306325b60bd45732`. Only the existing local visual
writer is authorized. The complete delivery queue was reread at restoration;
no additional product lane, workflow, subagent or foreign worktree is activated.
The tooling packet records the narrow application/protocol extension before
implementation. Production rendering, GPUI pin, immutable upstream and all real
baselines remain unchanged.

The optimized real application now emits **28 Light/Dark stills**: the original
eight ordinary/representative browser and Sidebar cases, populated History,
Clear Browsing Data (cancel only), empty Saved Pages and seven Settings panes.
Fixed undated fixture visits avoid clock-dependent labels and real user data.
Production actions and typed readiness drive each surface. The feature-only,
authenticated `CaptureSettings` reads the existing Settings window's own renderer,
explicitly drawing it first; closed Settings and nested output paths are rejected
with exact error codes. Semantic state is preserved across readback. Browser,
Sidebar and Settings dimensions remain exact RGBA8, respectively 2160×2814,
608×2814 and 1560×1240, without resampling or masking. The independent catalog
requires all 28 and explicitly withholds remaining application coverage.

Evidence root: `/private/tmp/bbr-visual-app-surfaces`.
- The first existing-browser transport stopped at `runtime probe deadline`.
  A bounded read-only runtime diagnostic subsequently passed; its Nix byte check
  took 16.016s. Host load was high, not a proven timeout cause. Fresh transport
  then passed ordinary Light/Dark but failed the representative `pinned=1`
  assertion. Existing `favorite => pinned` requires all four Favorites pinned;
  the correction asserts both count four and every tab pinned, without product
  changes or removing checks. Both failed attempts retain logs and cleanup.
- Fresh expanded `attempt2/transport` succeeded: run `app-surfaces-20261002-a`,
  caller exit0, 481.484s. An initial full contract gate then caught the remaining
  synthetic expected count eight; it was corrected to 28, not skipped. Expanded
  portable tests also reject a missing Settings image and browser-sized Settings
  output. Exact error-code assertions and rendering metadata were strengthened.
- Fresh final payload under `attempt3` binds 1,516 sources. Its first transport
  failed at the SSH listener audit (`Connection timed out during banner exchange`)
  and cleaned up before producer execution. New workspace `attempt3/transport2`
  succeeded, caller exit0, 110.547s. No changed deadline, audit bypass or in-place
  retry. All timings here are tooling, not product-performance measurements.
- Final report:
  `/Users/patwoz/dev/Personal/macos/bbr/reports/visual/app-surfaces-20261002-c/index.html`.
  Final sealed run:
  `/Users/patwoz/dev/Personal/macos/bbr/reports/visual/runs/app-surfaces-20261002-c`.
  Scope `collection`, review false, baseline writes false. Eight changed historical
  references, twenty missing baselines and environment mismatch remain review
  requirements; suite validation errors are empty. Collection coverage additionally
  fails on the explicit `remaining-app-surfaces` withheld requirement.
- Final payload SHA256 `4f31bdbd5607a86f3ec3006ce02abd0bb2b33196887a4601ddbf05fb26866b04`;
  capture SHA256 `746bedaf56c64dc010d09c8b73f305d6575f3f6d33c8a9d736117527f1a18014`;
  request SHA256 `f4f0e295ea96e1ebeead7e739a2fc1f6567133582dcea8b1456a9013bfd2c5a7`.
  Final clone `bbr-visual-49032-bcc4797564b0`, former IP192.168.66.40, Tart PID51335.
  Existing offline closure import, authenticated/private-NAT audits, original
  source/build identity and before/after guest checks pass through the same route.
- `final-inspection.json` independently rechecks all 1,516 current source hashes,
  28 original PNGs, 28 report actual copies and all 46 collection-sealed files.
  The review's capture hash matches the original seal. All 28 PNGs match expanded
  run `a` byte-for-byte; **source seals differ** across those runs. This does not
  satisfy the two-complete-captures-of-one-final-tree migration acceptance gate.
- The same independent inspection repeats clone/key/process/disk absence and
  unreachable SSH for all five transports, including the three failures. Both
  capture locks were observed unheld; no lock metadata was erased. Golden Image,
  ordinary host app and other workloads/worktrees remain untouched.

All 28 originals have been opened and visually inspected, and the actual static
HTML is open in the existing managed Chrome-patwoz report tab. The report's 44
unique nonempty image sources load, including all 28 actual images at exact
original dimensions. Sidebar, History and Settings content is legible. The Clear
Data sheet has a conspicuously tall empty body in the production render; Shortcuts
shows only its initial scroll viewport (the next section reaches the bottom edge).
Neither is cropped, masked, repaired or promoted to full-content/paint-framing
acceptance. Native WKWebView, traffic lights and Portal-child pixels remain absent;
ordinary browser content is therefore blank, not a screenshot of a webpage.
Populated Saved Pages, recently closed History, Settings menus/sheets/scrolled
content, Command/Find/Downloads/Peek portals, permissions, welcome, About and other
transient states remain explicitly uncaptured. This is not the complete app.

Verified checks under the evidence root:
- Negative-first catalog mismatch (`catalog-red.log`), then passing independent
  required-pair contract (`catalog-green.log`).
- `RUST_TEST_THREADS=1 just visual-test` passes twice after the count correction,
  including the final missing/wrong-sized Settings fixtures (`visual-test2.log`,
  `visual-test-final.log`): 171 Python discoveries (170 pass, one integration skip
  separately passed), unchanged upstream and 111 top-level consumer tests plus
  fixtures. The earlier count failure remains in `visual-test.log`.
- Bounded mbx strict Clippy: app visual-capture target (`app-clippy-final.log`),
  consumer all targets (`consumer-clippy-final.log`) and ordinary app testkit all
  targets (`testkit-clippy-final.log`) pass. No lint suppression. The first app
  build's by-value WindowHandle call correction is retained in `app-clippy1.log`.
- Both capture protocol wire contracts pass (`protocol-tests.log`); the pure
  feature-on output-path regression passes (`capture-path-test-final.log`). No
  ignored GUI test ran on the host. Existing `block v0.1.6` warning remains.
- Final workspace/consumer format, roadmap/index-roadmap and whitespace checks
  precede the local checkpoint. Only documentation changed after final capture.

Self-review checked actual-window routing rather than reconstruction, explicit
draw without a leased Root, feature-off refusal, isolated deterministic history,
no deletion/settings mutation, before/after state, complete independent inventory,
wrong-window dimensions, unchanged PNG bytes, retained native exclusions and
source/cleanup binding. No independent agent was requested or used. This is an
intermediate tooling checkpoint: no new full `verify`, stable functional aggregate,
performance pass, baseline approval, migration closeout or Python removal is
claimed. Earlier aggregate Command benchmark failures remain open. Public capture
and Gallery endpoint guards are unchanged. Total implementation/review wall time
was not instrumented. The broader input/framing/parity work remains open; further
capture-affecting source changes require fresh staging rather than stale resealing.
No push/publication or additional writer accompanies this checkpoint.

### Acceptance/coverage correction activated (2026-10-02)

Clean starting checkpoints: BBR `f26335b4a290d7d0ff1c7e08c2bcd2a95ac25c9e`,
standalone tool `c2723cac924b98b8a2c8c170d580bd2e434b674a`. Same sole local
visual writer; complete queue reread, no agent/lane/worktree/publication activated.
Current runnable work is the requested acceptance bug and screenshot proof;
unrelated product work, broader Gallery pilot and Python cutover remain deferred.

Patrick approved the exact `app-surfaces-20261002-c` 28 images and Tart environment.
The official unchanged `accept-collection` refused `Cannot accept suite with invalid
coverage/comparison: browser`; `/private/tmp/bbr-visual-app-acceptance` retains the
selection, pre-state and JSON exit2. Baselines/report stayed unchanged, no attempt
journal was created. The only collection error is the deliberately withheld
`remaining-app-surfaces`; reviewed comparison errors are empty. This is not missing
user consent and must not be solved by hiding coverage or bypassing the collection.

The updated tooling packet bounds an upstream-first exact withheld acknowledgement,
negative-first contracts and strict gates, acceptance of the still-valid approved
report before changing source-bound tool inputs, immutable import, then fresh Tart
captures and actual report screenshots. Defaults, all other rejection rules and
red coverage remain intact. No new/changed image is implicitly approved. Existing
benchmark failures and native/full-app exclusions remain open. Stop on real failed
verification or integrity/runtime uncertainty with original attempts preserved.

### Exact approved reference adoption verified

Standalone `4743437699567ba16fdcabf8fc12010a367f9134` implements the bounded
opt-in; 183 portable results, strict all-target Clippy and fmt pass. Before any
consumer tool-source change, the retained verified CLI revalidated all 1,516 BBR
sources, 28 original/report image pairs and 46 report-sealed files against the
approved capture `746bedaf56c64dc010d09c8b73f305d6575f3f6d33c8a9d736117527f1a18014`
and collection seal `938d87a3d63bdec919542f2b095582d11f00c2365a8fb0bb38e11384a03b8184`.
The baseline pre-state still equals the failed attempt's retained pre-state.

Official collection acceptance now succeeds (exit0, 1.693s tooling time) with all
28 exact `browser:case` IDs, explicit `--adopt-suite browser` and only
`--acknowledge-withheld browser:remaining-app-surfaces`. Its schema-2 journal is
complete for browser; every accepted PNG is byte-identical to its approved original
and the environment equals that capture exactly. Eight references replaced, twenty
added; the upstream normal schema-2 writer deduplicates historical source maps.
No image conversion, report mutation, manual copy or acceptance-time capture.

Fresh stored-run comparison at
`/Users/patwoz/dev/Personal/macos/bbr/reports/visual/app-surfaces-accepted-20261002-a/index.html`
proves 28/28 exact, zero comparison errors and no environment mismatch. Overall
coverage correctly remains FAIL with exactly the unchanged withheld row, exit1.
This is a new comparison of the approved capture, **not a new guest capture**.
Evidence: `/private/tmp/gpui-visual-withheld/bbr-{before,accept,post-adoption}.json`.
Roadmap and whitespace checks pass. This adoption slice is committed as
`ee409d0e73399ea10ca42468a9d2ba6c5283dbef`. The following checkpoint records
the subsequent immutable import and fresh guest screenshots; no migration,
full-app or performance pass follows.

### Immutable acceptance-fix import and screenshot proof

The complete unchanged **84-file** standalone export at
`4743437699567ba16fdcabf8fc12010a367f9134` replaces the prior immutable import.
All upstream/export/vendor/provenance hashes match; LICENSE/NOTICE and prior SDK
provenance are preserved. No consumer implementation, product renderer, dependency,
image format, tolerance, coverage catalog or accepted baseline changes accompany
this import. Default Collection acceptance stays strict. The new repeatable
`--acknowledge-withheld` opt-in requires the exact withheld bases of the selected
or adopted suites, never grants image selection or environment adoption, and never
makes coverage pass. Opt-in attempt journals retain those IDs, including interrupted
attempts; ordinary journals retain their existing schema.

Fresh runtime evidence under `/private/tmp/gpui-visual-withheld/attempt1`:

| Run | Caller duration | Capture SHA-256 |
|---|---|---|
| `app-acceptance-fix-20261002-a` | 158.983s | `526cbd73e81404ace453ba387ef2165f7bd90021cf42eb5990e4d704ce6ef23b` |
| `app-acceptance-fix-20261002-b` | 102.483s | `7a6c8ce1fb0034e1d6ff8da20bea3186ca4853711a8b9ab43edf67eda3f17e20` |

Both use one freshly prepared payload
`cd7c84ee2cc438a807b25c4fe6d832b78c02c92c861ad5879b173fe766b04723`
and the same 1,517 source hashes/cases/policy. Both callers exit0 with 28 images,
`review_passed:false` and no baseline writes. Every original/baseline/report PNG
matches exactly; **all 28 images are byte-identical between the two fresh runs**.
Independent checks verify every source and all 86 sealed files of each report.
These are new optimized real-app Tart captures through existing `vm_capture::run`
and review APIs, not a replay of old raw output or acceptance-time recapture.

Reports:
- `/Users/patwoz/dev/Personal/macos/bbr/reports/visual/app-acceptance-fix-20261002-a/index.html`
- `/Users/patwoz/dev/Personal/macos/bbr/reports/visual/app-acceptance-fix-20261002-b/index.html`

The only environment differences versus the approved reference are
`/host_build/tool_revision` (`c2723ca` → `4743437`) and
`/guest_rendering/displays/0/metal/registry_id`: approved4294967700,
A4294967701, B4294967713. All other observed values match. Both reports correctly
retain environment mismatch and the unchanged `remaining-app-surfaces` failure,
despite 28 `same` results and zero comparison errors. No registry normalization,
new environment adoption or false full-suite pass. Same-tree image repeatability
is now demonstrated for this bounded matrix; same-environment/full-migration
acceptance is not. Redesigning environment identity is separate bounded work.

Transport A owns clone `bbr-visual-50559-d017eae32b58`, former192.168.66.41,
Tart PID50975, request
`397e6f79bc595bfbb54c9cf12dd6d326c84b0ac91d08ea1b01687451ff47f679`.
Transport B owns clone `bbr-visual-63323-d07298a48b38`, former192.168.66.42,
Tart PID63794, request
`256bcd5c734217b460b475bba5e448423bf1ac2348caaa0753bafe5b0d839980`.
Both repeat offline runtime transfer, private-NAT/key-only SSH/listener audits,
installed source/build checks, before/after observations and supervised teardown.
The independent host recheck proves clone/key/disk/process absence and unreachable
SSH for both; both capture gates are unheld. Exact report seals and cleanup records
are in `/private/tmp/gpui-visual-withheld/final-inspection.json`. Ordinary host app,
Golden Image, other workloads and foreign worktrees remain untouched.

Seven actual managed-Chrome screenshots, each3344×1926, are retained at
`/Users/patwoz/dev/Personal/macos/bbr/reports/visual/evidence/acceptance-fix-20261002/`:
- `approved-report-after-acceptance.png`: stored-run 28 exact, coverage-only FAIL.
- `fresh-report.png` and `repeat-report.png`: fresh 28 exact, environment and
  coverage still FAIL.
- `sidebar-light-dark.png`, `history-light-dark.png`, `settings-light.png` and
  `settings-dark.png`: rendered production-image sections in the original report.

The existing pinned task tab now displays report B. The nonempty reference/
actual image sources of report A loaded without errors. Report screenshots and
repeat originals Settings Appearance Dark / History Light were opened for visual
inspection. Browser screenshots preserve red verdicts; scrolling alone changes
viewport, not HTML/assets/styles. A first batch JavaScript quoting error produced
no screenshots; guarded stdin evaluation corrected it without changing reports.
Original PNGs remain unmodified, including the documented native/viewport gaps.

Verification under `/private/tmp/gpui-visual-withheld`:
- Upstream missing-option negative-first regression (`red.log`), then28 Collection
  tests (`collection.log`) and183 passing results (180 top-level plus3 child
  fixtures; `tests.log`); strict all-target Clippy/fmt pass.
- Imported `RUST_TEST_THREADS=1 just visual-test` passes (`bbr-visual-test.log`):
  171 Python discoveries (170 pass, one integration skip separately passed),
  upstream contracts and111 top-level consumer tests plus existing fixtures.
- Both imported-tool and consumer `mbx clippy --locked --offline --all-targets
  -- -D warnings` pass (`bbr-upstream-clippy.log`, `bbr-consumer-clippy.log`);
  both Cargo-root fmt checks and roadmap pass (`bbr-import-roadmap.log`).
- Final documentation-only updates are outside capture source policy.
  `/private/tmp/gpui-visual-withheld/precommit-inspection.json` rechecks all84
  upstream/export/vendor/provenance hashes, unchanged LICENSE/NOTICE and adopted
  baselines, the original report seal, both1,517-source/28-image/86-file fresh
  reports and all7 screenshot hashes/dimensions. Read-only doctor reports1,517
  inputs with no producer/probe. An initial manual inspection expected a boolean
  environment mismatch; its stored schema contains both environments. The
  corrected check requires those exact objects, without changing stored evidence.
  Final roadmap and whitespace pass (`bbr-final-roadmap.log`); index roadmap and
  both Cargo-root formatting are rechecked before local commit.

Scheduling/self-review audit: entire delivery document reread at restoration;
only this local acceptance-fix/import/evidence slice is authorized, no additional
lane/workflow/subagent, other-consumer write or publication. Self-review checks
strict defaults, exact acknowledgement sets, retained invalid coverage, original
approval versus new environment drift, immutable import and image/source/resource
identity. The requested acceptance error is resolved and screenshot proof exists.
Broader Gallery/pilot, complete app/native-state coverage, public guards, Python
cutover, unchanged Command benchmark failures and stable functional aggregate
remain open. No completion checkbox, full verify/E2E/performance or independent
agent-review claim. Measured durations are tooling only; implementation/self-review
wall time was not instrumented. No additional unchanged aggregate retry or silent
environment adoption accompanies the local checkpoint.

### Rendering identity/provenance correction verified (2026-10-03)

Patrick authorized “ok mach das” for the bounded environment correction, explicit
metadata migration and two fresh supervised Tart runs. Starting BBR checkpoint
`ca67fd02dc9a21440ac4130226514b926f7595d1`; same sole local visual writer,
no subagent/workflow/other-consumer/foreign-worktree/publication scope. At restoration,
HEAD, empty pre-stage index, preserved candidates and upstream clean commit were
rechecked. The entire delivery queue was reread in bounded chunks. Only this
correction's documentary closeout and commit are runnable under the current request;
other product lanes and broader migration remain deferred, not newly activated.

Portable authority is committed upstream as
`e3050c78cc1347ea0164218d7bcd0d74addcdb8a`. The complete immutable **90-file**
export, tracked set, provenance and unchanged LICENSE/NOTICE are verified in BBR.
It includes the prior `ce4ff11` compact-navigation layout regression correction.
The comparison-schema-1 envelope is explicit and reversible, not a generic ignore
list. Legacy JSON remains fully exact. Layer, contract, rendering fields and the
provenance pointer set compare exactly; whole capture before/after identity remains
strict. Sealed JSON and original HTML preserve separately classified differences;
acceptance recomputes them and never refreshes historical provenance implicitly.

BBR authors `bbr-tart-rendering-v1`: only validated guest Metal registry instance ID
and complete tool revision become provenance. All other observed facts stay exact.
Nine source-bound capture/codec/execution/dependency fingerprints retain meaningful
tool-change detection. Native display validation rejects unknown/malformed fields,
missing device facts and invalid geometry. New capture modules require reviewed
contract/list changes. No product/native producer, GPUI pin, pixels, dimensions,
coverage catalog, baseline PNG, budget, tolerance or mask changes accompany this
correction.

Official `partition-environment`, after inspected dry-run and exact old-hash guard,
changes only the baseline environment representation:
- Before SHA256: `fac87538eec85882e9781b9ece83541d06444ceaee20ff41c252447cde0233c1`.
- After SHA256: `593f2ad806c8065915c7b5247e74423bbe4250a05ffe3b69f878be5981488bc4`.
- All nine rendering fingerprints derive from historical source seals and match
  the current rendering inputs. Original tool `c2723ca` and Metal `4294967700`
  reconstruct losslessly; every non-environment metadata value is preserved.
- Independent preservation check: 677 files, 676 byte-identical, only the explicitly
  authorized metadata file changed. Approved PNGs and historical reports/seals
  are untouched. Old reviews intentionally become stale against new baseline
  metadata; no old report is edited/resealed and no new image is accepted.

Fresh runtime evidence under `/private/tmp/gpui-visual-environment/attempt1`:

| Run | Caller seconds | Capture SHA256 |
|---|---|---|
| `app-environment-20261003-a` | 84.579018083 | `d08be997c8bdf91a3e05bddbfcb8a191ef04abb6f311c7a6ddad1719e585f32b` |
| `app-environment-20261003-b` | 81.569718791 | `1fafc8dc4ad8d04a9ef3039fbda97c213c03dc5020e10772450df7f8f7f9e0c9` |

Both use one fresh 1,526-file payload
`95b38ca92bf1a8bdc119d5dfb4598349ef914f063e2221da20b37ce80261d323`
and identical **1,523 source hashes**. All **28 PNGs match references and each other
byte-for-byte**; zero changed pixels/comparison errors, `environment_mismatch:null`
and `review.passed:true`. Each Collection/suite remains false solely for the same
withheld `remaining-app-surfaces` requirement. Captured tool `e3050c7`, Metal A
`4294967701` and B `4294967703` remain explicit provenance in the sealed reviews
and original HTML. No environment/image adoption accompanies those captures.

Reports:
- `/Users/patwoz/dev/Personal/macos/bbr/reports/visual/app-environment-20261003-a/index.html`
- `/Users/patwoz/dev/Personal/macos/bbr/reports/visual/app-environment-20261003-b/index.html`

Independent inspection verifies all payload/source/installed bytes, baseline-state
bindings, 86 sealed report files per run, before/after guest observations and
request/payload-bound completed terminals. Both transports verify the original
51-path offline runtime, key-only SSH/no forwarding, only SSH listeners and closed
ports 88/5900. Read-only resource rechecks again prove both exact clones, keys,
disks and owned processes absent and former SSH unreachable. Both host leases
have no observed holder. Host app, Golden Image and unrelated workloads are
untouched. Timings above are tooling, not product-performance evidence.

Verification and retained attempts under `/private/tmp/gpui-visual-environment`:
- Early runtime preflight succeeds before implementation: prepare 398.67s,
  caller 156.876s, 28 exact images/coverage red and independently absent clone.
  A scratch caller's `clap_derive` toolchain error is resolved through the pinned
  upstream wrapper, without target/cache overrides.
- Upstream negative-first regressions then **188 top-level tests + 3 nested
  fixtures**, strict all-target Clippy/fmt pass; tests 130.34s, Clippy 28.89s.
  The intermediate missing `provenance_notice` qualification error remains in logs.
- Consumer **114 top-level tests + 16 nested executions**, two normal-discovery
  ignored fixtures explicitly exercised by their protocol contract, pass in
  57.92s. Four initial synthetic fixtures lacked the `.lock` source-policy suffix;
  fixtures were corrected, not production fingerprint completeness weakened.
- `RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true just visual-test` passes in 219.75s:
  171 Python discoveries (170 pass, one integration skip separately passing),
  imported upstream and consumer contracts. Strict all-target Clippy for both
  Cargo roots and fmt pass. Full logs retain expected injected failure transcripts;
  those are not failed real tests. Existing `block` warning is not suppressed.
- Official `validate` passes for each fresh run with sources/images valid and
  producer unexecuted. It explicitly does **not** check baseline environment;
  rendering equality is separately proven by the sealed review inspection.
  Read-only `doctor` reports 1,523 sources, no producer/environment probe.
- Focused self-review covers reversible migration, exact classification, stale/
  pending/locked no-write paths, capture-time drift, retained coverage and approval,
  immutable export and source/resource identity. No independent agent review.

Original A/B reports were inspected in the existing managed Chrome task tab.
Desktop B at 1672×963 DPR2 and narrow B at 390×844 DPR1 show 28 loaded images,
32 navigation rows, no duplicate IDs, broken anchors/images, label overflow,
row overlap or horizontal overflow. The provenance disclosure and coverage-only
FAIL are visible. Original screenshots and measurements are retained under
`/Users/patwoz/dev/Personal/macos/bbr/reports/visual/evidence/environment-20261003/`,
alongside migration/pre-state and integrity evidence. The first combined browser
image-decode command timed out; follow-up found no pending images and screenshots
succeeded. The bounded static Impeccable layout/type scan returns nine existing
compact-typography warnings (eight leading, one hierarchy), not a full-audit pass.
No observed overlap/clipping in the checked viewports; no vendor/report polish,
new browser/profile or unrelated tab changes. The task tab returns to report B
with its desktop viewport restored.

Final documentary edits are outside the capture source policy; exact current
source/export/preservation checks, roadmap/index-roadmap, formatting and whitespace
are rechecked before the local commit. This closes only the requested correction.
Complete app/native coverage, Gallery interaction/framing, public capture guards,
Python cutover, current stable functional aggregate and historical aggregate
Command p95 failures remain open. No full verify/E2E/performance pass, broader
migration checkbox, automatic reference acceptance or push. Total implementation/
self-review time was not instrumented. Do not silently extend into other queue
items or repeat unrelated aggregates to turn this bounded evidence into closeout.

### Application-state extension activated (2026-10-03)

Patrick's “ja lets go” authorizes the proposed real-app state extension, starting
with populated Saved Pages, recently closed History and Settings menus/sheets/
scroll positions. Clean base `main@3a60ef367120aa14430ec725632a6021c8e22bee`;
same sole local visual writer. Entire delivery queue reread at restoration.
Both capture locks have no observed holder; no foreign process was signalled.
No workflow/subagent/new lane, foreign worktree, publication or baseline approval.

The tooling packet owns the bounded matrix and gates. Runnable now: one populated
Saved Pages Light/Dark pair using existing production save actions and real views,
then a fresh supervised Tart proof before extending the matrix. Other product
queue items, Gallery interaction/framing and public/Python cutover are outside
this request; downstream expanded stills depend on that first runtime proof.
Territory is the existing capture test/helpers, consumer browser catalog/contracts,
necessary visual-only testkit support and documentation, serial on this worktree.
Shared integration points have one writer and no merge conflict/parallel lane.
The approved 28 PNGs and all historical reports stay unchanged. New images are
review candidates only; coverage remains incomplete and independently red.

### Application-state extension verified (2026-10-03)

The bounded real-app matrix now contains **66 Light/Dark PNGs**: all 28 approved
references remain byte-identical, with **38 new unapproved candidates**. A 30-image
populated-Library pilot passed before expansion. Saved Pages adds collections,
single/multiple selection, move/removal sheets and a cancelled collection draft.
History adds real keyboard filtering, empty results, Clear Data range menu and two
actually opened/closed fixed data documents. Settings adds General/Search/Motion
menus, cancelled New/Edit Profile sheets and three overlapping Shortcuts viewports.
No product design, settings choice, profile deletion or real user data is changed.

The authenticated visual-only wheel request draws the actual Settings window,
dispatches production GPUI wheel input at its measured content center, then reads
its existing Div's ScrollHandle. Normal rendering supplies no observer. Zero only
observes; closed/unmeasured Settings and out-of-bound deltas fail. Measured viewport
is `[0,96,780,524]`, scroll extent1002, offsets0/−400/−800/−1002, reset0. Every
capture preserves its semantic and scroll state. This proves that finite scroll
journey, not arbitrary nested paint, accessibility or typography completeness.

Final same-source evidence:
- Reports: `/Users/patwoz/dev/Personal/macos/bbr/reports/visual/app-states3-20261003-a/index.html`
  and `/Users/patwoz/dev/Personal/macos/bbr/reports/visual/app-states3-20261003-b/index.html`.
- Payload `fd460773f4b2c1cdd583a2665bb267a45a0c1e7accc6752c25c4726d53de6d66`,
  **1,526 identical current source hashes**, same cases/policy and rendering identity.
  Capture A `17de882c8e143cff90012bf0dee5c289fcfd6de3b22f79196e80fdaccd898c85`;
  B `be24502243c33902768ed9d443591fde6a5d6f378bb703a6e90a9fbc0091644f`.
- All66 actual PNG hashes match across A/B. Every original/reference/report seal
  is rechecked, including124 sealed report files per run. Zero comparison errors
  or rendering drift;28 `same`,38 `missing-baseline`, baseline writes false.
  Metal registry IDs4294967743/4294967710 remain explicit run provenance.
- Independent coverage still fails on seven named families: remaining transients,
  Library, History, Settings, portals, permissions/native dialogs and welcome/About.
  Native WebKit/AppKit exclusions remain; no synthesized page or portal pixels.
- Both clones repeat the existing frozen-payload/offline-runtime/private-NAT/key-only
  SSH route and original identity checks. Final independent resource rechecks cover
  all **six** transports (pilot, failed expansion, expanded2 A/B, expanded3 A/B):
  clones/keys/disks/owned processes absent and former SSH unreachable. No host
  producer, Golden Image mutation, foreign-worktree change or new exposure.

Measured tooling times: pilot prepare493.03s/caller319.544s; final expanded3
prepare102.48s, A caller114.365s/outer116.80s, B caller109.192s/outer110.55s.
Final integrity/cleanup audit13.95s. These are not product-performance measurements;
implementation and self-review wall time was not instrumented.

Retained failures and narrow corrections:
- Invalid integer Page metadata was replaced by explicit Shortcuts still moments,
  never guessed page ranges. The expanded guest exposed the exact production
  primary-tab recency increment after real close/reactivation; expected `last_used`
  advances3 while all other tab/session assertions remain. Data-document spaces
  are percent-encoded to satisfy persistence validation. Failed clone cleanup is
  independently verified, not hidden by subsequent success.
- The full gate caught the retained Python inventory still at28; its compatibility
  inventory now requires66. Missing-metadata negative tests assert the actual
  earlier regular-file failure, not the later aggregate inventory error. No guard
  or assertion was weakened. The first focused app command used nonexistent
  `--lib`; corrected `--bin bbr` executes the intended one test (not zero).
- Expanded2 PNGs also match all66 final images, supporting prior pixel inspection
  reuse only. Its different source seal is retained as historical evidence; source
  changes triggered fresh expanded3 preparation/captures, never stale resealing.

Verification:
- `RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true just visual-test`: pass,219.80s;
  171 Python discoveries (170 pass, one integration skip separately passed),
  immutable upstream188 top-level passes plus nested fixtures, consumer116
  top-level passes plus nested fixtures; two normally ignored inert protocol
  fixtures are explicitly exercised. Injected failure transcripts are retained.
- Bounded mbx protocol17/17 and app capture-path1/1 pass; strict all-target Clippy
  passes for visual-capture, testkit, normal app/UI and the consumer,98.96s combined.
- Fresh full **mbx-equivalent `just verify` passes**,79.25s, including the unchanged
  Command50ms benchmark, workspace checks/tests/Clippy, roadmap, UI boundaries and
  49 script tests14.449s. The external recipe substitutes only compiler Cargo calls
  with mbx, preserving targets/order/budgets. Earlier aggregate p95 failures remain
  historical evidence; this pass is not canonical real-app performance acceptance.
- Final source/protected-input audit confirms immutable upstream/export90 files at
  `e3050c78cc1347ea0164218d7bcd0d74addcdb8a`, unchanged GPUI/dependency graph,
  all29 baseline-store files unchanged and exact metadata hash
  `593f2ad806c8065915c7b5247e74423bbe4250a05ffe3b69f878be5981488bc4`.

All new Light/Dark original state pairs were visually inspected across proven
byte-identical runs. Existing duplicate-position Library selection-box appearance
is retained, not repaired or accepted as a side effect of tooling work. Original
HTML B was inspected in the existing managed Chrome task tab at1672×963 and390×844.
All66 unique nonempty image sources load at their exact original dimensions; the
initial244 unloaded elements were all lazy.82 navigation rows show no measured
label overflow/overlap, broken anchors, duplicate IDs or horizontal overflow.
A navigation/decode batch timed out without evidence output; a separate readback
and screenshot confirm the final Dark Shortcuts anchor and image are visible.
The red verdict, missing references and withheld coverage stay visible. Viewport
and report-top position were restored; report files/styles were never edited.

Logs, retained failures, safe transport/cleanup records, original report screenshots
and hash inventory are preserved at
`/Users/patwoz/dev/Personal/macos/bbr/reports/visual/evidence/app-states-20261003/`.
Scratch originals remain at `/private/tmp/bbr-visual-states`; the evidence bundle
excludes keys, known-host files, payload executables and runtime archives. Final
format, roadmap/index-roadmap and whitespace checks precede local commit; only
documentation changes after source-bound captures. Self-review checked feature
boundaries, real-input/real-window routing, exact state preservation, independent
inventory/negative cases, source/rendering/provenance separation and no acceptance.
No independent agent was requested or used.

Queue/ownership audit: same sole local writer on the recorded base; no other lane,
workflow, subagent, push or publication is activated. This is a verified intermediate
capture-tooling slice, **not stable migration closeout**. Public capture and Gallery
endpoint guards, interactive/framing pilot, full application/native coverage, Python
cutover and fresh canonical functional aggregate remain open. No full real-app
E2E/performance aggregate or migration checkbox is claimed. New references still
need explicit approval; historical approval covers only the original28.

### Explicit application-state reference approval (2026-10-03)

Patrick answered “ja” to accepting exactly the38 new images from
`/Users/patwoz/dev/Personal/macos/bbr/reports/visual/app-states3-20261003-b/index.html`.
On clean `main@329449861d836ca538e9f377e4e11b993184efe1`, the existing pinned
90-file export was checked and its official CLI rebuilt through bounded mbx.
Before acceptance, all1,526 current sources,66 original PNGs,124 report-sealed
files and the original28 baseline images/records were independently verified.
No capture, producer, environment probe, tool-source edit or VM operation occurs
in this reference-only slice. Same sole writer; no agent, lane or publication.

Official `accept-collection` succeeds with exactly38 explicit `browser:case` IDs
and all seven exact `browser:base` withheld acknowledgements, **without environment
adoption**. Its completed schema-2 journal names only browser. All38 stored PNGs
match the approved originals byte-for-byte and point to capture
`be24502243c33902768ed9d443591fde6a5d6f378bb703a6e90a9fbc0091644f`.
The original28 PNGs, case/capture records and complete approved environment are
unchanged. Historical report/seal bytes remain unchanged; only the official
acceptance journal is added to the approved report. Baseline metadata SHA256:
`593f2ad806c8065915c7b5247e74423bbe4250a05ffe3b69f878be5981488bc4` →
`f324a93f983da0ff7518a9d8a07182332155e39b9c3f182eabac74f11dfc3d79`.

New comparisons of the two still-valid existing captures:
- `/Users/patwoz/dev/Personal/macos/bbr/reports/visual/app-states-accepted-20261003-a/index.html`
- `/Users/patwoz/dev/Personal/macos/bbr/reports/visual/app-states-accepted-20261003-b/index.html`

Both prove66/66 `same`, no image/comparison errors and exact rendering equality.
Provenance differences stay visible. Collection exit1/FAIL remains solely the
seven unchanged withheld families. These are **fresh stored-run comparisons, not
new guest captures**. No unrecorded acceptance-time recapture or historical reseal.

The first CLI invocation supplied unnecessary global `--repo`; collection routing
rejected it before writes. Baseline hashes and absent journal were checked before
removing only that flag. The exact same approved report supplies suite authorities;
no guard was bypassed. Rejected invocation and successful attempt are retained.
Measured tooling time: pinned build21.85s, acceptance1.700s, reviews2.021s/1.982s.
Self-review verifies exact selection, journal, unchanged unselected references and
environment, new PNG/source provenance, report integrity and still-red coverage.
`RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true just visual-test` passes in237.48s,
including Python, immutable upstream, consumer and explicit real-CLI acceptance
fixtures. No production source changed; roadmap/index/whitespace and post-adoption
source/report/baseline revalidation precede the local commit. No new product E2E
or performance execution is claimed by this baseline-only operation.

Evidence is preserved at
`/Users/patwoz/dev/Personal/macos/bbr/reports/visual/evidence/app-states-acceptance-20261003/`;
scratch originals at `/private/tmp/bbr-visual-states-acceptance`.
No broader migration, Gallery/native coverage, public capture/Python cutover or
canonical functional/performance completion is implied. Earlier “unapproved38”
statements remain accurate historical checkpoints, superseded only by this explicit
reference approval. No push or additional work is activated.

### Command Palette capture activated (2026-10-03)

Patrick's “ja lets go” activates the proposed smallest real overlay capture,
then only matched/no-match search variants and an exact repeat/report if the
first pair works. Clean base `main@9e72e9fe`; session `01a0f8f4` remains the sole
local writer. Complete delivery queue read; index/worktree initially clean,
no observed capture-lock holder or visual/Tart/compiler process. Golden Image
is stopped. Foreign worktrees and ordinary apps remain untouched.

Queue audit: the bounded Command Palette child readback and deterministic fixture
are runnable now. Additional overlay families, Gallery pilot, public cutover and
unrelated product queue items are not activated. No workflow/subagent/new lane.
Exclusive territory is visual-only testkit dispatch, capture fixtures, consumer
inventory/catalog/tests and documentation, serial on this base with no competing
writer or merge order. Use existing frozen-payload, offline-runtime, private-NAT,
key-only SSH supervision and independently verified cleanup. No host producer,
Golden Image mutation, new exposure, publication or baseline adoption. All66
approved references remain unchanged. The first real pair is the dependency gate
before the search matrix; hidden-window renderer evidence is not on-screen native
visibility or WebKit composition evidence.

### Command Palette candidate and verification stop (2026-10-03)

The real production Portal child now supplies empty-query, matched-actions
(`> tab`) and no-match-actions (`> qqqzzz`) Light/Dark stills. Authenticated
`CaptureCommandPalette` is visual-feature-only, requires the existing Palette
presentation/handle, and refuses closed, other-presentation and nested-path calls.
The child draws after releasing the app lease; no Gallery reconstruction, parent
readback, pixel composition, renderer/lifecycle or dependency change. The real
fixture proves unchanged command/tab/session/focus state across readback and
Escape restoration. RGBA8 dimensions remain2160×2814 with transparent exterior;
hidden-parent/native-child flags are explicit, not on-screen visibility evidence.

Pilot1 exposed AppKit's startup height1408 instead of1407. The fixture now uses
the existing production resize/layout-settled boundary, keeping exact assertions.
Fresh pilot2 passed before searches were added. The final two runs each contain
**72 byte-identical PNGs**, including the unchanged **66 approved references**:
- `/Users/patwoz/dev/Personal/macos/bbr/reports/visual/command-expanded-20261003-a/index.html`
- `/Users/patwoz/dev/Personal/macos/bbr/reports/visual/command-expanded-20261003-b/index.html`

Both bind the same1,527 current sources, cases/policy and exact rendering identity.
Payload SHA256 `3115d8f2273bdf60699ae341975b637f629a1f38a6db52673981615661b15fe1`;
capture A `40bf8e25bf84382dd123b02ad41b2a0c8c9e84e9e91c984a8cb2c2ae4ddd0db8`;
B `80fbf672952451ab0df92d4b71cd955b0a066f2369398bf91cd5d16fe7142af4`.
Each review has66 `same`, six `missing-baseline`, zero comparison errors or
rendering drift and seven withheld families. All206 report-sealed files per run
and17 earlier collection seals pass read-only checks. Baseline metadata remains
`f324a93f983da0ff7518a9d8a07182332155e39b9c3f182eabac74f11dfc3d79`;
all baseline-store bytes, immutable90-file export and GPUI pin are unchanged.
No reference/environment adoption or historical resealing occurs.

Original new PNGs and the actual managed-Chrome report were inspected. Matching
actions have ten semantic results but the image shows only the current viewport;
no full-result-list framing is claimed. The no-match state retains the current
production copy “Type a destination”, not a tooling-invented no-results message.
All72 unique local image sources load at their original dimensions. Desktop
1672×963 and narrow390×844 checks find no horizontal overflow, broken anchors,
duplicate IDs or overlap/label overflow in88 navigation rows. All six new anchors
resolve. A visible-image `complete` wait timed out in the background tab despite
nonzero natural dimensions and readable screenshots; this is retained separately
from successful explicit asset loading. The first narrow screenshot retained an
earlier scroll position; re-anchoring produced the correctly labeled screenshot.
Original report bytes/styles stay unchanged; desktop viewport is restored.

All four transports, including failed pilot1, pass request/payload-bound independent
clone/key/disk/process/SSH-absence rechecks. The existing frozen-payload/offline
runtime/private-NAT/key-only SSH route is unchanged. Golden Image stays stopped;
no host producer, foreign-worktree mutation or new exposure. A transient unclassified
capture-lock descriptor was observed during inspection and disappeared before its
process could be identified; no signal, takeover or gate-idleness inference.

Verification and measured tooling times:
- Negative-first protocol/catalog regressions, then18 protocol tests and one
  capture-path test pass. Missing Palette PNG/metadata and wrong dimensions are
  rejected by the portable raw-return contracts. Strict visual-feature/testkit/
  normal app/UI and consumer Clippy pass.
- Full serial offline `just visual-test` passes in377.35s:171 Python discoveries
  (170 pass, one integration skip separately passed),188 upstream and117 consumer
  top-level passes, plus nested fixtures. Earlier compatibility-inventory66/72
  failure and unchanged upstream-test timeout124 remain in their original logs.
- Fresh mbx-equivalent non-GUI `just verify` fails after85.94s at the unchanged
  Command benchmark: p50=6.545ms, p95=34.192ms, max=133.806ms against50ms. One
  isolated unchanged recheck also fails: p50=9.856ms, p95=44.433ms, max=154.387ms.
  Host load was elevated; causation is unproven and unrelated workloads untouched.
- Remaining Command integration/doc tests, other workspace tests, workspace strict
  Clippy, roadmap/UI boundaries and49 script tests pass in64.62s. These explicitly
  separate checks do not convert the failed aggregate into a pass. No skipped or
  relaxed benchmark, product fix, canonical functional/performance aggregate or
  stable migration acceptance is claimed.
- Pilot2 caller137.448s; final callers145.148s/138.601s. Independent source/report/
  history/resource audit10.319s. None is product performance. Implementation and
  self-review wall time were not instrumented.

Evidence is preserved under
`/Users/patwoz/dev/Personal/macos/bbr/reports/visual/evidence/command-palette-20261003/`;
scratch originals remain at `/private/tmp/bbr-visual-command`. A scratch inspector
initially shadowed Python's standard `inspect` module; renaming only that scratch
file resolved the diagnostic import failure. Source files did not change afterward.
Self-review checked actual-child routing/lease lifetime, feature-off refusal,
semantic field existence and preservation, independent inventory, exact repeats,
remaining coverage and approval separation. No independent agent was requested.

Queue/stop audit: only this serial candidate is authorized; no subagent/workflow,
new lane, publication or additional overlay/product work. The implementation stays
**uncommitted** because the requested fresh non-GUI gate remains red. Six images
are review candidates, not references. The next verification step must investigate
the unchanged Command maximum-latency failure without weakening it; changing that
product subsystem requires a separate bounded scope. Broader migration/coverage,
Gallery interaction/framing, public guards and Python cutover remain open. Preserve
this candidate and evidence rather than repeatedly retrying or claiming completion.

### Explicit Command Palette reference approval (2026-10-03)

Patrick answered “ja” to the explicit question about adopting exactly six new
Command Palette images from
`/Users/patwoz/dev/Personal/macos/bbr/reports/visual/command-expanded-20261003-b/index.html`.
The approved bases are `command-palette-open`, `command-palette-actions-match`
and `command-palette-actions-empty`, each Light/Dark. This supersedes only their
preceding unapproved-candidate status, not the recorded benchmark failure or the
capture implementation's uncommitted state. No additional product work is activated.

Preflight at `main@9e72e9fed2061973b1a213084a557ac31ae9d22e` preserves the known
15-file candidate and empty index. The sole local writer reread the complete
queue; only selective adoption and its verification are runnable under this
approval. No subagent/workflow/new lane, foreign-worktree mutation, producer,
VM, environment probe or publication. The pinned official CLI was rebuilt through
bounded locked/offline mbx; all90 immutable export files,1,527 current capture
sources, both72-image runs and20 historical collection directories were verified.

Official `accept-collection` succeeds with exactly six `browser:case` selections
and the seven complete explicit withheld acknowledgements, **no environment
adoption**. Its schema-2 journal is complete for browser. New PNGs exactly match
approved capture `80fbf672952451ab0df92d4b71cd955b0a066f2369398bf91cd5d16fe7142af4`.
All66 previous PNGs/case records, both previous capture records and the complete
approved rendering/provenance environment remain unchanged. Historical report and
run files are byte-identical; only the official acceptance journal was added to
the approved report. No historical resealing or source modification occurs.

- Baseline metadata SHA256 changes from
  `f324a93f983da0ff7518a9d8a07182332155e39b9c3f182eabac74f11dfc3d79` to
  `d9b23b6161cabf9b84525c3392d5f70c86a8a36fc6662783efc476112bef649a`.
- Acceptance journal SHA256:
  `4e6b81af8bb9c250ebbc8dad8159735da385526d2442cea0dca6917ff3d242a8`.
- Fresh stored-run comparisons, not new captures:
  `/Users/patwoz/dev/Personal/macos/bbr/reports/visual/command-accepted-20261003-a/index.html`
  and `/Users/patwoz/dev/Personal/macos/bbr/reports/visual/command-accepted-20261003-b/index.html`.
  Both show72/72 `same`, zero comparison errors, exact rendering equality and218
  verified sealed report files. Collection exit1/FAIL remains solely the same
  seven withheld families; provenance differences stay visible.

Fresh serial offline `just visual-test` passes in322.45s:171 Python discoveries
(170 pass, one integration skip separately passed),188 upstream and117 consumer
top-level passes plus existing nested fixtures. No real reference writes occur in
those disposable acceptance tests. Acceptance took1.994s; comparisons3.971s/5.910s,
all measured tooling time, not product performance. No new functional/performance
aggregate is run or claimed; both recorded Command maximum-latency failures remain.

The existing managed Chrome task tab displays the new B report. It shows zero
changed images, zero missing references and72 unchanged; all six approved assets
load at2160×2814, with `same` labels. The initial browser anchor check incorrectly
reused the missing-baseline `f-` finding anchor. Static report inspection identified
the correct `s-` same-image anchors; following its actual navigation link and
checking again succeeded. Both screenshots are retained, without editing the
report or its styles. Desktop viewport remains1672×963 with no horizontal overflow.

Evidence and explicit allowlisted hashes are preserved at
`/Users/patwoz/dev/Personal/macos/bbr/reports/visual/evidence/command-palette-acceptance-20261003/`;
scratch originals remain at `/private/tmp/bbr-visual-command-acceptance`.
Self-review checks selection/journal identity, unchanged prior references and
source/report seals, independent coverage and approval versus implementation
acceptance. Roadmap/index/whitespace and final adoption/comparison integrity checks
precede the reference-only local commit. The source-bound capture implementation
is deliberately not staged with these approved references; its bytes are preserved.
No migration checkbox, stable-slice completion, benchmark repair or push is implied.

## Status vocabulary

| Status | Meaning |
|---|---|
| `planned` | durable contract exists; dependencies or activation are incomplete |
| `ready` | dependencies and ownership preflight permit one writer to start |
| `running` | exactly one writer owns the named worktree |
| `review` | candidate settled; independent read-only review active |
| `remediation` | one writer addresses concrete review findings |
| `frozen` | direct review pass; exact tree awaits serial integration |
| `blocked` | no defensible local action remains without the named unblocker |
| `integrating` | Orchestrator owns merge/conflict resolution and integration gates |
| `integrated` | exact reviewed change is on the control ancestry with required evidence |
| `closed` | full bounded milestone acceptance and documentary closeout are proven |

## Active product wave

The reference-alignment implementation wave is complete on `main`: Theme merged through `f7fe3d6f`, Favicon through `a79ff95e`, Reference Shell through `4d1a5343`, and Sidebar Polish plus interaction fixes through `35c91189`; follow-up functional E2E fixes and the centered New Tab palette land through `e464ca32`. The completed reconciliation records those facts and Patrick's later folder design decision without changing runtime behavior.

| ID | Slice | Status | Worktree | Owner | Contract | Next gate |
|---|---|---|---|---|---|---|
| CONTROL-RECONCILE | Product/delivery authority reconciliation | `closed` | commits `d2b507f5` and `f8e945c2`; locally integrated to `main` | none; reconciliation complete | [Roadmap maintenance](../roadmap/README.md) | Closed: worktree/index/head graph, script tests, targeted UI test/Clippy, ancestry, sessions, worktrees, and dirty-state preservation audited |
| FOLDER-PIN-INVARIANT | Folder membership and pinned-state invariant | `integrated` | `main` (serial, no worktree) | none; implementation integrated | [folder membership pins tabs](../roadmap/tasks/folder-pinned-invariant.md) | Functional gates green; independent review remediated (`d51a4b52`: cross-space grouping pins are persisted but not reported as active-space pins); performance evidence deferred to the end of the queue |
| UI-CONFORMANCE | UI contract conformance fixes | `integrated` | `main` (serial, no worktree) | none; slice complete | [UI contract conformance](../roadmap/tasks/ui-contract-conformance.md) | Integrated; independent review remediated (`fb79d688`: glyph extents checked against measured paths; E2E scope recorded in the packet) |
| DESIGN-CARD | Seated content card | `integrated` | `main` (serial, no worktree) | none | [seated content card](../roadmap/tasks/seated-content-card.md) | Integrated; independent review remediated (`c48b0799`: the circular traffic-light assertion exposed and fixed a native measurement bug); performance evidence deferred |
| DESIGN-ADDRESS | Address field and space identity | `integrated` | `main` (serial, no worktree) | none | [address field and space identity](../roadmap/tasks/address-field-space-identity.md) | Integrated; independent review remediated (`c4a0a20d`: heading aligned with tab titles, real zoom-chip press evidence, focus ring deferred to the Accessibility track); performance evidence deferred |
| DESIGN-FAVORITES | Favorites grid and drag offers | `integrated` | `main` (serial, no worktree) | none | [Favorites grid and drag offers](../roadmap/tasks/favorites-drag-offers.md) | Integrated as `e6ba14ab` with its review remediation; full `just e2e` minus perf green; performance evidence deferred |
| RA-MEDIA | Media-session capability and Sidebar media control | `planned` | `main` (serial, no worktree) | none; paused 2026-09-28 | [capability contract](../roadmap/tasks/media-session-capability-contract.md) → [media control](../roadmap/tasks/sidebar-media-control.md) | Portable contract integrated at `ec5e0bad`; resumes with the macOS adapter after V2-S12 |
| V2-S0 | v2 contract and delivery control | `integrated` | `main` (serial, no worktree) | orchestrator `01a0e454` | [v2 contract](../roadmap/tasks/v2-contract.md) | Integrated as `907a668e` (`just roadmap`, `just verify`) |
| V2-S1 | System contrast and motion preferences | `integrated` | `main` (serial) | orchestrator | [system preferences](../roadmap/tasks/v2-system-prefs.md) | Integrated; verify, testkit clippy, sidebar 17/17, smoke, gallery 45/45; self-review recorded |
| V2-S2 | Palette, radius and elevation tokens | `integrated` | `main` (serial) | orchestrator | [tokens](../roadmap/tasks/v2-tokens.md) | Integrated; verify, testkit clippy, sidebar/hover/spaces/gallery + command/tab-menu/library/organize green; self-review recorded |
| V2-S3 | Rail: lift, veils, heading, footer | `integrated` | `main` (serial) | orchestrator | [rail](../roadmap/tasks/v2-rail.md) | Integrated; verify, testkit clippy, sidebar/hover/tabs/organize/dense/spaces/gallery green, visual capture read; self-review recorded |
| V2-S4 | Space paper and active chip lift | `integrated` | `main` (serial) | orchestrator | [space paper](../roadmap/tasks/v2-space-paper.md) | Integrated; verify, testkit clippy, spaces/sidebar/gallery/organize green, visual capture read; self-review recorded |
| V2-S5 | Floating surfaces | `integrated` | `main` (serial) | orchestrator | [floating surfaces](../roadmap/tasks/v2-overlays.md) | Integrated; verify, testkit clippy, command/tab-menu/context-menu/find/downloads/library/auth/gallery green; self-review recorded |
| V2-S6 | Motion core | `integrated` | `main` (serial) | orchestrator | [motion core](../roadmap/tasks/v2-motion-core.md) | Integrated; verify, testkit clippy, sidebar/hover/errors/tabs/favicons/gallery green; attention E2E not expressible (no producer); self-review recorded |
| V2-S7 | After-commit moments | `integrated` | `main` (serial) | orchestrator | [motion moments](../roadmap/tasks/v2-motion-moments.md) | Integrated; verify, testkit clippy, sidebar/downloads/command/spaces/organize/gallery green; self-review recorded |
| V2-S8 | Drawn empty states | `integrated` | `main` (serial) | orchestrator | [empty states](../roadmap/tasks/v2-empty-states.md) | Integrated; verify, testkit clippy, errors 11/11 (new illustrations E2E), tabs, states, gallery green, visual capture read; self-review recorded |
| V2-S9 | Phosphor-b app icon | `integrated` | `main` (serial) | orchestrator | [brand icon](../roadmap/tasks/v2-brand-icon.md) | Integrated; verify, package-test 14/14, package + determinism identical, package-smoke alive; self-review recorded |
| V2-S10 | Curated intensity and grain | `integrated` | `main` (serial) | orchestrator | [theme picker](../roadmap/tasks/v2-theme-picker.md) | Integrated; verify, testkit clippy, persistence (new space-paper restart E2E), spaces, gallery green, visual capture read; whole appearance persists (schema v4); self-review recorded |
| V2-S11 | Gallery, documentation and closeout | `integrated` | `main` (serial) | orchestrator | [closeout](../roadmap/tasks/v2-closeout.md) | Integrated (`92263be8`); canonical aggregate green (31/32 locked, portal 7/7 after unlocking on the identical product tree) |
| V2-S12 | Paper alignment and comparison findings | `integrated` | `main` (serial) | product orchestrator `01a0e454` | [Paper alignment](../roadmap/tasks/v2-paper-alignment.md) | Implemented (`43703235`…`05015039`); fresh combined-tree functional sequence passes, including Portal 7/7 and Titlebar 5/5 (merge evidence below); product-tree Portal rerun 7/7 on the unlocked console; documentary closeout `3c70abfc` and this follow-up; performance evidence deferred |
| PAPER-BLUEPRINT | Complete browser blueprint in Paper (design-only) | `closed` | `main` (serial; Paper file `01KZY8FRMDSAYFJ4TJ9TYTV4AF`) | orchestrator `01a0e454` | [complete browser blueprint](../roadmap/tasks/paper-browser-blueprint.md) | Closed: W0–W7 committed, 116 boards, coverage 118/118, tokens 0 mismatches; 24 delta packets proposed (DESIGN §19) awaiting Patrick's adoption |
| BLUEPRINT-IMPL | Implement the adopted blueprint deltas D01–D25 (D25 added 2026-09-29, ledger correction) | `running` | `main` (serial) | orchestrator `01a0e454` | [complete browser blueprint](../roadmap/tasks/paper-browser-blueprint.md) → `blueprint-*.md` | One delta at a time in the DESIGN §19 order (serial on `main` by Patrick's instruction: no parallel lanes or subagents); each with its tests, affected real-app E2E and DESIGN text; performance deferred. Implemented 2026-09-29: D01, D20, D05, D14, D03, D04, D21, D22, D25 (first slice), D09, D08 (+ hit-region Portal, Peek pointer fix), D07, D19 (menus), D17, D18 (About window), D02 (command-field IME editor, 2-D drag targets with folder rows, floating rail in the Portal; exit fade and reveal perf scenario open), D16 (Settings window; rows only where behaviour exists), D06 (profile create/edit/delete), D10 (History as a Library page with global visits, Clear Browsing Data; `just e2e-history`), D11 (camera/microphone prompts, device marks, site settings, Settings › Privacy sites; `just e2e-permissions`; native delegate hand check open), D12 (client-certificate sheet with Keychain metadata and remembered choices; `just e2e-client-certificates`; live TLS hand check open), D13 (blocked pop-up banner, open-in-app sheet, space chooser; `just e2e-link-destinations`; sign-in pop-up window open), D15 (first-run welcome, import from Safari and Chrome, first space; `just e2e-first-run`), D24 (sidebar media card over a WebKit-state-gated client-world probe: Play/Pause, Seek, Mute, `+N`, dismiss, source activation; `just e2e-media`; keyboard traversal into the card open). Blocked: D23 (tiling contract first), D18 updates (§19 question 14). D19 second slice 2026-09-30: Duplicate Tab, Select Next/Previous Tab (⌃⇥/⌃⇧⇥, ⌘T actions 20–22). D19 several windows decided 2026-09-30 (answer A; last window closed quits): slices 1–6 implemented ([multi-window](../roadmap/tasks/multi-window.md): controller window contexts, engine roots, shared handles, New/Close Window, restore of every window, Reopen toast and ⇧⌘T, tabs dragged between windows; `just e2e-windows`). `e2e-perf` failed in the context-menu bookmark benchmark since before this work (reproduced on `89c97cdf`); cause found 2026-09-30 (the never-shown harness window has no display link and the bookmark sample forced no draw) and fixed, as was the dense scenarios' stale pinned-row target (DESIGN §11.11); `e2e-perf` now stops in the renderer-responsiveness scenario, which was added under the performance-execution override and has never run (performance deferred). Canonical `just e2e` 2026-09-30 after D19 several windows on the committed clean tree `49c87250`: all 53 functional test binaries green (271 tests); the run then stopped in `e2e-perf` at the startup ceiling (3691 ms > 750 ms at host load average 74; one rerun 3771 ms). D02 exit fade landed 2026-09-30. Canonical `just e2e` 2026-09-30 (after D24, D19 tab commands, D02 exit): every suite green — the run stopped at `e2e-client-certificates`, whose repeated-URL cancellation it exposed; after the fix the remaining suites from there through `e2e-auth` ran green; `e2e-perf` not run (deferred). D23 implemented 2026-09-30 ([split tiling](../roadmap/tasks/split-tiling.md), 6 serial slices: tiling contract, groups + `splits.txt`, tiling composition + keyboard commands, drop zones + rail block; `just e2e-split`; the focused-pane outline is open). Canonical `just e2e` 2026-09-30 after D23: every suite green — the run passed through `e2e-shell-dense` (43 suite results), then mbx's cache GC removed the target directory under `e2e-shell-downloads` ("spawn bbr: No such file"); after restoring it the remaining suites from `e2e-shell-downloads` through `e2e-auth` ran green; `e2e-perf` not run (deferred). Canonical `just e2e` 2026-09-29 (after D19): every suite green except one `e2e-auth` flake (2/2 reruns green); `e2e-perf` not run (deferred). `just verify` green after D18 |
| REALITY-GAPS | Make the tests run what users run (items A–F) | `integrated` | `main` (serial) | orchestrator `01a0e454` | [reality gaps](../roadmap/tasks/reality-gaps.md) | A–F implemented and functionally verified 2026-09-30 (`fc00be97`/`3bc8fb08` default profile directory, `48699c2a` native HTTP authentication, `40c48600` real input events, `200c1943` background links, `82d8ca9` pop-ups from script clicks, `307fe1d` persistent website data, `ffd7c3d` one-gesture clicks); real bugs found and fixed: ephemeral default launch, WebKit debug abort on HTTPS (`be35936c`), unconnected HTTP auth, Downloads panel ignoring the next outside press, ⌘/middle-click on ordinary links, middle button misread, script-clicked pop-ups, deleted profile store left on disk. Canonical `BBR_E2E_TIMEOUT=15m just e2e` on the committed clean tree `ffd7c3d` (22:00–22:12): all 55 functional test binaries green (281 tests, incl. `e2e-real-input` 6 and `e2e-web-data` 1); the run then stopped in `e2e-perf` at the startup ceiling (868 ms > 750 ms, host load average 43; one rerun 883 ms) — not green, performance deferred; the never-run renderer-responsiveness scenario stays open. Earlier canonical attempts on `307fe1d`: stopped once in `e2e-shell-command` (portal ids after close; 3/3 reruns green) and once in `e2e-real-input` (a real Back click; see the packet): intermittent, cause open. |
| KEPT-PLACES | Closed pinned tabs and Favorites that leave the list (Patrick, 2026-09-30) | `integrated` | `main` (serial) | session `01a0f303` (Patrick handed `main` over after REALITY-GAPS settled; `01a0e454` idle) | [closed pins and Favorites](../roadmap/tasks/closed-pins-and-favorites.md) | `3fd97e37` removes the unplanned row pin control; the follow-up commit adds `TabLifecycle::Closed`/`PinnedPage`, `favorite => pinned`/`no folder`, the grey closed row/member/tile, the tile's selected edge, DESIGN §10.12 and Paper PR-24/B-3. Gates: unit tests of every touched crate, testkit clippy, new `e2e_shell_closed_pins` (real ⌘W and clicks, 2/2), every `just e2e` suite without `e2e-perf` plus `e2e-peek` and `e2e-gallery` green (three Sidebar and one drag-offer assertion rewritten to the new contract). Performance deferred; closed state and Favorites stay session-local (schema) |
| KEPT-PLACES-2 | Follow-up "dann fix alle": ⇧⌘T for closed pinned rows, schema v5 persistence of kept places/Favorites/folders, Gallery closed families, perf scenarios, independent review | `integrated` | `main` (serial) | session `01a0f303` | [closed pins and Favorites](../roadmap/tasks/closed-pins-and-favorites.md) | Gates: unit/contract tests, `just verify`, testkit clippy, every `just e2e` suite except `e2e-perf` on the host and in a throwaway Tart clone (`macos-27-golden-gate`, NAT, `--no-graphics`, host-built binaries) so GUI runs no longer take the user's focus; `e2e-gallery` 48/48. New perf scenarios measured within budget in the VM (report-only); the whole `e2e-perf` stays red on the pre-existing responsiveness block and the startup ceiling under load. Independent review: no blocker, three findings remediated |
| DRAG-BACK | Drag pinned rows and Favorite tiles back out of the pinned area (Patrick, 2026-10-01) | `integrated` | `main` (serial) | this session | [dragging back out](../roadmap/tasks/drag-back-out-of-pins.md) | `Browser::unpin_tab_to`, `DragTarget::Unpin`, draggable Favorite tiles (activation + armed drag, ghost row, no drop cell). Gates: unit tests, testkit clippy, `e2e-shell-organize` (new `e2e_shell_drag_back`, rewritten pinned → regular step), sidebar, hover, dense, windows, split, real-input (new real tile drag), gallery green on the host; `just verify`. Performance scenario open (override); self-review only |
| FAVICON-CACHE | Keep acquired favicons across a relaunch (Patrick, 2026-10-01) | `integrated` | `main` (serial) | this session | [favicon disk cache](../roadmap/tasks/favicon-disk-cache.md) | Per-profile `favicons/` directory seeded at launch and written as a diff; profile deletion and Clear Browsing Data's cached images drop records. Gates: unit tests, testkit clippy, `just verify`, `e2e-shell-persistence` (new restart leg, red without the seeding), `e2e-shell-favicons`, `e2e-profiles`, `e2e-web-data`, `e2e-history`. Performance deferred; self-review only |

The orchestrator (session `01a0e454`) is the product writer and runs the V2 slices serially on `main` (KEPT-PLACES ran serially from session `01a0f303` at Patrick's request while it was idle): no subagent or background workflow is authorized for this program, so the packets' parallel-capable pairs are executed in order and each slice is reviewed by a documented self-review against its packet before integration. Independent reviews use `homelab-llm/opencode-go/deepseek-v4-flash:high`: the route named in `AGENTS.md` (`homelab-llm/deepseek-v4-flash`) no longer resolves in the model registry. All historical BBR worktrees remain preserved.

## Isolated Portal bug-fix lane (2026-10-01)

Patrick explicitly approved a separate worktree for the Command Palette
remaining above other apps. Sole writer: session `01a0f808-b22a-7311-8e49-dc2619e64d76`;
branch `parallel/portal-window-lifecycle`, worktree
`/Users/patwoz/dev/Personal/macos/bbr-portal-window-lifecycle`, clean committed
base `32b735dcd79372b4fe9722122579959da0f39c4a`; frozen source commit
`aff55e14ccc1869abcfb7545b37775ca984a12f5`. Status: `integrated` by the local
no-ff merge after Patrick authorized the bounded Titlebar synchronization repair
and fresh combined-tree Tart gates. Session `01a0f808` settles integration
ownership on `/Users/patwoz/dev/Personal/macos/bbr` with this merge; no ongoing
product ownership is claimed. Both earlier Titlebar failures remain recorded.
Favorites drag, favicon persistence and main's VM tooling are preserved;
internal-pages and cleanup stay outside this integration. Historical lane-only,
no-merge and blocked checkpoints below are provenance, not current state.
Contract: [Portal window lifecycle](../roadmap/tasks/portal-window-lifecycle.md).

Parallelism audit: only this bug fix is authorized in this session. Main's
Favorites drag writer and the isolated internal-pages writer remain untouched;
no subagent, workflow, extra lane or merge is authorized. Exclusive product
territory is the native Portal bridge. Shared integration points are additive
native/testkit diagnostics, Portal lifecycle requests, snapshot mapping and
roadmap/architecture documents. Main's ongoing work lands first, this isolated
patch follows serially through the integration owner; conflicts must preserve
both contracts. Native GUI gates use the common-directory host lock. No Git
index lock existed at activation; the main worktree's unrelated dirty diff
was not copied. Performance execution remains deferred; no performance pass is
claimed.

### Portal lifecycle checkpoint and GUI blocker

Implemented in the uncommitted isolated lane: current-intent deferred visibility,
app/key-window/active-Space observers with balanced removal, parent-relative
ordering and reattachment after AppKit detaches an ordered-out child. Native
level/Space policy is reset at attachment. Diagnostics now distinguish retained
ownership from actual `parentWindow()`. Hidden consumer fixtures assert logical
presentation plus no native orphan; shown visibility checks remain mandatory.
No dependencies, renderer or product focus authority changed.

Evidence:
- Old-code regression reproduced inactive-app visibility, standalone hidden-parent
  visibility and independent all-Spaces policy. Initial level hypothesis was
  corrected: GPUI creates level 101 but native attachment already lowers it.
- Initial lifecycle regression 3/3 green: `/tmp/bbr-portal-regression4.log`.
  After actual attachment diagnostics and the two-window test, the latter passed
  twice (`/tmp/bbr-portal-regression5.log`, `/tmp/bbr-portal-suite2.log`). First
  run still saw `app_active=true` after deactivate; the helper now avoids a
  redundant pending activation before deactivation. That fix is not yet fully
  revalidated across repeated runs.
- Portal suite `/tmp/bbr-portal-suite2.log`: 9/11, 41.07s measured test time.
  Failures: shown Space test did not gain native visibility; draft test received
  unexpected suffix `es ` (`preserved draftes `). Do not classify those earlier
  failures as lock-caused without contemporaneous evidence.
- Latest `/tmp/bbr-portal-consumers2.log`: Portal 3/11, 247.76s measured; shown
  owners stay `app_active=false`, `parent_is_key=false`, foreign frontmost window
  70321. Read-only `ioreg` then confirms `CGSSessionScreenIsLocked=Yes`.
  GUI work stopped; no lock-state changes, assertion relaxation or browser/VM
  fallback. Later consumer recipes did not execute because Portal failed.
- `/tmp/bbr-portal-verify3.log`: full `just verify` green, including workspace
  check/tests/Clippy, roadmap, UI boundary and 49 script tests. Earlier attempts
  caught missing DTO fixture fields and an untracked task link; both corrected.
- `/tmp/bbr-portal-testkit-clippy.log`: `mbx clippy -p bbr -p bbr-web-macos
  -p bbr-testkit --features testkit --all-targets -- -D warnings` passes.
  Existing `block` future-incompatibility warning remains.
- Commands use bounded `/tmp/bbr-portal-lifecycle.just`, a repository recipe
  copy replacing compiler Cargo calls with mbx and omitting only deferred perf
  from the aggregate. Repository justfile/targets/budgets unchanged. An earlier
  consumer attempt received SIGTERM during a cold build; no pass claimed.

Self-review checked live-intent/drop safety, observer removal from both centers,
non-key parent-relative ordering, truthful diagnostics and preservation of hidden
geometry coverage. Consumer E2E, full functional aggregate, Peek/Gallery and a
fresh full Portal pass remain open. Real desktop Space switching/fullscreen are
not proven here. No commit, merge, push, ownership transfer or completion claim.
Only the new task packet is staged so the roadmap validator can see its link.
Other code and documents remain unstaged. Planning/review elapsed time was not
instrumented.

### Unlocked retry and Tart-only continuation (2026-10-01)

The subsequently unlocked host retry passed all four new lifecycle regressions.
Portal minimize/restore failed once, then the focused rerun passed 1/1 (5.73s).
The later affected-suite retry passed Portal 11/11 (15.79s), Auth 7/7 (15.55s),
Sidebar 21/21 (122.75s), Zoom 7/7 (18.70s), the five Spaces binaries 14+4+2+5+2,
Windows 9/9 and Shutdown 3/3. Logs: `/tmp/bbr-portal-retry1.log`,
`/tmp/bbr-portal-minimize-retry.log`, `/tmp/bbr-portal-consumers-retry3.log`.
Command passed 28/28 and Find 6/6 in the preceding consumer retry logs. These
passes do not erase the earlier intermittent failures.

The canonical functional attempt `/tmp/bbr-portal-functional-final.log` then
stopped at Sidebar's fast-arrival assertion (`dwelling` instead of `floating`,
20/21 passed). No aggregate pass. Patrick requested VM-only further GUI tests
and approved private Tart NAT/SSH with a run-specific key, no host window,
bridge or port forwarding, and clone/key cleanup. No further host GUI test is
authorized or run.

The lane-local temporary runner `/tmp/bbr-portal-tart/run.py` snapshots only this
worktree's allowlisted source plus its fresh mbx `--no-run` artifacts; source
and binaries are SHA-256 checked again in the guest at their compiled absolute
paths. The host build passed (23.23s app test artifacts; 31.72s Gallery). A
throwaway `macos-27-golden-gate` clone uses 3200×1800 and a read-only payload
share, with the common host GUI gate retained. The first runner preflight found
an unsupported generic Cargo recipe while parsing the plan and stopped before
creating a VM; parsing now ignores recipes without an explicit test target.

Two VM preparations stopped before tests: the first checked the VirtioFS tag
instead of the mounted path, the second resolved the guest's Xcode-dependent
Python placeholder. Both clones and temporary private keys were deleted. The
runner now checks the mount path and realizes an explicit Darwin Python store
path inside the disposable guest; host setup and Golden Image stay unchanged.

First actual VM evidence: `/tmp/bbr-portal-tart/run-20261001-194050` verified
1,634 source/artifact hashes; Sidebar 21/21 (39.70s), Auth 7/7 (4.85s), Portal
10/11 (40.30s). Minimize/restore and two-window switching passed. Deactivation
failed because `NSApplication.deactivate()` left `app_active=true` even though
the parent lost key status and the Portal correctly hid. No assertion weakened:
the test now activates a second real BBR process, proving cross-process switching
rather than relying on this inconsistent driver. The unused `DeactivateApp`
request was removed; real hide/unhide stays. Rebuilt artifacts passed.

The first guest listener audit found inherited Screen Sharing/Kerberos and SSH
password/kbd-interactive login. These were disabled only in the disposable clone,
and subsequent fresh SSH plus listener/config checks proved key-only TCP 22,
no forwarding and only loopback test fixtures. Later clones apply this reduction
before tests. The first test clone/key were deleted successfully.

### Verified Tart handoff checkpoint

Final run `/tmp/bbr-portal-tart/run-20261001-194627` is green:
- Focused Portal 11/11, Sidebar 21/21 and Auth 7/7: 39 tests, 54.11s summed
  test time / 55.690s runner time. Cross-app switching uses two real processes.
- Recipe-derived canonical functional order, excluding only deferred perf,
  followed by Peek and Gallery: **58/58 binaries, 43 recipes, 345 tests**, zero
  failures or ignored tests; 361.56s summed test time / 384.822s runner time.
  Portal passes again in that sequence. All affected consumers are included.
- This executes host-built current-worktree artifacts directly inside Tart,
  not Cargo/`just` compilation in the guest. Each exact recipe's test flags and
  suite order are preserved; outer command budget is 15m, in-test deadlines
  unchanged, process-group cleanup remains bounded. No performance acceptance.
- `/tmp/bbr-portal-tart/verify.log`: fresh mbx-equivalent `just verify` passes,
  including 49 script contracts; testkit all-target Clippy `-D warnings` passes.
  Existing `block` future-incompatibility and mbx cache-prediction warnings are
  not suppressed. Formatting, roadmap and whitespace gates pass.
- Guest macOS 27.0 build 26A5353q, 8 vCPU, 16 GiB, 3200×1800; source and test
  artifacts verified by SHA-256. No Rust/test code changed after the VM snapshot;
  only documentary evidence changed afterward.
- Network probes from the host confirmed guest TCP 22 reachable and 88/5900
  unreachable after clone-only hardening. No bridge/forward/VNC/host window;
  test fixtures listen only on guest loopback. Cleanup records `stop_rc=0`,
  `delete_rc=0`, `private_key=deleted`; Tart inventory has no lane clone, the
  removed guest's SSH is unreachable, Golden Image remains stopped/unchanged.

Self-review checked retained-owner versus actual-child semantics, current-intent
coalescing, dismissal/drop supersession, removal from both notification centers,
public AppKit APIs, parent-relative ordering, testkit feature boundaries, hidden
fixture truth and strict shown-fixture assertions. No independent agent was
requested or used. The failed direct-deactivate test remains in the earlier log;
its replacement tests the actual cross-process user transition, not a relaxed
predicate. Historical intermittent host failures remain provenance, not erased.

The next local commit contains the coherent verified lifecycle fix and evidence.
No merge, push, other-worktree modification or ownership takeover. Integration
owner must preserve current main's additive contracts and run fresh combined-tree
Tart gates. Performance, real desktop Space switching and native fullscreen
completion remain unproven/deferred. Planning/review wall time was not measured;
test times above are measured, not product performance metrics.

Final parallelism audit: only this bounded lane is authorized here; main and
other lanes remain untouched. No new workflow/subagent/worktree is activated.
The slice is ready for serial integration, not a combined-main acceptance claim.

### Authorized Portal integration checkpoint (2026-10-01)

Patrick confirmed the ordinary development app and requested the local merge.
Main was clean at `80c56bb19361a99cb1630d3f0f64867e004db3f3`; the frozen source
was `aff55e14ccc1869abcfb7545b37775ca984a12f5`. Session/process, worktree and lock
preflight found no other main writer: the cleanup session awaited separate
worktree permission, and the internal-pages writer stayed in its isolated lane.
The no-ff merge is conflict-free and intentionally uncommitted until verification.
Shared testkit requests/snapshots, E2E helpers and architecture/roadmap text are
semantic unions: Favorites drag, favicon persistence and main's VM tooling remain.

Fresh combined-tree non-GUI gates and host `mbx --no-run` builds use
`/tmp/bbr-portal-merge/verify.sh` and `/tmp/bbr-portal-merge/build.sh`. The bounded
`/tmp/bbr-portal-merge/run.py` reuses the lane's hardened, source-allowlisted,
SHA-256-checked Tart runner with main paths and fresh manifests; it leaves the
newer repository VM tooling unchanged. Focused Portal/Sidebar/Auth precede the
recipe-derived functional aggregate without deferred perf, then Peek/Gallery.
The common GUI lock is retained conservatively; all GUI execution is in the
throwaway guest, never on Patrick's desktop.

First combined-tree gate `/tmp/bbr-portal-merge/run-20261001-202551`: focused
Portal 11/11, Sidebar 21/21 and Auth 7/7 pass (48.79s summed test time;
49.966s runner time). The aggregate completes all 59 binaries / 43 recipes,
including Peek/Gallery: 347 passed, one failed, no ignored tests (340.68s test
time; 364.774s runner time). Only Titlebar reload parity fails with `Some("")`
instead of `BBR Fixture Two`; all other binaries pass, including the new main
Favorites-drag and favicon-restart coverage. This is not a green aggregate.
The unchanged test and shared `wait_title` helper consume repeated title events;
`titlebar-event-ring.txt` shows old title → empty title → completed title during
reload, matching the previously documented pre-Portal failure. No test/source
change or assertion relaxation: one fresh VM was scheduled to run the focused
Titlebar suite, then the complete functional order on identical code/artifact
hashes if the focused gate passed.

Fresh `/tmp/bbr-portal-merge/verify.log` passes the full mbx-equivalent `just
verify` and touched-crate testkit all-target Clippy `-D warnings`; 49 script
contracts pass. Initial clone cleanup proves `stop_rc=0`, `delete_rc=0` and
`private_key=deleted`; Golden Image remains stopped and unchanged. The first
guest verifies 1,648 source/artifact hashes; host probes see TCP 22 only, with
88/5900 unreachable. Source/build/tooling hashes remain unchanged after gates.

Fresh recheck `/tmp/bbr-portal-merge/run-20261001-203439` reproduces the same
Titlebar assertion at `nav_clicks_match_keyboard_shortcuts` (line 490): 6/7 pass,
5.22s test time / 7.139s runner time. Its event ring again records completed
`BBR Fixture Two` after the transient empty title. The focused failure correctly
prevents another aggregate run; no blind further retry or acceptance claimed.
Both disposable clones and private keys are deleted, removed guest SSH is
unreachable, Golden Image unchanged, and the GUI lock is released. All 1,190
recorded code/build/tooling hashes still match the verified build. Only this
integration ledger changed after the runtime snapshot.

Integration stops with the conflict-free merge staged and uncommitted:
`HEAD=80c56bb19361a99cb1630d3f0f64867e004db3f3`,
`MERGE_HEAD=aff55e14ccc1869abcfb7545b37775ca984a12f5`. Next required input is
permission for the bounded Titlebar reload-synchronization repair and renewed
Tart gates before committing this merge. Do not drop the test, relax its final
title/session assertions, claim historical lane evidence as a combined pass,
or commit the red aggregate.

Integration parallelism audit: only this requested local merge was authorized;
no new workflow, subagent or lane. Other worktrees, branches and the user's
ordinary app are preserved. No push, branch/worktree removal, unrelated cleanup
or performance execution. The repeated gate failure is the blocked stop condition.

### Authorized Titlebar synchronization remediation

Outcome: repair only the existing reload parity test's readiness boundary and
finish the prepared local merge with fresh combined-tree functional evidence.
The two recorded real-app failures are red regression evidence. Scope is the
Titlebar suite's local helper/calls and integration documentation; no production
navigation or generic testkit behavior changes. For each reload, take a fresh
event cursor before the action, require that view's new loading/commit/completion
sequence, then observe the exact final title with loading false. Keep all title,
availability and session assertions and existing in-test deadlines. Verify with
focused Titlebar E2E, the full functional order plus Peek/Gallery in disposable
Tart, mbx-equivalent verify and testkit Clippy. Stop at a genuine failing gate;
no sleeps, relaxed assertions, host GUI fallback, push or unrelated fixes.

Parallelism audit: only this explicitly resumed integration/remediation is
runnable here. No subagent/workflow or new lane is authorized; other writers and
worktrees remain untouched. Performance execution remains deferred/unpassed.

### Verified local Portal integration

The Titlebar helper now takes the gateway's fresh event cursor before each
reload, requires `LoadingChanged(true)` → `DidCommitNavigation` →
`LoadingChanged(false)` for the same WebViewId, then waits for the exact title
and loading=false snapshot within the existing 30s deadline. Both keyboard and
click paths retain their title, availability and session assertions; URL and
loading-state assertions are added. No production navigation, generic testkit,
dependency, budget or timeout changes. The two earlier failing runs are the red
regression evidence; no failing predicate was removed or weakened.

Fresh combined-tree evidence:
- `/tmp/bbr-portal-merge-titlebar/verify.log`: full mbx-equivalent `just verify`
  and touched-crate testkit all-target Clippy `-D warnings` pass, including 49
  script contracts. Fresh host `mbx --no-run` builds pass. The existing `block`
  future-incompatibility warning remains unsuppressed.
- `/tmp/bbr-portal-merge-titlebar/run-20261001-211102`: focused Titlebar 7/7,
  Portal 11/11, Sidebar 21/21 and Auth 7/7 (46 tests; 62.65s summed test time,
  64.777s runner time). Two additional Titlebar processes each pass 7/7
  (4.13s / 3.52s test time; 4.298s / 3.664s runner time).
- The complete recipe-derived functional order, omitting only deferred perf,
  then Peek/Gallery passes **59/59 binaries, 43 recipes, 348 tests**, zero
  failures/ignored tests; 533.75s summed test time, 569.895s runner time.
  Titlebar passes for the fourth time; all Portal consumers, main's Favorites
  drag and favicon persistence remain green. This is fresh combined-main
  evidence, not reused source-lane evidence or performance acceptance.
- All GUI execution used the disposable macOS 27 Tart guest with 3200×1800,
  8 vCPU and 16 GiB. Source/artifact hashes were checked in the guest; all
  1,190 code/build/tooling input hashes still match after the run. Only
  integration documentation changed afterward. Original recipe test flags,
  sequential order, bounded process-tree cleanup and the conservative common
  GUI gate were retained; no host GUI test was run.
- Clone-only listener/authentication hardening and host probes prove key-only
  TCP 22 with 88/5900 unreachable and no forwarding. Cleanup records
  `stop_rc=0`, `delete_rc=0`, `private_key=deleted`; removed guest SSH is
  unreachable, Golden Image stopped/unchanged, and this run released its gate.

Self-review checked fresh event correlation, same-view matching, commit-before-
completion ordering, the final snapshot boundary, unchanged deadlines and
session/title assertions, additive merge unions and source-hash provenance.
No independent agent was requested or used. Final format, roadmap and whitespace
checks pass. Test timings are measured tooling runtime, not product performance;
implementation/review wall time was not instrumented.

The local merge has parents `80c56bb19361a99cb1630d3f0f64867e004db3f3` and
`aff55e14ccc1869abcfb7545b37775ca984a12f5`; it includes only the Portal slice,
its authorized Titlebar test remediation and integration evidence. No push or
branch/worktree removal. Performance execution, actual desktop Space switching
and native fullscreen completion remain unproven/deferred. Final parallelism
audit: no other lane or subagent authorized; other owners and worktrees remain
untouched and no additional work is activated by this completed integration.

## Isolated visual-tooling lane (2026-09-28)

Patrick initially authorized this isolated worktree alongside the product writer.
He subsequently requested its local merge into `main`; the integration record
below supersedes the historical no-merge/branch-only statements. Those earlier
checkpoints remain provenance, not current ownership or acceptance claims.

| ID | Slice | Status | Worktree | Owner | Contract | Next gate |
|---|---|---|---|---|---|---|
| VISUAL-SNAPSHOTS | Reusable visual snapshots | `integrated` | local merge into `main`; preserved source `parallel/visual-snapshots@7f5f6243` | session `01a0e635`, integration settled by this merge; no ongoing product ownership | [visual snapshots](../roadmap/tasks/visual-snapshot-tooling.md) | Combined-tree functional/capture/contract gates pass; all eight browser references require explicit re-review after Paper alignment. No baseline updates or push |

Parallelism audit: only this requested tooling lane is authorized here; V2-S12
is owned by the other session. RA-MEDIA retains its explicit sequencing;
Split, performance and the other blocked/deferred items stay untouched. No
subagent/workflow is authorized. Territory is the capture test, visual adapter,
CLI vendor and baselines. Expected shared conflicts: justfile, roadmap/current
ledger and E2E documentation; resolve as semantic unions. Product alignment
integrates first, tooling second, with fresh baseline review on the combined
tree. GUI captures acquire the same common-directory lock. Base is a clean
committed checkpoint; this lane must run its own gates, not infer a current
pass from historical evidence.

### Tooling evidence and handoff boundary

Outcome: unchanged CLI vendored as `539c4cbc`, BBR adapter and eight reviewed
baselines delivered in the following local implementation commit. No merge,
push, main-worktree mutation or ownership transfer was performed.

Evidence (all paths below are on this host):

- `just visual-test`: 15 BBR adapter tests and 29 unchanged upstream contracts.
  Self-review remediated added vendor-source detection, non-regular PNGs and
  holding the existing GUI lock through timeout/process-group cleanup. The two
  input-validation regressions failed before the fix; the new lock-lifetime
  test exercises a SIGTERM-resistant child and proves reaping before unlock.
- First valid capture `reports/visual/runs/1790570337671813000-55313` failed only
  for eight missing baselines. All eight PNGs were opened and inspected, then
  accepted with eight explicit IDs. Independent run
  `1790570427719032000-82082` passed 8/8 exact, zero errors. Native content/traffic
  lights remain intentionally absent, not synthesized.
- After adapter hardening, fresh run `1790571479914399000-46151` and explicit
  `just visual-compare` pass 8/8 exact. Its suite takes 6.24s (measured); the
  earlier independent suite took 4.21s. These are capture timings, not product
  performance. An old run is correctly rejected with exit 2 after source edits.
  Report: `reports/visual/latest/index.html`; `visual-doctor`: 1130 source inputs.
- Workspace `verify` passes, including 49 existing script tests. Testkit
  all-target Clippy and visual-capture test Clippy pass with `-D warnings`.
  Existing Cargo recipes were invoked through a temporary mbx-equivalent
  `/tmp/bbr-visual-verification.just`; no target-dir/cache-wrapper override.
- Canonical functional sequence (only deferred perf omitted) stops at
  `e2e-shell-titlebar-navigation`: reload parity at `main.rs:488` sees `Some("")`
  instead of `BBR Fixture Two`. One focused rerun reproduces it. Event ring
  `/tmp/bbr-st-titlebar-nav-parity-86914/event-ring.txt` shows the old title
  during reload followed by an empty title, then the completed title.
- Remaining recipes were run sequentially, not silently skipped. Portal is
  6/7: `host_hides_on_minimize_and_restores` fails hit-test convergence with
  foreign frontmost window `28977` versus parent `29431` / portal `29432`.
  Diagnostics: `/var/folders/8j/xtk9v77x03q_cqw3fmh7hdw40000gn/T/bbr-st-portal-minimize-31641`.
  Host occlusion is observed; no product cause or repair is claimed.
- Overall 28/30 functional recipes pass, with the two named failures. Peek
  passes 7/7, Gallery 46/46. Product source and both failing suites have no lane
  diff. Neither the aggregate nor performance is marked passed.

Logs: `/tmp/bbr-visual-verify.log`, `/tmp/bbr-visual-final-contracts.log`,
`/tmp/bbr-visual-functional-gates.log`, `/tmp/bbr-visual-remaining-gates.log`,
`/tmp/bbr-visual-snapshots-sealed-final.log`; machine outcomes are in
`/tmp/bbr-visual-functional-results.json` and
`/tmp/bbr-visual-remaining-results.json`. Measured first gate sequence: testkit
Clippy 50.477s, capture Clippy 12.499s, functional aggregate to failure 97.188s,
Peek 19.509s, Gallery 32.217s; remaining 22 recipe invocations 358.710s.
Planning/implementation/review wall time was not instrumented.

Final parallelism audit: no additional lane authorized; main remains owned by
`01a0e454`; media/Split/performance boundaries unchanged. Stop at the two
functional blockers and handoff boundary rather than changing unrelated
product tests. Next owner needs explicit handoff, product-first integration,
resolution/recheck of the failing gates, and fresh combined-tree captures with
explicit review before any new baseline acceptance.

### Gallery coverage extension (user-approved)

| ID | Slice | Status | Worktree | Owner | Contract | Next gate |
|---|---|---|---|---|---|---|
| VISUAL-GALLERY | Offscreen Metal Gallery coverage | `planned` | capture/command foundation locally integrated into `main`; source worktree preserved | no active writer after integration; continuation requires activation | [Gallery visual coverage](../roadmap/tasks/gallery-visual-coverage.md) | Combined-tree capture validates 1,872 pages; all Gallery baselines remain absent. Locked-session proof, supported-state/framing audit and visual review remain open |

Activation preflight: clean index/worktree at the named base, no lane test
processes or Git index locks. Existing GUI lock remains mandatory. Main is at
`a10a52262facbdb5a4eddec447f6ebf3c515ffa7` (worktree metadata only); no main or
other lane source is inspected or mutated. The earlier visual adapter's exact
capture/contract gates are green; its two functional blockers remain open.

Parallelism audit: Patrick approved Gallery-first coverage followed by bounded
app-composition slices. Only this local writer is authorized; no subagents or
extra worktrees. Product/Paper remains owned by `01a0e454`. Gallery, adapter,
visual configs and baselines are this lane's territory. Shared conflicts are
Gallery story wiring, recipes, roadmap/current ledger and E2E documentation;
resolve semantically product-first, tooling-second, then recapture/re-review.
No dependency/runtime contract changes. RA-MEDIA, Split and performance remain
deferred; additional browser composition capture awaits its own bounded packet.

### Gallery fixture checkpoint and blocked capture evidence

The first bounded increment fixes Gallery-only fixture wiring: Reduce Motion now
reaches the rendered theme (the regression failed before the fix), Command no
longer hardcodes Dark, Sidebar model tint/pitch and renderer share a resolved
family theme, and Tab/Favorite menu fixture heights contain the production rows.
Family densities remain intrinsic; named increased-contrast/reduced-motion
families retain their requirements. No bbr-ui or browser production source change.
Self-review found a half-point menu underallocation; the natural-layout regression
failed, then passed after per-row rounding. No snapshot masks/tolerance/acceptance.

Evidence before the host locked:

- Gallery unit/contract tests 155 passed with the capture feature (150 without
  it); feature all-target Clippy `-D warnings` passed. Existing semantic Gallery
  E2E plus two new regressions: **48/48**, 25.69s measured. These tests use the
  established TestPlatform and are not native screenshot evidence.
- Native optimized probe `bbr-gallery-native-settled-c` under `/private/tmp`:
  49 stories, 1,260 selector/theme cases, 1,872 PNG pages; first-to-last image
  write span 169.95s (measured, not total runtime or product performance).
  Menu, Sidebar and Command images inspected. Earlier focused Command repeats
  `bbr-gallery-command-fixed-a` / `-b` matched 144/144 PNG bytes, but preceded
  final family-theme reconciliation and are not final baseline candidates.
- Partial full repeats `bbr-gallery-native-settled-a` (1,334 pages) and `-d`
  (1,593) timed out; incomplete runs have no coverage TSV and are unusable.
  Comparison of `-c`/`-d` isolates 12 RGBA differences to Tab Row page 0; the
  16×16-pixel attention dot retains its finite opacity pulse under Reduce Motion.
  A production-duration endpoint wait was added to the uncommitted prototype;
  it still needs native verification. Reduced Motion fixed the other animated
  States/Favicon probe differences, not all animation by definition.
- The final `bbr-gallery-native-diagnose-a` produced no frames and hit its 300s
  deadline. `ioreg` then reports `CGSSessionScreenIsLocked=Yes`. Do not infer the
  earlier intermittent stalls have the same cause without contemporaneous data.
  Native GUI work is stopped, process cleanup completed, GUI lock released.

Logs: `/tmp/bbr-gallery-settled-gates.log`, `/tmp/bbr-gallery-regressions.log`,
`/tmp/bbr-gallery-framing.log` (red), `/tmp/bbr-gallery-framing-after.log` (green),
`/tmp/bbr-gallery-settled-repeat.log`, `/tmp/bbr-gallery-native-diagnose.log`.
All native probes are unsealed diagnostics, not accepted comparator runs.
The exact all-state audit, nested/shadow framing, bounded sealed adapter/recipes,
independent final comparison and broader functional gates remain open. Initial
Titlebar/Portal blockers remain unchanged; performance remains deferred.

Checkpoint parallelism audit: no other lane/subagent/worktree is authorized.
Product/Paper remains with its existing owner; Split/media/performance are not
runnable here. Preserve this lane's prototype dirty tree and ownership.

Headless continuation (Patrick approved): replace native-window scheduling with
pinned `HeadlessAppContext`, injected macOS text system and the existing offscreen
Metal renderer. No NSWindow, activation, DisplayLink or hidden-window fallback.
Keep the existing host lock through process cleanup for conservative GPU
serialization. First prove small probes, animation endpoints, independent exact
repeats and locked-session operation without changing the user's lock state;
then expand. Saved native probes are diagnostic references, not baseline authority.
No additional lane, subagent, worktree or browser-composition scope is activated.

### Offscreen Metal foundation checkpoint

Implemented `just gallery-capture [story-id]`: optimized, production Gallery
components with real macOS text and existing Metal offscreen textures, no native
window/event loop/activation. Registry-derived selector/theme cases, measured
pagination, production-duration finite opacity endpoints, arena clearing and
ordered app shutdown are explicit. The small bounded adapter resolves the mbx
artifact, retains the existing host lock through process-group cleanup and
rejects missing/extra/duplicate pages, broken child ranges and wrong dimensions.
This is intentionally an **unsealed probe**, not comparator/baseline acceptance.

Runtime evidence:

- `/private/tmp/bbr-gallery-headless-full-a` and `-b`: each 49 stories, 1,260
  selector/theme cases, 1,872 PNG pages plus completion TSV; all files byte-exact
  across independent processes. Measured wrapper wall time 50.64s / 50.26s.
  CoreGraphics observations: 838 / 822 samples, zero native producer windows.
- Compared with saved native `bbr-gallery-native-settled-c`: 1,860 PNGs identical;
  only the 12 known Tab Row attention-dot pages differ. Each difference is
  confined to its 16×16-pixel dot (density-specific y); the production-duration
  endpoint is now repeatable. No mask, tolerance, baseline or production change.
- Real recipe run `reports/visual/gallery-probes/1790594701123197000-6956`:
  1,260 cases / 1,872 validated pages, 50.95s capture+validation; all PNGs and TSV
  byte-exact with independent A. Focused Command observer run
  `/private/tmp/bbr-gallery-headless-observed-final`: 328 observations, no native
  window, zero producer-frontmost samples, same foreground PID before/after.
- Every observation reported an on-console session with no locked flag; **locked
  operation is still unproven**. The user's session was never locked/unlocked by
  tooling. Metal API availability alone is not treated as locked-session proof.
- Two first pilots exposed and fixed lifecycle errors: typed WindowHandle update
  leased Root during explicit draw; app context initially dropped before the held
  Gallery entity. Untyped app update and entity-before-context drop correct both.
  The completion marker is now written only after successful shutdown.

Verification: 150 normal Gallery tests / 156 capture-feature tests; normal and
capture-feature all-target Clippy with `-D warnings`; 48/48 semantic Gallery E2E
both normally (24.10s) and with capture enabled (24.34s measured); format and
roadmap checks. Final `just visual-test`: 25/25 Python adapter/probe tests and
29/29 unchanged upstream contracts, including the added lock-ownership contract. A GC-evicted mbx target
symlink exposed an existing provenance bug, fixed separately in `7be94ca` after
a red regression, without touching comparator sources or weakening file checks.

Logs: `/tmp/bbr-gallery-headless-{build,full-a,full-b,final-gates,e2e,recipe,final-contracts,observed-final}.log`,
`/tmp/bbr-gallery-visual-contracts.log`; observation JSONs beside the temporary
probe directories. Throughput is tooling evidence, not product performance.
Self-review checked dependency boundaries, real renderer/text injection, no
window fallback, finite readiness, cleanup, completeness and acceptance limits.

Next bounded work remains the supported-state/applicability and nested/shadow
framing audit, then sealed unchanged-comparator integration and HTML review.
Locked-session proof awaits an actually locked host, not a tooling-induced lock.
No full acceptance, functional aggregate rerun, performance, browser-composition
activation, baseline acceptance, merge, push or handoff is claimed. Final
parallelism audit: only this writer is authorized; other owners and deferrals
remain untouched. The current checkpoint does not activate additional lanes.

### Unified visual commands (user-requested follow-up)

Patrick requested one command for Gallery **and** the existing browser snapshots,
plus one command to update reviewed snapshots. Sole writer and existing branch
remain unchanged; clean activation base `47b6ea31`. No extra lane/subagent or
browser product changes. Scope is the adapter/config/recipe/report wiring only.

`just visual` runs adapter/upstream contracts, then both capture/compare/report
suites. `just visual-update <batch> <suite:case>...` requires explicit reviewed
selections; it never chooses latest, accepts all, recaptures or adopts environment
drift. Each suite keeps its own baseline/environment authority and upstream seals.
Batch HTML links the original comparator reports; no fork or second comparison
algorithm. Gallery identities are renamed to put appearance last without changing
PNG bytes. Both checks run on review-needed/failed suite results. Cross-suite
updates preflight every selection but remain explicitly nontransactional.

First real `just visual` batch `1790597536807106000-94418`: Gallery 1,872 sealed
candidates, all missing baselines as expected, zero validation errors (86.68s
including build/seal/report); browser 8/8 exact, zero errors (33.45s). Aggregate
exit 1 is correct, not a failed capture or a green acceptance claim. HTML:
`reports/visual/batches/1790597536807106000-94418/index.html`. Follow-up source
edits invalidate that first batch for acceptance; recapture rather than reseal.

The actual update router is exercised with the unchanged CLI in a temporary Git
fixture: one selected image per suite is accepted, unselected images stay absent,
stale report/baseline and changed source attempts fail without new writes. Real
Gallery baselines are not accepted as part of command implementation. Existing
Titlebar/Portal blockers, native-browser window requirement, locked-session proof,
all-state/framing audit and product-first integration remain open.

Final fresh batch `1790598062285304000-41557` proves the final source tree:
Gallery 1,872 missing-baseline candidates with zero errors (156.72s including
managed-target rebuild), browser 8/8 exact with zero errors (45.13s). Aggregate
correctly exits 1. Both manifests pass upstream source/image validation; all
1,880 image hashes match the first independent batch. Report:
`reports/visual/batches/1790598062285304000-41557/index.html` (generated, not opened
in a managed browser). No real baseline changes. A real update attempt against
the first stale batch was rejected before writes; the final wrapper reports that
error without a Python traceback.

Final `just visual` includes 35 Python unit contracts (the integration class is
skipped in discovery), 29 unchanged upstream contracts and one separately run
real-CLI two-suite update integration test. `just script-tests`: 49/49;
`just roadmap` and whitespace checks pass. No Rust production code changed;
full functional aggregate/performance are not rerun for this bounded tooling
increment. Logs: `/tmp/bbr-visual-unified-{final,contracts,update-tests,seals,script-tests}.log`
and `/tmp/bbr-visual-update-stale.log`. Timings above measure complete tool phases,
not product performance.

Self-review checked independent namespaces, source/environment/PNG seals, complete
Gallery pairing without pixel edits, continuing both checks after a non-pass,
explicit-only acceptance, stale/invalid no-write paths, signal/lock lifetime and
managed-target eviction recovery. Parallelism audit: only this requested adapter
lane is runnable here. Product/Paper ownership and Split/media/performance deferrals
are unchanged; no merge/push or handoff. Current all-state/framing/locked-session
and baseline-review limits remain explicit.

### Authorized local integration (2026-09-28)

Patrick requested “merge es in main bitte”. Preflight found clean, settled
`main@a10a52262facbdb5a4eddec447f6ebf3c515ffa7` and source
`parallel/visual-snapshots@7f5f6243adf30f447873c5c46c6e0b720c95bb86`.
Product session `01a0e454` was paused after committing S12 evidence; no BBR
compiler/capture or Git-lock owner was active. Session `01a0e635` took only the
bounded integration work on the existing main worktree, preserving every Paper
commit and the source branch/worktree. The no-ff merge was conflict-free;
Command/Sidebar fixture unions preserve Paper model additions and capture theme
resolution. No product source was changed by integration.

Combined-tree verification before the merge commit:

- `just verify`, through `/tmp/bbr-visual-merge.just`, passes (the temporary
  recipe copy substitutes mbx for compiler-invoking Cargo commands only).
  Normal Gallery tests: 150; script contracts: 49.
- Bounded mbx all-target testkit Clippy and Gallery capture-feature Clippy pass
  with `-D warnings`; capture-feature Gallery tests: 156. Browser visual-capture
  test Clippy also passes.
- Canonical sequential `just e2e` with only the explicitly deferred `e2e-perf`
  omitted, plus Peek and Gallery: **32/32 recipes, 281 tests, zero failures**,
  385.14s measured including builds. Titlebar passes 5/5 and Portal 7/7;
  this supersedes the earlier functional blockers on this combined tree.
  Gallery passes 48/48 normally (24.70s test time) and 48/48 with capture enabled
  (24.80s test time). The console was observed unlocked before the sequence;
  no lock-state change was induced. S12 product closeout is not performed here.
- `just visual`: 35 Python unit contracts, 29 unchanged upstream contracts and
  the separate real-CLI two-suite acceptance integration test pass. Fresh batch
  `1790600840022959000-69552` produces 1,872 Gallery pages (all missing baselines)
  and eight browser images (all changed against pre-Paper references). Both
  reports have **zero validation errors and no environment mismatch**. Correct
  aggregate exit is 1 (review required), not green visual acceptance.
  Gallery phase 142.71s, browser phase 40.88s, whole command 205.41s measured;
  these are tool timings, not product performance.

Report: `reports/visual/batches/1790600840022959000-69552/index.html`.
It is generated, not claimed visually reviewed. Both suites pass the upstream
source/image seal validator after the documentation updates. No Gallery or
browser baseline is accepted or rewritten. Final roadmap, format and whitespace
checks pass. Logs: `/tmp/bbr-visual-merge-{verify,feature-gates,e2e,gallery-feature-e2e,visual,capture-clippy,seals,final-docs}.log`.
The existing comparator, pixel exactness, native exclusions and explicit-update
policy are unchanged. Self-review checked the shared fixture unions, absence of
product-source edits and truthful separation of merge verification from visual
acceptance. Locked-session capture proof and all-state/nested-framing audits
remain open. Historical source-branch runs are not combined-main evidence.

Integration parallelism audit: only the requested merge is authorized; no
subagents, new lanes or worktrees. Product-first/tooling-second order is retained.
RA-MEDIA sequencing, Split prohibition and performance deferral remain unchanged.
No push, branch deletion or worktree removal; no unrelated product work starts.

## Post-F2a parallel wave

The former post-F2a and Daily-driver waves are closed historical evidence. Their lane details remain in Git and [the completed Daily-driver delivery](2026-08-12-daily-driver-delivery.md); they grant dependencies but schedule no work.

| ID | Status before F2a close | Worktree / owner | Exclusive territory | Start condition | Integration priority |
|---|---|---|---|---|---|
| LEGACY-F2A | `closed` | historical worktrees; no current owner | completed footer/appearance/swipe and dependent UI foundation | complete | none; evidence only |

## Serial integration order

1. `CONTROL-RECONCILE` is closed; its exact durable-contract commit is `d2b507f56b9da068e7df0aa08c3f8b12cdec506c`, and no writer remains active.
2. Do not infer a product lane from an idle historical worktree or session. Every new writer requires a fresh exact-base, status, process, lock, and ownership preflight.
3. `FOLDER-PIN-INVARIANT` has landed, so Folder-menu behavior may assume folders contain pinned tabs. The design-branch packets (`DESIGN-CARD` → `DESIGN-ADDRESS` → `DESIGN-FAVORITES`) are integrated.
4. V2 ran serially on `main`: S0 → S1 → S2 → S3 → S4 → S5 → S6 → S9 → S7 → S8 → S10 → S11, all integrated (through `92263be8`); S12 (Paper alignment, from Patrick's 2026-09-28 comparison request) followed serially and is integrated through `05015039` (Portal 7/7 on the unlocked console; documentary closeout done).
5. RA-MEDIA resumes after V2-S12 as a capability-first sequence: macOS adapter → Sidebar media control (its portable contract is integrated).
6. Split stays blocked; no preflight, inspection, port, review, or mutation of `bbr-split-app-composition` is authorized.
7. GUI suites remain host-exclusive through the repository lock and need an unlocked macOS session (a locked session presents no frames, so render-gated E2E time out). Performance execution is deferred to the end of the queue.

## Downstream dependency queue

| ID | Status | Depends on / boundary |
|---|---|---|
| DD-FOLDER-MENU | `blocked` | Requires the integrated shared menu seam; the delete truth is settled (Unpack) and `FOLDER-PIN-INVARIANT` is integrated |
| DD-SHIFT-LINK | `planned` | Shift-modified link behavior remains intentionally unclassified; no writer may infer foreground/window semantics |
| DD-MANUAL-SMOKE | `planned` | Requires a fresh deterministic package; representative-site smoke supplements rather than replaces automation |

## Open product and evidence work

| ID | Status | Owner / blocker | Required evidence or decision |
|---|---|---|---|
| SPLIT-COMPOSITION | `blocked` | Patrick must explicitly re-enable Split work | Fresh preflight from current `main`; stale worktree remains untouched |
| PERFORMANCE | `blocked` | Patrick deferred performance execution to the end of the queue (2026-09-27, after a first un-deferred run) | Preserve code/budgets/baselines/sample guards; execution and acceptance remain unpassed. The un-deferred run (unlocked session) showed that matrix blocks added under the override were never executed: four driver assumptions were fixed in `2b15ffb8`. The matrix now stops at the renderer-responsiveness block, which requires a two-tab warm pair (exactly two active-space tabs, global session counts relative to a baseline) that no matrix position provides (three active-space tabs, tab 0 pinned by the folder invariant) and reads the injected unresponsive state without waiting for it. Planned fix: give the block its own two-tab space, wait for the derived state under a deadline, restore the matrix; a partial attempt is kept outside the repository as a reference only |
| STARTUP | `planned` | Phase 9 | Last recorded `535.866375 ms > 500 ms`; hard budget unchanged |
| RELEASE | `planned` | Phase 9 | Packaging, Accessibility, canonical runner, energy/resource, polish, and release closeout |
| MVP-BOUNDARY | `blocked` | Patrick owns the final finite In/Out classification | No feature-complete First-MVP claim before explicit decision and evidence review |
| RENDERER-A5 | `blocked` | External Apple WebKit callback/error hop | A1–A4 integrated; A5 remains explicitly unpassed |

## Standing decisions and constraints

- The Reference Shell packet is complete at exact reviewed tree `5f2b8a04c0c5bbf026e891058a3fabce17cfc191`, integrated as `4d1a534300344af48d28d8cb7f435b95c0548f02`. Its recorded performance/startup/packaging/Accessibility exclusions remain open; they do not reopen the visual slice.
- Sidebar Polish is complete through `35c91189a4a895e3c7f627534acf00a56899fc7b`; its lane commits `e1081aeb`, `2ffc27fa`, and `891c720a` are all ancestors of current `main`.
- Patrick's 2026-08-14 design decision supersedes the old “separate regions” wording: folders are grouped pinned tabs in one organized area, while loose pinned rows remain above folders. Runtime currently does not enforce `folder member => pinned`; only the dedicated open packet may change that authority.
- Patrick's 2026-09-27 decisions: the design-branch drift ledger is adopted as proposed (DESIGN §17.14/§17.16; eight rows adopted, the compact media bar and non-catalog footer glyphs rejected); the media-card presentation answers live in the sidebar-media-control packet; deleting a folder is Unpack; Peek follows the shell appearance; the dark selection tint is recalibrated. Performance execution was lifted for one evidence run, then deferred again to the end of the queue: `e2e-perf` is excluded from slice gates until then, and no budget, probe or assertion is weakened.
- `design/browser-shell-paper@414c97c7` preserves the wider Paper exploration. Content card, revised address field, Favorites affordances, and other unimplemented targets on that branch are not shipped claims and must receive bounded packets before implementation.
- Exactly one writer per worktree. Forward recovery only: no reset, clean, stash, amend, force-push, or silent ownership takeover.
- No second renderer or product authority in `bbr-ui`/Gallery; no private WebKit SPI; profile isolation and native WKWebView child composition remain invariant.
- Performance code remains mandatory while performance execution and canonical aggregate acceptance remain deferred/unpassed. Until the deferral ends, the stable-slice gate is the `just e2e` aggregate without `e2e-perf`, plus `e2e-peek` and `e2e-gallery`.
- No remote is configured and no push occurs as part of this generation.

## Cutover and rollback protocol

This generation replaces stale execution text that still described Sidebar Polish as awaiting integration after it had already landed. Historical detail remains available through Git and the linked ledgers, but it cannot schedule work.

Before each activation or integration, verify branch, exact base, worktree status, processes, Git locks, GUI lock, ownership, and expected shared conflicts. If documentation, worktree, and runtime ownership disagree, fail closed and preserve state before changing authority.

Rollback means returning to the named committed control pointer after preserving every newer branch and dirty worktree; it never means resetting or deleting active work. If no defensible path remains, stop with exact fingerprints, evidence, attempted paths, blocker, and the smallest required user input.
