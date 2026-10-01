# Rust-first visual tooling migration

Active goal: `1790607331142-4a53bba10bf7a8` (2026-09-28). This document owns
cross-consumer migration scope and evidence, not product visual acceptance.
Canonical home moved from the SDK to this repository with Patrick's explicit
extraction approval. [Extraction evidence](extraction.md) records the current
checkpoint; the SDK keeps a pointer and its consumer-specific progress.

## Outcome and boundaries

One portable `cargo-gpui-visual` library/CLI owns validation, exact RGBA8
comparison, original HTML review and explicit acceptance. Small project-local
Rust producers own production views, deterministic fixtures, supported variants,
semantic actions/assertions and named capture moments. Gallery and capture reuse
those definitions. Existing GPUI tests remain the execution basis; no new general
test framework, plugin system, scenario DSL or consumer report frontend.

Keep existing HTML/CSS/JS assets, external Git/mbx/Xcode and authorized device
tools. Replace owned Python orchestration on the visual path, not unrelated SDK
or consumer scripts. GPUI-bound helpers stay separate from the portable tool;
consumers retain their own pins and root-owned dependency graphs. New package
boundaries require demonstrated reuse rather than speculative abstraction.

Non-goals: changing product pixels, accepting references, completing independent
native/accessibility/performance gates, publishing, upgrading GPUI or migrating
unrequested consumers. BBR Split, RA-MEDIA and deferred performance stay outside
this work. No listeners, device resets or use of real consumer data/credentials.

## Approved extraction-first amendment

The portable authority is now developed in standalone `gpui-visual`, not inside
the SDK. First preserve behavior and provenance, verify the standalone package,
and turn the SDK into a pinned consumer. The helper stays in the SDK for now.
All R1–R7 requirements and exclusions below still apply; extraction does not
prove the Rust API, capture pilots, multi-suite UI or native coverage.

## Approved report redesign (2026-09-30)

Patrick approved the Paper design „A · Prüfstand“ and asked to implement it
(„bitte umsetzen“). The original report generator and its embedded CSS/JS are
restyled in place; it stays one static HTML file with the same CSP, controls and
single-report/collection contracts, not a replacement frontend. Comparison,
seals and acceptance are unchanged; the report only reads the stored diff image
for per-row columns. Consumers keep their pinned exports until a separately
verified import; no consumer, baseline or native capture was touched.

## Initial ownership and dependency audit

| Target | Observed base | Next allowed work |
| --- | --- | --- |
| GPUI Visual | new owned local repository from SDK `7263178ffcb9921990aadd34bfe8d6918426cbc7` | Portable tool, report and migration contract; no remote |
| SDK | rechecked clean `7263178ffcb9921990aadd34bfe8d6918426cbc7` | Pinned tool import and SDK fixture; previous visual session is paused |
| BBR | clean `b35c33f2e980a2e6fdf44f98a369c6d3f042cf2c` | Read-only until product-writer ownership rechecked; product session is active again |
| Hackerschau | previous audit found dirty app/tool/helper extensions | No writes/import of dirty overlays; recheck and coordinate clean immutable reconciliation |
| Respawn FM | no existing visual integration in prior tracked-file audit | First adoption; current Gaming fixtures only; recheck ownership before edits |

Only this local writer is authorized. No workflows/subagents, extra worktrees or
independent-review claim. SDK portable checks do not require the GUI gate. Real
capture uses host-wide exclusion through descendant cleanup, including offscreen
Metal. Do not infer writer ownership from a clean status alone.

BBR integration belongs to its existing `docs/roadmap/tasks/visual-snapshot-tooling.md`
and `gallery-visual-coverage.md`, with volatile ownership in
`docs/delivery/current.md`. Link this contract there when that worktree is available;
do not write another product owner's ledger concurrently.

## Ordered slices and acceptance evidence

| Slice | Required evidence | State |
| --- | --- | --- |
| R0 contract | Exact clean bases, ownership, scoped gates and preserved exclusions | Recorded; extraction-first amendment approved |
| R0a extraction | Immutable provenance; unchanged executable payload; standalone tests/Clippy/fmt; SDK pinned import | Verified: tool `94f4437`, SDK `2f6203a`; see extraction evidence |
| R1 portable API/execution | Reusable Rust capture/config API; bounded subprocesses, cleanup and locks; CLI regressions and strict Clippy | API, bounded Unix producer/probe/Git and guarded host lease verified on macOS; consumer gate adoption remains R2/R4–R6 |
| R2 SDK pilot | Rust environment/fixture glue; same eight identities/dimensions/pixels before/after, two independent complete runs; no baseline change | Verified in SDK `1388807`: frozen old/new exact parity, two scenario runs, two final configured runs; Python probe/duplicate producer removed, real host gate used |
| R3 upstream review collection | One original report for independent suites, collision-safe IDs/assets, per-suite failures/environments/seals; explicit selection routing | R3a report/catalog and R3b exact acceptance verified in portable fixtures; consumer adoption remains R4–R6 |
| R4 BBR pilot/migration | Static, interaction and multipage proof, then complete Gallery/browser repeats; remove replaced Python glue; full stable-slice gates | Isolated portable import verified at BBR03bbe7f; bounded scenario pilot recorded at8804ab2, not yet run |
| R5 Hackerschau | Reconcile clean group/sequence changes; host migration and native orchestration separately; preserve withholding/completeness failures | Host verified: fork merged as `441d3d4`, Hackerschau Rust host pipeline with exact old/new parity. Native capture ported to Rust with device-free parity on a fake simulator; real simulator run pending approval |
| R6 Respawn FM | Production Gaming view fixtures, isolated temporary store, graph/build/capture tests and separately authorized native journeys | Host pilot verified (`221aea4`, `b0a1211`, `31b08ff`); native install/exercise and baseline review open |
| R7 closeout | Requirement-to-artifact audit across every claimed scope; no missing work hidden by aggregate green | Open |

Do not hide unrelated runnable work behind a blocked consumer; do not mark the
multi-consumer goal complete with an omitted consumer. Local commits record exact
checks per coherent slice; no push, amendment or destructive Git operation.

## Current checkpoint

Portable extraction and SDK import are complete as a bounded slice. SDK pins
`94f4437981ed2e29112e8cf7af71362b9e591e32` in consumer commit
`2f6203ab36c1bb4af85db4e69c2ecbbdac42f59c`; it does not automatically track
subsequent development here. [Exact checks and limitations](extraction.md).

R1a exposes `config::Config` and `capture::capture`; CLI and Rust callers share
one producer/seal implementation. Five new disposable-fixture API tests initially
fail to compile because the modules are absent, then pass after extraction (not
a claimed runtime bug red). Final fresh full suite43/43 (4.905s total), strict
all-target Clippy (0.544s) and scoped fmt pass.
Guards cover strict schema, explicit paths/overlap, argv `{run}`, no overwrite,
source/environment drift, failures, duplicate/missing/unexpected/invalid images,
raw self-seal rejection, sealed policy/source/hash identity, and no review or
baseline writes. Sealed images are now validated before the public API returns,
rather than relying on a subsequent compare. No dependencies, report assets or
acceptance/comparison implementation changed. Logs `/tmp/gpui-visual-api-red.log`
and `/tmp/gpui-visual-api-final-{tests,clippy,fmt}.log`; first green checks also
retained. No consumer import or real capture in R1a.

R1b adds bounded `process::run` and routes capture/environment probes through it.
Private Unix groups retain their leader until group kill, then reap and require
group disappearance before returning. Default producer30min/probe30s, cleanup10s,
probe stdout1MiB (anonymous temporary file; disk may overshoot between ticks).
CLI INT/TERM/HUP cancellation uses maintained `signal-hook`0.3.18/registry1.4.8;
existing libc becomes direct and tempfile moves from dev to runtime. No existing
package upgrades. Offline dependency resolution initially lacked the signal-hook
index; one normal Cargo metadata resolution fetched it, subsequent gates offline.
Source inventory Git calls remain unbounded and need their own follow-up.

Process tests first failed at absent API. Actual first runtime exposed Darwin
EPERM when signaling an unreaped zombie-only group; isolated waitid/killpg probe
confirmed it. Correction still requires known exited leader and complete group
disappearance, never treats arbitrary EPERM as successful cleanup. Tests cover
TERM-ignoring deadline, successful/failed leader leaving children, grandchildren,
pre/during cancellation, output overflow, exit/status/stdout, invalid limits and
missing executable. Capture integration covers producer/probe deadlines plus CLI
INT/TERM/HUP and verifies child disappearance/no seal/review/baseline.
No native producer run; escaping process groups and SIGKILL/crash recovery remain
explicitly unproven. Host locking and poisoned/stale-gate handling must land
before consumer/native adoption. Final fresh **54/54** tests pass (8.004s total),
strict all-target Clippy passes (0.705s), rustfmt and diff checks pass. Existing
38 contracts, original report assets, comparison/acceptance/store and licensing
bytes unchanged. Tests use only owned short-lived fixtures; final process audit
found no remaining test descendants. Logs
`/tmp/gpui-visual-process-{red,first,second,integrated,final-tests,final-clippy,final-fmt}.log`.

R1c routes all three Git inventory commands (`ls-files`, submodule root and HEAD)
through the same bounded runner. Default30s/64MiB stdout per Git process; capture
passes cancellation into discovery and filesystem traversal. Existing store API
re-export preserved; extraction into `inventory.rs` keeps responsibilities bounded.
Temporary fake-Git child processes exercise each hanging command and verify owned
children are gone after deadline. No parent PATH mutation or real-repo override.
New API compilation first fails as expected; final **57/57** tests (including one
child-test entry, 10.977s total), strict Clippy (2.308s), fmt and diff pass. Logs
`/tmp/gpui-visual-inventory-{red,tests,clippy,fmt}.log`. Original source hashes,
submodule/baseline protections and 38 contracts preserved; no consumer update.
Only macOS execution proven; non-Unix discovery currently fails closed.

R1d adds `gate::{Gate, Options}` and `capture::capture_guarded`, with CLI opt-in
`check --host-gate` / repeated `--compatibility-gate`. Stable-inode same-user host
flock spans probes, capture, validation and cleanup. Active/pending/PGID evidence
survives Drop/crash; only explicit proven-clean finish or exact-hash, dead-owner,
absent-group recovery can clear it. Symlink/hardlink/foreign/replaced/unsafe files
fail closed. Legacy flock metadata stays unchanged. See [host-gate.md](host-gate.md)
for spawn-window, mixed-caller, escaped-process and power-loss limitations. Recovery
is currently a Rust API; no automatic or CLI stale-lock deletion. Native adapters
must opt in; unguarded APIs remain available for portable fixtures.

Fresh final **70/70** tests (including two child-test entries, 6.961s total), strict
all-target Clippy (1.151s), rustfmt and diff pass. Gate10, capture API6, capture
process4, inventory3, process9 and original contracts38. Guarded disposable captures
match direct fixture pixels/manifests without reference/review writes. Crash test
SIGKILLs its own supervisor, proves the producer survives, refuses acquisition and
recovery, then kills that owned producer and recovers only after group disappearance.
Temporary locks only; the real host gate was neither acquired nor recovered.

A final rerun exposed an actual test-handoff race: existence of `producer.pid`
preceded its contents. The parent panic left one test sleep because it did not yet
know the PGID. Its exact temp cwd/PID/PGID were verified before explicit cleanup,
and group disappearance checked. The fixture now atomically publishes its PID and
asks the bounded supervisor to clean up on pre-handoff failures. Log retained at
`/tmp/gpui-visual-gate-handoff-race.log`. A later full run hit the existing inventory
fixture's 250ms deadline before its child marker appeared; targeted recheck and the
next full run passed without changing deadlines/assertions. Retained logs
`/tmp/gpui-visual-gate-inventory-{startup,recheck}.log`; this is not proof that
short-deadline fixtures are immune to host scheduling load.

Final logs `/tmp/gpui-visual-gate-final-{tests,clippy,fmt}.log`; final process audit
found no surviving test descendants. Original report assets, comparison/acceptance,
manifest/lock, LICENSE/NOTICE/provenance remain byte-identical to R1c. No consumer
import, real GUI/GPU/native capture or baseline/environment adoption in R1d.

SDK subsequently imported the complete39-file immutable portable4094066 export in
`15ae4be03c117e478d05b69f9fcb312a16299669`:70/70 wrapper tests, strict Clippy/fmt,
187-source doctor and exact export hashes. No portable consumer fork. Original
SDK derived-upstream drift/missing Taffy was not overwritten: an isolated immutable
export at `/private/tmp/gpui-visual-sdk-pilot-xa4oovq7` rebuilt the exact four pinned
upstreams plus99 SHA-checked patches from local Git objects. Metal test built
`--no-run`; this was not GPU execution.

SDK R2a `5584c9278617ca20973d7dcf287cdac5b22e9875` adds a Rust environment probe
matching all14 typed Python fields, including arm64 spelling/upstream lock hash:
3/3 tests, owning all-target Clippy/fmt, actual probe parity. R2b
`66cc085f3236f676a006d6f4dff2fd0b1f7723df` adds reusable production SharedImage
scenarios with actual child geometry, generation-stamped readiness and independent
eight-case requirements:5/5 non-GPU GPUI tests, Clippy/fmt, mutation red for stale
measurement then restored green. Python/Metal capture path and references stay
unchanged pending two full eight-case runs and exact before/after parity. Full
consumer evidence/logs live in SDK `PROGRESS.md`; native ownership remains unsettled.

R3a now provides one original multi-suite report, independent source-bound catalogs,
ordered groups/sequences/moments/page sets, namespaced assets/DOM/radios, separate
capture/environment/baseline authorities and visible missing/withheld/unsupported
coverage.12 new tests plus original70 entries pass (**82/82**,12.91s), strict
Clippy1.09s/fmt0.53s. Existing managed Personal Chrome proves original controls,
CSP, unique IDs/anchors and desktop/narrow rendering using disposable fixtures.
No producer/real baseline touched. [R3 contract and exact evidence](collections.md).

R3b adds exact explicit `suite:case` acceptance and per-suite environment adoption
from the reviewed collection, full report-file integrity, selected-suite locks,
all-selected preflight and persistent no-replay attempt evidence. Existing
single-suite acceptance rules are shared, not duplicated. **92/92** tests pass
(16.79s), strict Clippy1.36s/fmt0.49s; mutation of seal checking fails on altered
HTML then restores green. Deterministic interruption tests prove completed-suite
and partial-current-suite evidence, not filesystem transactionality. Original R3a
UI remains unchanged. [Exact evidence and limitations](collections.md).

SDK R3 import completed as `3f8890b0bcf052b34eebac78c10ad684f2943bc8`:
complete48-file immutable1e8cf04 export,92/92 SDK-wrapper tests21.46s,
strict Clippy2.36s/fmt and read-only doctor200 inputs. Root pins/locks/helper/
fixtures/config/baselines unchanged; no capture or adoption. Evidence in SDK
`PROGRESS.md` and `/tmp/gpui-visual-sdk-r3-{tests,clippy,fmt,doctor}.log`.

SDK R2c `212f5ef3b894d99875cfc7112716095b3ed5c50e` prepares an ignored Metal
candidate using shared scenarios and independently required coverage. Actual
production img-loader state is observed with current-generation child geometry;
fixed geometry with a missing image fails readiness. A geometry-only mutation
fails that regression (exit101), then restored tests pass. Exact source bytes
rechecked in the owned isolated export against clean3f8890b (550 files); only the
expected19 R3 files refreshed via an inspected immutable archive. No derived
original upstream or foreign source overwritten.

Fresh final SDK owning macos all-target tests **7/7**, two Metal tests explicitly
ignored (5.77s), strict Clippy2.21s/fmt1.04s. Logs
`/tmp/gpui-visual-sdk-scenario-capture-final-{tests,clippy,fmt}.log` and
`/tmp/gpui-visual-sdk-readiness-mutation.log`. Old producer/Python probe still
configured; candidate is temporary migration staging, not a second permanent
capture path. No screenshot/native/pixel-parity claim.

Source/API reuse evaluation found matching AccessKit types but no real-tree/action
bridge in BBR's pinned GPUI test platform; the SDK pin has no AccessKit integration.
[Exact seams, sources and non-adoption decision](test-reuse.md). Keep standard
Rust/GPUI tests and existing semantic IDs/actions; no new runner/dependency/pin.

## R2d — SDK capture replacement verified after resumption

Patrick resumed with “ja lets go”. The earlier pause was assistant-initiated, not
user-requested. Merely open Xcode/Simulator runners were not valid evidence of a
conflict. Actual Respawn screenshot/pressed-state work was observed and respected;
after its owner finished and reported playback paused, capture-command inspection
and the free BBR compatibility lock permitted a quiet SDK offscreen interval.
No foreign process or device was stopped or controlled.

SDK commit **`1388807bf2b008cd7336c36acbe8ac549bd44e50`** completes R2:

- Preserved frozen old/new pilot `/private/tmp/gpui-visual-sdk-pilot-xa4oovq7`:
  all552 SDK files match212f5ef, plus two source-sealed diagnostic configs.
  `reports/visual/r2-parity-old`, `r2-parity-scenario-a`, `r2-parity-scenario-b`
  prove identical ordered eight-case inventory,480×640 decoded RGBA8, PNG hashes
  and bytes, source inputs/policy and all14 environment fields. The latter two
  are independent complete processes. Whole commands4.61s/6.84s/6.62s; Metal
  tests0.90s/0.85s/0.82s. Existing Python route remained unchanged until this proof.
- Final `metal.rs` uses the reusable production scenarios, semantic present action,
  current-generation measured geometry, decoded bundled asset and independent
  required inventory. Existing nonempty/readback/dimension/reversible assertions
  remain. Removed old Python probe and temporary candidate producer; configured
  environment now uses Rust. No helper, portable vendor, dependency or pin change.
- New final export `/private/tmp/gpui-visual-sdk-r2-final-54_qsqnf` reconstructs
  the same four immutable upstreams/99 hash-checked patches and verifies550 SDK
  files. Original drifted/missing-Taffy `.upstream` untouched. Owning macos tests
  7/7 (capture ignored there), strict Clippy and workspace fmt pass in
  34.14s/19.52s/33.33s. Final default workspace257 plus two child-test invocations
  pass41.49s; strict workspace all-target Clippy12.95s. Post-resume unchanged
  portable92/92 tests19.06s also pass. Existing upstream warnings remain visible.
- First final command failed before capture at the unchanged30s probe deadline;
  no run directory/seal written, cleanup/lease finished clean. Diagnosis measured
  redundant Nix evaluation32.05s/34.88s versus direct Rust probe0.02s. The outer
  documented pinned wrapper now supplies the probe toolchain via inherited
  `mbx run`; no inner Nix evaluation and no relaxed deadline. This is not a
  guarantee that arbitrary cold builds finish within30s: failure remains bounded.
- Final configured `reports/visual/r2-final-b` and `r2-final-c` execute the real
  Metal test independently (0.86s/0.83s;87.59s/14.56s whole command). Their seals,
  sources/environment, exact decoded pixels and PNGs match each other; PNGs and
  environment also match the frozen unchanged-old producer. Final source seals
  appropriately differ from the pre-replacement pilot. Both final runs also pass
  post-commit portable `validate --repo <SDK>` against clean1388807 (not merely
  the export's files). Six distinct images inspected; push/pop midpoint
  duplication is explicitly asserted.
- All five successful captures use `--host-gate` plus BBR compatibility
  `/private/tmp/bbr-gui-gate-6ff732a0fdc0bb09.lock`. Final lease idle, no pending
  spawn/groups/owned descendants/held locks; no recovery. Cooperative locks cannot
  exclude unrelated nonparticipating applications. No native window, device,
  gesture, accessibility, performance or locked-session acceptance is claimed.
- All nine reference files remain byte-identical to212f5ef. Reports intentionally
  return1 with8/8 exact/zero errors: only baseline `upstream_lock_sha256` differs
  (`686fec1d…` versus current `5f14b9fd…`). No acceptance, environment adoption or
  reseal. Disposable Rust verification uses the portable store/seal/PNG APIs,
  not a second comparator authority; strict Clippy passes. One GC-evicted checker
  binary caused exit127; normal `mbx run` rebuilt it without cache/target overrides.

Logs `/tmp/gpui-visual-sdk-parity-{old,scenario-a,scenario-b,comparison,workspace-tests,workspace-clippy,workspace-fmt}.log`
and `/tmp/gpui-visual-sdk-r2-final-{prepare,tests,clippy,fmt,workspace,workspace-clippy,a,b,c,comparison}.log`.
Frozen/final `reports/r2-*-source.json` records exact inputs; SDK `PROGRESS.md`
contains command details. This upstream checkpoint changes documentation only;
SDK vendor remains the immutable1e8cf04 executable export. No publication.

## R4 activation and macOS cleanup correction

Patrick explicitly activated the existing isolated BBR visual worktree with
“ja lets go”. Read-only preflight found clean preserved `parallel/visual-snapshots`
at7f5f6243, entirely ancestral to clean product checkpoint
3e95937d58480304fbce7ef90cfe9f6c0fb76ac7 (0/20 divergence). Only that worktree
was fast-forwarded to the fixed checkpoint; main/Blueprint remains owned by
session01a0e454. Lane ownership, territory, serial shared-kernel boundaries and
product-first integration order are recorded in the lane's Current delivery.
No extra worktree, workflow, main write, merge, publication or acceptance.

The initial complete48-file1e8cf04 import matched the verified SDK export
exactly. Its first test run hit the already-recorded250ms fake-Git startup
race (deadline before child.pid existed); targeted recheck passed without
changing limits/assertions. The next full run exposed a distinct actual cleanup
bug: after reaping a exited leader, observing its killed descendant zombies can
briefly return Darwin EPERM. Previously this errored immediately. A bounded
100-process diagnostic using only owned shell/sleep groups observed EPERM→ESRCH
in100/100 cases and separately verified every child disappeared.

The portable correction waits through that macOS-only observation within the
original cleanup deadline; it never treats EPERM as absence, never signals a
reaped numeric PGID and still requires ESRCH before successful cleanup. A
persistent permission error times out and poisons the caller's gate as before;
unexpected errors still fail immediately. No caller/baseline/report/pixel policy
change. Deterministic transient/persistent regressions fail before the fix;
three new unit checks and existing real-process/host-gate tests pass afterward.

Fresh upstream **95/95** tests17.90s, strict all-target Clippy1.66s and fmt1.46s.
First Clippy found test-module placement, corrected without lint suppression;
first fmt found one wrapping difference, corrected. Process inspection found
no remaining fixture children. Logs `/tmp/gpui-visual-r4-cleanup-red.log` and
`/tmp/gpui-visual-r4-cleanup-final-{tests,clippy,fmt}.log`; BBR triggering logs
`/tmp/bbr-r4-import-tests.log`, `...-inventory-recheck.log`, `...-visual-test.log`.
Developed only here. BBR subsequently imports complete49-filed3bae01 as
`03bbe7f03903e6546e887db1e626a378dbab798e` in its existing isolated visual worktree:
35 Python unit contracts,95 upstream tests and one separate real-CLI acceptance
integration pass33.40s; portable strict Clippy4.99s/fmt0.43s, doctor1175 inputs0.76s,
exact provenance and roadmap pass. The unchanged `just visual-test` entrypoint
also passes under BBR's ambient Rust1.97.1 (34.16s); its root graph/toolchain is
unchanged. Logs `/tmp/bbr-r4-import-final-{visual-test,clippy,fmt,doctor,roadmap}.log`
and `/tmp/bbr-r4-import-bbr-toolchain-visual-test.log`.

BBR8804ab2 records the bounded next pilot: existing Tokens id0, Checkbox id16
unchecked→pointer-checked→pointer-unchecked, and multipage Sidebar id44/state0,
Light/Dark with standard/normal selector settings and Reduced Motion. No pilot
code or fresh pixel capture yet. Python producer/probe and link-only batch wiring
remain configured; no reference or main/Blueprint changes. Stable BBR aggregate,
full Gallery/browser migration and all existing coverage limits remain open.
SDK retains its previous pin until a separately verified import; historical
successful SDK capture proof is not relabelled as proof of this correction.

### R4 consumer-local execution without nested supervisors

BBR's in-process Rust orchestrator needs the portable sealing path without an
outer CLI process group around another process-group supervisor. Added
`capture::capture_local`: producer/environment closures run in the caller,
sharing the exact same before/after source/environment validation, raw inventory,
PNG checks and seal publication as external commands. The consumer still owns
bounded builds, real producer groups and host gates through portable APIs; this
is not an unbounded subprocess alternative or a second sealing authority.

Fresh **96 tests** pass, strict all-target Clippy and pinned fmt pass. A new
local-producer contract covers success, source/probe drift, cancellation, bad
pixels and producer failure with no reference/report writes or failed seals.
The first fixture lacked the mandatory explicit capture layer and was corrected,
not the validator. The known 250ms fake-Git fixture startup failure reproduced
twice; it now executes a non-hanging warmup under an 8s bound before measuring
its unchanged 250ms hanging-operation/descendant cleanup contract. Targeted and
full suites pass without increasing the actual operation deadline or relaxing
PID disappearance assertions. Logs `/tmp/gpui-visual-local-{tests,tests-final,inventory-recheck,inventory-warm,tests-green,clippy-green}.log`.
No comparator/report/acceptance or baseline changes; BBR imports a complete
immutable export before using this API. SDK keeps its previous pin.

## R5 — Hackerschau host migration

Goal `1790629039744-e6e72764381ee` (Patrick: "orientier dich an bbr und mach es für
hackerschau genau"). Hackerschau's local Kit fork (`vendor/gpui-mobile` gitlink
`0f394760926d`, based on SDK `9fe5320`, on no SDK branch) carried sequences,
gallery groups, German statuses, the bounded terminal verdict, unchanged-still
comparison controls and the deferred filmstrip. `441d3d4` merges exactly that
`tools/cargo-gpui-visual` delta (3-way apply, recorded in
`docs/provenance/hackerschau-sequence-import.json`) onto `2d9458f`:

- `sequence.rs`, `sequence_report.rs`, `comparison_view.rs`, `gallery.rs` and
  `terminal.rs` are byte-identical to the fork; `review.js` equals the fork (the
  standalone repository had not changed it). CSS keeps both the fork's general
  narrow layout and R3a's collection-only rules.
- `capture_steps` seals an optional raw `sequences.json`, so `capture`,
  `capture_guarded` and `capture_local` share it. `config::Config` gains the optional
  validated `[review]` section; `compare_with_options` passes it to the report.
- Merged acceptance keeps R3b's prepare/revalidate/write split: whole-sequence
  expansion, reviewed-sequence checks and environment-adoption refusal happen in
  `prepare`; obsolete frames of a selected journey are removed in `write`.
- The single-suite report still publishes `review.json` last. Collections refuse
  suites with timelines (they would neither show nor safely accept them) until a
  collection presentation exists.

Red→green: new raw-producer timeline sealing and malformed/symlinked/unpaired timeline
tests, plus the collection refusal, fail with the respective change removed and pass
with it. Fresh gates in an isolated clone before integration: **119/119** tests
(25.2s; 58 contracts incl. 20 fork contracts), strict all-target Clippy and rustfmt.
Canonical main was clean at `2d9458f` when fast-forwarded; no BBR/SDK file touched.
Logs `/tmp/gpui-visual-r5-sequences-{red,tests,clippy}.log`.

Hackerschau `580650432..3136bf6c9` imports the complete 58-file immutable `441d3d4`
export as `vendor/cargo-gpui-visual` (hash provenance in
`tests/gpui-visual/tool/tool-source.json`) and adds the BBR-style consumer crate
`tests/gpui-visual/tool` (own Cargo workspace): harness build via `mbx test --no-run`
outside the host lease and deadline, in-process `capture_local` under
`gate::Options::host()` (+ optional compatibility flocks), a 16-field environment
probe with exact parity to the retired Python probe, independent coverage
(states, page floors, component/route/tab mappings, 48 withheld sequences), completeness,
helper-graph checks and `produce`/`environment` for the generic CLI protocol. The
Python host pipeline and its unit tests are removed; the native `capture_ios.py`
adapters stay (device approval) and call the Rust tool for sources/validation.
31 consumer contracts and the 119 vendored contracts pass under Hackerschau's
pinned toolchain; strict Clippy/fmt pass. Rust/Python outputs were compared on the
same inputs: environment 16/16, completeness (103 findings, flattened) and helper
graph identical; inventory reproduces the same eight currently-missing intro states.

Parity on one frozen product state (clones `/private/tmp/hs-r5/export-{a,b}` of
Hackerschau `60a7857c4` and `3136bf6c9`; no product diff under `apps/` or
`vendor/gpui-mobile`, materialized `.upstream` trees file-identical apart from
materializer stamps). Old path: `make gpui-visual` through `tool.py` and the fork
CLI, held under the host lease by a throwaway `gate-run`; new path: `make gpui-visual`
twice. Every run with the BBR compatibility flock:

| Run | Capture | Result |
| --- | --- | --- |
| old `r5-parity-old` (Python + fork CLI) | 3072 PNGs, 48 sequences, 23m45s; compare ≈12 min (debug) | 144 changed, 58 without reference of 414 units, 0 errors |
| new `r5-parity-new-a` | 3072 PNGs, 48 sequences, 23m46s (24m38s incl. build) | identical verdict; coverage complete, 48 withheld |
| new `r5-parity-new-b` | 3072 PNGs, 48 sequences, 23m53s (24m34s incl. build) | identical verdict; coverage complete, 48 withheld |

A throwaway Rust checker (portable `load_manifest`/`read_png`, no second comparator
authority) proves for both new runs against the old one: identical ordered inventory
(3072), timelines (48 sequences, 8688 ticks, `sequences.json` bytes), 16 environment
fields, decoded RGBA8 + dimensions **3072/3072** and even PNG bytes 3072/3072.
`review.json` results/sequence reviews/baseline state are identical across all three.
All 967 product inputs under `apps/` and `vendor/gpui-mobile` have identical hashes;
the new seal additionally covers nine SDK reference files that the fork CLI skipped
through its generic `baselines` path rule (GPUI Visual excludes only the configured
store). Remaining seal differences are tooling files only. No reference, environment
adoption or reseal; Hackerschau reference bytes unchanged. Logs
`/tmp/hs-r5/parity-{old,new,final}.log`, `/tmp/hs-r5/parity-a-vs-new-a.log`.

The findings reflect the already-known realistic-fixture/intro-card reference drift
and the deliberately withheld sequences; they are review work, not migration output.
Native iOS capture adapters (`capture_ios.py`, `native_*.py`), collection review of
`gpui` + `ios` and SDK reconciliation of the fork's non-tool commits remain open
Hackerschau TODOs.

## Portable individual-story selection (separate development slice)

Patrick requested shared individual-story execution, referencing session
`01a0ebf9-f5c4-73a3`. Built on settled portable `56bf7a6`; this request does not
resume paused BBR R4 or grant Hackerschau/product checkout ownership.
[Usage, producer protocol and compatibility](story-selection.md).

- Added read-only JSON `list`, repeatable exact `check --story`, and
  `Config::select`. Resolve independent source-authored complete story requirements
  before producer invocation. All pages, Light/Dark pairs and whole timelines
  remain required; unknown/duplicate IDs, ignored filters, missing/mismatched
  acknowledgements and output drift fail closed.
- External raw producers receive resolved `GPUI_VISUAL_SELECTION`; local callbacks
  read `config.selection`. Both must implement pre-render selection and emit the
  exact receipt. Full commands/probes clear inherited scope. No silent consumer
  fallback or claim that portable validation can measure hidden rendering work.
- Selected capture/review schema 2 makes old schema-1 readers reject partial
  evidence. Ordinary captures remain schema 1. The original report and terminal
  show TEILLAUF; comparison validates the entire baseline store while comparing
  only selected requirements. Acceptance/environment adoption and full-suite
  collection use are refused. Selected check/compare requires an explicit report.
- Fresh final **133/133** tests (119 existing +14 new test entries, including a
  producer child entry), strict all-target Clippy and rustfmt pass under the
  pinned wrapper, locked/offline and hard timeouts. Measured wall time:
  tests27.00s, Clippy1.98s, fmt0.58s. Initial compilation caught a fixture borrow
  error, corrected before passing; no assertion/deadline/validator weakening.
- Existing managed Personal Chrome reviewed a synthetic scoped report with8 PNGs,
  two complete3-tick timelines and two Light/Dark page pairs.48 unique DOM IDs,
  valid anchors, all6 viewer modes respond globally, timeline scrubbing and
  findings filter preserve the scope banner. Desktop and390px viewport have no
  horizontal overflow. Loaded image URLs resolve; initial image probe counted
  six existing src-less crop placeholders, corrected to inspect actual img[src]
  assets. Screenshots visually inspected; only the task-created tab was closed.
- No dependency, GPUI pin, CSS/JS asset, consumer export, native/device capture,
  real baseline, listener, workflow or publication change. This proves portable
  selection and presentation, not consumer speed/integration or migration closeout.

Logs `/tmp/gpui-visual-selection-final-{tests,clippy,fmt}.log`, synthetic report
`/private/tmp/gpui-visual-selection-review/index.html`, screenshots
`/tmp/gpui-visual-selection-{desktop,narrow}.png`. BBR remains paused with its
previous checkpoint/dirty work preserved; R4 and native consumer gates stay open.

## R5 follow-up — native port and SDK reconciliation (2026-09-29)

Patrick: "ja mach das" for both remaining Hackerschau items.

- SDK: merge `a59c1b0` (+ evidence `98b5256`) takes the branchless fork's frame-demand,
  sequence-clock and list-anchor commits; git rename detection tried to replay the fork's
  tool edits into `vendor/gpui-visual`, resolved by keeping the tool removal and the
  byte-identical export. Isolated-clone gates: 15 foundation Python suites, fmt,
  workspace 259/259, headless navigation examples, strict default/macOS Clippy, retained-
  line/layout-measure harnesses, dependency graphs, iOS-sim UIKit Clippy; `--all-features`
  blocked offline by uncached `java-locator` (pre-existing). Metal covers 8/8 exact; only
  `upstream_lock_sha256` differs as before. Hackerschau's fresh host capture after the
  gitlink move equals its previous final run exactly (3072/3072, 48 timelines).
- Hackerschau native: `native.rs` replaces `capture_ios.py`/`native_*.py` and uses
  `capture_local` under the host lease. Retired Python and new Rust capture were run for
  all 24 cases against the same deterministic fake `xcrun`/`agent-device`: identical
  72 sealed PNGs, 360 PNGs incl. stability samples, 507 evidence files and environment;
  645 device commands identical apart from two intended reorderings. Mutations removing
  the pre-build device check, the failure-path override clear or the bundle check fail
  the contracts. A real run is blocked on device/session and listener approval: the
  agent-device XCTest runner listens on a wildcard TCP port.
- Found while waiting 26 minutes for the shared lease: mbx collected the consumer's
  managed target and the built harness disappeared (exit 126, no seal). Hackerschau now
  runs a retained hard link. Other consumers that build before waiting for the lease can
  hit the same GC window.

## Agent-ready CLI — slice A1

Patrick requested the complete AI-ready plan in separate tested commits. The first
slice adds command-specific Clap help, opt-in schema-1 `agent::Response` JSON,
logical-unit summaries and typed diagnostics. Unix CLI-only stdout isolation keeps
noisy producers off protocol stdout without changing library descriptors, probe
capture or process/gate supervision. Existing raw list/sources and human output
remain. The historical `doctor --run` no-op is preserved (hidden) after an existing
presentation contract exposed that compatibility requirement.

Three new disposable CLI contracts fail on the absent `--json` behavior, then pass
for noisy producer output, parsing/errors, selected scope, stale sources and no
reference writes. First green attempt caught missing fixture parent creation,
corrected in the fixture. Baseline suite passed before edits (22.68s). Final full
suite passes (29.28s), strict all-target Clippy (1.96s), fmt (0.55s), diff whitespace.
Logs `/tmp/gpui-ai-cli-red.log` and `/tmp/gpui-ai-cli-final-{tests,clippy,fmt}.log`.
The shell lacked `timeout`; checks used `/tmp/gpui-ai-check.py` with a new process
group, 300s test/Clippy or 120s fmt deadline and TERM/10s/KILL cleanup. All compiler
commands still use the pinned wrapper and mbx, locked/offline. No new dependency,
consumer write, capture, original report asset or baseline change. JSON schema is
separate from seals; full scope is explicitly not independent consumer coverage.

### A2 — read-only review inspection

Added `inspect::inspect` and CLI `inspect`: bounded logical findings/suite summaries,
exact case or namespaced suite:case assets, one sequence tick (default first
abnormal tick), explicit missing sides and pagination. Stored evidence is always
labelled unvalidated; no current-validity/acceptance claim. No config, Git, probe,
lock creation, report/baseline writes or browser is required. IDs, symlinks, lock
markers and before/after document identity are checked; no hostile-writer or
filesystem-transaction guarantee is made.

New CLI test fails before the command exists, then passes; two Rust contracts
cover sequence metadata/held frames, out-of-range ticks, symlink refusal and
collection namespaces/invalid suites. Full locked/offline suite passes35.84s,
strict all-target Clippy3.08s, fmt0.56s. Logs `/tmp/gpui-ai-inspect-red.log` and
`/tmp/gpui-ai-inspect-{tests,clippy,fmt-check}.log`. Original comparator/HTML/assets,
seals and acceptance untouched; disposable fixtures only.

### A3 — execution preview

Added `plan::check` / `check --dry-run`, preserving strict explicit selected-report
requirements. Reports resolved argv, paths, independent catalog requirements,
requested gates and effects without running probes/producer or writing/acquiring
anything. Checks sources through existing bounded Git inventory; rejects overlap,
existing run, invalid catalog binding and unsupported legacy selection. Native
permission, environment, gate availability and actual producer support remain
explicitly unverified. No preview reuse/implicit reservation.

Regression fails before the flag exists; then verifies trap producers/probes never
run, no output/reference writes and invalid scope/path rejection. Full suite passes
37.73s, strict Clippy2.89s, fmt0.56s; bounded locked/offline wrapper as A1. Logs
`/tmp/gpui-ai-plan-{red,tests,clippy,fmt-check}.log`. No consumer or capture change.

### A4 — observational timings

Added shared `Timings` and additive timed capture/local/guarded/compare APIs, with
existing entrypoints delegating unchanged. CLI responses retain total, producer,
probe, capture, gate wait/finish, compare/asset and report wall times as available,
including failed execution attempts. No seal fields or timing-based readiness;
producer time includes any embedded build, never falsely labelled GPU time. Local
consumers can separately measure their existing bounded build/lease operations.
Comparison asset I/O remains grouped with validation/comparison until profiling
justifies finer instrumentation or optimization.

JSON regression fails before instrumentation; final full locked/offline suite
passes28.22s, strict Clippy1.45s, fmt0.51s. Logs `/tmp/gpui-ai-timing-red.log` and
`/tmp/gpui-ai-timing-final-{tests,clippy,fmt}.log`. Existing real process/gate contracts
and exact image/report contracts remain green. No consumer performance claim,
compiler/storage policy change or native capture.

### A5 — bundled skill and consumer onboarding

Added one short Agent Skills-format `skills/gpui-visual/SKILL.md`, a focused consumer
reference/config/catalog/API example and reviewed AGENTS snippet. CLI `skill`
(`--reference consumer` optionally) serves the exact embedded files without a repo,
installation, global setup or network. README routes consumers separately from
historical migration evidence. The normal loop uses pinned entrypoint, discovery,
dry-run, authorized selected capture, compact inspect/images and explicit full
coverage/approval boundaries. No automatic skill install or consumer config write.

Contract verifies bundled byte parity, concise frontmatter, plain/JSON output,
reference selection, absent config and no generated outputs. Full suite29.76s,
strict Clippy3.79s and fmt0.61s pass; logs `/tmp/gpui-ai-skill-{tests,clippy,fmt-check}.log`.
Examples are integration guidance, not a new renderer/test DSL; no independent-agent
usability claim (no delegation authorized).

Read-only consumer recheck: Hackerschau is clean at3e58eef99 and already imports
1070c36 story selection (85feb7065), plans capture schedules (c66ab26d7) and exposes
Make story/list routes (3e58eef99). Its owner session01a0ebf9 last explicitly required
source stability during a full capture; no handoff exists. Its docs record a72-PNG/
2-timeline selection at33s capture/49s total. That is historical consumer evidence,
not a fresh benchmark of A1–A5. No duplicate integration, export overwrite or consumer
native execution was performed. New export/measurement remains ownership-gated.

### A6 — selection work proof and real stored-report smoke

The disposable producer now records actual scheduled story invocations separately
from emitted inventory. A full run invokes tokens/opening/other; three selected
CLI runs independently exercise static, two-page and whole-sequence stories,
assert zero unselected invocations, complete Light/Dark/ticks and exact PNG hashes/
timeline equality to the full run. A mutation adding hidden unselected work fails
the new assertion; restored full suite passes24.79s, Clippy1.04s, fmt0.56s. Logs
`/tmp/gpui-ai-pilot-{targeted,mutation,tests,clippy,fmt-check}.log`. This is synthetic
portable producer evidence, not native consumer render performance.

Fresh read-only smoke with the built upstream executable against Hackerschau's
existing `reports/gpui-visual/comment-count-after`: summary and exact
`sequence-story-cold-60hz--light` detail both exit0 in0.08s wall (internal38.0/36.6ms).
The stored review has64 frame findings/242 ticks and375602 bytes; summary1192 bytes,
one-tick detail1455 bytes (about315x/258x smaller by bytes, not a token benchmark).
Both correctly say selected, unvalidated, missing baselines and acceptance blocked;
actual PNG path exists, expected/diff are null. Report hash unchanged across reads;
consumer checkout still clean3e58eef99. Logs `/tmp/gpui-ai-consumer-{inspect,detail}.json`.
No fresh consumer capture, vendor import, baseline write or pixel approval; native
pilot/update remains blocked on the active owner's source handoff. No speculative
comparator optimization: measured output reduction is delivered without weakening
exact comparison or report controls.

### A7 — failed-run identity and final protocol gates

Local final review identified that a producer failure with an automatically generated
run lost its attempted path in JSON. New assertion reproduces that failure; fixed by
retaining resolved single-suite attempt paths on errors, explicitly not claiming files
or seals exist. Early errors still have null data. Added disposable explicit acceptance,
exact compare and sources JSON coverage; no real reference is touched.

Final full suite **143 top-level test entries** passes28.29s (plus3 nested producer
invocations; zero failed/ignored), strict all-target Clippy8.98s and fmt1.40s.
Logs `/tmp/gpui-ai-failure-path-red.log`, `/tmp/gpui-ai-final-{tests,clippy,fmt}.log`.
Documentation relative links resolve; schema JSON parses; `cargo package --list
--locked --offline` includes embedded skill/references/schema with publication still
false (list only, not a package build/upload). No JSON Schema validator was installed;
CLI contracts assert actual payload shape/semantics. Process inspection found no
matching surviving fixture processes. Original assets, PNG semantics, acceptance,
license/provenance and dependency graph remain unchanged. Consumer native pilot and
immutable import of these new features remain the explicit ownership-gated remainder.

## Respawn first adoption — test-handoff correction

Patrick explicitly requested Respawn integration. Its prior app writer finished;
clean consumer checkpoint `628871d` and unchanged SDK `29b927d` were inspected.
Initial complete 76-file `27b741a` import is consumer commit `221aea4`; its first
143-entry portable run passed. A subsequent full consumer verification reproduced
a PID-publication race in `capture_process`: existence of `child.pid` preceded
its contents, and signalling the CLI could interrupt the shell before `echo`.
The test then parsed an empty PID, after successful CLI cleanup. Process audit
found no surviving test producer/child. Failing evidence:
`/tmp/respawn-visual-tests-final.log`.

Publish that fixture PID via a temporary file and atomic rename, as other owned
process fixtures already do. No runtime/timeout/assertion or dependency change.
Fresh standalone 143 top-level tests (+3 nested producer invocations), strict
all-target Clippy and fmt pass in 22.52s/1.71s/0.49s, locked/offline through the
pinned wrapper with hard process-group deadlines. Logs
`/tmp/gpui-visual-pid-handoff-{tests,clippy,fmt}.log`. Consumer imports the next
immutable export; its host/native coverage remains separate and incomplete.

## Agent-ready consumer rollout (2026-09-29)

Patrick: "ja bitte überall. lets goal" — bring every consumer to `0c221e7`.
One local writer; no subagents/workflows, listeners, devices, acceptance or push.

### R6 Respawn host pilot

Respawn's previous app writer had finished; clean `628871d`, SDK pin `29b927d`.
Commits `221aea4` (initial 76-file `27b741a` import), `b0a1211` (immutable `0c221e7`
refresh) and `31b08ff` (integration). Independent tool root `tests/gpui-visual/tool`
retains copies of the pinned CLI and glue (surviving mbx GC), binds prepared renderer
sources/executable digests and exposes `scripts/gpui-visual`; checks always add the
host gate. Opt-in macOS-only `visual-tests` feature; the iOS normal/build graph has
no tool, helper, `gpui_macos` or fixture crate; all prior lock entries are preserved.

Six production Home/Episode/Terms cases, two fixed-dark appearance slots each
(12 PNGs, 402×874 pt at 2x), in-memory SQLite, fake audio, HTTP-count assertion,
measured production rows/viewport and blank-image checks. Independent catalog plus
invocation traces reject missing/duplicate/extra/hidden work. Two full gated runs
are exactly equal in decoded RGBA8 (12/12); a selected Episode run renders only its
4 images, equal to the full runs. All exit 1 for 12/4 missing references, 0 errors.
Consumer gates: portable 143+3, 6 glue and 94 screen tests (1 ignored Metal), strict
Clippy/fmt, `make check`, `make mobile-check`, `make ios-build`; details in Respawn
`docs/gpui-visual.md`. Respawn session `01a0e9aa` later pushed these three local
commits to its `origin/main` together with its own work; this migration pushed
nothing. **Open:** baseline review/acceptance and native install/exercise on the
`respawn-fm` simulator, pending approval of the agent-device XCTest wildcard listener.

### SDK import `f6af706`

SDK clean `98b5256`, previous SDK session idle. Complete 76-file `0c221e7` export
(hash-equal to Respawn's), no Cargo manifest/lock change, commit `f6af706`.
SDK-wrapper vendored suite 146 passing test results, strict Clippy, fmt. Agent smoke:
`skill`, `doctor --json` (226 sources), gated dry-run without writes; `list` fails
closed with `selection_unsupported` because the SDK has no story catalog.
Isolated clone of the identical tree with a stamp-verified copy of the fresh
materialization for the same lock/102 patches: owning tests 7/7, strict helper Clippy,
workspace fmt. Two gated captures `a8-final-{a,b}`: 8/8 same, 0 errors; decoded
RGBA8, PNG bytes, inventory and environment equal prior `merge-a` and references.
Only vendored-tool source hashes differ. Both runs validate against canonical
`f6af706`. Exit 1 remains the known baseline `upstream_lock_sha256` mismatch; no
adoption. Evidence and logs: SDK `PROGRESS.md`.

### Hackerschau landed on `main`

After Patrick approved the `count-update` references and its owner session committed
them (`c4e966247`) and closed, the verified candidate was rebased without conflicts
and fast-forwarded as `ba9694c5c`; evidence commit `b50f36978`. Re-verified on
`main`: exact 76-file provenance, 57 consumer + 146 vendored results (1 ignored native
parity fixture), strict Clippy/fmt for both roots, agent smoke (`skill`, `list` 181,
`doctor`, gated dry-run, refusals, `inspect` of the acceptance report). Fresh gated
run of both `count-update` stories (234 PNGs, 4 timelines): 4/4 exact against the
new references, same environment, `validate` ok. No other capture or acceptance.
Open upstream follow-up recorded by Hackerschau: acceptance stores the full source
map per accepted case, so 234 frames grew `baseline.json` from 32 to 68 MB.

### Hackerschau and BBR — verified handoff candidates (historical for Hackerschau)

Both canonical checkouts have active owners, so nothing was written there.
Hackerschau `main` `d660829ce` (clean) is owned by session `01a0ebf9`, which is
working on count-update references and may start a ~26-minute source-stable full
run. BBR's lane `parallel/visual-snapshots` `74b94eb0` is single-writer to session
`01a0e635` per its Current delivery and carries six uncommitted Gallery/catalog
files. Each candidate was built in an isolated `--shared` clone:

- **Hackerschau** `3368dada0` in `/private/tmp/gpui-visual-hs-a8.9Y1lvY/hackerschau`
  (patch `/tmp/gpui-visual-hs-a8-handoff.patch`, applies cleanly to canonical).
  Complete export, provenance, `make gpui-visual-agent` limited to read-only commands
  and dry-run, docs/AGENTS/TODO. SDK gitlink `98b5256` and `apps/ios/.upstream` (stamp
  input and working-tree fingerprints verified) unchanged. 56 consumer + 146 vendored
  results (1 ignored native parity fixture), strict Clippy and fmt through the app's
  pinned wrapper. Agent smoke `skill`, `list` (181 stories), `doctor`, gated dry-run,
  unknown-story/capture/accept refusals, `inspect`, `validate`. Fresh host-gated
  `sequence-story-count-update-60hz` (82 PNGs, 2 timelines) equals owner run
  `1790675550418558000-4238` in decoded RGBA8, PNG bytes, `sequences.json`,
  environment and selection; only vendored-tool hashes differ.
- **BBR** `0359a582` in `/private/tmp/gpui-visual-bbr-a8.G1vzNv/bbr` (patch
  `/tmp/gpui-visual-bbr-a8-handoff.patch`, disjoint from the lane's dirty files).
  2d9458f → 0c221e7 needed one consumer fixture change: `store::Manifest` gained
  public `selection`/`sequences` fields. `tool.py` forwards `skill`/`list`/`inspect`;
  `just visual-agent` is read-only/dry-run only. BBR dev shell (Rust 1.97.1): 15
  consumer entries, strict Clippy both roots, fmt, `just visual-test` (36 Python,
  146 upstream, real-CLI integration), `just roadmap`, `just script-tests`. Combined
  with the lane's dirty diff the consumer suite, Clippy and fmt also pass. Fresh
  host-gated Gallery capture through the pinned CLI and existing producer: 1,872
  pages equal batch `1790629600131104000-63811` in decoded RGBA8 and PNG bytes;
  environment differs only in `tool_revision`. Browser windows not recaptured.

Only a lane/checkout owner handoff or explicit reassignment can land these commits.
No reference, environment, device, listener or push. `/tmp` evidence is transient.

## Baseline provenance deduplication — Hackerschau visual-review preparation

Patrick selected visual review after closing the prior Pi sessions. Hackerschau
was clean at `b50f36978`, upstream at `9e29fb1`; one local writer. Schema 2 now
shares sources per historical capture hash while retaining case image/capture
hashes, environment and sequence receipts. Legacy reads are read-only; explicit
acceptance keeps unselected history and prunes only unused capture records.
`compact-baselines` provides an explicit exact-input-hash metadata-only migration,
with no capture/current-source lookup or visual acceptance. [Contract](baselines.md).

Initial regression failed on schema 1 vs 2; a second red test caught actual
consumer Git SHA-1 submodule entries before migration. Both corrected without
changing historical hashes. Fresh full locked/offline suite: **152 top-level
entries +3 nested producer invocations**, zero failures/ignored; strict all-target
Clippy and pinned rustfmt pass. Nine new contracts cover sharing, lossless migration,
CLI behavior, legacy/selective acceptance, stale seals, invalid provenance,
locks/pending writes/hash drift/images and storage growth. Commands use
`timeout --kill-after=10s` (300s compiler gates, 120s fmt) and the pinned wrapper.
Logs `/tmp/gpui-visual-baseline-dedup-{red,focused,tests,clippy,fmt}.log` and
`/tmp/gpui-visual-baseline-git-red.log`.

Read-only dry-run against Hackerschau's real GPUI store validates590 cases/53
captures and predicts67,838,312 →6,038,852 bytes (~91.1% reduction). All590 images
were hashed and decoded; no consumer file changed. Preview took54.79s; not a
capture/comparison performance benchmark. Input SHA-256
`d7ed62476085f8acc1d673c7509730297368eb47bdae2eaf9edb4eeea06f281e`, predicted output
`2bba87a1f7c070c83c9c790e8870710ea1329738d8b0d35bec81410561f96bd6`.
Log `/tmp/gpui-visual-hs-compact-preview.json`. Immutable consumer import, reader
updates, actual migration and fresh full host report follow separately. No PNG,
reference environment, native device/listener, dependency, tolerance or report
asset change; SDK/BBR/Respawn remain on their existing pins.

### Consumer import, metadata migration and fresh host evidence (2026-09-29)

Hackerschau imported the complete79-file immutable `661c9ce` export in `c0d850d74`,
updated coverage/completeness to the typed baseline reader, and committed the
explicit exact-hash metadata migration as `61b94f500`. GPUI:590 cases/53 captures,
67,838,312→6,038,852 bytes (−91.1%); iOS:24/3,2,133,568→272,655 bytes (−87.2%). Both
outputs match their dry-run hashes. Re-expanding schema2 reconstructs the complete
historical schema1 documents identically; all614 PNG hashes/filenames, timelines
and reference environments remain unchanged. Full consumer/import tests pass again
(58 consumer +155 vendored results, one explicitly ignored fake-native parity
fixture); strict all-target Clippy/fmt passed for both roots before compaction.
Completeness is byte-identical before/after:103 findings, all eight groups open.

Fresh full host `make gpui-visual` on clean `61b94f500`, under the shared host lease
and BBR compatibility gate, produces3306 PNGs/52 sequences in26m51s (27m58s incl.
build). Report/run stem `visual-review-20260929-provenance-v2`; all181 stories and
required capture cases present. Verdict:210 exact/146 changed/10 missing stills,
4 exact count-update/48 withheld sequences; no errors or environment mismatch.
Exit2 is the review-required verdict, not capture failure. Separate read-only
`validate` exits0 with sources/images valid and no producer execution; that command
does not validate the baseline environment.

All3306 actual PNGs and canonical results/sequences/errors/environment fields equal
the historical `count-update-accepted` report. Capture hash
`6cafa506efbd34d3482f3ef18004ea1526ed5d02ce116ebf41cd30a190a6e33e`, review SHA-256
`a3c1a859a0e73cd921f6be626560c23e803d4e0b80315c62c7ebc842b2ab83e4`.
This is migration-regression evidence, not acceptance of pre-existing differences.
Representative grouped still review in Hackerschau's `docs/visual/README.md`
separates a26-ID Settings/Icons candidate from reading-fixture changes, existing
Saved spacing/punctuation defects and the48 unreviewed/withheld sequence variants.
No baseline PNG/environment adopted, historical reseal, native operation/E2E,
listener, push or publication. This upstream documentation follow-up is not a new
consumer import; preserve the `661c9ce` export and fresh capture seals.

## Parallel per-image verification (2026-09-30)

Patrick asked to minimize Hackerschau's complete host run without quality loss and
approved committing on `main` after the report restyle `26cec7e`. After Hackerschau's
own producer work, its remaining time was single-threaded hashing/decoding and exact
comparison of 3,306 images. `parallel::map` is an order-preserving scoped-thread map:
items are claimed in increasing order, every item below the lowest failure completes,
so the reported error is always the first failing item in input order, exactly as the
former loops; worker panics propagate. It now runs the independent per-file work in
`store::validate_images`, `store::baseline_state` (directory listing errors still
first), the raw-producer seal's image digests and `compare`'s per-case loop (disjoint
report folders; findings keep inventory order). Every image is still hashed and fully
decoded; sequence-pair diffs, checks, seals and publication order are unchanged. No
tolerance, dependency or schema change.

Fresh gates on this tree: **162 test results**, zero failures/ignored (three new
`parallel` tests: order equality for 1–64 workers, first failure in input order with
a delayed earlier failure, panic propagation); strict all-target Clippy and rustfmt
pass. Logs `/tmp/gv-main-par-{tests,clippy,fmt}.log`.

Prototype evidence on the vendored `661c9ce` (scratch copies, same code): 158 results,
Clippy, fmt pass; Hackerschau's sealed `pngcache-b-full` compared in 4.4/3.9s instead
of 27.7/25.8s with all 4,722 report files byte-identical and the same exit code; one
`validate` pass (release) 1.1/1.6s instead of 10.1/8.9s. Consumer import and fresh
host evidence are recorded in Hackerschau's `docs/visual/README.md`. SDK, BBR and
Respawn remain on their existing pins.

## Explicit `accept --all` (2026-09-30)

Patrick asked for one command that accepts everything a reviewed report flags,
implemented here rather than per consumer. `accept --all` is an explicit bulk
selection, never a default (`accept` without IDs still fails): `preview_all` derives
`Reviewed` (changed and new stills, complete changed/new sequences; no unchanged
cases or single journey frames) from the report's sealed `review.json`, refusing a
review of another capture. `confirm_all` prints every ID on stderr and asks on an
interactive terminal; without one only `--yes` confirms (so agents and scripts cannot
accept silently). `accept_all` derives the list again under the report lock, requires
it to equal the confirmed list, then runs the unchanged `prepare`/revalidate/write
path, so every source, image, capture/baseline-state, full-capture, sequence and
environment check applies exactly as for explicit `--case` IDs. `--all` conflicts
with `--case`; `--yes` without `--all` is rejected. Collections keep exact
`suite:case` acceptance.

Fresh gates: **166 test results**, zero failures/ignored (four new: exact reviewed
selection with an untouched unchanged reference, stale confirmation and nothing to
accept without writes; capture/environment gates including `--adopt-environment`;
complete journeys only; CLI refusal without terminal or `--yes` listing every ID,
flag conflicts, JSON `accepted`); strict all-target Clippy and rustfmt pass. Logs
`/tmp/gv-acceptall-{focused,tests,clippy,fmt}.log`.

## Exact lossless WebP references (2026-10-01)

Patrick asked to shrink reference stores losslessly for every consumer, implemented
here. Codec measurement on Hackerschau's references (copies only): libwebp 1.6.0
lossless preset 8 with `exact` stores 3,306 references in 59.7 MB of unique files
instead of 271.7 MB of PNG (22%), 50.7 s on 16 cores, deterministic, zero pixel
differences; preset 9: 19.8% vs 20.1% at ~12x time; pure-Rust `image-webp`
encoding 47.1%; oxipng/zopfli PNG 41-44%; AVIF lossless 69-84%; JPEG XL effort 10
18.2% but ~6x slower and not shown by default in Chrome 152. `image-webp` decodes
libwebp output exactly (~2 ms per 804x1748 image), so only encoding uses C
(`libwebp-sys` 0.14.4, MIT, vendored BSD-3 libwebp via `cc`; `zenwebp` is AGPL).
Playwright 1.62 (`toHaveScreenshot('*.webp')`) and Roborazzi use lossless WebP
references too; Skia Gold identifies images by a hash of decoded pixels.

Slices: `5cd0eda` adds `codec` (strict PNG/lossless-WebP decoding, verified encode,
`pixel_hash`). The store slice adds schema 3 (`image_format`, per-case
`pixel_hash`), format-aware reference paths/state/inventory, WebP acceptance for
`accept`, `accept --all` and collections (store format governs; configured format
only for new stores), comparison with the byte/fingerprint fast path plus
container check and exact decode fallback with fingerprint verification, PNG report
assets, `compact-baselines` fingerprint verification and `convert-baselines`.
PNG stores keep schema 2 and exact capture bytes; public `accept`/`accept_all`
signatures are unchanged (`*_with_format`, `store_format` added).

Fresh gates on the store slice: **178 test results**, zero failures/ignored (nine
new WebP contracts plus the codec unit tests from `5cd0eda`); strict all-target
Clippy and rustfmt pass. Logs `/tmp/gv-webp-{tests,clippy,fmt}.log`. Consumer
import, conversion and fresh host evidence are recorded in Hackerschau's
`docs/visual/README.md`; SDK, BBR and Respawn remain on their existing pins.

Dev builds of the CLI (`cargo run`, the default of consumers' agent commands) now
optimize every dependency (`[profile.dev.package."*"] opt-level = 3`). A read-only
`convert-baselines --dry-run` over Hackerschau's 3,306 references took 288 s with
only the codec crates optimized and 100.5 s afterwards, with identical output
(`new_hash` afa4b215…, 146,414,206 image bytes). Gates: 178 test results, strict
Clippy and rustfmt. Logs `/tmp/gv-prof-{tests,clippy,fmt,dry}.*`.

## Current consumer ownership gates

Read-only recheck after R2d (2026-09-28): BBR's complete live delivery document
now activates `BLUEPRINT-IMPL`; main `ce6015dd` has foreign DESIGN/Gallery/Sidebar
edits and session01a0e454 is actively implementing. Hackerschau main
`0ff20f72640c9d4465ac9df736c33d48e07e9eab` still has overlapping foreign visual
scenarios, inventory/config and dirty SDK changes. Respawn main
`87cc6e7e77b37e88c1a56f5c3c678d6eab010951` now contains the owner's merged app work,
but foreign flake/toolchain changes and explicit fixture/device handoff remain.
These observations do not transfer ownership; no consumer files were changed.

| Remaining gate | Actual boundary | Minimal unblocker |
| --- | --- | --- |
| BBR R4 pilot/full migration | Existing isolated `bbr-visual-snapshots` is explicitly activated; main remains Blueprint-owned. | Continue immutable import and bounded pilot in that lane; shared-kernel changes/integration remain serial, GUI gate mandatory. |
| Hackerschau R5 native | Host and native code migrated; no visual Python left. SDK fork reconciled (SDK `a59c1b0`, Hackerschau gitlink `98b5256`, exact host parity). | Approved iPhone 17 / iOS 26.2 simulator, exclusively owned agent-device session and approval of its wildcard runner listener for a real native run. |
| Agent-ready rollout | SDK `f6af706`, Respawn and Hackerschau `ba9694c5c` landed; BBR `0359a582` verified in an isolated clone only. | Lane-owner handoff or explicit reassignment of `parallel/visual-snapshots` (six uncommitted owner files). |
| Respawn R6 | Host pilot integrated at `0c221e7`; ownership settled. Native install/exercise requires the agent-device XCTest runner, which listens on a wildcard TCP port. | Explicit listener/device approval (or local-only runner binding); separate explicit baseline review. |
| R7 completion | Consumer migrations and their mandatory capture/native gates remain unproven. | Finish preceding gates; retain missing references/withholding/framing/locked-session exclusions. |

No workflow, extra worktree, takeover, simulator operation, baseline adoption,
remote or publication is authorized by a finished SDK pilot. Continue on the next
ownership-cleared consumer; do not add speculative portable abstractions merely
to avoid a real handoff boundary.

## Invariants and negative evidence

- Exact decoded RGBA8 and dimensions; reject lossy/high-precision input. No masks,
  tolerance, resizing or synthetic production screenshots.
- Semantic readiness, measured layout and controlled clocks with production
  durations; never sleeps or consecutive equal PNGs as readiness.
- Separate source seals, environments and baseline stores per suite. One report
  does not imply a common environment or cross-suite filesystem transaction.
- Explicit reviewed batch and exact `suite:case` selections; no implicit latest,
  wildcard, accept-all, recapture on acceptance or silent environment adoption.
- Independent required-coverage authority, not merely emitted inventory; required,
  unsupported, partial and withheld cases remain distinguishable.
- Test missing/duplicate/unexpected cases, changed sources/images/review/baselines,
  environment drift, suite/DOM/asset collisions, timeouts, aborted producers,
  descendant cleanup, stale locks and no-write rejection. Acceptance fixtures
  are disposable; actual project reference bytes remain unchanged.
- Preserve BBR's missing Gallery and changed Paper references, existing framing
  and locked-session gaps, and Hackerschau's withheld sequences. Host captures
  do not prove UIKit/AppKit/WebKit, AVPlayer, gestures or device accessibility.

## Reuse evaluation gate

Use ordinary Rust/GPUI tests first. Evaluate `kittest` only against real AccessKit
updates/actions at existing pins; no fabricated tree or forced upgrade. Evaluate
`libtest-mimic` only if dynamic discovery is needed; `nextest` groups never replace
cross-invocation host locking. `insta` may support semantic data assertions, never
a second image-baseline store. [Current evidence-backed decisions](test-reuse.md)
record why none of these additional dependencies is adopted at this checkpoint.

## Verification and iteration

Portable gates run here through `scripts/with-rust-toolchain.sh`:
`mbx test --locked --offline`, strict all-target Clippy, and
`cargo fmt -- --check`. SDK pinned-import gates use its wrapper with
`--manifest-path vendor/gpui-visual/Cargo.toml`. Run affected
SDK helper/fixture checks and actual Metal capture for renderer changes. Use hard
timeouts with process-group cleanup; never override mbx target/cache policy.

BBR stable handoff requires `just verify`, strict testkit Clippy via mbx, affected
real-app suites and the canonical sequential `just e2e` under the current explicit
performance deferral (plus Peek/Gallery as its ledger requires). If timing or
interactions change, preserve/add the affected optimized performance coverage;
do not claim deferred execution. Other consumers follow their own live guides.

Before each slice: recheck status/ownership, choose the smallest runnable scope,
add regression coverage where automatable, implement, run targeted gates, inspect
artifacts/diff, then record exact evidence and commit owned paths. Diagnose failed
attempts before retrying. No historical result substitutes for a fresh gate.

Stop blocked only when no defensible authorized action remains. Report preserved
state, commits, commands/artifacts, hypotheses attempted, exact blocker and minimal
input needed. Missing ownership or native permission is not approval to improvise.
