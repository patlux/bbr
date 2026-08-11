<!-- bbr:current-delivery -->
# Current delivery

Start at [ROADMAP.md](../../ROADMAP.md), then read this file completely before choosing or scheduling work. This is the sole owner of volatile execution state. Durable acceptance lives in the linked packets. The completed Renderer/G2/G3 Sidebar+Command chronology remains reachable from Git history through control checkpoint `568cdae0008d3cc9008209007ae32442ee0ff80e`; it is evidence, not a second queue.

## Control metadata

| Field | Value |
|---|---|
| Generation | `daily-driver-2026-08-12` |
| Cutover commit | `781e862ca4547cee16be23e251767f6b2fa938fb` |
| Verified product base | `main@0dc22fee16d737a077aa1b097e359e2f1d08a764` |
| Integrated product checkpoint | `8355dcd59e0d4b8da1c0387a349a76a5c427ca02` / tree `75a2b23fbda9598529694c4a47f72de7674790bb`; G3 Sidebar and Command integrated |
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
| DD-LINK | Cmd-click/middle-click grouped background tabs | `running` | `/Users/patwoz/dev/Personal/macos/bbr-dd-background-links`; branch `parallel/dd-background-links`; rebound tree `4c067b3c…`, 47 paths | forward-recovery writer `dd-link-finalizer`; first report transport-truncated before verdict/evidence | [background link tabs](../roadmap/tasks/background-link-tabs.md) | Preserve exact rebound state, finish gates/fingerprint/report, then whole-candidate review |
| DD-GESTURE | Current-base real-app gesture revalidation | `review` | `/Users/patwoz/dev/Personal/macos/bbr-dd-gesture-revalidation`; branch `parallel/dd-gesture-revalidation`; exact tree `36dd5ab8…`, 2 paths | independent reviewer `dd-gesture-review` | [gesture revalidation](../roadmap/tasks/web-gesture-revalidation.md) | Review pump ordering fix + regression + exact red→green evidence; no mutation |
| DD-TITLEBAR | Full-height expanded Sidebar header/navigation | `running` | `/Users/patwoz/dev/Personal/macos/bbr-dd-titlebar`; branch `parallel/dd-titlebar`; rebound tree `0872e1da…`, 14 paths | forward-recovery writer `dd-titlebar-continuation`; first report ended in tool markup without verdict | [titlebar navigation](../roadmap/tasks/titlebar-navigation-controls.md) | Preserve state, complete product/app/E2E/gates or return exact platform blocker |
| DD-SPACE | Create/activate/configure Space and icon-only footer | `running` | `/Users/patwoz/dev/Personal/macos/bbr-dd-space-appearance`; branch `parallel/dd-space-appearance`; rebound tree `6746d676…`, 10 paths | forward-recovery writer `dd-space-continuation`; current UI render is non-compiling | [Space creation and appearance](../roadmap/tasks/space-creation-appearance-ui.md) | Fix preserved callback/id compile defect, then finish app/E2E/Gallery/docs/gates |
| DD-SIDEBAR | Daily-driver tab/pinned/Favorites composition and drag | `running` | `/Users/patwoz/dev/Personal/macos/bbr-dd-sidebar-main`; branch `parallel/dd-sidebar-main`; rebound tree `5b8f832b…`, 22 paths | forward-recovery writer `dd-sidebar-continuation`; prior Metal claim conflicts with known repository Xcode path | [Daily-driver Sidebar main](../roadmap/tasks/daily-driver-sidebar-main.md) | Diagnose command environment, compile through mandated inner DEVELOPER_DIR, finish behavior/evidence/report |
| DD-TAB-MENU | Production Tab/Favorite context menus | `running` | `/Users/patwoz/dev/Personal/macos/bbr-dd-tab-menu`; branch `parallel/dd-tab-menu`; rebound tree `51ea656f…`, 9 paths | forward-recovery writer `dd-tab-menu-continuation`; only bbr-ui pointer seams and portable groundwork exist | [Tab/Favorite menus](../roadmap/tasks/tab-favorite-context-menus.md) | Finish app source binding/menu lifecycle/actions/testkit/E2E/Gallery/docs/gates |
| DD-PAGE-MENU | Enriched conservative page-background menu | `running` | `/Users/patwoz/dev/Personal/macos/bbr-dd-page-menu`; branch `parallel/dd-page-menu`; rebound tree `c1b4a1c7…`, 29 paths | forward-recovery writer `dd-page-menu-continuation`; 9/11 E2E with two concrete test/setup findings | [Page-background menu](../roadmap/tasks/page-background-context-menu.md) | Fix exact two findings, run context-menu+Portal/gates/docs, return candidate fingerprint |

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
