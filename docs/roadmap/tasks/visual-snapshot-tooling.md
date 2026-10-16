# Task packet — Reusable visual snapshot tooling

Parents: [Phase 4](../phases/04-ui-shell.md) · [Testing track](../tracks/testing.md) · [Design contract](../../../DESIGN.md).

## Latest evidence — 2026-10-10

Downloads recovery and history/missing-file expansion now produce 84 exact-repeat
Light/Dark PNGs across two disposable Tart guests. All 72 approved references are
unchanged; no new reference acceptance occurred. The `history4-20261010` pair
binds identical current sources and passed independent resource-cleanup rechecks.
A preceding pair exposed a real unresponsive-tab overlay; the capture now asserts
healthy tabs rather than masking the overlay or changing watchdog policy.

Review report:
`/Users/patwoz/dev/Personal/macos/bbr/reports/visual/downloads-history4-20261010-20261009-b/index.html`.

2026-10-11: the [Paper fidelity slice](paper-sidebar-downloads-fidelity.md)
changed the rail footer, titlebar arrows and Downloads panel. Its `paper3` pair
is byte-identical (84 PNGs); 62 references now differ only in those regions
and await Patrick's review before any acceptance.
See the delivery ledger's Downloads recovery entry for checks and retained failures.
Remaining families, visual review and stable aggregate acceptance stay open.

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

### Serial Palette closeout → Find → Downloads → Peek (authorized 2026-10-03)

Patrick activated this sequence with “lets goal”. First assess the recorded
Command benchmark and commit the coherent Palette capture intermediate slice
when its relevant checks are proven. Source/dependency assessment confirms that
the dependency-free benchmark and its build configuration are byte-identical to
the last passing base; it never links the app/testkit/capture seam. The recorded
maximum failures remain open, with no proven load attribution or further unchanged
retry. Relevant protocol/capture/Clippy/visual gates and two source-equal Tart runs
support the intermediate commit, not stable aggregate or migration acceptance.
Details: [Palette closeout](../../delivery/current.md#palette-intermediate-closeout-and-serial-continuation-2026-10-03).

Next bounded Find matrix: opened with empty query, query with actual WebKit matches,
and query without matches, each Light/Dark. Use an isolated deterministic local
page, actual production Cmd-F/input and typed find readiness/counts. Read the real
production Find child renderer only; never composite the native webpage or parent.
First prove the smallest opened pair through the existing frozen-payload/offline
Tart route before expanding the query matrix. Keep exact geometry/transparency,
closed/wrong-presentation/path refusal, feature-off behavior, semantic preservation
and independent required/withheld coverage. A narrowly authenticated visual-only
readback is allowed if required; no product lifecycle/rendering change.

After Find, define each minimally meaningful Downloads and Peek matrix from actual
production capabilities before implementation, and prove its smallest real capture
before expansion. No synthetic download engine, Gallery page stand-in or fabricated
native pixels. A necessary product change beyond observation/fixture scope is a
blocker for that slice, not permission to rewrite the product.

For each expanded matrix: two independent same-source Tart captures, exact repeat
and preserved72-reference checks, real PNG/HTML inspection, original payload/build/
source binding and independent resource cleanup; negative-first protocol/catalog/
output tests, feature-on/off strict Clippy, serial offline visual contracts,
format/roadmap and the applicable intermediate/stable gates. Preserve historical
reports/seals and all72 approved reference bytes/records/environment. New images
remain candidates pending explicit approval. Same sole local writer, no subagents,
new worktrees, host GUI, exposure change, publication or upstream/GPUI pin change.
Stop on a genuine unresolved runtime/integrity/determinism or relevant-gate failure
with retained evidence and the minimal prerequisite; do not manufacture a permission
blocker or treat an agent-authored precaution as a user restriction.

Find's bounded matrix is now verified as an intermediate tooling increment:
opened pilot before expansion, then two source-equal78-image Tart runs with exact
PNG/rendering equality and all72 approved references unchanged. Production Cmd-F
opens Find; the existing shared query-input seam drives actual native WebKit
counts, not fake results or a claim of keystroke typing. All six new Light/Dark
images and both HTML reports are inspected. Feature/normal Clippy, protocol/output
contracts, serial visual suite and independent source/build/report/cleanup checks
pass. The [Find checkpoint](../../delivery/current.md#find-production-child-capture-verified-2026-10-03)
records exact evidence and limitations. No reference adoption, stable aggregate,
Command benchmark repair or broader migration completion follows. Downloads then
Peek retain their pilot-first dependency and separate bounded-matrix requirement.

### Downloads production-panel matrix (2026-10-03)

Outcome: three bounded Light/Dark production states, captured through the existing
`CaptureChrome` parent renderer: `downloads-empty`, `downloads-history` (a completed
64 KiB fixture plus an HTTP-500 failure), and `downloads-missing-file` (the same
history after removing only the verified run-created download). This matrix uses
the docked sidebar panel confined above its footer (DESIGN §11.12), not the
floating rail's Portal composition. First prove only the empty pair in Tart before
implementing downloads; docked pixels cannot establish floating correctness.

The expanded fixture must use the existing loopback FixtureServer, authenticated
`DownloadStart` and native WKDownload/controller path, never seeded machine states
or a fake engine. Wait for actual terminal outcomes and settled production moments;
assert exact size/status/action semantics, History ordering, enabled Reveal and
Retry, then disabled Reveal with the existing missing-file reason. Do not activate
Finder or retry the failure. Before deleting the disposable result, prove its
expected guest Downloads path, absence before start and exact content bytes.
Production footer activation and Escape must preserve tab/WebView/session/profile/
space identity; readback must also preserve the entire download model and focus.

Retain full2160×2814 RGBA8 frames, no new crop or compositing. Native webpage and
AppKit pixels remain excluded. Waiting/running/unknown-length/zero-byte/cancelled,
retry/restart outcomes, scrolling, pointer states and animation sequences remain
explicitly withheld rather than implied by three terminal/empty stills. No product
renderer/lifecycle/protocol change is needed for this matrix. Owned territory is
only capture fixtures, independent inventories/output contracts and evidence docs.

After the empty pilot: two fresh same-source expanded Tart runs, exact PNG repeat,
unchanged72 approved references and previous Find candidates, original PNG/HTML
inspection, source/build/payload/receipt and independently observed cleanup checks.
Run negative-first catalog/output contracts, strict feature/normal/consumer Clippy,
serial offline visual tests, format/roadmap/whitespace gates. This remains an
intermediate tooling slice, not a stable aggregate or performance acceptance.
Preserve failed attempts. Stop at an unresolved native-download, capture, identity,
cleanup or determinism failure; do not fabricate state, change the product, weaken
checks, adopt references or fall back to host GUI. Peek follows separately.

The first empty pilot produced80 structurally valid images but visually failed:
the Downloads panel is absent in both original PNGs despite `open=true`, semantic
IDs and empty motion moments. Independent audit preserves all78 prior images,
72 references, source/build/receipt binding and verified clone cleanup. Before
expansion, add only a visual-feature readback guard observing the existing fade's
cached rendered opacity, goal and elapsed transition delta after the actual draw.
Poll that guarded readback within the existing deadline; never reset/jump/clone the
transition or synthesize opacity. A fresh Tart run must distinguish an early frame
from a completed transition that still paints zero alpha. If production lifecycle
repair is necessary, retain the failing candidate/evidence and stop this sequence
for that separately scoped repair; no Downloads coverage is proven by the pilot.

### Authorized Downloads fade repair (2026-10-03)

Patrick answered “ja” to the bounded production fade-lifecycle fix and continuation
of this serial sequence. This supersedes the capture-only boundary only for that
repair. Keep the pinned GPUI transition timing, continuous retargeting, open/close
focus and native boundaries unchanged. Retain the transition's entity state across
frames, but obtain a fresh frame-local interpolation wrapper in the owning
browser render; never carry its evaluated cache across browser draws. Both docked
and floating panel composition read that presented value. Floating composition
runs against the browser window outside its draw phase, so it must not request
animation frames there; the owner render advances and schedules the transition. The capture seam must only
observe the actual rendered value. No vendor patch, blanket animation rewrite,
instant-opacity shortcut or altered references.

The retained alpha0/goal1/delta1 Tart failure is the real-app red regression. Prove
the smallest fixed empty-panel capture first, then add open→close→reopen endpoint
coverage and run the affected normal Downloads suite in Tart. Targeted units,
strict feature/normal Clippy and portable visual contracts remain mandatory. A
verified intermediate repair may be committed separately; stable functional and
performance aggregate acceptance remain governed by the existing delivery rules.
Only after visible pilot proof and the affected ordinary-feature gates pass,
expand Downloads history/missing-file states and repeat the final same-source
captures; Peek follows separately. Ordinary artifacts must use `testkit` without
`visual-capture`, whose GPUI executor/test-support configuration differs. Require
exact listed test names/counts, individual successful results and independent
cleanup; no zero-test, skipped-test or transport-only success. No additional
network/runtime/host-GUI or reference-adoption permission is inferred.

The docked open→fully closed→reopen visual regression now passes with exact
reopened pixels, but the new ordinary floating regression is red at its missing
panel semantic ID. Source inspection identifies docked-only snapshot gating and
zero panel width when that sidebar is hidden; neither establishes floating pixel
visibility or authorizes a broader renderer rewrite. Two later full-suite attempts
stop before test execution at SSH banner exchange. Keep these distinct outcomes,
the failing assertion and historical payloads. The [current checkpoint](../../delivery/current.md#downloads-fade-docked-proof-and-ordinary-verification-stop-2026-10-03)
owns exact attempts and next gates. No expansion, baseline adoption or verified
repair commit follows until the ordinary gate is resolved.

### Authorized floating Downloads repair (2026-10-09)

Patrick's “ja lets go” approves the explained bounded continuation: diagnose the
SSH prerequisite on the smallest fresh VM → SSH → offline runtime → existing-test
path, then repair the floating Downloads width and associated truthful snapshot
mapping. The rail assembler's actual width must drive both docked and floating
panels; do not substitute docked visibility for a floating rail's geometry. Add
ordinary-testkit observation of the panel's actual painted layout/opacity and
assert open/close/reopen, positive confined geometry, focus and browser identity.
Semantic IDs alone are not rendering proof. No general Sidebar/animation rewrite.

Run the focused red regression before the repair, then the complete ordinary
Downloads and Sidebar suites (exact current listed names, no omitted failing case),
relevant unit/protocol/strict Clippy/visual contracts and source/cleanup checks.
A coherent verified intermediate repair may be committed locally, after which the
history/missing-file capture matrix and same-source A/B inspection resume, then
Peek. Existing performance deferral, stable-aggregate obligations, original failures,
72 references, immutable dependencies and all no-host/no-publication constraints
remain. Technical uncertainty calls for investigation, not a renewed consent stop
for this same scope. Stop only on an evidenced unresolved gate or actual boundary;
record the specific failure and next prerequisite without weakening checks.

The bounded floating repair now passes measured width/opacity red→green,
complete ordinary Downloads13/13 and Sidebar21/21, protocol20/20, strict Clippy
and serial offline visual contracts. Original failures and independently verified
resource cleanup remain preserved. This is actual ordinary-feature paint-layout
and identity evidence, not floating pixel/visibility or fresh capture proof.
The fresh visual caller compiled but stopped before VM start on an empty malformed
shared host-gate record. The scratch ordinary runner's raw `a+`/flock can create
that file without the required structured lease and must not be reused unchanged.
No gate deletion, invented idle state, alternate lock or immutable-vendor change
is allowed. The [latest checkpoint](../../delivery/current.md#floating-repair-verified-fresh-capture-blocked-at-host-gate-2026-10-09)
records exact identities and the separate host-wide coordination recovery boundary.
The candidate remains uncommitted; fresh source-bound image/report proof and A/B,
then the repair commit and history/missing-file extension, remain open. All72
references and historical seals stay unchanged; no acceptance is inferred.

### Authorized host-gate restoration (2026-10-09)

Patrick's “ja let sgo” authorizes investigation/restoration of the shared host
coordination file and moving the scratch ordinary runner to the existing
structured gate authority, then fresh captures. This supersedes only the latest
coordination-recovery pause. Inspect actual process/VM/lock users first; preserve
the original empty bytes, inode, hashes and all prior evidence. Under both exact
exclusive locks, a one-time same-inode initialization may record an **active
maintenance lease owned by the actual restoration process**, with the original
hash and audit reason. It must not invent a previous owner or claim unknown
historical cleanup. After that process exits, use unmodified `Gate::recover` to
verify its exact bytes, absent real owner and compatibility gates before idle.
Interruption must remain fail-closed; no automatic empty-record recovery path,
unlink, alternate lock, vendor change or checks weakened. Test stale inode/hash,
contention and unsafe-file refusal on private fixtures before touching real gates.

Wrap the ordinary scratch lifecycle in existing Rust `vm_supervision` and Python
`vm_control`, with independently retained VM/request/payload identity and host
cleanup recheck before lease finish. First prove a small existing ordinary test
in the same offline hardened Tart profile, then complete ordinary suites and
fresh visual source-bound runs. Preserve all failure outcomes; do not expand
networking, mutate Golden Image, install dependencies, accept references or run
host GUI. Stop only at an evidenced remaining technical/safety blocker, not for
another consent request covering these same authorized actions.

### Ordinary Downloads synchronization follow-up (2026-10-09)

The first structured full run retains two exact failures in `start_download`:
its next asynchronous snapshot already sees `Succeeded` or `Running`, not the
asserted initial `Waiting`. Keep the Waiting/Cancel assertions and all downstream
terminal/progress/identity checks. Bound remediation to an explicit single-response
hold in the loopback fixture server and the Downloads test helper: withhold the
real HTTP response until the newly minted row is observed in Waiting, then release
the original fixture response unchanged. Correlate a new attempt by excluding
pre-action IDs, including repeated filenames after terminal history. No production
state injection, protocol change, sleeps, slower fixture or timeout increase.
Test the fixture's hold/release/abort/duplicate paths locally, then the two failing
real-app scenarios and complete Downloads/Sidebar in fresh structured Tart runs.
Preserve the failed run and independently verified cleanup; no green aggregate or
repair commit until the relevant gates pass.

This follow-up now passes actual native execution: both reproduced failures,
the floating regression and complete Downloads/Sidebar suites. Review additionally
requires deadline enforcement for already observed requests, not just sleeping
waiters; its negative-first regression and fresh native recheck pass. Preserve
identical response bytes/retry semantics and abort/notify after expiry. The
structured runner and same-inode authorized gate restoration are also verified.
Fresh visual work is still blocked by intermittent offline metadata deadlines
and an unexpected Tart VM shutdown before import. The [structured checkpoint](../../delivery/current.md#structured-gate-and-ordinary-regression-verified-runtime-remains-intermittent-2026-10-09)
records passed gates, exact failed attempts and independent cleanup. No timeout
increase, repeated unchanged run, host fallback, reference acceptance or repair
commit follows without the remaining fresh PNG/HTML and same-source proof.

### Subject-only stills (Patrick, 2026-10-12)

The `footer5` report showed sidebar changes in every Saved Pages and
History still although those stills are about the page. Patrick decided
that a still photographs only its subject. The testkit `CaptureElements`
request (visual-capture only) draws the main window, takes the rects the
named elements' probes recorded in that frame, reads back only their union
grown by 4 pt (rings, lift, the near part of `elev:2`) and replies with the
kept window-pixel rectangle; a missing element fails, never a larger
readback. History and Saved Pages stills read `bbr-content-card`, Downloads
stills `bbr-downloads-panel`, footer stills `bbr-footer` (with
`bbr-plus-menu` or `bbr-tab-drag-proxy` for the states that open them). The
snapshot records `visual_region`; the host requires the PNG to have exactly
that size inside the window and never the whole window. The shell pairs,
Settings, Find and the Command Palette are unchanged.

- [x] Protocol wire shape, region arithmetic (union, outward rounding,
      clipping, missing element) and host region validation tests.
- [x] Fresh Tart A/B capture with the element stills and a recheck of the
      report; changed references stay for Patrick's review. `element2-20261012`:
      100 PNGs byte-identical across both guests, both cleanups verified,
      working tree unchanged, no baseline writes; 50 element stills carry
      their region. Cut to the same region, the 14 old History references
      are pixel-identical and the 14 Saved Pages ones differ only in the
      24×24 px "New collection" plus (round caps) — the sidebar no longer
      reaches them. The first attempt (`element1`) kept a real failure: the
      Downloads reopen check compared a whole-window frame with the panel
      still; it now compares the two settled whole-window frames, byte for
      byte as before. Tart E2E smoke, sidebar, spaces, downloads, organize,
      hover, real input and gallery: 21/21 executables, 139 tests.

### Withheld coverage, step 1: Saved Pages, History, Settings (Patrick, 2026-10-12)

Patrick asked why the withheld families were never captured and approved
working through them in order: (1) Saved Pages, History and Settings,
(2) Welcome, import and About, (3) Peek, Command and Find, (4) permissions,
(5) an explicit list for the remaining transient states. Step 1 adds 15
required states (30 stills), all from the real app:

- Saved Pages (end of the existing flow, every staged change cancelled):
  search with and without a match, title sort, inline row rename, staged
  collection rename and removal, 50 entries scrolled 12 rows into the
  24-row window.
- History in its own session: a profile seeded with 25 dated visits over
  five days plus two pre-D10 visits, and the testkit `SetDisplayClock`
  seam pinning the display clock to Tuesday 2026-09-29 12:00 UTC, so day
  titles, visit times and the Today range never depend on the guest's
  clock or zone (production always reads the system clock): dated groups,
  Today range, keyboard-selected row, after Clear Browsing Data (All time).
- Settings: the contrast menu; after every other still (creating a
  profile activates its first space) a named New Profile sheet, two
  profiles and the Delete confirmation (cancelled).

The Saved Pages family is fully covered and leaves the withheld list.
History keeps only the scrolled list (no typed scroll observation);
Settings keeps populated Privacy lists (with the permissions step), other
committed choices and other long panes. The stale Command/Find/Downloads/Peek
reason now names what is captured.

- [x] Catalog, inventory and adapter contracts (130 cases), clock-pin wire
      shape and unit test.
- [x] Fresh Tart A/B capture with the 30 new stills and report check.
      `step1-20261012`: 130 PNGs byte-identical across both guests, both
      cleanups verified, working tree unchanged, no baseline writes; review
      62 changed, 10 same, 58 without a reference (30 new). Inspected: dated
      groups Today/Yesterday/Sunday 27/Saturday 26/Friday 25/Earlier with
      fixed times, Today range, selected row, empty page after All-time
      clearing, every Saved Pages and Settings state as named. Tart E2E
      history, client certificates, Settings, profiles, Saved Pages Library
      and command: 7/7 executables, 45 tests. Observation for Patrick: the
      Settings select menus open beside, not under, their button (already
      so in the approved motion menu); not changed here.

### Withheld coverage, steps 2–5 (Patrick, 2026-10-12: “ja bitte alle”)

Sixteen more required states (32 stills), each from the real app:

- Step 2, Welcome and About (own session): the first-run welcome, the
  import sources found in a fixed Safari/Chrome import home, the default
  browser step after the import toast left, the named first space, and the
  About window through BBR › About BBR (`CaptureAbout`).
- Step 3, Peek, Command and Find: Find's second of three matches; the
  Command Palette with a typed address and with a plain search; Peek's bar
  over a loaded preview (an eligible new-window request through the engine
  event path; the bar names the page by its title, never the port).
- Step 4, permissions and sheets (own session on a fixture page, display
  clock pinned): the camera and microphone prompt, site settings with the
  remembered block, the client-certificate sheet, the populated Settings ›
  Privacy lists, the no-app banner and the mailto open-in-app sheet, and
  the HTTP Basic credential sheet. Prompts and sheets are portal-child
  stills (`CapturePortal`); every one names the fixture host without its
  port.

The withheld families narrow to what stays uncaptured (catalog reasons).
Step 5 is a proposal for Patrick's confirmation, not yet captured: the
“Remaining transient application states” family becomes these explicit
groups — tab rows (hover with close, pressed, keyboard focus, loading arc,
crashed, unresponsive overlay); Favorites and folders (hover, pressed,
expanded folder, drop target); tab drag insertion rule; address (focused
editing, not-secure mark, zoom chip, back/forward history menu); titlebar
control hover and press; inactive window, full-screen card, collapsed and
floating rail; content surfaces (empty new tab, offline error, crash panel,
recovery pane); toasts and the media card; split view; tab and sidebar
context menus; Increased Contrast and the S/XL densities for the shell
windows.

- [x] Catalog, inventory and adapter contracts (162 cases), About/portal
      readback wire shapes.
- [x] Fresh Tart A/B capture with the 32 new stills and report check.
      `steps6-20261012`: 162 PNGs byte-identical across both guests, both
      cleanups verified, working tree unchanged, no baseline writes; review
      62 changed, 10 same, 90 without a reference (32 new). Retained
      attempts: `steps2` (the prompt is a portal overlay, not a
      presentation: `CapturePortal` now needs only the synced child frame),
      `steps3` (the same, now with named waits; Find also hit the known
      unresponsive-tab watchdog once under load), `steps4` (a credential
      challenge holds the page loading: portal waits accept reduced-motion
      static marks of the parent window), `steps5` (green, but the 8 s
      no-app banner showed in the next two stills; the capture now waits
      for it to leave). Observations for Patrick, not changed: the
      open-in-app and certificate sheets stretch to the full card height,
      and site settings call `http://127.0.0.1` “Connection is secure”.
- [ ] Patrick confirms or edits the step 5 list before it is captured.

### Sheet height fix (Patrick, 2026-10-12: “viele der dialoge sind viel zu hoch”)

The open-in-app, client-certificate, Settings (New/Edit/Delete Profile) and
Clear Browsing Data sheets sat in a `wash.active` veil that is a flex row
with `justify_center` and no cross-axis rule, so flex stretched each
`h_auto` sheet to the whole card or Settings window. The veils now use
`items_start`: the sheet hangs from the card top at its content height
(the credential and space-delete sheets were already centred). Regression:
the four sheet panels probe their drawn rect into the new snapshot field
`sheet_rects` (any window; the portal and Settings windows clear their
rects per frame) and the real-app suites assert
`assert_sheet_hugs_content` (open-in-app ≤ 260 pt, certificate ≤ 420,
New Profile ≤ 480, Delete Profile ≤ 220, Clear Browsing Data ≤ 460).
`FlushFrame` also draws an open Settings window, so its drawn state never
lags the model.

- [x] Red without the fix: link destinations, client certificates, history
      and profile management fail on the new assertion (sheets drew 666 pt
      in the card, 602 pt in Settings; run-20261012-123506-96820).
- [x] Green with the fix: profiles, Settings, permissions, certificates,
      history, link destinations, external URL and first run 9/9
      (run-20261012-125208-60444); smoke, portal and real input earlier
      green with the probes (run-20261012-123838-30717).
- [x] Fresh Tart visual A/B with the corrected sheets: `sheets1-20261012`,
      162 PNGs byte-identical across both guests, both cleanups verified,
      working tree unchanged, no baseline writes. Open-in-app, certificate,
      New/Edit/Delete Profile and Clear Browsing Data sheets now hang from
      the top at their content height.

Patrick then decided (“lass alles mittig machen”, against the Paper boards'
top attachment, P-4 note 2): every blocking sheet sits centred on both axes
over its veil. The four `bbr-ui` veils centre by layout (`items_center`, no
top margin); the credential and Delete Space portal surfaces now place their
sheet inside the veil and centre it by layout too (the credential sheet was
centred on the window, not the card; Delete Space was attached to the top).
The veils probe their rects as well, and the shared check became
`assert_sheet_centered` (content height and centre within 1 pt of the
veil's) in the open-in-app, certificate, history, profile, credential and
Delete Space suites. DESIGN §11 records the rule.

- [x] Real-app E2E: link destinations, certificates, history, profiles,
      Settings, spaces (5), auth (7), portal (11), permissions, smoke
      15/15 (run-20261012-165811-30063); persistence, recovery, gallery
      3/3 (run-20261012-170400-87370).
- [x] Fresh Tart visual A/B with the centred sheets: `sheets2-20261012`,
      162 PNGs byte-identical across both guests, both cleanups verified,
      working tree unchanged, no baseline writes; Clear Browsing Data,
      open-in-app, certificate, credential and the Settings profile sheets
      inspected centred on their veils.

## Non-goals

- No browser appearance or behavior changes, portal capture beyond the bounded
  Command Palette/Find/Downloads/Peek follow-ups above, native WKWebView
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
and production child-window testkit readbacks described above.
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

### Route activation (Patrick, 2026-10-16: visual evidence is official only)

The composed internal route had no repository caller: every report so far
came from out-of-repo scripts, and `just visual-check` pointed at the refused
host adapter. Patrick's rule that chrome pixels are proven only through the
sealed route made the route itself the deliverable. AGENTS.md now records the
standing Tart authorization for it (same profile as `just e2e-vm`).

- [x] `check <suite>…` (`just visual-check`) runs prepare → supervised Tart
      transport → upstream seal → Collection review per explicit suite; the
      host producer stays refused and the `--dry-run` preview is unchanged
      apart from reporting `tart_vm` execution.
- [x] The guest bootstrap binds the payload's compiled checkout to the main
      clone **or** a `bbr-<lane>` sibling worktree under the same fixed
      ancestor (never a remap), so lanes capture before integration.
- [x] `gate-recover` (`just visual-gate-recover`) exposes upstream's attested
      host-gate recovery after a failed/killed run.
- [x] First official run from a lane worktree: batch
      `1792102072680971000-69492` (seated card edge), sealed, 78 results,
      clone removed, gate finished.
- [ ] Pin a host `python3` ≥ 3.10 in the dev shell; the route currently
      depends on the caller's PATH (mise 3.12 on Patrick's shell).
- [ ] Retire the legacy host producers still reachable through `just visual`
      (`batch.py`/`capture.py`) and `tool.py check` as part of the Python
      cutover.

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
