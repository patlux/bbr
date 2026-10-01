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
| VISUAL-RUST | Standalone GPUI Visual consumer migration | `running` | main, activation base above | session `01a0f8f4` | [visual tooling](../roadmap/tasks/visual-snapshot-tooling.md) | Immutable standalone import and portable adapter/upstream contracts first; no native capture or baseline acceptance yet |

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
