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
| Target Orchestrator session | `01a062b1-9945-7d1b-aa68-02a11d6084fd`; reverify pane/session activity before any ownership transfer |
| Preserved design checkpoint | `design/browser-shell-paper@414c97c7a5139cf6070a54138969a3bb34403bc2`; design intent only, not runtime acceptance |

The UI contract conformance slice and the folder membership/pinned invariant are integrated on `main` (2026-09-27); the invariant's performance gate waits for the override to lift. The seated content card (DESIGN-CARD) is integrated the same way; the address field and space identity (DESIGN-ADDRESS) followed; the next serial slice is the Favorites grid and drag offers (DESIGN-FAVORITES). Historical lane worktrees retain evidence or abandoned experiments but own no current queue state. The stale Split worktree remains explicitly untouched.

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
| FOLDER-PIN-INVARIANT | Folder membership and pinned-state invariant | `blocked` | `main` (serial, no worktree) | none; implementation integrated | [folder membership pins tabs](../roadmap/tasks/folder-pinned-invariant.md) | Functional gates green on `main` (`just verify`, testkit Clippy, organize/dense/persistence, full `just e2e`); only `just e2e-perf` remains, blocked by the performance-execution override |
| UI-CONFORMANCE | UI contract conformance fixes | `integrated` | `main` (serial, no worktree) | none; slice complete | [UI contract conformance](../roadmap/tasks/ui-contract-conformance.md) | Integrated on `main`: `just verify`, testkit Clippy, affected suites, `e2e-peek`, `e2e-gallery` and the full `just e2e` aggregate green; orchestrator self-review (no independent subagent review without Patrick's opt-in) |
| RA-MEDIA | Media-session capability and Sidebar media control | `planned` | none | none | [capability contract](../roadmap/tasks/media-session-capability-contract.md) → [media control](../roadmap/tasks/sidebar-media-control.md) | Prioritize independently after an explicit delivery decision; capability contract first |

No product writer or background workflow is active. The Paper and Ladybird sessions are read-only and idle at their prompts; all historical BBR worktrees remain preserved.

## Post-F2a parallel wave

The former post-F2a and Daily-driver waves are closed historical evidence. Their lane details remain in Git and [the completed Daily-driver delivery](2026-08-12-daily-driver-delivery.md); they grant dependencies but schedule no work.

| ID | Status before F2a close | Worktree / owner | Exclusive territory | Start condition | Integration priority |
|---|---|---|---|---|---|
| LEGACY-F2A | `closed` | historical worktrees; no current owner | completed footer/appearance/swipe and dependent UI foundation | complete | none; evidence only |

## Serial integration order

1. `CONTROL-RECONCILE` is closed; its exact durable-contract commit is `d2b507f56b9da068e7df0aa08c3f8b12cdec506c`, and no writer remains active.
2. Do not infer a product lane from an idle historical worktree or session. Every new writer requires a fresh exact-base, status, process, lock, and ownership preflight.
3. `FOLDER-PIN-INVARIANT` has landed functionally, so Folder-menu behavior may assume folders contain pinned tabs; `UI-CONFORMANCE` precedes the design-branch packets (`DESIGN-CARD` → `DESIGN-ADDRESS` → `DESIGN-FAVORITES`).
4. RA-MEDIA remains a separate capability-first sequence and does not depend on the folder invariant.
5. Split stays blocked; no preflight, inspection, port, review, or mutation of `bbr-split-app-composition` is authorized.
6. GUI suites remain host-exclusive through the repository lock. Performance execution remains deferred until Patrick removes the active override.

## Downstream dependency queue

| ID | Status | Depends on / boundary |
|---|---|---|
| DESIGN-CARD | `blocked` | [Seated content card](../roadmap/tasks/seated-content-card.md); integrated on `main` with functional gates green; only `just e2e-perf` remains (override) |
| DESIGN-ADDRESS | `blocked` | [Address field and space identity](../roadmap/tasks/address-field-space-identity.md); integrated on `main` with functional gates green; `just e2e-perf` (override) and the Paper glyph redraw remain |
| DESIGN-FAVORITES | `ready` | [Favorites grid and drag offers](../roadmap/tasks/favorites-drag-offers.md); FOLDER-PIN-INVARIANT and DESIGN-ADDRESS integrated |
| DD-FOLDER-MENU | `blocked` | Requires the integrated shared menu seam; the delete truth is settled (Unpack) and `FOLDER-PIN-INVARIANT` is integrated |
| DD-SHIFT-LINK | `planned` | Shift-modified link behavior remains intentionally unclassified; no writer may infer foreground/window semantics |
| DD-MANUAL-SMOKE | `planned` | Requires a fresh deterministic package; representative-site smoke supplements rather than replaces automation |

## Open product and evidence work

| ID | Status | Owner / blocker | Required evidence or decision |
|---|---|---|---|
| SPLIT-COMPOSITION | `blocked` | Patrick must explicitly re-enable Split work | Fresh preflight from current `main`; stale worktree remains untouched |
| PERFORMANCE | `blocked` | Patrick's active execution override | Preserve code/budgets/baselines/sample guards; execution and acceptance remain unpassed |
| STARTUP | `planned` | Phase 9 | Last recorded `535.866375 ms > 500 ms`; hard budget unchanged |
| RELEASE | `planned` | Phase 9 | Packaging, Accessibility, canonical runner, energy/resource, polish, and release closeout |
| MVP-BOUNDARY | `blocked` | Patrick owns the final finite In/Out classification | No feature-complete First-MVP claim before explicit decision and evidence review |
| RENDERER-A5 | `blocked` | External Apple WebKit callback/error hop | A1–A4 integrated; A5 remains explicitly unpassed |

## Standing decisions and constraints

- The Reference Shell packet is complete at exact reviewed tree `5f2b8a04c0c5bbf026e891058a3fabce17cfc191`, integrated as `4d1a534300344af48d28d8cb7f435b95c0548f02`. Its recorded performance/startup/packaging/Accessibility exclusions remain open; they do not reopen the visual slice.
- Sidebar Polish is complete through `35c91189a4a895e3c7f627534acf00a56899fc7b`; its lane commits `e1081aeb`, `2ffc27fa`, and `891c720a` are all ancestors of current `main`.
- Patrick's 2026-08-14 design decision supersedes the old “separate regions” wording: folders are grouped pinned tabs in one organized area, while loose pinned rows remain above folders. Runtime currently does not enforce `folder member => pinned`; only the dedicated open packet may change that authority.
- Patrick's 2026-09-27 decisions: the design-branch drift ledger is adopted as proposed (DESIGN §17.14/§17.16; eight rows adopted, the compact media bar and non-catalog footer glyphs rejected); the media-card presentation answers live in the sidebar-media-control packet; deleting a folder is Unpack; Peek follows the shell appearance; the dark selection tint is recalibrated. The performance-execution override stays in force.
- `design/browser-shell-paper@414c97c7` preserves the wider Paper exploration. Content card, revised address field, Favorites affordances, and other unimplemented targets on that branch are not shipped claims and must receive bounded packets before implementation.
- Exactly one writer per worktree. Forward recovery only: no reset, clean, stash, amend, force-push, or silent ownership takeover.
- No second renderer or product authority in `bbr-ui`/Gallery; no private WebKit SPI; profile isolation and native WKWebView child composition remain invariant.
- Performance code remains mandatory while performance execution and canonical aggregate acceptance remain deferred/unpassed.
- No remote is configured and no push occurs as part of this generation.

## Cutover and rollback protocol

This generation replaces stale execution text that still described Sidebar Polish as awaiting integration after it had already landed. Historical detail remains available through Git and the linked ledgers, but it cannot schedule work.

Before each activation or integration, verify branch, exact base, worktree status, processes, Git locks, GUI lock, ownership, and expected shared conflicts. If documentation, worktree, and runtime ownership disagree, fail closed and preserve state before changing authority.

Rollback means returning to the named committed control pointer after preserving every newer branch and dirty worktree; it never means resetting or deleting active work. If no defensible path remains, stop with exact fingerprints, evidence, attempted paths, blocker, and the smallest required user input.
