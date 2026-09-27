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
| RA-MEDIA | Media-session capability and Sidebar media control | `planned` | `main` (serial, no worktree) | none; paused 2026-09-28 | [capability contract](../roadmap/tasks/media-session-capability-contract.md) → [media control](../roadmap/tasks/sidebar-media-control.md) | Portable contract integrated at `ec5e0bad`; resumes with the macOS adapter after V2-S11 |
| V2-S0 | v2 contract and delivery control | `integrated` | `main` (serial, no worktree) | orchestrator `01a0e454` | [v2 contract](../roadmap/tasks/v2-contract.md) | Integrated as `907a668e` (`just roadmap`, `just verify`) |
| V2-S1 | System contrast and motion preferences | `integrated` | `main` (serial) | orchestrator | [system preferences](../roadmap/tasks/v2-system-prefs.md) | Integrated; verify, testkit clippy, sidebar 17/17, smoke, gallery 45/45; self-review recorded |
| V2-S2 | Palette, radius and elevation tokens | `integrated` | `main` (serial) | orchestrator | [tokens](../roadmap/tasks/v2-tokens.md) | Integrated; verify, testkit clippy, sidebar/hover/spaces/gallery + command/tab-menu/library/organize green; self-review recorded |
| V2-S3 | Rail: lift, veils, heading, footer | `integrated` | `main` (serial) | orchestrator | [rail](../roadmap/tasks/v2-rail.md) | Integrated; verify, testkit clippy, sidebar/hover/tabs/organize/dense/spaces/gallery green, visual capture read; self-review recorded |
| V2-S4 | Space paper and active chip lift | `running` | `main` (serial) | orchestrator | [space paper](../roadmap/tasks/v2-space-paper.md) | after S3 |
| V2-S5 | Floating surfaces | `planned` | `main` (serial) | orchestrator | [floating surfaces](../roadmap/tasks/v2-overlays.md) | after S4 |
| V2-S6 | Motion core | `planned` | `main` (serial) | orchestrator | [motion core](../roadmap/tasks/v2-motion-core.md) | after S1 and S5 |
| V2-S7 | After-commit moments | `planned` | `main` (serial) | orchestrator | [motion moments](../roadmap/tasks/v2-motion-moments.md) | after S6 |
| V2-S8 | Drawn empty states | `planned` | `main` (serial) | orchestrator | [empty states](../roadmap/tasks/v2-empty-states.md) | after S7 |
| V2-S9 | Phosphor-b app icon | `planned` | `main` (serial) | orchestrator | [brand icon](../roadmap/tasks/v2-brand-icon.md) | after S6 |
| V2-S10 | Curated intensity and grain | `planned` | `main` (serial) | orchestrator | [theme picker](../roadmap/tasks/v2-theme-picker.md) | after S8 |
| V2-S11 | Gallery, documentation and closeout | `planned` | `main` (serial) | orchestrator | [closeout](../roadmap/tasks/v2-closeout.md) | after S10; canonical aggregate |

The orchestrator (session `01a0e454`) is the only product writer and runs the V2 slices serially on `main`: no subagent or background workflow is authorized for this program, so the packets' parallel-capable pairs are executed in order and each slice is reviewed by a documented self-review against its packet before integration. Independent reviews use `homelab-llm/opencode-go/deepseek-v4-flash:high`: the route named in `AGENTS.md` (`homelab-llm/deepseek-v4-flash`) no longer resolves in the model registry. All historical BBR worktrees remain preserved.

## Post-F2a parallel wave

The former post-F2a and Daily-driver waves are closed historical evidence. Their lane details remain in Git and [the completed Daily-driver delivery](2026-08-12-daily-driver-delivery.md); they grant dependencies but schedule no work.

| ID | Status before F2a close | Worktree / owner | Exclusive territory | Start condition | Integration priority |
|---|---|---|---|---|---|
| LEGACY-F2A | `closed` | historical worktrees; no current owner | completed footer/appearance/swipe and dependent UI foundation | complete | none; evidence only |

## Serial integration order

1. `CONTROL-RECONCILE` is closed; its exact durable-contract commit is `d2b507f56b9da068e7df0aa08c3f8b12cdec506c`, and no writer remains active.
2. Do not infer a product lane from an idle historical worktree or session. Every new writer requires a fresh exact-base, status, process, lock, and ownership preflight.
3. `FOLDER-PIN-INVARIANT` has landed, so Folder-menu behavior may assume folders contain pinned tabs. The design-branch packets (`DESIGN-CARD` → `DESIGN-ADDRESS` → `DESIGN-FAVORITES`) are integrated.
4. V2 runs now, serially on `main`: S0 → S1 → S2 → S3 → S4 → S5 → S6 → S9 → S7 → S8 → S10 → S11. Each slice lands its DESIGN sections and §18.6 row with its implementation.
5. RA-MEDIA resumes after V2-S11 as a capability-first sequence: macOS adapter → Sidebar media control (its portable contract is integrated).
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
