<!-- bbr:current-delivery -->
# Current delivery

This is the sole current owner of BBR's volatile execution state. Start at
[ROADMAP.md](../../ROADMAP.md), then read this file completely before choosing
or scheduling work. Durable scope and acceptance remain in the linked task
packets. Full chronology and detailed evidence remain in the preserved
[2026-08-08 post-MVP ledger](2026-08-08-post-mvp-delivery.md).

## Control metadata

| Field | Value |
|---|---|
| Generation | `post-mvp-f2a` |
| Candidate prepared from | `main` `0dc22fee16d737a077aa1b097e359e2f1d08a764` |
| Canonical-ledger refresh through | `main` `0dc22fee16d737a077aa1b097e359e2f1d08a764` (F2a integrated at `7180128`; documentary closeout and its green `just roadmap`/`just verify` evidence committed at `0dc22fe`) |
| Cutover commit | `781e862ca4547cee16be23e251767f6b2fa938fb` (tree `8e87a7931bab248faa707dbb131f7df6e341db80`, parent `0dc22fee16d737a077aa1b097e359e2f1d08a764`) |
| Atomic handoff | acknowledged after committed-HEAD audit and independent `COMMITTED_READBACK_PASS` |
| Verified product base | `main@0dc22fee16d737a077aa1b097e359e2f1d08a764` |
| Current integrated control checkpoint | `6ff70eab66432655e184ff3e09147650ef6bf39f` (tree `55f6f10e05d14a08c6b34bf47e69106725688fcb`); includes SHUTDOWN, deterministic error-fixture recovery, and deferred native Portal visibility ordering |
| First-wave branch base | `e9f5d30214e7efc46586cb3ae11c6472e0b1801c`; SHUTDOWN, B1, and G1a were all created clean from this exact commit |
| Previous canonical pointer | `docs/delivery/2026-08-08-post-mvp-delivery.md` |
| Rollback pointer | `docs/delivery/2026-08-08-post-mvp-delivery.md@0dc22fee16d737a077aa1b097e359e2f1d08a764` |
| Target Orchestrator session | `019fdc57-7afb-7f81-8963-756586fe0a6b`; current verified pane `tws_personal_macos_bbr-orchestrator:3.0` (pane is runtime data and must be reverified immediately before handoff) |

The atomic cutover is complete. `ROADMAP.md` and this committed snapshot are
now the sole execution authority; the old ledger is preserved evidence and the
rollback target, never a parallel queue. A filesystem-only copy, stale
worktree, historical workflow, or uncommitted main-worktree document has no
product authority.

## Status vocabulary

| Status | Meaning |
|---|---|
| `planned` | durable contract exists; dependencies are not all satisfied |
| `ready` | dependencies and ownership preflight permit a writer to start |
| `running` | exactly one writer owns the lane |
| `review` | writer settled; independent read-only review is active |
| `remediation` | one writer addresses concrete review findings |
| `frozen` | direct review pass; tree waits for serial integration |
| `blocked` | no defensible local action remains without the named unblocker |
| `integrating` | Orchestrator owns the serial merge and integration gates |
| `integrated` | change is on `main` with lane-specific evidence |
| `closed` | full milestone/closeout acceptance is proven |

Dependencies and integration priority are separate fields; they are not encoded
as additional status words.

## Active product wave

F2a and its documentary closeout are integrated. The atomic cutover and
fresh-operator acknowledgement are complete. SHUTDOWN and the two bounded F2
recoveries are integrated on the current control checkpoint. All 27 functional
E2E suites are green. Patrick explicitly prioritizes a feature-complete first
MVP over further startup investigation: the known startup-performance debt and
full release closeout move to Phase 9 without changing the `≤500 ms` contract.
B1 and G1a are integrated and independently verified. G1b Gallery MVP and B2
Saved Page command/provider started from one exact common base in isolated
worktrees with disjoint primary territories. Both original writers settled on immutable forward states. Independent static
reviews found bounded remediation in each lane: G1b has three Gallery-only
lint/test/doc findings; B2 needs production owning-window Saved Page flush parity
plus stale packet metadata repair. Two disjoint bounded remediation writers may
run in parallel; GUI/E2E/performance gates remain host-exclusive and serial.

| ID | Slice | Status | Worktree | Owner | Contract | Next gate |
|---|---|---|---|---|---|---|
| CUTOVER | Delivery-Control authority cutover | `closed` | `/Users/patwoz/dev/Personal/macos/bbr-delivery-control-reconcile` | Orchestrator only; no second writer | [completion audit](delivery-control-completion-audit.md) | complete at `781e862`; committed-HEAD audit and independent readback passed |
| F2-STARTUP | Cold fresh-profile startup regression remediation | `planned` | `/Users/patwoz/dev/Personal/macos/bbr-f2-startup-remediation`; branch `parallel/f2-startup-remediation`; clean unchanged base `1eb74b1df4a7651cd6357eabb75346389110da00` / tree and index `0941b044e100edc26914ffe1a817a781f3959dcc` | no active writer; further startup work deliberately deferred until the first feature-complete MVP | [performance E2E foundation](../roadmap/tasks/performance-e2e-foundation.md) | Phase-9 release hardening after MVP feature delivery: retain the hard `≤500 ms` contract and consumed unchanged reproduction, then define new causal territory or canonical-runner policy. No automatic budget, harness, sample, baseline, Xcode, milestone, Portal, or initial-tab change is authorized |

## Post-F2a parallel wave

All three lanes start from exactly the same verified post-F2a `main`, after a
full queue and active-writer audit. Implementation and non-GUI checks may run
in parallel; all GUI/E2E/performance gates remain host-exclusive. Read-only
source maps, conflict maps, regression-first plans, and handoff drafts may be
prepared beforehand only outside repository/worktree/index state; the current
preflight `post-f2a-parallel-readonly-preflight-mskx3emg-aqoxwp` grants no
product authority and every bundle is stale until refreshed after F2a.

| ID | Status before F2a close | Worktree / owner | Exclusive territory | Start condition | Integration priority |
|---|---|---|---|---|---|
| SHUTDOWN | `integrated` | `/Users/patwoz/dev/Personal/macos/bbr-shutdown-lifecycle`; branch `parallel/shutdown-lifecycle`; no writer; exact reviewed lane commit `48f2841b9db46f19ebbd8f8b9b3637496f07e37d` | app lifecycle, real SIGINT and owning-window close, portal-child isolation, testkit response flush, fresh `e2e_shutdown`, required E2E registry | merged as `12431c95…`; deterministic error-fixture recovery merged as `1e42d835…`; deferred Portal visibility recovery merged at control checkpoint `6ff70eab…`. `just verify` and all 27 functional E2E suites are green, including Portal 7/7 and errors 10/10; no `RefCell already borrowed`. Startup performance remains explicit Phase-9 debt rather than a feature-delivery blocker | **1 — product integrated; functional checkpoint accepted** |
| B1 | `integrated` | `/Users/patwoz/dev/Personal/macos/bbr-saved-pages-b1`; branch `parallel/saved-pages-b1`; frozen lane commit `a23d9335240ff4c6cbc0d5d8b80744339f4a335e` / tree `0c728df48ecf495375b6b4ad096dce35d41cfbbf` remains clean | no writer; integrated by Orchestrator merge `eaa0078d8e1c98560055ee1eeb723060fed53570` / tree `ffccb1349d2b5448003c5ec16e09a1ffd4af37ee` | focused portable domain/persistence Saved Pages modules, canonical URL equality, CRUD/search/list, schema v3 bump+discard, fixtures/benchmarks | exact 39-path merge; no conflict or unstaged mutation; all B1 gates green and independent exact-index `REVIEW_PASS`. B2 may satisfy only its integrated-B1 dependency; its remaining contract decisions and ownership gates still apply |
| G1a | `integrated` | `/Users/patwoz/dev/Personal/macos/bbr-ui-g1a-foundations`; frozen lane `e6d3ab85a49b8754764553b31d0eac996015ce25` / `8c441f2ccee86792b4475a314b472299717d11fb` remains clean | no writer; integrated by Orchestrator merge `e076911a1a0f6938cc394020c1735628ce580f91` / tree `1643060ddcb1a61cab8308c95c66258bfa3848ec` | one production `bbr-ui` authority; exact direct deps `gpui` + `bbr-domain`; only `bbr-domain → bbr-web` transitivity; app consumes the same production symbols | `ui-deps`, clean full verify, testkit Clippy, persistence, roadmap, and all functional E2E except the carried host-evidence case passed; independent `REVIEW_PASS_HOST_EXCEPTION`. Portal minimize/restore remains explicitly unpassed on this host because persistent `loginwindow` window `48252` layer 2004 wins Window-Server hit-testing; native Portal and test files are byte-identical to the green F2 checkpoint. No retry or test relaxation. Startup performance remains Phase-9 debt |
| G1b | `remediation` | `/Users/patwoz/dev/Personal/macos/bbr-gallery-g1b-mvp`; branch `parallel/gallery-g1b-mvp`; base `c41b06c3ff466b87fefaaf6c3f353c62c0d4ee50` / tree/index `1df4b846e23a0c70485e50e7484dd193eba5310d`; exact reviewed input tree `ecb84ab89d0b66f5562a3137740bfd31b97412b7`, binary diff SHA-256 `af559aa11f22eb767d24c1f51a328da9ac7571e77682c5985dbcb46f1088346e`, status-z SHA-256 `22388e8081c853eacdf952a7d84423f7b76a76a534e0657eeb4076a9473d0886`, 28 paths, real index base | original writer settled `BLOCKED`; independent static review `g1b-blocked-settlement-and-static-review-msm1iy8i-4cnwhv` returned three bounded `REVIEW_FINDINGS`. One replacement writer may own only G1b until direct `CANDIDATE_READY` or `BLOCKED` | Preserve the implemented pointer/core-control/copy/macOS-gating product. F1 remove unused `AppContext` imports in `main.rs` and `harness.rs`. F2 repair five collapse-E2E expected counts to subtract only the collapsed group's stories; do not change product behavior or weaken assertions. F3 correct `dependency_contract.rs` module docs to the exact five-dependency allowlist and rationale. No Status line, G2/G3, B2, `bbr-ui`, ranking, or browser-app edits | Run fmt, gallery all-target check/test/Clippy `-D warnings`, workspace check/test/Clippy, `just ui-deps`, `just roadmap`, and `just verify`. Clean locked `just e2e-gallery` remains mandatory but must wait until Patrick-owned PID `82530` exits and foreign GUI/build load is gone. Return only exact `CANDIDATE_READY` or `BLOCKED` fingerprints | parallel with B2 remediation; GUI gates remain serial |

Shared registry, manifest, lockfile, Justfile, composition-root, and E2E-list
conflicts are predeclared serial integration costs, not reasons to duplicate
product authority. G1a's dependency gate audits direct/source authority and the
first-party closure separately: `bbr-ui` may consume canonical `ProfileHue`
through `bbr-domain`; the existing `bbr-domain → bbr-web` `ProfileId` edge is
accepted, while a direct `bbr-ui → bbr-web` edge or second hue/identity type is
forbidden.

## Serial integration order

1. Complete the atomic Delivery-Control cutover, committed HEAD audit,
   independent fresh-operator readback, and acknowledgement. Any mismatch
   leaves the historical ledger canonical and starts no product lane.
2. Re-read this complete document and audit every open queue item, active
   workflow, writer, worktree, authority dependency, and shared kernel point.
3. Start SHUTDOWN, B1, and G1a maximally safely from the exact same verified
   base. Start G1b immediately when G1a settles. All substantive work uses
   isolated background workflows with explicit DeepSeek-v4-flash high.
4. Integrate SHUTDOWN first.
5. Accept the integrated F2 functional checkpoint for continued MVP feature
   delivery: `just verify` and all 27 functional E2E suites are green. Preserve
   the reproduced startup p95 `535.87 ms > 500 ms` as explicit Phase-9 release
   debt; do not retry the unchanged tree or change an acceptance threshold.
   Required feature-specific regression and performance scenarios remain in
   force, but no further dedicated startup-remediation work runs before the
   first feature-complete MVP.
6. Integrate B1, G1a, and G1b serially according to the actual reviewed trees;
   default order B1 → G1a → G1b unless concrete dependency evidence requires
   another semantic integration order.
7. Continue MVP feature delivery through the dependency queue. Before release,
   return to F2 startup and run the complete closeout sequence from
   `just verify` → `just e2e` → `just e2e-perf` → `package-verify`; record
   canonicality, Bundle/Binary SHA, and clean process/GUI-lock state.
8. Keep G2, G3, B2, B3, and Z packages dependency- and ownership-gated; at
   every scheduling checkpoint refill every safely runnable bounded lane.

## Downstream dependency queue

| ID | Status | Depends on / boundary |
|---|---|---|
| B2 | `remediation` | `/Users/patwoz/dev/Personal/macos/bbr-saved-pages-b2-command`; branch `parallel/saved-pages-b2-command`; base `c41b06c3ff466b87fefaaf6c3f353c62c0d4ee50` / tree/index `1df4b846e23a0c70485e50e7484dd193eba5310d`; exact reviewed input tree `989185c1f45da6c306fd4bf6baab2e75cb86fe30`, binary diff SHA-256 `6f0059eaa8ff00698f3a39d6d57648a2f169e03778e150818f7d30380a55901f`, status-z SHA-256 `d7b072ce9f0e9686501462fa1142649a389d3fa265f04a2ec36afe08a6b9e5ec`, 49 paths, real index base. Independent static review `b2-static-candidate-review-msm16ivc-ir57si` returned two bounded `REVIEW_FINDINGS`; preflight `b2-review-findings-remediation-preflight-msm1owoh-zqtdug` returned `PREFLIGHT_READY`. One replacement writer may implement production quit-flush parity: un-gate `BbrApp::flush_saved_pages`, synchronously call it after `flush_browser_state` and before `cx.quit()` on owning-window close, remove the now-dead lint allowance, align app/architecture/E2E docs, and add a deterministic fourth Saved-Pages E2E that Cmd-D saves, proves the writer idle, wipes the DB out-of-band, closes through production `Request::CloseWindow` without `FlushSavedPages`, waits for successful exit, restarts the same profile, and proves one restored record. Correct task-packet `§9b→§9c` and stale tree/path metadata. No ranking/provider/budget/menu/Gallery/`bbr-ui` change; command-history quit parity remains out of scope. The historical U2 stash remains untouched. After remediation run focused testkit/non-testkit Clippy, unit tests, `just verify`, and `just roadmap`. Clean-host GUI gates remain `just e2e-shell-saved-pages` (now 4/4), `just e2e-shutdown`, `just e2e-shell-command`, `just e2e-perf`, then canonical sequential `just e2e`; no overrides. PID `82530` and foreign GUI/build load still block canonical evidence. |
| B3 | `planned` | integrated B1 + B2 + G1a; ordinary GPUI Saved Pages Library, no permanent sidebar region |
| G2 | `planned` | Start immediately after integrated G1b when production territories are ownership-free. First serial cutover wave: Status line, Tab Row, Address/Input pieces, Favorites/Folders/Footer/Space chip, Downloads, Find, Error/Crash, Auth, Context Menu. Stories never precede production extraction/migration; no Saved Pages row before integrated B2/B3. |
| G3 | `planned` | Start immediately after integrated G2. First serial composition wave: Sidebar, Command Palette/Overlay, Downloads/Find/Error/Auth/Menu groups, Split. Saved Pages remain append-only after integrated B2/B3. |
| Z1 | `planned` | integrated F2a + G1a; Bookmark row additionally waits for B2 |
| Z2 | `planned` | integrated F2a + G1a; flat session-local Folder authority only |
| Z3 | `planned` | integrated functionally green F2 checkpoint and free Portal/context-menu territory; Bookmark row additionally waits for B2 |
| Z4 | `planned` | integrated F2a + G1a; one titlebar/navigation authority |
| Z5 | `planned` | integrated F2a + G1a and public capability-first media contract |
| ICON | `integrated` | reviewed icon commits are included in F2a integration `7180128`; obsolete Spaces glyph authority remains absent |
| ENGINE-SPIKE | `planned` | research only; no product or release claim until a canonical product task accepts the result |

## Open product and evidence work

| ID | Status | Exact remaining condition |
|---|---|---|
| MANUAL-SMOKE | `blocked` | Patrick runs the packaged-app smoke checklist after the next stable bundle |
| UI-2B | `blocked` | Patrick provides a screenshot or concrete problematic UI location |
| PORTAL-MINIMIZE | `closed` | deferred `NativePortal::show/hide` ordering is integrated at `6ff70eab…`; the real integrated-head Portal suite is 7/7 including `host_hides_on_minimize_and_restores`, with no `RefCell already borrowed`. Portal is no longer the F2 blocker. |
| F2-CLOSEOUT | `planned` | Deferred by Patrick until the first feature-complete MVP so delivery does not over-invest in startup performance now. The one allowed unchanged-tree reproduction remains consumed and red at startup p95/max `535.866375 ms > 500 ms`; command first-result p95/max `40.241083/40.528958 ms` passed. No retry, threshold/sample/baseline relaxation, or silent closeout claim is permitted. The six previously allowed adapter seams were measured below `6 ms` total versus approximately `36 ms` required recovery; no candidate exists and the changed-candidate performance run remains unused. Before release, define new causal territory or canonical-runner policy, then execute the full canonical verify/E2E/performance/package sequence. Existing evidence fingerprints remain recorded in the F2 rows below. |
| PERF-500-50 | `blocked` | stable measured 500-tab/50-session frame-time evidence on a dedicated canonical runner; functional 500/50 and measured 230/20 are not substitutes |
| PERF-RUNNER | `blocked` | name/provision the canonical runner and accepted baseline publication surface |
| PERF-RESOURCES | `planned` | CPU, wakeups, energy, memory pressure, and suspend/resume evidence |
| PHASE-6 | `planned` | complete real two-pane split product acceptance and persistence |
| PHASE-8 | `planned` | remaining daily-driver capabilities and manual representative-site matrix |
| PHASE-9 | `planned` | accessibility, visual/release hardening, signing/notarization and clean-machine acceptance |
| PHASE-10 | `planned` | Windows and Linux adapter spikes and implementations |

## Standing decisions and constraints

- No persistence migrations or backward compatibility during development;
  schema change means version bump plus discard/recreate.
- `bbr-command` owns the portable provider-neutral global Search Settings
  contract; `bbr-app`/`bbr-persistence` own composition and storage. Ranking
  and render never query SQLite.
- Space appearance uses the integrated nine-token `SpaceTheme` catalog (six
  ProfileHue-compatible plus three Space-only) and validated curated identities.
- Downloads: standard suggested destination, active-only badge, persisted
  terminal history, Cancelled Restart/Delete, Reveal-only success behavior.
- Pin, Favorite, and Saved Page remain distinct authorities. Saved Page is a
  global durable canonical-URL record without Tab/WebView/Session/Profile and
  opens exactly one fresh ordinary tab in the currently active Space/Profile.
- B1 canonical-URL equality is settled for this slice: normalize/default the
  navigable scheme through the existing authority, then ASCII-case-fold only
  the scheme and host. Preserve path, query, fragment, explicit port, trailing
  slash, `www.`, and percent-encoding byte-for-byte; do not elide default ports
  or add any bookmark-only rewrite. Unsupported/dangerous URLs reject whole.
- B2 Saved Page command/provider decisions are settled: Saved Page results rank
  after standalone Spaces and before History (`ExactTab < CurrentPinned <
  CurrentTab < OtherSpace < Space < SavedPage < History < Action < Fallback`).
  `saved_page_search_first_result` uses the existing warm command-type budget
  p95/max `50/100 ms`; `saved_page_open` uses the existing session-recreation
  budget p95/max `100/150 ms`. No new budget class is introduced.
- One writer per worktree; one product authority per concern; only the
  Orchestrator commits, merges, and integrates.
- Never weaken tests, validators, budgets, sample counts, deadlines, or
  acceptance criteria.
- Dedicated startup optimization and the full release closeout are deliberately
  deferred until the first feature-complete MVP. This changes scheduling only:
  the `≤500 ms` target and all recorded red evidence remain authoritative.
- Feature-specific regression coverage and required bounded performance E2E
  remain part of each user-visible implementation slice.
- GUI, real-app E2E, and performance gates are strictly serialized through the
  repository GUI lock.
- Preserve unrelated dirty files and worktrees by explicit scope and
  fingerprint.

## Scheduling checkpoints

| Checkpoint | Runnable audit | Decision / evidence |
|---|---|---|
| Atomic handoff | CUTOVER only until committed verification completed | `781e862` committed; tree/parent/pointer/scope audit passed; independent `COMMITTED_READBACK_PASS`; historical ledger demoted to evidence |
| First post-cutover audit | SHUTDOWN, B1, G1a runnable; G1b and all downstream rows dependency-gated | no current workflow owns their territories; proposed branch/worktree names are free; repository GUI lock is unheld; stale dirty historical worktrees grant no authority |
| Parallelism decision | start all three first-wave lanes from one identical committed scheduling base | expected shared conflicts are declared integration costs: SHUTDOWN—app composition/testkit/E2E list; B1—schema version/repository DTO aggregates; G1a—root manifest/lockfile/Justfile/ROADMAP/app composition. SHUTDOWN still integrates first; GUI/perf gates remain serialized |
| Dispatch | SHUTDOWN, B1, and G1a all runnable | three clean sibling worktrees created from `e9f5d302`; three background workflows started with explicit `homelab-llm/deepseek-v4-flash:high`; exactly one writer per worktree |
| Preflight reconciliation | refreshed source maps are advisory, committed contracts and real ownership are authoritative | G1a refresh re-grounded integrated F2a sources and closure. B1's stale script misclassified settled dirty historical F2a worktrees as active writers; Shutdown's stale script rejected the now deliberately tracked canonical packet and used unsupported macOS `grep -P`. Neither stale script owns a start gate; no source-territory writer exists. |
| G1a writer settlement/refill | original writer completed with a coherent but incomplete new-crate-only tree; no review was authorized | 20 untracked `crates/bbr-ui/**` files preserved; no app/manifest/lockfile mutation and no surviving build process. Original ownership released, one continuation writer started in the same worktree with exact fingerprints. SHUTDOWN/B1 remain active; no new downstream dependency became runnable. |
| B1 writer settlement/review | writer completed but its final report was transport-truncated, so wrapper completion is not acceptance | preserved source/index state; temporary-index tree `b63becc1…`; 31 files, 3301 insertions/122 deletions; no app/UI/manifest scope; independent SHA-bound reviewer started. B2/B3 remain dependency-gated until reviewed integration. |
| B1 review/remediation | direct `REVIEW_FINDINGS`; no blocker, five concrete remediation items | F1 Unicode search mojibake/SQLite case-fold divergence; F2 mutation can persist `updated_at < created_at`; F3 single-record reads skip row validation; F4 byte-identical update writes; F5 B1-owned >500-line modules. One bounded writer started from the unchanged reviewed tree; focused recheck follows. |
| G1a continuation/recovery | continuation writer terminated after a rejected oversized edit and reported a known-broken mid-cutover tree | old definitions were deleted while `main.rs` declarations, app dependency, four elevation duplicates, compile, gates, E2E, and perf remained incomplete. Mixed real index is preserved (only staged palette/theme deletions); full temporary-index tree `6c4fbce4…`. One recovery writer owns forward repair; no commit/review/G1b start is permitted before coherent gates. |
| SHUTDOWN idle settlement | original writer launched long GUI/perf gates in background, then emitted no final report; wrapper was stopped only after no process still owned the worktree | preserved full tree `8bc35bfd…`; raw status `cd645d39…`; clean real index. `e2e_shutdown` 3/3 and non-GUI verify are green. Canonical `just e2e` remains failed at the known portal-minimize host condition, and unchanged startup performance failed twice above the 500 ms ceiling. One settlement writer now owns bounded truth/gate completion before independent review. |
| B1 remediation settlement/recheck | remediation writer completed F1–F5 with all prescribed portable gates green and released ownership | independently rebound base `e9f5d302…`, final tree `0c728df4…`, raw status `9d8ad03e…`, clean real index; remediation delta SHA-256 `db46d528…`. One fresh read-only reviewer rechecks Unicode semantics, timestamp clamping, single-read validation, true no-op writes, module budgets, and every unchanged B1 contract. B2/B3 remain blocked. |
| SHUTDOWN settlement/review | settlement writer audited the preserved tree, fixed only lane-introduced doc attribution and packet truth, ran final lane gates, and released ownership | independently rebound final tree `69c9badd…`, raw status `a27c434e…`, clean real index; seven expected paths. Fresh read-only reviewer owns lifecycle/flush/E2E/docs review. Portal-minimize and startup-performance failures remain explicit F2-closeout blockers, neither candidate passes nor automatic SHUTDOWN findings. |
| B1 focused pass/freeze | focused reviewer returned direct `REVIEW_PASS` on the exact remediated tree with all portable gates green and zero mutation | Orchestrator committed the unchanged reviewed tree as `a23d9335240ff4c6cbc0d5d8b80744339f4a335e` (tree `0c728df4…`); worktree clean and frozen. No main integration occurred. B2/B3 remain blocked until B1 is actually integrated after SHUTDOWN/F2 closeout. |
| SHUTDOWN review/remediation | independent review found the lifecycle, response-flush, E2E, package, and lane-specific contracts satisfied, but returned `REVIEW_FINDINGS` for one false packet path count plus optional related window-model wording precision | one bounded writer may edit only the packet, `window_seam.rs` docs, and ARCHITECTURE implemented-fact wording; no behavior/test/budget change. Focused roadmap/unit recheck follows before freeze/integration. |
| SHUTDOWN docs remediation/recheck | writer corrected the seven-path ledger count and portal-versus-find/auth window-model wording with no behavior mutation, then released ownership | final tree `d2b206c6…`; three-file delta only; fmt, roadmap, and seam unit tests green. Fresh focused reviewer returned direct `REVIEW_PASS`; Orchestrator committed the exact tree as `48f2841b…`. |
| SHUTDOWN serial integration | exact reviewed lane merged on the clean control trunk; main worktree remained protected and foreign-dirty | integration merge `12431c95…`; corrected-CWD `just verify` green, canonical E2E proves all suites through persistence plus SHUTDOWN before the known portal blocker, serial perf green, serial package green. |
| F2 error-fixture review/integration | independent reviewer returned direct `REVIEW_PASS` on exact tree `8314793d…`; Orchestrator froze it unchanged as `e1f9e523…` and merged it as `1e42d835…` | at this checkpoint `just verify`, all post-portal suites including errors 10/10, performance startup 484.532 ms with `regressions: []`, and deterministic package were green; canonical E2E still stopped at the then-open Portal minimize condition. This is preserved checkpoint evidence, not the current blocker. |
| F2 Portal visibility recovery | independent review returned direct `REVIEW_PASS` on tree `55f6f10e…`; freeze readback returned `FREEZE_PASS`; Orchestrator committed `6a54cc9…` and integrated it at `6ff70eab…` without changing the reviewed tree | `NativePortal::show/hide` now defer `orderFront:`/`orderOut:` to a bare main-runloop turn while logical visibility changes immediately. Portal 7/7 including minimize, Find 6/6, Command 25/25, Auth 4/4, Context Menu 9/9, all 27 functional canonical E2E suites, and no `RefCell already borrowed`. Portal blocker closed. |
| F2 canonical closeout after Portal recovery | exact integrated control checkpoint `6ff70eab…`; no product mutation | first run proved all functional E2E green, then failed performance with startup p95 597.81 ms and command first-result max 185.56 ms under concurrent Tart verification and an operator Xcode-26.6 override; artifact `canonical: false`. Evidence is invalid for closeout and not a Portal finding. |
| F2 idle-host recovery V2 | exact clean HEAD/tree `6ff70eab…` / `55f6f10e…`; bounded read-only operator | external Tart native/functional runs and runner SSH occupied the full 30-minute window. GUI lock stayed free and no competing BBR process remained, but no idle pair was possible. No performance, E2E, package, build, or GUI gate ran; the single controlled reproduction was still unused at this historical checkpoint. F2 remained blocked on a future genuinely idle host, not on product Portal behavior. |
| F2 controlled startup reproduction | exact clean control HEAD/tree `a6750d0…` / `0467dc9…`; exclusive operator after idle observations `10:20:48Z` and `10:24:10Z` | one `CARGO_BUILD_JOBS=4 nix develop -c just e2e-perf` ran with repository Xcode 26.2 and no override. Fresh revision-bound evidence failed exit 101 only on startup p95 `535.87 ms > 500 ms`; command first result passed at p95/max `40.24/40.53 ms`; `regressions` contains the startup failure. No full closeout or retry ran; final HEAD/tree stayed clean, no BBR child remained, and the GUI lock was free. The unchanged-tree reproduction is consumed. One isolated causal startup-remediation lane is the next executable product gate; B1 and G1a remain frozen. |
| F2 startup-remediation settlement | original workflow `bbr-f2-startup-remediation-writer-mslog7lh-qclwc7` failed only while producing required structured output after three provider `503` responses; no product verdict was emitted | no gate or product fix completed. Preserved state: modified `crates/bbr-app/src/main.rs` plus untracked temporary `crates/bbr-persistence/tests/zz_diag_restore_cost.rs`; temporary-index tree `6a3b0ee018d9c8cb2dd2019e128a40e93c60d73e`; binary diff SHA-256 `a8f88496eb99896b5e9b261a3930dcc61cb3257126e0d3687a0c670a8b69ece3`; raw-status SHA-256 `3ed4050388b2f371ccb59471bcd1cf77eb7de4924ea914257e8de6e58db64756`; real index remains base tree `0941b044…`. No BBR process remains and the GUI lock is free. Ownership was released for the completed continuation below. |
| F2 startup causal diagnostic | continuation workflow `bbr-f2-startup-remediation-continuation-mslpfsaj-shyj45`; no product fix and no acceptance performance gate | one bounded GUI-locked diagnostic of the unchanged failing code measured parent spawn→Startup commit `545.79 ms` and app-internal `process_started`→commit `230.94 ms`. Phase points: application run `46 ms`, store open `52 ms`, engine built `118 ms`, window complete `125 ms`, initial tab `202 ms`, Portal `207 ms`, gateway/pump `207 ms`; committed state had view 1, visible sidebar, and non-zero content rect. This rejects the 10k-history/store-open hypothesis but does **not** remove pre-main/dyld/AppKit/scheduling time from the product budget or authorize a Portal mutation. Latest preserved state adds temporary `crates/bbr-app/tests/zz_diag_startup_commit.rs`; temporary-index tree `e445b91b1413d98b6f2d0d3cd5b5bcf042fced48`; tree-diff SHA-256 `79872cbc83476811a81315a883aca5f726b57db1ebb1c5c5f96437ec57613b22`; raw-status SHA-256 `92bc037a44be71318f3f278cb8df24cc6b2b24140545415575acc98e4e4ed943`; real index remains `0941b044…`. All three TEMP diagnostics must disappear before handoff. No BBR process or GUI-lock holder remains. |
| F2 post-frame deferral design settlement | final investigation workflow `bbr-f2-startup-final-remediation-mslrck40-0xmf9p`; worktree unchanged from `e445b91b…`; no gate run | GPUI's exact API contract confirms `Window::on_next_frame` runs directly after the current frame is rendered. The synchronous interval after window construction and before initial-tab completion contains find, error, zoom, menu, geometry, and downloads composition. Deferring window-independent work was only a candidate mechanism. Error and download capture remain initial-navigation-critical: they must be installed before any corresponding event or use a tested single-authority lossless buffer/replay. Portal, startup milestone, harness, budgets, samples, and initial-navigation semantics remain unchanged. The bounded measurement below rejected this candidate on materiality. |
| F2 allowed-seam materiality blocker | bounded workflow `bbr-f2-startup-bounded-implementation-mslsp99m-l84a3h`; one diagnostic only; no acceptance or changed-candidate gate | release+testkit diagnostic under the repository GUI lock measured harness spawn→commit `588.83 ms` and app process→commit `228.02 ms`. App-relative points: application run `60 ms`, store open `66 ms`, window options `67 ms`, engine build `120 ms`, window complete `129 ms`, each of find/error/zoom/menu/geometry/downloads start and finish at `129 ms`, initial-tab open `129→202 ms` (`73 ms`), Portal `202→210 ms` (`8 ms`), PresentationCommitted `228 ms`; cold-profile restore-window-state was absent. Because each of the six allowed seams was below one-millisecond resolution, their combined cost is less than `6 ms`, far below the approximately `36 ms` required recovery. The only large measured interval, `open_initial_tab`, and Portal were outside this lane. The writer therefore made no product candidate and did not consume the changed-candidate performance run. Evidence log `/tmp/f2-diag-startup-20260809T125051Z.log`, SHA-256 `c9248864ae54036b2d1a3b04ecf779e6aa91a9f30ab17f02fc394fc925b5f54d`. All `F2DIAG` code, both `zz_diag` tests, diagnostic profiles under `/tmp/bbr-f2diag`, and the failed no-op log were removed. Lane is byte-clean at `1eb74b1…` / `0941b044…`; status-z SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`; no BBR process or GUI-lock holder remains. |
| B2 static review and remediation preflight | immutable candidate review returned two bounded findings; read-only preflight proved a deterministic forward fix | Exact reviewed input remains tree `989185c1f45da6c306fd4bf6baab2e75cb86fe30`, diff SHA-256 `6f0059eaa8ff00698f3a39d6d57648a2f169e03778e150818f7d30380a55901f`, status-z SHA-256 `d7b072ce9f0e9686501462fa1142649a389d3fa265f04a2ec36afe08a6b9e5ec`, 49 paths, real index base. `b2-static-candidate-review-msm16ivc-ir57si` found that Saved Pages docs claimed a production quit flush not wired by `quit_when_browser_window_closes`, leaving immediate normal-close durability to the detached writer; it also found stale task-packet tree/path and `§9b` metadata. `b2-review-findings-remediation-preflight-msm1owoh-zqtdug` returned `PREFLIGHT_READY`: make the existing Saved Page flush production-live beside browser-state flush and add a causal same-profile production-close restart regression without `FlushSavedPages`; update only B2-owned docs. Quarantined performance artifact/raw SHAs `f0698dd5309199cb32ac16c4b0850a82e809d59aa849cdcca7c5b58851891412` / `d93c4809d73871b54e7bf9af9e923cfdb5b907ac473c697bc7256e886c91608a` remain provisional and cannot authorize acceptance. |
| G1b independent static review | exact evidence-gated tree received three bounded findings; no product-authority widening | Authority half returned `REVIEW_PASS` and authorized settlement commit `ce4b3c236e1b58383f2ca25dadc4e69663d3cebf` / tree `ccc7f5cd0a60b1901c92e607c15f87db822ed903`. Product half returned `REVIEW_FINDINGS`: unused `AppContext` imports in `main.rs` and `harness.rs` break Clippy `-D warnings`; five pointer/keyboard collapse E2E assertions expect all stories hidden although only group 0 collapses; `dependency_contract.rs` docs describe the obsolete three-dependency/absent-framework contract. The implemented disclosure authority, real framework controls, append-only StoryIds, copy, macOS gating, dependency code, and module budgets were otherwise accepted. One Gallery-only bounded remediation writer follows; GUI evidence remains host-blocked. |
| G1a recovery settlement/review | recovery candidate `49bee8bf…` survived independent gates unchanged | first reviewer found only F1 duplicate `SpaceChipVisuals`, F2 crate-local lock, and F3 stale architecture wording; dependency closure, behavior cutover, module budgets, non-GUI gates, affected E2E, and repeated perf passes were accepted. |
| G1a remediation and serial completion | bounded writer resolved F1–F3, then Orchestrator added only the required root workspace and Justfile serial points | one canonical `bbr-ui::state::SpaceChipVisuals`; no crate-local lock; implemented architecture truth; root `Cargo.toml` includes `crates/bbr-ui`; `just verify` includes `ui-deps`; 46 script tests registered. Complete tree `8c441f2c…`, status `5c7f058f…`, mixed real index unchanged. |
| G1a focused pass/freeze | fresh reviewer returned direct `REVIEW_PASS` with every fingerprint stable and all focused gates green | Orchestrator committed the exact reviewed tree as `e6d3ab85…`; commit tree `8c441f2c…`; worktree clean. It originally waited behind successful F2 closeout and B1; the MVP-first scheduling decision below removes only the F2 performance dependency. B1 remains the serial predecessor, and G1b remains blocked until G1a is integrated. |
| Downstream read-only readiness | B1/G1a/G1b integration preflights plus B2/B3, G2/G3, Z1–Z5, and evidence-lane maps completed without mutation | durable packet-grounded dependencies are prepared, but findings derived from the unrelated dirty `bbr@0dc22fe` worktree were rejected. No readiness workflow authorizes a writer; only this committed snapshot and fresh identity/ownership audits may unlock work. |
| MVP-first scheduling decision | F2 startup closeout deferred; B1 and G1a frozen; G1b and downstream dependency-gated | Patrick explicitly moved dedicated startup optimization and full performance/package closeout behind the first feature-complete MVP without changing the `≤500 ms` target. Functional F2 evidence is green, so B1 integration is now executable; G1a follows B1, then G1b. Other items remain gated only by their product and ownership dependencies, not by the deferred startup closeout. |
| G1b live-product remediation settlement | Patrick's binding findings are implemented on an exact immutable forward state, but the writer correctly returned `BLOCKED` because minimum repository and GUI evidence is incomplete | Exact G1b state: tree `ecb84ab89d0b66f5562a3137740bfd31b97412b7`, binary diff SHA-256 `af559aa11f22eb767d24c1f51a328da9ac7571e77682c5985dbcb46f1088346e`, status-z SHA-256 `22388e8081c853eacdf952a7d84423f7b76a76a534e0657eeb4076a9473d0886`, 28 paths, real index base, diff-check green. The former missing pointer path now has stable disclosure targets and one shared `TreeState::activate` authority for pointer plus keyboard. Seven append-only real framework-control stories and their interaction E2E exist; copy and macOS gating are corrected; Status line/G2/G3 remain absent; all Gallery Rust modules remain below 500 lines. Writer `g1b-live-feedback-remediation-msm04by8-bzgeqw` is settled with no commit or ownership. Its non-GUI evidence is preserved but does not create `CANDIDATE_READY`. Patrick-owned Gallery PID `82530` and a foreign Xcode-26.6 workload still block canonical Gallery GUI evidence. Next: independent static review, remaining unchanged repository gates, then one clean locked `just e2e-gallery`; no new writer unless review or gates identify a concrete defect. B2 remains immutable in its own static review. |

## Cutover and rollback protocol

The cutover conditions were satisfied at `781e862`: complete F2a on the
verified product base, status-equivalence audit, one documentation/validator
commit, exact rollback metadata, green focused and repository gates,
committed-HEAD audit, independent fresh-operator readback, and atomic handoff.
The commit is self-resolving through the exact SHA transmitted above; it could
not contain its own SHA before creation.

For recovery, first verify the current pointer and commit object. If this
snapshot is corrupt, ambiguous, or unreachable, fail closed: stop product
writers, preserve every lane/worktree/commit, and restore execution authority
to `docs/delivery/2026-08-08-post-mvp-delivery.md@0dc22fee16d737a077aa1b097e359e2f1d08a764`.
Do not mix queue rows from both authorities or reconstruct state from stale
workflow wrappers.
