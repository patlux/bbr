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

F2a and its documentary closeout are integrated. The first post-cutover queue
audit must activate every defensible ready lane below from the exact same
verified base; until the atomic cutover and fresh-operator acknowledgement
complete, none has product authority.

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
| SHUTDOWN | `integrated` | `/Users/patwoz/dev/Personal/macos/bbr-shutdown-lifecycle`; branch `parallel/shutdown-lifecycle`; no writer; exact reviewed lane commit `48f2841b9db46f19ebbd8f8b9b3637496f07e37d` | app lifecycle, real SIGINT and owning-window close, portal-child isolation, testkit response flush, fresh `e2e_shutdown`, required E2E registry | merged on clean control trunk as `12431c95e76fb4ca53cc1e0102a8d102b21809f8`; post-merge verify, SHUTDOWN 3/3, serial perf, and serial package green. F2 closeout remains open at loginwindow-blocked portal minimize and reproducible truncated-response error fixtures | **1 — complete** |
| B1 | `frozen` | `/Users/patwoz/dev/Personal/macos/bbr-saved-pages-b1`; branch `parallel/saved-pages-b1`; no writer; focused recheck returned direct `REVIEW_PASS`; exact reviewed lane commit `a23d9335240ff4c6cbc0d5d8b80744339f4a335e` | focused portable domain/persistence Saved Pages modules, canonical URL equality, CRUD/search/list, schema v3 bump+discard, fixtures/benchmarks | commit tree exactly `0c728df48ecf495375b6b4ad096dce35d41cfbbf`; worktree clean; F1–F5 and unchanged B1 contracts passed independent recheck; waits for F2 closeout before serial integration | after F2 closeout, normally before G1a |
| G1a | `review` | `/Users/patwoz/dev/Personal/macos/bbr-ui-g1a-foundations`; branch `parallel/ui-g1a-foundations`; recovery completed/released ownership; independent read-only review to be dispatched | extraction of integrated production Foundations/Primitives authority into `bbr-ui`; no copies; exact direct dependencies `gpui` + `bbr-domain`; approved transitivity only `bbr-domain → bbr-web`, never direct `bbr-ui → bbr-web` | settled full tree `49bee8bfeab0392cda793c3f7a8870505a7b369e`, raw status `faf65683…`, mixed real index `66eb5e8c…`; app cutover and writer gates reported complete; exact dependency, source-authority, scope, index, E2E, perf, and serial-point review required | after F2 closeout, normally after B1 |
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
5. Run the canonical F2 closeout: `just verify` → `just e2e` →
   `just e2e-perf` → `package-verify`; record Bundle/Binary SHA and clean up
   process/GUI locks.
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
| PORTAL-MINIMIZE | `blocked` | rerun the real WindowServer hit-test after the display is unlocked |
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
| F2 closeout blockers | canonical closeout rerun on exact integration HEAD | `PORTAL-MINIMIZE`: fullscreen `loginwindow` window 42993, layer 2004, blocks AppKit hit-test. `e2e-shell-errors`: truncated 500 and flaky-first truncated response reproducibly complete without `NavigationFailed`; valid isolated retries remain red. No assertion/deadline/budget weakening. One bounded fixture/evidence recovery lane is ready from `12431c95…`. |
| G1a recovery settlement/review | recovery writer completed the app cutover and released ownership; no process owns the mixed-index worktree | full candidate `49bee8bf…`, status `faf65683…`, real index `66eb5e8c…`. Independent reviewer must distinguish source imports from build artifacts, audit the root lockfile serial-point mutation, and require direct gate evidence before freeze. |
| Deliberate deferrals | G1b, B2, B3, G2, G3, Z1–Z5, ENGINE-SPIKE, blocked evidence rows | exact dependencies remain those in the tables above; no item is deferred merely for conflict probability |

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
