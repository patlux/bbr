# Task packet — Reusable visual snapshot tooling

Parents: [Phase 4](../phases/04-ui-shell.md) · [Testing track](../tracks/testing.md) · [Design contract](../../../DESIGN.md).

## Outcome

BBR's real production chrome has a repeatable Light/Dark screenshot matrix,
reviewed exact-pixel baselines and the shared cargo-gpui-visual HTML review and
selective acceptance workflow, without adopting the mobile SDK's GPUI graph.

## Scope

- Pin the unmodified, Apache-2.0 portable CLI from GPUI Mobile Kit, including
  license, notices, source hashes and its upstream contract tests.
- Adapt the existing optimized real-app capture to ordinary/representative
  fixtures in Light/Dark, full chrome and a precisely defined Sidebar crop.
- Add a source/environment seal, explicit inventory, bounded recipes and
  integration tests; document native-pixel exclusions and baseline review.
- Migrate the portable authority to an immutable standalone GPUI Visual export,
  with complete provenance and unchanged licenses. Preserve baseline stores.
- Replace the existing visual Python orchestration with a separate Rust consumer
  after parity evidence, reusing the existing Gallery/browser production renderers
  and Tart infrastructure. Never import the mobile SDK's renderer graph.
- Provide exact producer-side story selection, independent required coverage,
  static multi-suite Review Collections and read-only JSON discovery/inspection.
  Selected runs are explicitly non-accepting development evidence.
- Run all actual captures in disposable Tart guests. Record host build identity
  separately from guest rendering environment; validate sources/artifacts before
  and after capture and return bounded diagnostics and raw PNGs before sealing.
  Preserve cleanup and interrupted-acceptance evidence.

### Real application surfaces follow-up (2026-10-02)

Patrick requested the actual Sidebar and application, including History and
Settings, rather than only Gallery examples. The next bounded slice retains all
eight browser cases and adds Light/Dark captures of populated History, its Clear
Browsing Data sheet, the empty Saved Pages library and all seven Settings panes.
Use the real production app, isolated fixture profiles, production actions and
existing renderers. History uses fixed undated fixture visits, not live data or
clock-dependent visit labels. Settings captures its own actual window through a
narrow authenticated, visual-feature-only readback request; no reconstructed view.

This extends the earlier no-new-surfaces/no-protocol scope only for those cases.
Keep exact RGBA8 and dimensions, typed readiness before/after readback, independent
required coverage, fresh source/build/guest binding, existing hardened Tart
transport and observed teardown. Preserve native WebKit/AppKit/Portal exclusions
and explicitly list other missing application surfaces; this is not full-app or
all-state coverage. Never accept baselines automatically. Test the smallest real
browser path first, then window routing/absence/path rejection and the complete
expanded producer in Tart; run portable visual contracts, affected feature Clippy,
format and roadmap checks. Self-review only; no subagents. Stop with retained
attempts and cleanup evidence at a genuine unresolved capture/verification blocker.

The first real browser run exposed the stale representative assertion `pinned=1`:
all four Favorites are now pinned by the existing product invariant. Correct the
fixture assertion to require all four pinned Favorites, without altering product
behavior, deleting assertions or accepting changed baseline pixels.

### Approved references versus incomplete coverage (2026-10-02)

Patrick approved the exact 28-image `app-surfaces-20261002-c` report and its Tart
rendering environment, then requested fixing the acceptance refusal with screenshot
evidence. The refusal is technical: upstream currently blocks every reference in a
suite with any withheld requirement. Preserve `remaining-app-surfaces` and the red
coverage verdict. Implement only an explicit exact withheld-ID acknowledgement in
the standalone authority first; default acceptance and all integrity/missing-required
checks remain strict. No portable vendor fork or unreviewed single-suite route.

After upstream regression tests/Clippy/fmt and local commit, the verified CLI may
accept that exact still-valid historical report with all 28 explicit case IDs,
explicit browser environment adoption and acknowledgement of the withheld row.
Then import the complete immutable export with provenance and freshly stage/capture
the same production app in supervised Tart. Verify every image against the accepted
references, independently recheck sources/cleanup, and inspect actual PNGs plus
managed-Chrome report screenshots. A fresh exact image match must still show failed
coverage. New or changed images require separate approval. Preserve old failed
attempts and source seals; never reseal. Commit independently verified upstream,
reference-adoption and consumer-import slices, without push or migration closeout.

### Rendering identity versus run provenance (authorized 2026-10-03)

Patrick approved the bounded environment-contract correction (“ok mach das”).
Keep every raw observation, exact image/source checks and before/after receipt
identity. Introduce an explicit versioned, reversible split: only the validated
Metal registry instance ID and complete portable-tool revision move to provenance.
All remaining observed fields compare exactly, including OS/build, compiler/SDK,
GPUI pin, Metal name, display geometry/scale, fonts, locale and capture policy.
A separate fingerprint of portable capture/codec inputs binds rendering-relevant
tool changes; a whole-repository documentation/report revision is not that identity.
Unknown fields must never be silently dropped or classified as provenance.

Implement portable comparison/validation and exact-hash guarded metadata migration
upstream first, with negative-first contracts. Legacy environments retain exact
comparison until explicit migration. Migration must reconstruct the old approved
environment losslessly, preserve every reference byte and historical capture/source
record, refuse locks/pending writes/stale hashes, and invalidate old review state.
Import the complete immutable export before consumer changes. Verify the runtime
boundary early, then two fresh source-identical supervised Tart captures against
the unchanged 28 approved PNGs; coverage must still fail on the existing withheld
row. Preserve historical reports/seals. No new image acceptance, rendering change,
other consumer, public capture activation or migration-closeout claim.

Bounded evidence is now recorded in the [rendering-identity addendum](../../visual/migration-evidence-audit.md#rendering-identity-and-provenance-addendum-2026-10-03):
upstream regression contracts, complete immutable 90-file export, consumer exact
classification tests, official metadata-only migration and two fresh source-equal
Tart runs. All 28 PNGs and rendering conditions match; provenance remains sealed
and coverage stays red. Every historical file except the authorized environment
metadata is unchanged. This closes the requested correction only, not any broader
migration checkbox, full application/native coverage or stable aggregate gate.

### Application-state extension (authorized 2026-10-03)

Patrick's “ja lets go” activates populated Saved Pages, recently closed History
and additional Settings states. Start from `main@3a60ef367120aa14430ec725632a6021c8e22bee`.
Retain the existing 28 stills and references. First prove one additional populated
Saved Pages pair end to end in supervised Tart, then expand deterministic fixtures:
collections, single/multiple selection and cancel-only move/removal sheets; History
recently closed, search/empty results and range menu; Settings select menus, profile
sheets (cancel only) and additional Shortcuts scroll positions. Use production
state/actions/views, not Gallery replicas. Add a narrowly feature-gated Settings
input/geometry observation only if the existing harness cannot prove real scrolling.

Own only capture fixtures, required/withheld browser inventory, their contracts,
necessary authenticated visual-only harness seams and supporting documentation.
No production appearance or behavior changes, other worktree, portable vendor
patch, host GUI, new runtime/exposure route or baseline acceptance. New PNGs remain
unapproved. Preserve exact dimensions, RGBA8, rendering identity, provenance,
source binding, original report seals and independent cleanup checks. Replace the
catch-all coverage prose with explicit remaining families without dropping any
requirement or claiming all-state/native coverage.

Evidence gates: negative-first independent catalog/output contracts; bounded mbx
feature Clippy/format; smallest actual optimized guest capture early; complete
expanded Light/Dark capture and fresh same-source repeat; exact preserved-reference
and repeat checks, original PNG/HTML review, source/report/teardown revalidation,
portable visual suite and roadmap. This is another intermediate tooling slice,
not stable migration closeout or a product-performance gate. Keep failed attempts.
Stop on an unresolved runtime/integrity/determinism failure rather than weakening
checks, resealing stale output, adopting references or falling back to the host.

The bounded extension is verified: a30-image pilot preceded the66-image matrix;
two final source-identical Tart captures match all66 PNGs and the exact rendering
identity. The28 approved references remain unchanged;38 new candidates are not
accepted. Real wheel input covers the measured Shortcuts extent with overlapping
viewports, not all nested paint. Seven explicit withheld families keep coverage
red. Portable visual contracts, feature/normal Clippy, current non-GUI verify,
original PNG/HTML inspection and independent source/report/cleanup checks pass.
Retained failures, exact identities, measured timings and evidence paths are in
[Current delivery](../../delivery/current.md#application-state-extension-verified-2026-10-03).
This intermediate result does not check broader migration completion boxes or
claim a fresh canonical functional/performance aggregate.

Patrick subsequently approved exactly the38 new images in report B. Official
collection acceptance adds those references without environment adoption or
rewriting the original28; all seven withheld families remain unchanged. New
comparisons of both stored captures show66/66 exact. The [approval record](../../delivery/current.md#explicit-application-state-reference-approval-2026-10-03)
binds the precise report/capture, unchanged historical records and verification.
This approval does not close any broader migration or coverage requirement.

### Command Palette child-window capture (authorized 2026-10-03)

Patrick's “ja lets go” activates one bounded real Command Palette capture path.
First prove a Light/Dark empty-query pair in a fresh about:blank profile, reading
only the existing production portal window's own Metal scene. No Gallery replica,
parent-window substitution, WebKit pixels or synthetic background composition.
Retain the full child-frame RGBA8/2x dimensions and transparency, and record its
native/window/presentation identity separately from on-screen visibility (the
harness remains hidden). Assert closed/wrong-presentation/path refusal and unchanged
command/browser state across readback. Typed readiness and production motion
completion precede capture; pixels never synchronize the application.

After that first supervised Tart proof, extend only to an actions search with
matches and an actions search without matches; ordinary free-text search always
has a web-search fallback and must not be mislabeled empty. Capture two fresh runs
of the final source tree, inspect originals and HTML, require exact repeatability
and unchanged existing66 references. New images stay unapproved. Preserve all
remaining Command/Find/Downloads/Peek and other withheld requirements explicitly.
Own only visual capture/testkit routing, deterministic fixtures, independent output
catalog/contracts and evidence documents. No product rendering/lifecycle changes,
upstream patch, host GUI, foreign worktree, subagent, new exposure or acceptance.

Verify negative-first protocol/catalog/output contracts, affected feature/normal
Clippy, portable visual contracts, non-GUI verify, fresh optimized guest capture
and same-source repeat, source/report/resource integrity and roadmap. This is an
intermediate tooling increment, not migration or performance closeout. Diagnose
any first-capture failure before building more variants; stop with retained logs,
owned-resource cleanup and the concrete blocker if the path cannot be proven.

The candidate has two source-identical72-image Tart runs:66 approved references
remain exact, six new child-frame stills remain unapproved, and seven withheld
families remain open. Actual PNG/report inspection and independent source/report/
cleanup checks pass, as do portable visual contracts and affected Clippy. The
fresh non-GUI aggregate and one isolated unchanged Command benchmark both fail
its50ms maximum-latency gate. Remaining non-GUI checks pass separately; they do
not erase that failure. The candidate stays uncommitted, without stable-slice or
migration completion. Full evidence and the bounded stop are recorded in
[Current delivery](../../delivery/current.md#command-palette-candidate-and-verification-stop-2026-10-03).

Patrick subsequently approved exactly those six images from report B. Official
selective collection acceptance now stores72 references; both existing captures
compare72/72 exact. The prior66 PNGs/records, rendering environment, historical
reports and source seals are preserved. All seven withheld families remain red.
The [reference-approval record](../../delivery/current.md#explicit-command-palette-reference-approval-2026-10-03)
contains the exact selection, capture/journal identities and fresh verification.
This reference-only approval does not accept the uncommitted implementation or
resolve its recorded Command maximum-latency failure and broader migration gates.

## Non-goals

- No browser appearance or behavior changes, portal capture beyond the bounded
  Command Palette child-window follow-up above, native WKWebView
  pixels, AppKit traffic-light pixels, animation-frame or full UI-state matrix.
- No renderer helper dependency, GPUI migration, registry publication, server,
  browser launch, automatic baseline acceptance, tolerance or masking.
- No changes to another writer's worktree or automatic integration of its dirty
  state. Baselines must be re-reviewed after product changes land.
- No baseline acceptance, environment adoption, WebP/LFS conversion, new browser
  composition or sequence claim merely from completing the tooling migration.
  Gallery state/framing coverage remains owned by its linked packet. New browser
  surfaces/sequences require their own bounded follow-up contracts.

## Parallel-lane contract

Base condition: integrated visual-capture feature and production shell.
Depends on: existing authenticated CaptureChrome and typed snapshot seams.
Provides/API: visual.toml plus visual-check/compare/accept recipes.
Exclusive paths: scripts/visual/, tests/visual/, vendor/cargo-gpui-visual/,
visual.toml, crates/bbr-app/tests/e2e_visual_shell_capture.rs.
Shared integration points: justfile, .gitignore, E2E_ARCHITECTURE.md,
ROADMAP.md, testing track, Phase 4 and Current delivery. The existing
`/Users/patwoz/dev/Personal/macos/bbr/scripts/tart_vm.py` lifecycle is shared
infrastructure: narrowly harden cleanup and add an optional offline-clone policy
hook; preserve the E2E runner's default boot/key policy and recipe behavior.
Integration order: Paper-alignment product changes first; snapshot tooling
second, followed by fresh captures and an explicit baseline review.

## Constraints

Use the real optimized app with isolated test profiles and real WKWebViews.
Keep capture behind visual-capture; no production runtime capture API. The
bounded application-surface follow-ups may add only the explicit Settings-window
and Command Palette child-window testkit readbacks described above.
Drive readiness through typed state and deadlines, never pixel stability or
fixed sleeps. Preserve mandatory functional E2E and performance budgets.
All compiler commands use mbx. GUI/Gallery/browser captures execute only in
throwaway headless Tart guests, never on the user's desktop. Preserve host-wide
GPU/compatibility exclusion where required and guest suite serialization;
long-running commands have hard deadlines with process-group/VM cleanup.
The existing private NAT/key-only SSH route needs the applicable network approval
before boot; no bridge, forwarding, unrelated services or host GUI fallback.
Source inventory is narrow and includes canonical renderer/assets/tool inputs,
never secrets, personal browser data, generated reports or baseline images.

## Evidence

The [requirement-to-artifact audit](../../visual/migration-evidence-audit.md)
maps every active migration requirement to existing evidence and explicit missing
runtime/approval gates. It is an incomplete-milestone audit, not acceptance; the
current portable green gates do not prove actual guest captures or Python parity.

### Unit and contract

- [x] Upstream comparator/acceptance contracts pass unchanged.
- [x] BBR adapter tests prove inventory, provenance, bounded execution and
  fail-closed configuration/output handling.
- [x] Source/environment mismatches and changed pixels require review; an
  explicitly selected case pair can be accepted without accepting others.

### Standalone Rust migration

- [ ] Complete immutable standalone import passes provenance, upstream contracts,
  strict Clippy and formatting; old baseline readers remain compatible.
- [ ] Rust consumer reuses the portable capture/seal/compare/collection/accept APIs;
  replaced Python visual entrypoints are removed only after verified parity.
- [ ] Story selection precedes rendering and preserves all appearances/pages;
  missing/extra/duplicate or ignored selection fails without accepting baselines.
- [ ] Independently authored coverage cannot become green by omitting cases from
  producer output. Invalid suites remain visible beside valid siblings.
- [ ] VM environment/build/source identity, output transfer, bounded cancellation,
  process/VM cleanup and stale/no-write acceptance paths have direct tests.
- [ ] The [Tokens / Checkbox / Sidebar pilot](gallery-visual-pilot.md) passes
  its production-input, endpoint, framing and exact-repeat gates.
- [ ] Two independent complete VM captures match decoded RGBA8 exactly; actual
  images/reports are inspected and existing eight browser cases are retained.
- [ ] Fresh current-tree functional gates pass; reviewed baseline approval remains
  a separate explicit user decision, never inferred from tooling success.

### Real-app E2E

- [x] Both fixtures pass semantic readiness/geometry/session checks in both
  appearances, before and after production Metal readback.
- [x] Eight images are inspected; two independent captures are exactly equal.
- [x] Affected capture suite, normal testkit checks and stable functional
  aggregate pass. Combined-tree evidence is recorded in
  [Current delivery](../../delivery/current.md); visual baseline re-review after
  Paper alignment remains separate from these functional gates.

### Performance

No user-visible interaction or timing is changed. The captures are optimized,
not performance measurements. Existing performance execution remains deferred
by Current delivery; no performance acceptance is claimed.

## Completion gates

```bash
just visual-test
just visual-check
just visual-compare <explicit-run>
just verify
# bounded mbx clippy -p bbr --features testkit --all-targets -- -D warnings
# bounded mbx clippy -p bbr --features visual-capture --test e2e_visual_shell_capture -- -D warnings
# standalone/consumer mbx tests --locked --offline, strict Clippy, fmt
# canonical just e2e sequence in Tart excluding deferred e2e-perf, plus peek/gallery
just roadmap
```

First missing-baseline check must fail for review. Acceptance names every
reviewed case explicitly; a fresh second run must pass against those baselines.

## Delivery sequence

1. Establish one exclusive writer and recorded clean base. Preserve historical
   migration commits and stopped-owner dirty work; no silent worktree takeover.
2. Import/verify the standalone tool before moving the consumer orchestration.
3. Prove the bounded VM pilot, then full Gallery/browser capture and explicit
   source/environment/coverage/selection/review parity before Python cutover.
4. Self-review source, failure modes and requirement-to-evidence mapping;
   no subagents are authorized for this task.
5. Commit coherent verified slices locally. No publication or baseline acceptance.
   Final completion requires all migration evidence above, not an import alone.

### Bounded transport-supervision slice

Reuse upstream process-group cleanup and gate ownership without modifying the
immutable export. A private per-run control directory carries an explicit
cooperative cancellation request; the Python Tart owner observes it only after
its signal/finally handler is installed. Allow a bounded cleanup grace before
upstream hard termination. Only a request-bound terminal acknowledgement after
successful lifecycle cleanup plus caller-validated resource evidence may finish
the gate. Missing/stale acknowledgements, failed cleanup and hard termination
leave unfinished gate evidence. Test with real local fixture processes and
private locks, including a detached stand-in for Tart; never boot a VM or native
producer in these contracts. This slice does not remove the native capture guard
or prove actual VM teardown. No new lane or subagent is authorized.

### Bounded guest-execution slice

Include the independent release consumer as a hash-bound payload executable so
the guest reuses immutable upstream process-group execution instead of retaining
a second Python process runner. Expose only fixed suite inventory/capture calls,
with host refusal before files/processes and no arbitrary argv or environment
bypass. Compose fresh installation, pre/post source/artifact and guest observation,
fixed producer execution, receipt and raw return packaging. Test orchestration
with synthetic artifacts and controlled observation hooks; do not claim pixel
coverage, VM runtime, current catalog parity or bootstrap success. Host boot/
SSH transport and real pilot stay guarded until the approved route is complete.

### Bounded independent host-cleanup slice

Bind the visual lifecycle's owned-run/resource/cleanup records to the independently
expected clone name, supervision request and payload digest. Record ownership
before boot and concrete IP/Popen identity before producer access. After the
transport process exits, repeat clone/key/disk/SSH/process absence checks from the
host before the Rust supervisor finishes either lease. Reuse the existing resource
observer; do not add stop/delete/retry or signal a recorded numeric PID (signal 0
is observation only). Missing, stale, ambiguous or changed records fail closed.
Tests inject resource observations and use private fixtures only, never boot or
probe a VM. Partial boot remains unproven; this prerequisite does not open capture,
authorize networking or claim actual VM cleanup. Host bootstrap/SSH/return wiring
and the approved real pilot remain separate required evidence.

### Offline runtime provisioning follow-up

After the actual guest proved missing pinned libiconv, Patrick requested fixing
all reported blockers (2026-10-02). The pilot packet's [offline provisioning
contract](gallery-visual-pilot.md#authorized-offline-runtime-provisioning) now
permits a hash-bound export/import of the already verified host closure into the
single disposable guest through existing supervised SSH. This supersedes only
the earlier preinstalled-only requirement. Host/Golden Image setup, downloads,
substitution/repair, signature-policy changes and host GUI remain prohibited.
The original complete pinned closure and post-import byte verification stay
mandatory; no runtime version or dependency is substituted to make tests pass.

### Bounded guest-runtime prerequisite

Before host bootstrap/SSH wiring, pin the existing Darwin Python interpreter and
its complete Nix runtime closure, including each producer's non-system dylibs.
Use a source-bound lock containing NAR hashes, sizes and references; validate the
actual staged Mach-O dependency lists and closure bytes with offline read-only
Nix queries/verification. Require this preflight before `visual-prepare` returns
host build evidence; failed staging remains diagnostics, not capture readiness. Reject unknown dylib origins, unpinned store paths,
incomplete/changed closure metadata and changed interpreter bytes. Never resolve
`python3` through the guest's Xcode placeholder or inherit host search paths.
No package installation, Nix realization/repair, new network access, guest launch,
producer execution or capture-enable flag belongs to this slice. Test rejection
and command ordering with local fixtures; actual host checks are prerequisite
proof only, not guest availability, pixel or teardown evidence. Runtime transfer/
realization and source-bound bootstrap/SSH/return integration remain open.

### Bounded source-bound bootstrap slice

Generate the fixed guest bootstrap only from a hash-verified staged payload,
including its pinned runtime lock and guest helper bytes. Refuse arbitrary shell
arguments, destination remapping and unbound imports. Before any guest mutation,
require actual admin/VirtualMac identity; verify every pinned NAR and interpreter
byte offline, with no realization, repair, download or Python fallback. Mount only
the existing read-only Tart payload share, create a fresh compiled-path ancestor
without recursive ownership changes, then execute verified helper bytes in memory
under isolated Python with a cleared deterministic environment. Reuse the fixed
Rust producer and receipt/return modules; do not introduce another renderer or
comparison/process authority. Exercise local fixtures, malformed payload/helper
tripwires, strict path/quoting and shell syntax/host-refusal contracts only.
This slice does not launch SSH/Tart, grant network permission, open capture or
prove real guest runtime/mount compatibility. Supervised transport/return wiring
and real pilot remain mandatory subsequent evidence.

### Bounded supervised transport/return slice

Compose the existing source-bound bootstrap, private hardened Tart lifecycle,
request-bound cancellation and independent cleanup verifier. Stream only the
fixed guest return archive through the existing isolated SSH command; preserve
raw bytes with explicit output/log/deadline bounds and no automatic retry. Direct
transport children must stay in upstream's owned group and be reaped before VM
teardown. Validate archive and payload before publishing transport success; expose
raw candidates only after observed cleanup. Rust validates the original payload,
host build and guest receipt after supervisor success, retaining upstream as the
sole gate/process/seal authority. Test ordering, malformed/swapped/truncated
return, failure/cancellation and unfinished cleanup using private local fixtures
and injected resource observers; no SSH/Tart/native operation. Normal CLI capture
remains blocked. Actual VM runtime availability, return/pixel parity and network
approval remain required before route activation.

The internal route is now composed (still no capture CLI caller). Thirteen Python
transport contracts and three Rust return/chain contracts pass with fixed inert
resource/SSH fixtures; both private leases are observed held through child and
host cleanup. Failed exchange, cancelled return, changed payload/archive, swapped
receipt and host cleanup rejection cannot succeed. Uncertain cleanup retains the
active gate; known-clean failed operation finishes leases without exposing raw
candidates. Rust binds exact returned bytes and separate host/VirtualMac receipt,
not a synthetic pixel claim. `just visual-test` (141 Python units, including one
integration skip rerun separately, unchanged upstream, 50 consumer tests plus
signal fixtures), strict all-target Clippy, fmt and shared script tests pass.
Full evidence and remaining real-runtime/transport boundaries are in the delivery
ledger. No network/native producer/acceptance action or milestone completion
accompanies this checkpoint.

### Retained adapter cleanup regression

The selected-plan gate exposed Darwin `EPERM` during the retained Python
adapter's group-existence probe after TERM. Do not treat permission uncertainty
as absence or weaken descendant checks. On that path, poll/reap the owned direct
child and repeat signal-0 observation within the existing cleanup deadline;
return only on confirmed ESRCH plus reaped leader, otherwise fail cleanup with
no further termination signal to the uncertain numeric group. Verify injected
transient/persistent EPERM and real local timeout/descendant fixtures. This is a
narrow repair of retained compatibility tooling, not another VM/native process
authority or permission to execute the legacy host producer.

### Bounded Gallery selected-plan contract

Translate the immutable upstream resolved Selection into an ordered numeric
Gallery schedule before a future producer call. Bind both the selection catalog
and the independent page plan to the original source hashes. Each selected story
must own every authored state/contrast/density/page pair, including asymmetric
Light/Dark child ranges; reject unknown, partial, duplicate, reordered, sequence
or cross-story requirements. Validate selected raw inventory/coverage/images
against that narrowed independent plan, not against emitted output, and publish
only an exact upstream selection receipt after complete validation. Exercise
synthetic portable fixtures through upstream sealing/review to prove that missing
or extra pages/appearances, ignored selection, stale sources and acceptance or
Collection promotion fail. Do not enable a real story catalog, change Gallery
rendering/dependencies, boot a guest or claim pre-render native evidence. The
existing host guard remains; guest request/producer integration and approved
real selection tests are subsequent required steps.

### Bounded selected guest-execution slice

Extend only the internal guest entrypoint with an explicit hash-bound request
at a fixed work-directory path. Bind it to the expected payload, configured
source-controlled story catalog and independent plan before any producer call.
Validate the selected subset of the existing non-rendering registry inventory
before rendering. Invoke the existing numeric `--story` capture command once per
selected story in authored order, through the same upstream executor and cleared
guest environment; no new renderer or Gallery product changes. Verify exact
per-story outputs, aggregate raw bytes without pixel edits, and validate the
complete selected plan before publishing its receipt. Keep one overall capture
deadline, cancellation and installed/shared source checks around every invocation.
Failure must leave only unusable diagnostics, never a capture seal. Test through
an internal execution seam with synthetic PNGs, pre-call argv/order assertions,
invalid/drifted requests, ignored selection, extra/missing output and cancellation.
Host refusal and ordinary capture guards stay unchanged. Host-to-guest request
transport, real config activation and real VM selection evidence remain later
requirements; no boot or native producer runs in this portable slice.

### Bounded selected-request transport slice

Retain an immutable host-side selected request, source-bound configured catalog
and independent schedule before transport. Convey exact request bytes and its
independently retained hash through the fixed bootstrap, never an arbitrary guest
path/command or a child-redefined expectation. The guest writes only the fresh
fixed request path and invokes the existing selected Rust action. On return,
require the exact expected selection receipt from hash-verified bounded bytes;
missing/swapped receipts and unexpected selection on a full run must fail before
any seal. Keep selected scope explicit in the returned typed capture. Reuse
existing supervision, cancellation and cleanup; test with portable synthetic
transport/producer fixtures and no SSH/Tart/native execution. No real catalog
activation, ordinary guard removal, baseline acceptance or parity claim.

### Bounded returned-raw to upstream-seal slice

Connect only the internal supervised transport to upstream `capture_local`, with
its source snapshot preceding the fresh transport invocation. Use the actual
validated guest before/after observations for environment callbacks; never probe
the host as rendering evidence or expose an import/reseal-old-directory command.
Retain and recheck original transport byte/selection evidence around bounded
copying into a fresh upstream run. Prepare Gallery names/inventory against its
source-bound independent plan (or exact selected schedule), and browser output
against all eight existing cases, reusing the existing validators. Extra metadata,
missing/extra/duplicate pages, wrong precision/size, source/environment drift and
cancellation must leave diagnostics without an upstream seal. Preserve raw PNG
bytes and selected schema/nonacceptance. Test the internal transport seam with
fresh synthetic PNGs and real upstream sealing/review APIs, never VM/native code.
No normal CLI activation, catalog activation, baseline write or parity claim.

### Bounded explicit stored-run review slice

Add a Rust `compare` command taking explicit Gallery/browser run paths and a
fresh report path; never discover latest, build, probe, capture, reseal or write
baselines. Use current configured baseline paths and canonical coverage catalogs.
Full still runs use unchanged upstream Collections, including invalid sibling
visibility and independent coverage failures. An explicitly supplied single
selected Gallery run uses a separate upstream partial report, never a Collection;
reject selected/full mixtures and sequence inputs rather than hiding their scope.
Keep current-config/source-policy binding before review and preserve stale-source,
image/report-integrity and explicit-only acceptance checks. Test fresh synthetic
sealed fixtures plus actual CLI tripwires and no-write failures. This adds stored-
run review plumbing only; normal capture guards, real catalog activation, Python
cutover and all VM/native/visual-approval requirements remain unchanged.

## Iteration policy

Run targeted contracts first, diagnose failures before changing code, and
never accept unexplained differences. Re-check source and environment seals
after capture-affecting edits. Keep integration-baseline review separate from
acceptance of the implementation on its recorded base.

## Blocked stop condition

Stop if GUI ownership/environment, capture determinism, native boundaries or
an unresolvable required gate prevents truthful evidence. Preserve the worktree,
exact run paths, passed/failed checks and smallest needed user input. Do not
change product rendering or weaken comparisons to obtain a pass.

## Delivery ledger

**Fixed guest-execution prerequisite:** both suites now stage the separately
built release Rust consumer, whose internal guest-only fixed commands reuse
upstream process cleanup. Fresh installation, pre/post guest facts and installed
bytes, receipt and raw transport are composed without producing capture seals.
Ten new synthetic Python contracts and three pure Rust tests pass; current totals
are 96 Python units plus separate CLI integration and 42 consumer tests, unchanged
upstream, strict Clippy/fmt and roadmap. Actual host-only builds stage both suite
formats and the real consumer; all hashes independently rechecked. Neither native
producer nor VM was launched. Host bootstrap/SSH/supervision integration, current
coverage and real pilot remain open. Old payloads without the consumer role are
superseded diagnostics; concrete runtime evidence belongs in Current delivery.

**Guest installation prerequisite:** strict bounded transport now validates the
Rust manifest against independent host identity, installs only in a fresh tree
at compiled paths and returns rechecked observed hashes. Eight Python contracts
and one Rust→Python two-suite fixture pass; consumer total 33, Python units 74.
Earlier real payloads parse read-only, but are not installed or executed on the
host. Actual guest bootstrap/installation/producer execution and supervised
cleanup remain open. See current delivery for exact logs and limitations.

**Current Rust host-preparation checkpoint:** `just visual-prepare` now builds
and stages actual Gallery/browser payloads using bounded mbx and the existing
Xcode route; 1,449 sealed sources plus one/two exact Cargo executables were
independently hash-verified. Nothing was executed. Evidence under
`/private/tmp/bbr-visual-prepare.DW3Hlm`; release builds 1m29s / 1m40s.
32 consumer tests, 66 Python units + separate CLI integration, immutable upstream
contracts, strict Clippy/fmt and roadmap pass. These are build/staging artifacts,
not guest or pixel evidence, and become stale after subsequent source edits.
Fresh preparation is required before capture; installation/supervision remain
open. Native Rust execution guard and baseline approval boundary remain closed.

**Earlier Python adapter/integration history (not Rust migration acceptance):**

The adapter, unchanged portable CLI, documentation and eight reviewed baseline
images are implemented. Fifteen adapter contracts and 29 upstream contracts
pass, as do real optimized captures, exact independent repeat comparison,
source-staleness rejection, workspace verification and both Clippy variants.
Self-review added timeout/GUI-lock cleanup, extra-vendor-source and non-regular
PNG checks; no product rendering was changed.

The user-authorized local integration preserves the Paper product commits.
Fresh combined-tree functional gates, including titlebar reload parity, portal,
Peek and Gallery, now pass. The old branch failures remain historical evidence,
not current functional blockers. All eight browser snapshots differ from their
pre-Paper references and still require explicit review and acceptance; integration
alone does not approve pixels. Exact evidence and integration state live in
[Current delivery](../../delivery/current.md). No performance pass is claimed.
