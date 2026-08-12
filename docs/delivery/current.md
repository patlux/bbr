<!-- bbr:current-delivery -->
# Current delivery

Start at [ROADMAP.md](../../ROADMAP.md), then read this file completely before choosing or scheduling work. This is the sole owner of volatile execution state. Durable acceptance lives in the linked packets. The completed Renderer/G2/G3 Sidebar+Command chronology remains reachable from Git history through control checkpoint `568cdae0008d3cc9008209007ae32442ee0ff80e`; it is evidence, not a second queue.

## Control metadata

| Field | Value |
|---|---|
| Generation | `daily-driver-2026-08-12` |
| Cutover commit | `781e862ca4547cee16be23e251767f6b2fa938fb` |
| Verified product base | `main@0dc22fee16d737a077aa1b097e359e2f1d08a764` |
| Integrated product checkpoint | `4b3f40c5f736d3703682d001181d7c597ef317eb` / tree `874fb4d4618dee12992e5a02874ead3cd66cbd78`; grouped background links, deterministic gestures, and full-height Sidebar navigation integrated |
| Daily-driver preparation base | `568cdae0008d3cc9008209007ae32442ee0ff80e` / tree `90114c65cc068a744b43aeeec710b47fff5cbf4a` |
| Daily-driver activation base | `4d5be3116ec090aebb7eefa4861970710dba7a5b` / tree `9c8cae5cd5bca883fadc3ec198e232be4f436839`; all seven lane worktrees created clean from this exact commit |
| Daily-driver contract | [2026-08-12 Daily-driver delivery](2026-08-12-daily-driver-delivery.md) |
| Previous canonical pointer | `docs/delivery/2026-08-08-post-mvp-delivery.md` |
| Rollback pointer | `docs/delivery/2026-08-08-post-mvp-delivery.md@0dc22fee16d737a077aa1b097e359e2f1d08a764` |
| Target Orchestrator session | `019fdc57-7afb-7f81-8963-756586fe0a6b`; live pane is volatile and must be reverified at handoff |

The current branch is the integration/control authority. Only the Orchestrator creates worktrees, commits, integrates, or edits this file. Product writers may update their canonical packet and implemented-fact docs but never this runtime snapshot.

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

All runnable Daily-driver lanes use the same committed authority ancestry and isolated sibling worktrees. Implementation/non-GUI checks may overlap. GUI suites remain host-exclusive. New substantive agents use `homelab-llm/deepseek-v4-flash:high`.

| ID | Slice | Status | Worktree | Owner | Contract | Next gate |
|---|---|---|---|---|---|---|
| DD-CONTROL | Daily-driver contract and lane activation | `closed` | `/Users/patwoz/dev/Personal/macos/bbr-delivery-control-reconcile` | Orchestrator | [daily-driver delivery](2026-08-12-daily-driver-delivery.md) | Contract `4d5be311…`, roadmap/index graph, clean common base, and seven isolated worktrees are proven |
| DD-LINK | Cmd-click/middle-click grouped background tabs | `integrated` | lane commit `abae5214c04900034da139e5148d7c79693158dc`; merge `b4b2b77c9d679dce30b48e2e302e24075956a449` | Orchestrator; focused remediation `REVIEW_PASS` and post-merge gates complete | [background link tabs](../roadmap/tasks/background-link-tabs.md) | Closed for this wave; performance remains globally unexecuted/unpassed |
| DD-GESTURE | Current-base real-app gesture revalidation | `integrated` | lane commit `f2183fdb2c3f3586c500d4c67a1bdc0257e6a06d`; merge `ebb8e137aa54077c47b5ae4c5603e47e8492d9ca` | Orchestrator; independent `REVIEW_PASS` and focused post-merge gates complete | [gesture revalidation](../roadmap/tasks/web-gesture-revalidation.md) | Closed for this wave; testkit-only fix, production command routing unchanged |
| DD-TITLEBAR | Full-height expanded Sidebar header/navigation | `integrated` | lane `6327b6b48b84f847061e5d55b85d58843dd676f0`; merge `69616b88be823b8ab35d1f806ada911d6256d9e6`; gate fix `4b3f40c5f736d3703682d001181d7c597ef317eb` | Orchestrator; focused `REVIEW_PASS`, semantic Justfile union and post-merge gates complete | [titlebar navigation](../roadmap/tasks/titlebar-navigation-controls.md) | Closed for this wave; fullscreen/multi-display/manual drag and performance evidence remain honestly open |

| DD-SPACE | Create/activate/configure Space and icon-only footer | `frozen` | `/Users/patwoz/dev/Personal/macos/bbr-dd-space-appearance`; branch `parallel/dd-space-appearance`; exact tree `c8dd884c3da20d557f1b3c767b48b6220f679d34`, 68 paths | Orchestrator; remediation and independent focused `REVIEW_PASS` complete | [Space creation and appearance](../roadmap/tasks/space-creation-appearance-ui.md) | Commit exact candidate and integrate now after DD-TITLEBAR; resolve shared Sidebar/Gallery/Justfile seams semantically, then run bounded non-performance gates |
| DD-SIDEBAR | Daily-driver tab/pinned/Favorites composition and drag | `frozen` | `/Users/patwoz/dev/Personal/macos/bbr-dd-sidebar-main`; branch `parallel/dd-sidebar-main`; exact tree `2bfa4c4b…`, 29 paths | Orchestrator; independent `REVIEW_PASS` complete | [Daily-driver Sidebar main](../roadmap/tasks/daily-driver-sidebar-main.md) | Preserve exact reviewed tree; integrate immediately after DD-SPACE passes post-merge gates |
| DD-TAB-MENU | Production Tab/Favorite context menus | `frozen` | `/Users/patwoz/dev/Personal/macos/bbr-dd-tab-menu`; branch `parallel/dd-tab-menu`; exact tree `c8bcaaf00af1004e691b36776056b69a21b9fcad`, 66 paths | Orchestrator; remediation and independent focused `REVIEW_PASS` complete | [Tab/Favorite menus](../roadmap/tasks/tab-favorite-context-menus.md) | Preserve exact reviewed tree; integrate after DD-SIDEBAR per serial order |
| DD-PAGE-MENU | Enriched conservative page-background menu | `blocked` | `/Users/patwoz/dev/Personal/macos/bbr-dd-page-menu`; branch `parallel/dd-page-menu`; preserved tree `e3c41377…`, 30 paths | no writer; macOS console is locked and two foreign qemu windows remain | [Page-background menu](../roadmap/tasks/page-background-context-menu.md) | User unlocks screen; rerun only `just e2e-shell-portal` for required 7/7, then whole-candidate review |

## Post-F2a parallel wave

The former post-F2a implementation wave is closed and preserved in Git history. It grants dependencies to the current wave but has no active writer or queue authority.

| ID | Status before F2a close | Worktree / owner | Exclusive territory | Start condition | Integration priority |
|---|---|---|---|---|---|
| LEGACY-F2A | `closed` | historical worktrees; no owner | completed footer/appearance/swipe and prerequisite UI foundation | complete | none; evidence only |

## Serial integration order

1. Commit and validate this Daily-driver control cut.
2. Create every defensible isolated lane from one recorded clean authority base and start writers maximally in parallel.
3. Integrate DD-LINK first because it changes the shared new-window/controller contract.
4. DD-GESTURE closes evidence-only if green; a remediation candidate integrates after DD-LINK.
5. Integrate DD-TITLEBAR, then DD-SPACE, then DD-SIDEBAR. These lanes may implement concurrently, but their shared Sidebar composition resolves in that order.
6. Integrate DD-TAB-MENU, then DD-PAGE-MENU. Shared `bbr-ui::surfaces::menu`, Gallery, testkit, and roadmap conflicts are semantic unions.
7. Activate DD-FOLDER-MENU only after the shared menu seam and folder command/delete truth are settled by committed authority.
8. After every writer/review/remediation/integration/gate event, reread `ROADMAP.md → this file`, audit every queue row, and keep runnable lanes occupied.
9. Do not start Split or inspect its stale worktree.

Parallelism audit at activation: runnable candidates `DD-LINK`, `DD-GESTURE`,
`DD-TITLEBAR`, `DD-SPACE`, `DD-SIDEBAR`, `DD-TAB-MENU`, and `DD-PAGE-MENU`;
all seven received isolated clean worktrees from exact base `4d5be311…` and
exclusive workflow-writer labels. Deliberately deferred: `DD-FOLDER-MENU`
(shared menu seam plus delete/command truth), `DD-SHIFT-LINK` (product
semantics), Split (Patrick override), and Phase 9. Expected conflicts are the
app composition root, controller/testkit envelopes, Gallery registry, Justfile,
and roadmap roll-ups; semantic integration order is fixed above.

Writer settlement audit: workflow `bbr-daily-driver-parallel-writers-msp95k82-okl41r`
completed with `running=0`, `queued=0`, `error=0`, but wrapper completion is
non-authoritative. Independent all-files bindings from unchanged base
`4d5be311…` are: DD-LINK `4c067b3c…`/47 paths; DD-GESTURE
`36dd5ab8…`/2; DD-TITLEBAR `0872e1da…`/14; DD-SPACE `6746d676…`/10;
DD-SIDEBAR `5b8f832b…`/22; DD-TAB-MENU `51ea656f…`/9; DD-PAGE-MENU
`c1b4a1c7…`/29. Every real index is unstaged, every Git/GUI lock is free,
and no lane process survives. DD-GESTURE alone supplied a complete direct
`REMEDIATION_READY` report; six incomplete lanes retain their exact forward
state and one replacement owner each. The writer-reported DD-GESTURE path hash
is superseded by the independently computed NUL-sorted path hash
`3676bf05f9dcf7cc028c6d5ad5a756de6ffbd45caddf6f16ae626928e75f4d8a`.

Settlement-wave audit: workflow `bbr-daily-driver-settlement-wave-mspar0h3-4kvy80`
completed all seven agents with no wrapper error. Direct independent rebinding
accepts DD-GESTURE `REVIEW_PASS` at exact tree `36dd5ab8…`; DD-LINK is a
reviewable 47-path candidate at `17325002…`; DD-SIDEBAR is a reviewable 29-path
candidate at `2bfa4c4b…`. DD-TITLEBAR stopped on one warning-denied test
assignment at forward tree `94425ba3…`; DD-SPACE and DD-TAB-MENU preserve
unfinished trees `a4891930…` and `73cd9597…`. DD-PAGE-MENU fixed both causal
context-menu findings and passed 11/11, but Portal remains 6/7 because the
macOS console is currently locked (`IOConsoleLocked=Yes`) above foreign Android
qemu windows; it remains blocked rather than mislabeled ready. All seven HEADs
remain activation base `4d5be311…`, real indices have zero staged paths, no lane
process survives, and Git/GUI locks are free. Canonical fingerprints are the
Orchestrator's temp-index computations; writer hashes produced from incomplete
working diffs or alternate path serialization do not supersede them.

Review/recovery-wave audit: workflow
`bbr-daily-driver-review-recovery-wave-mspcesf6-td6z5h` completed five agents.
Independent rebinding accepts DD-SIDEBAR `REVIEW_PASS` at exact tree
`2bfa4c4b…`. DD-LINK remains technically green at `17325002…`, but review
found one actionable overclaim: its packet says Portal regressions remain green
although the locked host only proves 6/7 and the lane does not touch Portal;
remediation is documentation-only. DD-TITLEBAR is a reviewable 47-path
candidate at `cb964905…`. DD-SPACE and DD-TAB-MENU preserve unfinished trees
`de966b1e…`/65 paths and `d284788e…`/63 paths. All real indices remain
unstaged, no lane process or Git/GUI lock survives, and the macOS console still
reports `IOConsoleLocked=Yes`; DD-PAGE-MENU therefore remains externally
blocked. Orchestrator NUL-sorted hashes supersede alternate writer path hashes.

Findings-wave audit: workflow `bbr-daily-driver-findings-wave-mspe8dft-ws9shg`
completed five agents. DD-LINK passed its focused documentation recheck at
exact tree `0bd4e5da…` and enters the serial integration kernel. DD-TITLEBAR
code/evidence remain green at `cb964905…`, with one documentation-only finding:
a checked item bundles unproven fullscreen/multi-display claims. DD-SPACE and
DD-TAB-MENU are reviewable 66-path candidates at `d1f0ab82…` and `a4f1f6c8…`.
DD-GESTURE and DD-SIDEBAR remain frozen. All real indices remain unstaged, no
lane process or Git/GUI lock survives, and `IOConsoleLocked=Yes` continues to
block the Page-menu Portal closeout. The Orchestrator-computed DD-TAB-MENU
porcelain-z hash is `5bdd3c506aa4f87c10b183b0a1694be3ae041747e69c50453ab63cac73165156`;
the writer's alternate status hash is non-authoritative.

DD-LINK integration audit: exact candidate commit
`abae5214c04900034da139e5148d7c79693158dc` merged as
`b4b2b77c9d679dce30b48e2e302e24075956a449`, tree `978e6078…`.
Post-merge format, domain/web/browser/web-macos/app tests, both fresh-target
warning-denied Clippy gates, roadmap, UI-dependency and script gates are green.
Affected GUI suites are green: background links 4/4, new-window 4/4, Peek 6/6,
web gestures 9/9, tabs 1/1, profiles 1/1. Logs live under
`/tmp/bbr-dd-link-postmerge{,-gui}/` with SHA-256 manifests in the Orchestrator
session. Performance execution remains intentionally unrun. The clean merge
checkpoint promotes DD-GESTURE to the next exclusive integration position.

DD-GESTURE integration audit: exact two-file candidate commit
`f2183fdb2c3f3586c500d4c67a1bdc0257e6a06d` merged conflict-free as
`ebb8e137aa54077c47b5ae4c5603e47e8492d9ca`, tree `b82b7f44…`, on top of
DD-LINK. Format, app unit tests, a fresh warning-denied bbr all-target Clippy,
roadmap and diff checks are green; the real-app gesture suite is 10/10,
including the 16-event drain-before-command regression. Logs live under
`/tmp/bbr-dd-gesture-postmerge/` with SHA-256 values in the Orchestrator
session. No performance execution occurred. DD-TITLEBAR remains the next
serial product integration position once its focused documentation recheck
passes.

Final-review audit: workflow `bbr-daily-driver-final-reviews-mspfijvd-flnwef`
completed four agents. DD-TITLEBAR passed its focused documentation recheck at
exact tree `99351773…`; the Orchestrator's canonical NUL-sorted 47-path hash is
`5ab1350847691c8fc8bbb44849355e2368541d0f93724db008c2923018b68fcf`,
superseding the reviewer's alternate serialization. DD-SPACE review found
Gallery StoryId 46 correctly appended but eight pinned-prefix unit tests still
expect 46 stories, a stale ledger tree, and a soft 551-line module-budget
finding. DD-TAB-MENU review found two major product bugs: production submenu
geometry is never assigned, and activating a closed submenu parent dispatches
the placeholder Reload action; suspended Copy Link availability and docs also
need correction. All states remain independently rebound and unstaged with no
lane process or Git/GUI lock. `IOConsoleLocked=Yes` continues to block only the
Page-menu Portal closeout.

DD-TITLEBAR integration audit: exact candidate commit
`6327b6b48b84f847061e5d55b85d58843dd676f0` merged as
`69616b88be823b8ab35d1f806ada911d6256d9e6`; the sole conflict was the
canonical `justfile` E2E aggregate and was resolved as a semantic union that
keeps both Titlebar and background-link recipes. Fresh canonical Clippy found
two integration-only warning failures: a constant test assertion and a
release-visible testkit Unhover helper. Forward fix
`4b3f40c5f736d3703682d001181d7c597ef317eb`, tree `874fb4d4…`, makes the
density invariant runtime-evaluated and correctly feature-gates the helper.
Both canonical warning-denied Clippy gates, targeted units, roadmap/UI/script
checks and GUI suites are green: Titlebar 5/5, Sidebar 11/11, split geometry
1/1, hover/semantics 3/3 and tabs 3/3. Logs live under
`/tmp/bbr-dd-titlebar-postmerge-final/`. No performance execution occurred.
DD-SPACE remains the next serial integration position after its remediation
and focused recheck.

Product-remediation audit: workflow
`bbr-daily-driver-product-remediation-mspfxzl9-99e931` completed 4/4 with no
terminal error. The first DD-SPACE focused-recheck attempt was idle-aborted at
the configured 480000 ms boundary and grants no authority; configured
`agentRetries=1` resumed the same bounded read-only call, whose second attempt
returned a complete direct `REVIEW_PASS`. Independent Orchestrator rebinding
accepts DD-SPACE tree `c8dd884c3da20d557f1b3c767b48b6220f679d34`, binary
diff SHA-256 `61ac9c566e39be1bcc50843dd8febf4e98bf8410b5db388dd31ec2514b4b085f`,
status-z SHA-256 `965f652e9b41d4a2fa4b8ca114fc1e983104fa1cf46f49290ce81f8fda3b45dd`
and NUL-sorted 68-path SHA-256
`0facad357c91eaf16b6c39177eb779aa37ecaee0a7876828a6e20535bc7e6ba1`.
Gallery 145/145, both fresh warning-denied Clippy gates, targeted units, Spaces
20/20, Sidebar 11/11, Organize 6/6, Gallery E2E 44/44, roadmap and diff checks
are green. The soft module-budget finding is resolved by a behavior-preserving
split; one non-blocking packet count-label inversion remains documentary only.

The same workflow returned a direct independent `REVIEW_PASS` for DD-TAB-MENU.
Orchestrator rebinding accepts tree
`c8bcaaf00af1004e691b36776056b69a21b9fcad`, binary diff SHA-256
`b7dc926fe3803ea8a9c1bca5dc9379f9c34b677ebaa4f7d24fb705391d68d81d`,
status-z SHA-256
`5bdd3c506aa4f87c10b183b0a1694be3ae041747e69c50453ab63cac73165156`
and NUL-sorted 66-path SHA-256
`c936061240e3a5cfacc614649fa8fa3ab6ee3e2510d8244480d73ea4bc531a40`.
Production submenu geometry, parent activation and suspended Copy Link are
causally covered; Tab menu 12/12, Organize 6/6, Sidebar 11/11 and Gallery 42/42
are green. Both real indices remain at activation base with zero staged paths;
no lane process, Git lock or GUI-lock holder survives. DD-SPACE is promoted to
the exclusive integration position; DD-TAB-MENU remains frozen behind
DD-SIDEBAR. No performance execution occurred.

## Downstream dependency queue

| ID | Status | Depends on / boundary |
|---|---|---|
| DD-FOLDER-MENU | `blocked` | Waits for integrated shared menu source/renderer shape plus explicit folder delete/command availability truth; [Folder menu packet](../roadmap/tasks/folder-context-menu.md) remains canonical |
| DD-SHIFT-LINK | `planned` | Shift-modified link behavior remains intentionally unclassified; no writer may infer foreground/window semantics |
| DD-MANUAL-SMOKE | `planned` | Fresh deterministic package required after the bounded Daily-driver wave; manual representative-site smoke supplements automation |

## Open product and evidence work

| ID | Status | Owner / blocker | Required evidence or decision |
|---|---|---|---|
| SPLIT-COMPOSITION | `blocked` | Patrick must explicitly re-enable all Split work | No preflight, reconciliation, review, port, writer, or stale-worktree inspection |
| PERFORMANCE | `blocked` | Patrick's active execution override | Preserve code/budgets/baselines/sample guards; no recipes, samples, retries, `just verify`, or canonical `just e2e`; remains unexecuted/unpassed |
| STARTUP | `planned` | Phase 9 | `535.866375 ms > 500 ms`; hard `≤500 ms` contract unchanged |
| RELEASE | `planned` | Phase 9 | Packaging, Accessibility, canonical runner, energy/resource, polish, and release closeout remain open |
| MVP-BOUNDARY | `blocked` | Patrick owns final finite In/Out classification | No feature-complete First-MVP claim before explicit decision and truthful evidence review |
| RENDERER-A5 | `blocked` | External Apple WebKit callback/error hop | A1–A4 integrated; A5 remains explicitly unpassed |

## Standing decisions and constraints

- Daily-driver product decisions are committed in the linked delivery contract and `browser-interactions.md`: Cmd-primary and middle click create grouped background children immediately after the opener; opener remains selected/web-focused; the full BBR titlebar/navigation row disappears with the collapsed Sidebar; Space creation commits+activates before optional name/icon configuration; footer Space chips are icon-only.
- Exactly one writer per worktree. Writer → independent read-only review → concrete remediation → focused recheck → Orchestrator integration.
- New agents use `homelab-llm/deepseek-v4-flash:high`.
- Verdict authority requires a complete report bound to exact branch/base/tree/status/path fingerprints. Timeout, partial output, wrapper completion, preview, or stale run status grants no transition.
- Forward recovery only: no reset, clean, stash, wholesale restore, amend, force-push, or silent ownership takeover.
- Use porcelain-v1 `-z`, NUL-separated bytewise-sorted path lists, exact index/tree binding, free Git lock, and process/GUI-lock checks before activation/integration.
- Cargo lanes use `CARGO_BUILD_JOBS=4`, `RUSTC_WRAPPER=`, fresh separate target directories for warning-denied all-target Clippy, and repository Xcode `/Applications/Xcode-26.2.0.app/Contents/Developer` inside `nix develop`.
- GUI suites run sequentially through `/tmp/bbr-gui-gate-6ff732a0fdc0bb09.lock`; only a real `lsof` holder matters.
- Performance code remains mandatory, but performance execution and canonical aggregate E2E remain forbidden/unpassed under the active override.
- No second renderer or product authority in `bbr-ui`/Gallery; no private WebKit SPI; profile isolation and native WKWebView child composition remain invariant.
- StoryIds 0–45 remain append-only; new production Gallery stories append from 46 only after the owning product surface exists.
- Split remains scheduling-deferred even though it is the next G3 composition in its durable packet.

## Cutover and rollback protocol

This committed snapshot replaces the prior natural-handoff queue as the only execution authority. The old state remains recoverable through `568cdae0008d3cc9008209007ae32442ee0ff80e` and the preserved historical ledger, but neither may schedule work.

Before each lane activation or integration, verify branch, exact base, status-z, index tree, processes, Git locks, GUI lock, ownership, and shared conflicts. If this document, the worktree, or runtime ownership disagree, fail closed, preserve all state, and update committed authority before proceeding.

Rollback means returning to a named committed control checkpoint after preserving every lane's evidence and ownership state; it never means resetting or deleting active work. If no defensible path remains, stop with evidence, attempted paths, preserved state, blocker, and the smallest input required.
