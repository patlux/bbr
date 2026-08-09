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
| SHUTDOWN | `running` | `/Users/patwoz/dev/Personal/macos/bbr-shutdown-lifecycle`; branch `parallel/shutdown-lifecycle`; one exclusive writer, workflow `bbr-shutdown-lifecycle-writer-msl3ppbo-2jnadc` | app lifecycle, real SIGINT and owning-window close, portal-child isolation, testkit response flush, fresh `e2e_shutdown`, required E2E registry | started clean at `e9f5d302`; lifecycle/testkit/registry territory audited writer-free | **1 — first** |
| B1 | `running` | `/Users/patwoz/dev/Personal/macos/bbr-saved-pages-b1`; branch `parallel/saved-pages-b1`; one exclusive writer, workflow `bbr-saved-pages-b1-writer-msl3ppbx-tfy6z0` | focused portable domain/persistence Saved Pages modules, canonical URL equality, CRUD/search/list, schema v3 bump+discard, fixtures/benchmarks | started clean at `e9f5d302`; URL decision below recorded; repository/schema territory audited writer-free | after F2 closeout, normally before G1a |
| G1a | `running` | `/Users/patwoz/dev/Personal/macos/bbr-ui-g1a-foundations`; branch `parallel/ui-g1a-foundations`; one exclusive writer, workflow `bbr-ui-g1a-foundations-writer-msl3ppc1-b44jfs` | extraction of integrated production Foundations/Primitives authority into `bbr-ui`; no copies; exact direct dependencies `gpui` + `bbr-domain`; approved transitivity only `bbr-domain → bbr-web`, never direct `bbr-ui → bbr-web` | started clean at `e9f5d302`; theme/components/footer/sidebar/portal-render territory audited writer-free; dependency closure unchanged | after F2 closeout, normally after B1 |
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
