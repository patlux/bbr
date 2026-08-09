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
recoveries are integrated on the current control checkpoint. B1 and G1a are
reviewed and frozen, but remain intentionally unintegrated until the complete
F2 closeout passes.

| ID | Slice | Status | Worktree | Owner | Contract | Next gate |
|---|---|---|---|---|---|---|
| CUTOVER | Delivery-Control authority cutover | `closed` | `/Users/patwoz/dev/Personal/macos/bbr-delivery-control-reconcile` | Orchestrator only; no second writer | [completion audit](delivery-control-completion-audit.md) | complete at `781e862`; committed-HEAD audit and independent readback passed |

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
| SHUTDOWN | `integrated` | `/Users/patwoz/dev/Personal/macos/bbr-shutdown-lifecycle`; branch `parallel/shutdown-lifecycle`; no writer; exact reviewed lane commit `48f2841b9db46f19ebbd8f8b9b3637496f07e37d` | app lifecycle, real SIGINT and owning-window close, portal-child isolation, testkit response flush, fresh `e2e_shutdown`, required E2E registry | merged as `12431c95…`; deterministic error-fixture recovery merged as `1e42d835…`; deferred Portal visibility recovery merged at control checkpoint `6ff70eab…`. `just verify` and all 27 functional E2E suites are green, including Portal 7/7 and errors 10/10; no `RefCell already borrowed`. F2 remains open only because canonical performance/package closeout evidence is not yet complete | **1 — product integrated; closeout blocked** |
| B1 | `frozen` | `/Users/patwoz/dev/Personal/macos/bbr-saved-pages-b1`; branch `parallel/saved-pages-b1`; no writer; focused recheck returned direct `REVIEW_PASS`; exact reviewed lane commit `a23d9335240ff4c6cbc0d5d8b80744339f4a335e` | focused portable domain/persistence Saved Pages modules, canonical URL equality, CRUD/search/list, schema v3 bump+discard, fixtures/benchmarks | commit tree exactly `0c728df48ecf495375b6b4ad096dce35d41cfbbf`; worktree clean; F1–F5 and unchanged B1 contracts passed independent recheck; still waits for an actually successful F2 closeout, currently blocked by missing idle-host performance/package evidence | after F2 closeout, normally before G1a |
| G1a | `frozen` | `/Users/patwoz/dev/Personal/macos/bbr-ui-g1a-foundations`; branch `parallel/ui-g1a-foundations`; no writer; focused recheck returned direct `REVIEW_PASS`; exact reviewed commit `e6d3ab85a49b8754764553b31d0eac996015ce25` | extraction of integrated production Foundations/Primitives authority into `bbr-ui`; no copies; exact direct dependencies `gpui` + `bbr-domain`; approved transitivity only `bbr-domain → bbr-web`, never direct `bbr-ui → bbr-web` | commit tree exactly `8c441f2ccee86792b4475a314b472299717d11fb`; F1–F3, workspace membership, `ui-deps`, 46 script tests, dependency closure, affected E2E, module budgets, and architecture truth passed; worktree clean. Integration waits for successful F2 closeout and B1 | after F2 closeout and B1 |
| G1b | `planned` | no worktree or writer until G1a settles | dev-only Gallery MVP consuming G1a's exact production symbols; no WebKit/network/testkit listener | G1a settled; Gallery territory writer-free | after G1a |

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
5. Complete the canonical F2 closeout. First prove an idle host with no Tart,
   Tart-runner SSH, or competing BBR build/test/package process and two
   observations at least 30 seconds apart. Then run the single controlled
   repository-default-Xcode performance reproduction. Only if it is green,
   restart the complete sequence from `just verify` → `just e2e` →
   `just e2e-perf` → `package-verify`; record Bundle/Binary SHA and clean up
   process/GUI locks. No blind retry or acceptance change.
6. Integrate B1, G1a, and G1b serially according to the actual reviewed trees;
   default order B1 → G1a → G1b unless concrete dependency evidence requires
   another semantic integration order.
7. Keep G2, G3, B2, B3, and Z packages dependency- and ownership-gated; at
   every scheduling checkpoint refill every safely runnable bounded lane.

## Downstream dependency queue

| ID | Status | Depends on / boundary |
|---|---|---|
| B2 | `planned` | integrated B1; free command-provider territory; consumes one shared save/remove/open authority |
| B3 | `planned` | integrated B1 + B2 + G1a; ordinary GPUI Saved Pages Library, no permanent sidebar region |
| G2 | `planned` | integrated G1a + G1b and each corresponding production component; stories never precede production |
| G3 | `planned` | integrated G2 and corresponding production compositions |
| Z1 | `planned` | integrated F2a + G1a; Bookmark row additionally waits for B2 |
| Z2 | `planned` | integrated F2a + G1a; flat session-local Folder authority only |
| Z3 | `planned` | successful F2 closeout and free Portal/context-menu territory; Bookmark row additionally waits for B2 |
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
| F2-CLOSEOUT | `blocked` | The first canonical run after Portal recovery had all 27 functional E2E suites green but invalid performance evidence: startup p95 `597.81 ms > 500 ms`, command first-result max `185.56 ms > 100 ms`, `canonical: false`, concurrent Tart verification, and an Xcode 26.6 override instead of the repository default 26.2. Recovery V2 waited from `2026-08-09T09:24:51Z` through `09:54:51Z`, but Tart/native then Tart/functional runner plus SSH remained active; no idle pair was proven and **no controlled reproduction or later gate was executed**. The single reproduction remains unused. Retry only after the external verification fully ends, then record two idle observations ≥30 seconds apart and run exactly one `CARGO_BUILD_JOBS=4 nix develop -c just e2e-perf` with no Xcode override. |
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
- One writer per worktree; one product authority per concern; only the
  Orchestrator commits, merges, and integrates.
- Never weaken tests, validators, budgets, sample counts, deadlines, or
  acceptance criteria.
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
| F2 idle-host recovery V2 | exact clean HEAD/tree `6ff70eab…` / `55f6f10e…`; bounded read-only operator | external Tart native/functional runs and runner SSH occupied the full 30-minute window. GUI lock stayed free and no competing BBR process remained, but no idle pair was possible. No performance, E2E, package, build, or GUI gate ran; the single controlled reproduction remains unused. F2 is blocked on a future genuinely idle host, not on product Portal behavior. |
| G1a recovery settlement/review | recovery candidate `49bee8bf…` survived independent gates unchanged | first reviewer found only F1 duplicate `SpaceChipVisuals`, F2 crate-local lock, and F3 stale architecture wording; dependency closure, behavior cutover, module budgets, non-GUI gates, affected E2E, and repeated perf passes were accepted. |
| G1a remediation and serial completion | bounded writer resolved F1–F3, then Orchestrator added only the required root workspace and Justfile serial points | one canonical `bbr-ui::state::SpaceChipVisuals`; no crate-local lock; implemented architecture truth; root `Cargo.toml` includes `crates/bbr-ui`; `just verify` includes `ui-deps`; 46 script tests registered. Complete tree `8c441f2c…`, status `5c7f058f…`, mixed real index unchanged. |
| G1a focused pass/freeze | fresh reviewer returned direct `REVIEW_PASS` with every fingerprint stable and all focused gates green | Orchestrator committed the exact reviewed tree as `e6d3ab85…`; commit tree `8c441f2c…`; worktree clean. No control-trunk integration occurred because successful F2 closeout and then B1 remain serial predecessors. G1b remains blocked until G1a is integrated. |
| Downstream read-only readiness | B1/G1a/G1b integration preflights plus B2/B3, G2/G3, Z1–Z5, and evidence-lane maps completed without mutation | durable packet-grounded dependencies are prepared, but findings derived from the unrelated dirty `bbr@0dc22fe` worktree were rejected. No readiness workflow authorizes a writer; only this committed snapshot and fresh identity/ownership audits may unlock work. |
| Deliberate deferrals | G1b, B1 integration, G1a integration, B2, B3, G2, G3, Z1–Z5, ENGINE-SPIKE, blocked evidence rows | B1 and G1a are technically frozen but deliberately remain dependency-gated by the explicit successful-F2-closeout and B1→G1a order; no item is deferred merely for conflict probability |

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
