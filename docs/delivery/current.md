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
| Current integrated control checkpoint | `05765724e8ca5516e5cb060a77af0dea2c887bed` (tree `5ae07734fbeb043e6388e513c598ef787b792a4e`); includes B2, Z2-1, G2 Status Line, and the reviewed productive NW-SEAM after the earlier SHUTDOWN/F2 recovery chain |
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
B1 and G1a are integrated and independently verified. Z2-1's reviewed
FolderId-keying candidate is integrated as exact semantic merge `5e727d39…` /
tree `9748d5ba…`; all required non-performance and affected functional gates
passed while performance execution remained Patrick-deferred and unpassed.
G1b Gallery MVP and B2 Saved Page command/provider started from one exact common base in isolated
worktrees with disjoint primary territories. Both earlier bounded remediation writers settled on immutable evidence
candidates. G1b's first clean locked Gallery E2E exposed two concrete failures;
its bounded remediation then settled at exact candidate tree `368b07d7…` and
received independent `REVIEW_PASS`. One clean host-exclusive Xcode-26.2
acceptance run passed all 14 Gallery tests on that unchanged candidate. The
Orchestrator committed the exact candidate as `01b563b…` and merged it at
`e067c43…`. Full post-merge `just verify` is green on the unchanged merge tree
after removing proven cross-worktree compiler-cache artifacts. After two clean
host samples, the required post-merge Xcode-26.2 Gallery gate passed 14/14 under
the repository GUI lock on clean committed control `6829930…` / tree
`e1c1aa8…`; G1b is integrated. B2's focused product review and
packet-only recheck both passed; it still waits only for clean serial
GUI/performance evidence. Z2-1's reviewed advisory preflight proves a bounded
behavior-preserving FolderId-keying lane is dependency-ready once its clean
worktree/base/owner are committed. Z3's reviewed advisory reconciliation fixes
the durable activation direction already chosen by the committed MVP-first
decision; no product writer is authorized by that doc-only correction.
Earlier non-GUI gates remain green in both product candidates. The productive
NW-SEAM is now integrated from exact reviewed lane commit `3bc016b…` / tree
`bc6427e…` by merge `05765724…` / tree `5ae07734…`. Post-merge verification
passed `just verify`, explicit testkit Clippy, roadmap/diff checks, NW-SEAM 4/4,
Tabs 1/1, and Profiles 1/1. The delegate still returns `None`; portable policy
opens only direct user activations as regular tabs and reports script blocks;
the controller remains sole session authority with fail-closed source/profile
validation. Performance is unexecuted/unpassed under Patrick's override.

The complete DDA inventory was re-read after this Phase-8 checkpoint. N1 and
PEEK are the two next packet-owned first-MVP outcomes. Foreground/background
modifier opening remains `decision needed` and is not silently absorbed by
either writer. N1 and PEEK have satisfied their NW-SEAM dependency but remain
writer-free until the next committed activation checkpoint records clean
worktrees, exact base, exclusive territory, shared integration ownership, and
merge order.

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
| G1b | `integrated` | reviewed lane commit `01b563b528f6cad61c8ce693617e1a16a0b796a1` / exact tree `368b07d7479b48039f509a1fc3121d03c2f018f9`; merged by Orchestrator as `e067c43f8ddc151d2866e38ddc17a948ddbb4016` / tree `cfd867904ea92e3a9d783a7d85a8dea561c3b203`, parents `1d916108e6e867bbaa0364d5e5870055dd1854fc` + `01b563b…`; no writer | Candidate writer returned direct `CANDIDATE_READY`, independent exact-candidate review returned `REVIEW_PASS`, pre-merge Xcode-26.2 Gallery acceptance passed 14/14, and the conflict-free merge preserved the exact candidate tree as second parent. Post-merge complete `just verify` passed on the unchanged tree after derived-target cleanup and `RUSTC_WRAPPER=`; log `/tmp/bbr-g1b-integration-verify-clean-nocache-20260809T2018Z.log`, SHA-256 `6b6e87830c8f546d06cf031e134c85a2ac36a3bfae90ac4ac482fc4fb655f05b` | Final post-merge gate passed after two clean host samples on committed control `68299303ea10fcc4ddd49124c1b0605824974f8f` / tree `e1c1aa85ed708b4a8a4b3fd66956018eafd1ed22`: repository recipe set Xcode 26.2, acquired `/tmp/bbr-gui-gate-6ff732a0fdc0bb09.lock`, and passed 14/14, exit 0. Log `/tmp/bbr-g1b-integration-e2e-gallery-post-authority-20260809T2040Z.log`, SHA-256 `1dc961945b94399abeabdfd6416ab0401fc6ef90d0669e2afcab654327369262`, 53 lines / 3685 bytes. HEAD/tree/status stayed exact and no process/lock remained. B2 serial evidence, Z2-1, and G2 Status-Line scheduling may now proceed under their own ownership/host gates |

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
8. Keep G2, G3, B3, and Z packages dependency- and ownership-gated; B2 is
   frozen on its reviewed tree pending serial evidence and integration. At
   every scheduling checkpoint refill every safely runnable bounded lane.

## Downstream dependency queue

| ID | Status | Depends on / boundary |
|---|---|---|
| B2 | `integrated` | exact lane commit `f53aebe053cf2cc9a5c574cadff597764eb0e27c` / tree `ec2438c48d98e65319925774e5355e29a7d883bf`; semantic merge commit `c8604e0d6892d3f2e054e1f1bfed249cfd43e3cb` / tree `403e9740a2bfb7af5ff63fa3cb4abc1e9c396d18`, parents `c4be393d5e1f052e94edc2b2a65bedc1f8918397` + exact lane commit. No writer. Product/packet/evidence and exact merge-index reviews returned direct `REVIEW_PASS`. Functional candidate and merge evidence both passed Saved Pages 4/4, Shutdown 3/3, and Command 25/25; merge log `/tmp/bbr-b2-merge-functional-20260809T2155Z.log`, SHA-256 `9d20af640e46a8a655aae626f705581bec128ef111c627cd68c4992b605942c2`. Merge non-performance `just verify` passed in isolated target; corrected inner-Xcode-26.2 testkit Clippy/roadmap/diff passed, SHA-256 `2ba3ef0f37ce37ec2d17cb1d5b2be554a9a3647914a9b7cc794ce8f129570d25`. `justfile` is exact union blob `258d18a0878ac2124b8c8387cff415942cf4eece`: Gallery recipes retained outside canonical app E2E, Saved Pages joins it. Phase 8/N1/PEEK/DDA authority preserved. Performance tests/contracts/budgets/baselines/sample checks remain unchanged; current integration performance was not executed and remains Patrick-deferred, never passed. B3 and Z1/Z3 Saved-Page consumers may now satisfy only their B2 dependency; all other gates remain. |
| B3 | `planned` | integrated B1 + B2 + G1a; ordinary GPUI Saved Pages Library, no permanent sidebar region |
| NW-SEAM | `integrated` | Canonical packet: [productive new-window request seam](../roadmap/tasks/new-window-production-seam.md). Exact reviewed lane commit `3bc016b075df917af0c57f46f5cbf5d6e2f33509` / tree `bc6427e33a7cf12538ad3ec19e374c4d43dc6232`; independent exact-candidate verdict `REVIEW_PASS`; semantic merge `05765724e8ca5516e5cb060a77af0dea2c887bed` / tree `5ae07734fbeb043e6388e513c598ef787b792a4e`, parents `55b4ebe50673d51421b3035e6ab4f52ea09a545a` + exact lane commit. Post-merge non-performance log `/tmp/bbr-nw-seam-merge-nonperf-20260810T0110Z.log`, SHA-256 `d33b5638a7a05e3a555a20e7498bee3fa25748c357f349a16b8fed13a44af961`; functional logs passed NW-SEAM 4/4 (`33823feb…`), Tabs 1/1 (`0a20c8be…`), Profiles 1/1 (`80ecd941…`). User activation opens exactly one regular tab in the source Space/Profile; script request blocks/reports; inactive source Space stays inactive; stale/mismatched source fails without mutation; native and typed paths share `BbrApp::on_new_window_request`; delegate always returns `None`. No writer. Performance remains preserved, unexecuted, and unpassed. |
| N1 | `planned` | [Web-content Back/Forward gestures and child-tab opener return](../roadmap/tasks/web-content-navigation-gestures.md). Integrated NW-SEAM dependency is satisfied. Owns regular-tab opener provenance, exact-pane WKWebView history gestures, and atomic Back-at-settled-root close/activate-opener behavior; ordinary Cmd-W neighbor selection stays unchanged. It owns no disposition, background/insertion, or Peek policy. Next gate: Orchestrator creates a clean isolated worktree from the next committed control checkpoint and records the exclusive modules plus its shared-facade integration order before one writer starts. |
| PEEK | `planned` | [Link Preview / Peek](../roadmap/tasks/link-preview-peek.md). Integrated NW-SEAM dependency is satisfied. Owns one source-profile-isolated transient preview per pane, warm blocked origin, exact dismissal focus restoration, and same-WebView/session promotion without reload/state loss. It owns no regular-tab opener-return or duplicate disposition policy. Next gate: clean isolated worktree plus a committed ownership split proving no concurrent N1 writer owns transient-session adoption, native z-order/Portal, or Peek UI. |
| DDA | `planned` | [Daily-driver capability inventory](../product/daily-driver-capability-inventory.md) was fully re-audited after NW-SEAM integration. N1 and PEEK remain the next two concrete packet-owned MVP gaps. Foreground/background modifier opening remains `decision needed`; no unbounded Phase-8 writer or new packet is authorized at this checkpoint. Re-audit after the next Phase-8 integration. |
| G2 | `integrated` | First bounded Status-Line cut: exact reviewed lane commit `76dc9b50216a512449f42ab482d140af806e23b8` / tree `6a61e7a7529780d15e4bd455e37b0f4fb251166e`; conflict-free semantic merge `fe81721083bc96d9a9d9efff26564e137c023177` / tree `d4118dba3978d44e19f51dc8144eecb0a0858a49`, parents `647519df9725566812f9aeb2128722a11e16e396` + exact lane commit. Product review accepted extraction/identity/StoryId/tests/deps/module budgets; bounded remediation changed only three stale comments and focused recheck returned direct `REVIEW_PASS`. The app retains `status_model`; production anatomy is `bbr-ui::components::status`; Gallery appends the Browser Components Status-Line StoryId without renumbering G1b. `just verify`, explicit Clippy for bbr testkit/bbr-ui/bbr-gallery, roadmap, and diff-check passed. Functional Xcode-26.2 GUI-locked evidence passed Sidebar 7/7 and Gallery 14/14. Logs: `/tmp/bbr-g2-status-merge-nonperf-verify-20260809T2225Z.log`, SHA-256 `9247a01ec3d197758c3a8556f0de7e60a4445b6e84535511947844767bf4a64e`; `/tmp/bbr-g2-status-merge-functional-20260809T2232Z.log`, SHA-256 `e9cf41304cf59b44f9ce7b5324b101e957b8aee99efeeb2289c3a32a07795dc6`. Performance recipes did not run; Gallery report-only sanity artifacts are non-acceptance output and no performance pass is claimed. Remaining G2 groups stay packet-open and separately scheduled. |
| G3 | `planned` | Start immediately after integrated G2. First serial composition wave: Sidebar, Command Palette/Overlay, Downloads/Find/Error/Auth/Menu groups, Split. Saved Pages remain append-only after integrated B2/B3. |
| Z1 — Tab/Favorite context menus | `planned` | [Tab and Favorite context menus](../roadmap/tasks/tab-favorite-context-menus.md); integrated F2a + G1a, integrated G2 menu presentation, and exact `TabId` command facade; Bookmark row additionally waits for integrated B2 and consumes only `SavedPageCommand`. |
| Z2 — Folder context menu | `integrated` | [Folder context menu](../roadmap/tasks/folder-context-menu.md), bounded behavior-preserving Z2-1 only. Exact reviewed lane commit `211e17934512ea221bd28e6d6876e30415d66915` / tree `98f187c38f2ebd395f0ff190a570d145d3d4240d`; semantic merge `5e727d39bb635bab8776ccd28fc203298c3faa48` / tree `9748d5ba3fc708000649f74ed2107d31fe5b4fc9`, parents `e101bbbbae0a26a3d1223c2fc4fa11157924e810` + exact lane commit. The automatic union preserved B2 on the two shared testkit paths and retained all 19 Z2 paths. `just verify`, explicit testkit Clippy, roadmap, and diff-check passed; functional Xcode-26.2 GUI-locked evidence passed Sidebar 7/7, Organize 2/2, and Drag-Chip 4/4. Logs: `/tmp/bbr-z2-merge-nonperf-verify-20260809T2205Z.log`, SHA-256 `47525c3fae619677543a52c917ee14bd098aecd6229f2d2f67768327b6d50fa9`; `/tmp/bbr-z2-merge-functional-20260809T2210Z.log`, SHA-256 `20182a682cf8a6d1cd2c6cfb93b77ca7845febb3e5756781a76a80d9968b53b4`. FolderId now keys state/commands while position remains presentation/assertion metadata. No Delete/Z2-2/menu/icon/Move-to-Space/DESIGN-ID change. Performance tests/code/contracts are unchanged; no performance recipe ran and no performance pass is claimed. |
| Z3 — Page-background context menu | `planned` | [Page-background context-menu follow-up](../roadmap/tasks/page-background-context-menu.md) over the integrated [BBR/WebKit boundary](../roadmap/tasks/bbr-context-menus-over-web.md); integrated functionally green F2 checkpoint and free Portal/context-menu territory; Bookmark row additionally waits for B2. Durable activation wording is reconciled to the committed MVP-first direction; startup/release closeout remains red Phase-9 debt, never passed. Reviewed advisory `/tmp/bbr-z3-activation-reconciliation-handoff.md`, SHA-256 `c30a1995abd41ac64a0f2ea7c0efa83ec3797c5c08918b277ef8d00102cc2544`, received direct `REVIEW_PASS`. No Z3 writer before integrated B2 where Bookmark is included, integrated G2 menu presentation, free Portal/context-menu territory, and a fresh post-cut ownership audit. |
| Z4 — Unified titlebar navigation controls | `planned` | [Titlebar navigation controls](../roadmap/tasks/titlebar-navigation-controls.md); integrated F2a + G1a and one shared history authority with N1. No second Back/Forward truth. |
| Z5 — Sidebar media control | `planned` | [Sidebar media control](../roadmap/tasks/sidebar-media-control.md); integrated F2a + G1a and the public capability-first [media-session contract](../roadmap/tasks/media-session-capability-contract.md). |
| S1 — Global UI scale/density | `integrated` | [Global UI scale and density pass](../roadmap/tasks/global-ui-scale-density.md) is already in the control ancestry (`97bd541…`; product integration recorded as `bf54450` in the packet/track). Automated shell, Portal, dense, performance, canonical E2E, and package evidence are recorded. Only Patrick's manual packaged-app readability smoke remains external/deferred; S1 is not a new writer lane and must not be reimplemented. |
| WKEXT | `review` | Preserved branch `parallel/wkwebextension-viability-packet` at exact commit `1d3f0fc5f839b01b8e5325645924e95a5dd0a123` / tree `94457df516a067c5a4f0d2a4b47125d16fc2c5a0`, 12 commits unique to control from merge-base `142a4d803460a162dd8e600d693f0c53ea5fa6ac`. It contains a real-WKWebView viability spike, fixture/E2E, remediations, research, and canonical task packet. Next action is read-only semantic reconciliation against current WebKit/app/testkit/Justfile state; no writer or integration before direct review. Even if accepted, this proves viability only and does not promote Extensions beyond explicit post-alpha scope. |
| ENGINE-SPIKE | `review` | Preserved CEF research branch `parallel/engine-adapter-spike` at exact commit `0f4bd35c3db0a3393788393a5c9aab7d3529c8f7` / tree `0a1bb58bb027c6f8eaa8c6d56dad17e11bbe6b20`, 7 commits unique to control from merge-base `e097e2750692d0451fbad1ded6628b67e8305633`. It contains the implemented/verified `bbr-web-cef` research adapter and branch-local `docs/roadmap/tasks/engine-adapter-spike.md` packet. Next action is read-only reconciliation/archival decision; retain no product, default-engine, cross-platform, or release claim unless a later canonical product packet explicitly accepts one. |
| TART-RUNNER | `review` | Preserved branch `parallel/tart-e2e-runner` at exact commit `63e3d7c3eb6db8e79d18b0282816159cd8b7c41b` / tree `dd3597ffa32ad7825d11c2f69b734023657951df`, 12 commits unique to control from merge-base `47e9d9df4767d2568470bde9a898f2f447a10931`. It contains runner scripts, tests, task packet, partial-failure preservation, and Tart VM evidence. Next action is read-only reconciliation against current E2E/Justfile/performance-report code and an explicit canonical-runner decision. It does not by itself satisfy PERF-RUNNER, publish an accepted baseline, or authorize infrastructure integration. |
| SPLIT-COMPOSITION | `blocked` | Preserved dirty stale-base worktree `/Users/patwoz/dev/Personal/macos/bbr-split-app-composition`, branch `parallel/split-app-composition`, HEAD `af8983261245ae2dd9076a2314e360ce90fbec8b`, HEAD/index tree `6ac3ec08bd6c85cadb6d1763619f3b2338ea3bbf`, status SHA-256 `16fbf0d957d532399f1fbc9b862005d886e05ca77603f7fca302047550243403`. It has 27 tracked modifications (`+1265/-309`) plus 18 untracked split/performance/testkit/E2E modules. It is a preservation/review candidate only, not a resumable writer lane. Before reuse, an independent settlement must fingerprint the full temporary-index tree including untracked files, classify each behavior against current Phase-6/G3 architecture, and choose bounded forward ports or explicit obsolescence. |
| ICON | `integrated` | reviewed icon commits are included in F2a integration `7180128`; obsolete Spaces glyph authority remains absent |

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
| PHASE-8 | `planned` | roll-up only, never an executable writer. Concrete bounded outcomes are classified in the Phase-8 table below; N1/PEEK/DDA cover only their named navigation/preview/inventory scope. |
| PHASE-9 | `planned` | accessibility, visual/release hardening, signing/notarization and clean-machine acceptance |
| PHASE-10 | `planned` | Windows and Linux adapter spikes and implementations |

## Phase-8 bounded outcome register

Phase 8 is a product roll-up, not one executable lane. Every open outcome below
needs its own bounded packet or an explicit existing packet before a writer can
start. NW-SEAM, N1, PEEK, and DDA do not absorb these territories.

| ID | Status | Canonical scope / next condition |
|---|---|---|
| P8-PERMISSIONS | `planned` | Native camera/microphone/location/notification/clipboard adapter and UI consuming the completed [portable permissions contract](../roadmap/tasks/permissions-contract.md); requires a bounded adapter/UI packet and fresh WebKit/app/testkit ownership audit. |
| P8-CLIENT-CERT | `planned` | Supported native client-certificate enumeration and selection consuming the [authentication-challenges contract](../roadmap/tasks/auth-challenges-contract.md); secret redaction and portable identity remain authoritative. Requires a dedicated bounded implementation packet. |
| P8-FILE-PANELS | `planned` | Native AppKit/WebKit upload/save panels, sandbox access, cancellation, and real-app E2E consuming the [file-panel contract](../roadmap/tasks/file-panels-contract.md); downloads' settled silent suggested destination is unchanged. |
| P8-HISTORY-DATA | `planned` | Profile History UI plus per-profile clear-data controls. No canonical bounded implementation packet exists yet; DDA must split persistence/storage, command/search, UI, destructive confirmation, and real-app/performance evidence before any writer. |
| P8-SERVICES-LIFECYCLE-UPDATES | `planned` | Native menus/Services, broader multi-window/application lifecycle, and app updates. Aggregate wording only; DDA must create separate bounded packets and must not reopen completed deterministic shutdown semantics. |
| P8-SITE-SETTINGS | `planned` | Complete profile-isolated site-key/settings persistence and UI around the integrated zoom command/native wiring, under the [site-settings/zoom contract](../roadmap/tasks/site-settings-zoom-contract.md). Requires a fresh bounded integration/UI packet. |
| P8-SITE-MATRIX | `blocked` | Manual representative-site matrix for login, media, editors, conferencing, upload/download, payments, and passkeys; requires a stable integrated build and an explicit matrix/evidence surface. |
| P8-STATE-HARDENING | `planned` | Empty/loading/error/cancellation/destructive-state hardening across integrated features. DDA must turn concrete audited gaps into feature-owned bounded packets; no cross-product cleanup writer. |
| P8-PORTAL-HARDENING | `planned` | One-per-window focus, multi-window ownership, display geometry, teardown, and related native lifecycle through [native Portal hardening](../roadmap/tasks/native-portal-hardening.md). Existing minimize/restore functional evidence is preserved; the known `loginwindow` layer-2004 host condition remains an external manual-evidence blocker, not a reason to duplicate Portal authority. |
| PROFILE-FAVORITE-RUNTIME | `planned` | Optional [profile-scoped singleton favorite runtime](../roadmap/tasks/profile-scoped-favorite-runtime.md) remains explicitly deferred. Current concrete `TabId`/owning-space behavior stays authoritative; do not promote it from DDA or menu work. |

## Preserved stale-worktree settlement register

Dirty or stale worktrees are evidence, never product authority. Their HEADs are
ancestors of the control checkpoint and no writer may resume them. Preserve
until an explicit archive/removal decision; forward-port only from a fresh
bounded packet and current clean base.

| Worktree | Preserved fingerprint | Disposition |
|---|---|---|
| `bbr-f2a-space-footer` | HEAD `cd3b5771c1744a5e1714efeb0a8fe0f8bdde6d76` / tree+index `42adf2e533081865be06418e79ec2e373accb7fe`, 167 commits behind control, status SHA-256 `3418ac11eb1945bc9e0656a7d994eb7c7a5f6676f069a5aa524213daab52c90a`, 4 tracked + 2 untracked, `+156/-1` | `superseded`; F2a footer/theme/swipe authority is integrated through later reviewed lanes. No port path. |
| `bbr-gpui-ce-spike` | HEAD `ec52bf07aba70a3071aa98f3b4d635b252c68b20` / tree+index `6db035ca5bad86f2ad12e9094a43369c4e3817c5`, 351 commits behind, status SHA-256 `ebfd9445a8aef1f0e26c188155b3cf646b6bbd3fa0a4f1ad9294f38007262840`, 10 tracked + 1 untracked, `+1785/-1280` | `superseded`; GPUI-CE/root and G1a production authority are integrated. No wholesale port. |
| `bbr-g0-gallery-contract` | HEAD `75a22eebdac08415aa7792f71cc6a90fb18438f1` / tree+index `577013c7a70179e4b7fa0fc480adffb5bec33627`, 159 commits behind, status SHA-256 `591a5bb942e0247007c6dbecf371a1e2cde14550a9a856869b0c6029ebf5d2b6`, 4 untracked packets | `superseded`; canonical G0/G1/G2/G3 packets are committed in control and G1b is integrated. |
| `bbr-mvp-u1-palette` | HEAD `86c0799e856970e6919a3af6f9431f9e95b33a28` / tree+index `6b85503ea8745abda5e6b62bce56aba8632eab15`, 271 commits behind, status SHA-256 `a11b472f9936ba16f2712f2be0b22daa1328b37e48a5cbd89b39f644d928dd7b`, 1 tracked historical-ledger edit, `+6/-4` | `superseded`; U1 production and later Portal work are integrated. Historical ledger edit has no current authority. |
| `bbr-schema-v2-decision` | HEAD `3139939bbd5c911b515e079e2333fa9b37cfa335` / tree+index `354e16b61aca9fde994f1e670a7fe7f68cf47207`, 257 commits behind, status SHA-256 `ce3da4899c76179030abeaf4bd9ebaf1bf70228073fa3e0ce2dde86b69da08d7`, 1 untracked packet | `superseded`; canonical schema decision is already committed and consumed. |
| `bbr-url-input` | HEAD `5fcebb820b005b9adc6cf837a1d16f7b40f59cc3` / tree+index `b3321f975ffd92925f2ec2356e1100ae43797f95`, 394 commits behind, status SHA-256 `d88dbe9a9a1fa6414a99163304a202762b12c591fa988bf80b237ff4be898f8f`, 24 tracked + 1 untracked, `+466/-34` | `superseded fragment`; current address/input, command, testkit, performance, and G2 ownership have evolved. Any still-desired behavior must be specified in a fresh bounded packet and reimplemented/ported selectively; no direct resume. |

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
- Feature-specific regression coverage remains mandatory. Patrick currently
  defers all performance execution for MVP delivery because unrelated host
  processes make measurements non-actionable: do not run, retry, preflight,
  idle-wait for, or block integration on `e2e-perf`, `e2e-perf-full`, feature
  benchmarks, sample counts, or canonical-runner measurements. Preserve every
  performance test, budget, baseline, and contract unchanged; record the missing
  evidence as explicit debt and never claim it passed. This scheduling override
  applies to B2 and subsequent MVP slices until Patrick lifts it.
- GUI and functional real-app E2E remain strictly serialized through the
  repository GUI lock. `just e2e` currently includes `e2e-perf`, so under the
  override run the required functional recipes individually instead of changing
  or weakening the canonical recipe.
- Preserve unrelated dirty files and worktrees by explicit scope and
  fingerprint.

## Scheduling checkpoints

Patrick's current performance override removes host-idle/performance gates from
current MVP integration only. Foreign build load is not a reason to wait before
functional E2E or non-performance checks; the repository GUI lock and bounded
functional deadlines remain authoritative. Performance remains unpassed debt.

Patrick's current scheduling override defers all WKEXT, ENGINE-SPIKE,
TART-RUNNER, and SPLIT-COMPOSITION reconciliation regardless of the descriptive
inventory rows above. The aborted workflow
`post-authority-queue-reconciliations-msm9rxrm-ep7d9x` grants no verdict or
follow-up authority. Do not review, port, integrate, archive, settle, or launch
replacement work for those four concerns; preserve every source branch/worktree
unchanged and leave them outside active delivery until Patrick explicitly
re-enables them.

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
| G1b independent static review | exact evidence-gated tree received three bounded findings; no product-authority widening | Authority half returned `REVIEW_PASS` and authorized settlement commit `ce4b3c236e1b58383f2ca25dadc4e69663d3cebf` / tree `ccc7f5cd0a60b1901c92e607c15f87db822ed903`. Product half returned `REVIEW_FINDINGS`: unused `AppContext` imports in `main.rs` and `harness.rs` break Clippy `-D warnings`; five pointer/keyboard collapse E2E assertions expect all stories hidden although only group 0 collapses; `dependency_contract.rs` docs describe the obsolete three-dependency/absent-framework contract. The implemented disclosure authority, real framework controls, append-only StoryIds, copy, macOS gating, dependency code, and module budgets were otherwise accepted. The bounded remediation and focused recheck later proved the AppContext finding invalid and passed the valid test/doc repairs; exact G1b candidate `3b85ed80…` now waits only for clean Gallery E2E. |
| G1b/B2 evidence-candidate focused review | authority and G1b passed; B2 product and packet reviews passed | Settlement authority committed as `75aa11a329d7b2232f05705dc5716ff75ddf012c` / tree `0046d47486fc35f23680a5cbb754c26500758f6d`. G1b returned direct `REVIEW_PASS`: required `AppContext` trait imports retained, collapse expectations and hidden-child assertions correct, dependency docs exact, no unintended changes. B2 product review verified all eight-file remediation behavior and fingerprints, but found task-packet text that named pre-self-reference tree `7d2e787d…` and incorrectly said the ledger line was excluded from the hash; the actual reviewed tree was `7805cc9c…`. Packet-only authority committed as `11d1a0234924c75b143c92a568333a358b9332eb`; the sole writer corrected exactly that prose and returned `CANDIDATE_READY` at tree `ec2438c…`. Review settlement committed as `6cbb3a4bb2127232c8d72168c6d6ec4bd92308e5`; final authority and packet reviewers both returned direct `REVIEW_PASS`. |
| Deferred evidence and downstream read-only preparation | G1b/B2 remain frozen; no skipped gate is accepted | Patrick authorized temporary deferral of the outstanding GUI/performance execution while unrelated host Cargo/Clippy load continues. The stale detached `agent-device` Xcode-26.6 runner PID `48835` was explicitly authorized for termination and exited cleanly on `SIGTERM`; no other foreign process was manipulated. G1b/B2 fingerprints remain exact and unchanged. A temporary combined integration rehearsal preserved the semantic `justfile` union, produced tree `476401427ec620fd78da1a536dc147977030756a`, and passed complete `just verify`; log `/tmp/bbr-g1b-b2-integration-rehearsal-verify.log`, SHA-256 `612eca3714d16572724d34641aabbae14c572d87d199ec14b150e927fb87da50`. This is non-GUI rehearsal evidence only and authorizes no integration. Read-only, non-authoritative downstream handoffs were prepared at `/tmp/bbr-g2-readiness-handoff.md` SHA-256 `89e0c733ee1ea24c53a2de9a81c9f53c20ca78d3268cbcd8cd8077a420b7225e` and `/tmp/bbr-b3-readiness-handoff.md` SHA-256 `2bcc512d73bb0851401d5749186157f58ac244063f29f0ba2ecccce7cb3673a5`; G2 still waits for integrated G1b and B3 for integrated B2. No product writer is authorized. |
| G1a recovery settlement/review | recovery candidate `49bee8bf…` survived independent gates unchanged | first reviewer found only F1 duplicate `SpaceChipVisuals`, F2 crate-local lock, and F3 stale architecture wording; dependency closure, behavior cutover, module budgets, non-GUI gates, affected E2E, and repeated perf passes were accepted. |
| G1a remediation and serial completion | bounded writer resolved F1–F3, then Orchestrator added only the required root workspace and Justfile serial points | one canonical `bbr-ui::state::SpaceChipVisuals`; no crate-local lock; implemented architecture truth; root `Cargo.toml` includes `crates/bbr-ui`; `just verify` includes `ui-deps`; 46 script tests registered. Complete tree `8c441f2c…`, status `5c7f058f…`, mixed real index unchanged. |
| G1a focused pass/freeze | fresh reviewer returned direct `REVIEW_PASS` with every fingerprint stable and all focused gates green | Orchestrator committed the exact reviewed tree as `e6d3ab85…`; commit tree `8c441f2c…`; worktree clean. It originally waited behind successful F2 closeout and B1; the MVP-first scheduling decision below removes only the F2 performance dependency. B1 remains the serial predecessor, and G1b remains blocked until G1a is integrated. |
| Downstream read-only readiness | B1/G1a/G1b integration preflights plus B2/B3, G2/G3, Z1–Z5, and evidence-lane maps completed without mutation | durable packet-grounded dependencies are prepared, but findings derived from the unrelated dirty `bbr@0dc22fe` worktree were rejected. No readiness workflow authorizes a writer; only this committed snapshot and fresh identity/ownership audits may unlock work. |
| MVP-first scheduling decision | F2 startup closeout deferred; B1 and G1a frozen; G1b and downstream dependency-gated | Patrick explicitly moved dedicated startup optimization and full performance/package closeout behind the first feature-complete MVP without changing the `≤500 ms` target. Functional F2 evidence is green, so B1 integration is now executable; G1a follows B1, then G1b. Other items remain gated only by their product and ownership dependencies, not by the deferred startup closeout. |
| G1b live-product remediation settlement | Patrick's binding findings are implemented on an exact immutable forward state, but the writer correctly returned `BLOCKED` because minimum repository and GUI evidence was then incomplete | Exact G1b state: tree `ecb84ab89d0b66f5562a3137740bfd31b97412b7`, binary diff SHA-256 `af559aa11f22eb767d24c1f51a328da9ac7571e77682c5985dbcb46f1088346e`, status-z SHA-256 `22388e8081c853eacdf952a7d84423f7b76a76a534e0657eeb4076a9473d0886`, 28 paths, real index base, diff-check green. The former missing pointer path gained stable disclosure targets and one shared `TreeState::activate` authority for pointer plus keyboard. Seven append-only real framework-control stories and their interaction E2E exist; copy and macOS gating were corrected; Status line/G2/G3 remain absent; all Gallery Rust modules remain below 500 lines. Writer `g1b-live-feedback-remediation-msm04by8-bzgeqw` settled with no commit or ownership. Later focused review and non-GUI gates passed; the clean full Gallery gate subsequently exposed the two concrete failures recorded below. |

| G1b clean Gallery gate failure | two clean host samples, exact unchanged candidate, repository Xcode 26.2, and repository GUI lock; no override or competing process | `CARGO_BUILD_JOBS=4 nix develop -c just e2e-gallery` acquired `/tmp/bbr-gui-gate-6ff732a0fdc0bb09.lock` and ran all 14 ignored GUI tests. Result: 12 passed, 2 failed. `gallery_copy_is_honest_foundations_primitives_with_g2_g3_continuation` could not find rendered selector `bbr-gallery-title`; `real_input_focus_and_typed_editing` observed `helloworld` instead of `hello world`. Inner recipe exit 101 is explicit in the log. Orchestrator logging then returned 255 because it read the wrong pipeline-status slot; that wrapper defect does not alter the Cargo failures. Evidence log `/tmp/bbr-g1b-e2e-gallery-20260809T1935Z.log`, SHA-256 `6878d9984f4e8a7cf4769643fa67601cfb404bb90862099720edcbb8bae070bd`, 89 lines / 6395 bytes. Post-gate tree/diff/status/path fingerprints remained exactly `3b85ed80…` / `e049268c…` / `22388e80…` / 28, real index base. G1b moved fail-closed to bounded remediation; no retry and no B2 GUI gate ran. |
| G1b clean-gate remediation dispatch | reviewed one-file authority transition committed as `2bddd205ef3a38f52c38463b8f127fd7d72cfb8e` / tree `ed2a168a2ef5381517b77c26f36e46f1bad0480d`; control worktree clean after commit | Independent reviewer returned direct `REVIEW_PASS` on exact diff `ee7d40af…` and authorized exactly one bounded writer. `g1b-clean-gallery-gate-remediation-msm7kv2e-tpgh78` exclusively owned the unchanged failed-gate input and was forbidden to commit, integrate, weaken assertions, run B2 gates, or run the final full Gallery acceptance gate before focused review. |
| G1b remediation settlement and acceptance | writer settled without commit at exact tree `368b07d7479b48039f509a1fc3121d03c2f018f9`; independent candidate review and host-exclusive acceptance both completed on that unchanged tree | Writer returned direct `CANDIDATE_READY`; reviewer returned direct `REVIEW_PASS`. Two clean host samples preceded the recipe-owned Xcode-26.2 Gallery gate; `just e2e-gallery` passed 14/14, exit 0, under the repository GUI lock. Log `/tmp/bbr-g1b-e2e-gallery-20260809T2000Z.log`, SHA-256 `1ad8c42d3b0fe89d54d5c68c9947a4dba5f05ef84ca33144209517fa813e9c73`. Post-gate tree/diff/status/index/path fingerprints remained exact. G1b is frozen for Orchestrator commit/integration; B2 gates remain unrun. |
| Z2/Z3 advisory rechecks and queue refill | remediated read-only handoffs received independent direct `REVIEW_PASS`; no repository mutation or product proxy evidence | Z2-1 is dependency-ready as a bounded keying refactor. Its clean isolated worktree is now bound from committed scheduling base `1d916108…`; writer label `z2_folderid_keying_writer` is reserved but not active while the host-exclusive G1b post-merge gate waits for foreign build load to clear. Its expected G2 group-4 folders conflict is a serial semantic integration cost. Z3's doc-only activation reconciliation propagates the already committed MVP-first direction while preserving startup p95 `535.866375 ms > 500 ms` as red Phase-9 debt; it changes no status, product gate, or writer authority. |
| G1b serial integration and cache recovery | exact lane commit `01b563b…` merged conflict-free at `e067c43…`; source tree stayed clean throughout | Initial post-merge verification failures were traced to cross-worktree compiler-cache reuse: B2-only `bbr-command` enum metadata and persistence test binaries compiled from a removed temporary rehearsal path. Fresh-target check passed; complete derived-target cleanup plus no-cache compilation passed `just verify` on unchanged tree `cfd8679…`. Earlier post-merge Gallery attempts correctly started no recipe while foreign Cargo load was present. After the authority cut `6829930…`, two clean samples preceded one repository-locked Xcode-26.2 run; Gallery passed 14/14 on unchanged tree `e1c1aa8…`, log SHA-256 `1dc961945b94399abeabdfd6416ab0401fc6ef90d0669e2afcab654327369262`. G1b is integrated; no foreign process was touched. |
| B2 integration and post-B2 refill | B2 semantic merge complete; Z2-1 passed review; G2 has one bounded comment remediation; NW-SEAM preflight ready | B2 merged as `c8604e0…` / tree `403e974…` with exact lane second parent and direct merge `REVIEW_PASS`; functional and non-performance evidence green, current performance explicitly deferred/unpassed. Z2-1 received direct `REVIEW_PASS` and froze for serial integration. G2 product behavior was accepted; one writer edited only three stale Gallery comments and the focused recheck passed. NW-SEAM returned `PREFLIGHT_READY` but remains writer-free until Z2-1 and G2 Status Line integrate. N1/PEEK remain behind integrated NW-SEAM. Patrick's WKEXT/ENGINE-SPIKE/TART/SPLIT deferral remains unchanged. |
| Z2-1 serial integration | exact reviewed FolderId-keying candidate merged into the clean post-B2 control checkpoint | Lane commit `211e179…` / tree `98f187c…` merged as `5e727d39…` / tree `9748d5ba…`, preserving B2 additions on both shared testkit files and every reviewed Z2 path. Non-performance verification, explicit Clippy, roadmap, and diff checks passed. Functional real-app gates passed Sidebar 7/7, Organize 2/2, Drag-Chip 4/4 under Xcode 26.2 and the repository GUI lock. Performance execution remained Patrick-deferred/unpassed; tests, budgets, baselines, and contracts were not changed. |
| G2 Status-Line serial integration | exact reviewed first G2 cut merged after Z2-1 with no conflict | Lane commit `76dc9b5…` / tree `6a61e7a…` merged as `fe817210…` / tree `d4118dba…`. The app keeps snapshot-to-model behavior; bbr-ui owns the single production status row; Gallery appends the first Browser Components story. Full non-performance verification and explicit component Clippy passed; functional Sidebar 7/7 and Gallery 14/14 passed under Xcode 26.2 and the repository GUI lock. No performance recipe ran or passed. NW-SEAM's serial predecessors were satisfied; remaining G2 groups are independently packet-open, not implied complete by this cut. |
| NW-SEAM review, integration, and DDA refill | exact candidate reviewed, committed, merged, and functionally re-proven before queue advancement | Candidate tree `bc6427e…` / binary diff SHA-256 `11202831…` received direct `REVIEW_PASS`; Orchestrator committed it as `3bc016b…` and merged it at `05765724…`. Post-merge non-performance verification is green (`d33b5638…`); NW-SEAM 4/4, Tabs 1/1, and Profiles 1/1 passed through the repository GUI lock. Performance remained unexecuted/unpassed with contracts unchanged. Full DDA readback identified no new unowned bounded gap: N1 and PEEK are dependency-satisfied but await a separate committed worktree/ownership activation; modifier/background opening remains decision-needed. |

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
