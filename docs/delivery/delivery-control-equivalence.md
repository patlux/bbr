# Delivery-control old-to-new equivalence audit

This audit is the cutover checklist between the preserved historical execution
ledger and the concise current-delivery snapshot. It is not itself an execution
queue. Every row must be rechecked against the latest committed old ledger and
actual Git/workflow state immediately before integration.

## Sources and durability boundary

| Surface | Candidate-preparation evidence |
|---|---|
| Preparation base | `3bf31eec5559feca30ac41cf3b5339f920ab4414`; final reconciled candidate rebased onto `0dc22fee16d737a077aa1b097e359e2f1d08a764` |
| Old canonical pointer | `docs/delivery/2026-08-08-post-mvp-delivery.md` |
| Old ledger blob SHA-256 at preparation | `80f5940ba05fe1f3400af2859b9ff2737a5f4376f577cb353191bdf4b4afa542` |
| Latest committed old-ledger refresh included | `main` `0dc22fee16d737a077aa1b097e359e2f1d08a764`; includes the `02:10` F2a integration chronology plus documentary closeout truth; final hashes are regenerated in the cutover gate report |
| Old ROADMAP blob SHA-256 at preparation | `5209e6d89d76a099508da175bd4470969d24ea6b538086d26e3392f5c5057085` |
| New candidate pointer | `docs/delivery/current.md` |
| Preserved untracked Shutdown packet source | source SHA-256 `dc2fb80f7c4d3d6b30853df8e3bb43af4cf090951de0a0a69fa9fbab7de578c4`; imported byte-identically, then changed only at the two delivery-control metadata lines (`Status`/`Base commit` → durable Activation/Base condition). Current candidate SHA-256 `9924c00100447941cf86ad27962f3b9c23f9531ae9c1bef47a1224cf3e2ea249` |
| Main unrelated-dirty fingerprint before lane creation | `cf47c216371ee9dbc73a692332f6182abc916e941b9baf3333c98f6d0fef7b37` |

These hashes prove the preparation input only. The Orchestrator must replace or
supplement them with the final pre-cutover commit and post-F2a base during the
atomic integration.

## Queue mapping

| Old ID | Old state at preparation | New owner / ID | Equivalence rule |
|---|---|---|---|
| P0 | integrated `5009be5` | historical ledger only | completed evidence preserved; not duplicated in current executable queue |
| P0b | integrated `5009be5` | historical ledger only | content-probe regression evidence preserved |
| 1 | Patrick/manual | `MANUAL-SMOKE` | remains externally blocked/manual; no completion claim |
| 2 | integrated `963e94f` | historical ledger only | completed evidence preserved |
| 2b | needs concrete UI input | `UI-2B` | remains blocked on screenshot/location |
| 3 | integrated `cd1bc65` | standing decisions + history | Downloads behavior and evidence retained |
| 4 | integrated `bb54841` | standing persistence decision | no migrations/backcompat retained |
| 5 | integrated `ddd09cd` | durable task/phase links + history | no active work invented |
| 6 | integrated `9f53f44` | F2a dependency/history | appearance contract remains consumed by F2a |
| 7 | integrated F2a `7180128` + closeout `0dc22fe` | historical ledger only; post-cutover wave is current | all strip/tint/swipe/drag/perf scope, direct reviews, semantic unions, latest-main/navigation-cover preservation, documentation truth, and unchanged evidence rules retained |
| 8 | integrated `26e19c1` | history + standing architecture | command-history authority retained |
| 9 | integrated `a70e9a3` | history; remaining Phase-8 acceptance stays open | do not reinterpret the old contradictory note; final reconciliation must distinguish delivered surfaces from remaining broader phase work |
| 10 | integrated `53a9590` | history | auth/portal evidence retained |
| 11 | integrated `1d35693` | history | zoom wiring retained |
| 12 | integrated `1d35693` | history | existing context-menu authority retained |
| 13 | integrated `e5dab25` | history | native external URL behavior retained |
| 14 | one environment gate open | `PORTAL-MINIMIZE` | remains blocked until display unlock; no false full-hardening claim |
| 15 | integrated `72b85bd` | history/performance track | warm-start evidence retained |
| 16 | integrated `6f91b32`; subclaims open | `PERF-500-50`, `PERF-RUNNER`, `PERF-RESOURCES` | measured 230/20 and functional 500/50 remain distinct from missing canonical evidence |
| 17 | integrated with F2a `7180128` | `ICON` | Chevron additions/Find usage retained; obsolete Spaces glyph authority remains absent |
| 18 | integrated | history + remaining explicit blockers | no false canonical `just e2e` closeout claim |
| 19 | integrated `72b85bd` | history | one repository authority and visible-set hydration retained |
| 20 | post-F2a Lane A | `SHUTDOWN` | same territory, same fresh regression requirement, integration priority 1 |
| B1 | post-F2a Lane B | `B1` | same portable-only territory, schema-v3 bump+discard, post-audit start |
| B2 | after B1 | `B2` | same one shared action/provider authority and dependencies |
| B3 | after B1+B2+G1a | `B3` | same ordinary GPUI Library and bounded 1k rendering |
| Z1 | after F2a+G1a; Bookmark after B2 | `Z1` | unchanged |
| Z2 | after F2a+G1a | `Z2` | unchanged flat session-local Folder boundary |
| Z3 | after F2 closeout; Bookmark after B2 | `Z3` | unchanged Portal/context-menu ownership dependency |
| Z4 | after F2a+G1a | `Z4` | unchanged one titlebar/navigation authority |
| Z5 | after F2a+G1a | `Z5` | unchanged capability-first media ordering |
| G0 | integrated `edf83ee` | history/task contracts | no runtime implementation claim |
| G1a | post-cutover Lane C | `G1a` | Foundations/Primitives extraction-only/no-copy boundary and ownership preflight; exact direct dependencies `gpui` + `bbr-domain`, existing `bbr-domain → bbr-web` profile-identity closure accepted, direct `bbr-ui → bbr-web` and duplicate hue/identity authority forbidden |
| G1b | after settled G1a | `G1b` | dev-only Gallery MVP consumes the same production symbols; no WebKit/network/testkit listener |
| G2 | after G1a+G1b/production components | `G2` | unchanged production-first story rule |
| G3 | after G2/production compositions | `G3` | unchanged production-first composition rule |
| Dark WebKit background | integrated `2da0cae` after isolated concurrent session settlement | historical ledger only | exact four-path integration and smoke evidence retained; no remaining current queue item or F2a authority |

## Root ROADMAP roll-up preservation

The previous root roadmap duplicated detailed current-focus acceptance. The new
root intentionally keeps only phase roll-ups and discoverability links; none of
the removed acceptance disappears:

| Previous root roll-up | Canonical surviving owner |
|---|---|
| Parallel-delivery foundation | [parallel-delivery foundation packet](../roadmap/tasks/parallel-delivery-foundation.md), [development workflow](../development-workflow.md), and [parallel-delivery contract](../parallel-delivery.md) |
| Performance E2E foundation and remaining canonical-runner work | [performance E2E packet](../roadmap/tasks/performance-e2e-foundation.md), [Phase 4](../roadmap/phases/04-ui-shell.md), and [performance track](../roadmap/tracks/performance-release.md) |
| Tab-mutation focus matrix | [browser interaction contract](../product/browser-interactions.md) and completed Phase-4 shell acceptance |
| DESIGN creation and semantic tokens | [DESIGN.md](../../DESIGN.md) and [UI design track](../roadmap/tracks/ui-design-system.md) |
| First vertical-tab shell | [Phase 4](../roadmap/phases/04-ui-shell.md) shell/interaction checkboxes |
| Profile/space rendering and switching | [Phase 4](../roadmap/phases/04-ui-shell.md), [testing track](../roadmap/tracks/testing.md), and [cross-context hint packet](../roadmap/tasks/cross-context-profile-hint.md) |
| Adjustable/collapsible sidebar | completed [Phase-4 sidebar acceptance](../roadmap/phases/04-ui-shell.md) |
| Tab organization | completed [Phase-4 organization acceptance](../roadmap/phases/04-ui-shell.md) and `e2e-shell-organize` evidence |
| Dense-500 lifecycle/windowing | [dense-list packet](../roadmap/tasks/dense-list-slice.md) and completed functional Phase-4 item; measured 500/50 frame time stays open in the performance track/current blockers |
| Pointer states/internal semantic tree | [Phase 4](../roadmap/phases/04-ui-shell.md), [accessibility track](../roadmap/tracks/accessibility.md), and [testing track](../roadmap/tracks/testing.md) |
| Product-phase checkboxes | preserved one-for-one in the new root roadmap |

Checkbox totals are not expected to be byte-identical because mixed stale
roll-ups were split during reconciliation and the durable Shutdown packet adds
its own acceptance. Every added `[x]` is backed by the named completed packet,
integrated commit, and existing real-app/performance evidence; every unproven
remainder stays `[ ]` under a narrower owner.

## Decision mapping

| Old decision | New location | Required equality |
|---|---|---|
| No persistence migrations/backcompat | `current.md` standing decisions and B1 row | exact |
| Search Settings owner | `current.md` standing decisions | integrated resolution (`bbr-command` portable contract; app/persistence composition), not the superseded early recommendation |
| Space appearance palette | `current.md` standing decisions | integrated nine-token catalog, not the superseded early eight-tone proposal |
| Downloads destination/history/badge/retry/delete/reveal behavior | `current.md` standing decisions; historical ledger | exact |
| Saved Pages product boundary | `current.md` standing decisions and B1/B2/B3 rows | exact |
| Full F2a before A/B/C | active wave, post-F2a wave, integration order | exact |
| SHUTDOWN first, then F2 closeout | integration order | exact |
| B1 normally before G1a/G1b | integration order | exact, unless concrete dependency evidence changes it |
| G1a reuses one ProfileHue/identity authority | `current.md`, UI track, G1a packet, and ARCHITECTURE | exact: `bbr-domain` remains the hue owner; current `bbr-domain → bbr-web` ProfileId edge is audited rather than hidden; no direct UI-to-web edge or copied identity vocabulary |
| Downstream packages remain gated | downstream dependency queue | exact |
| GUI/E2E/perf host-exclusive | standing constraints | exact |
| Only Orchestrator commits/merges/integrates | standing constraints and cutover protocol | exact |
| No budget/sample/deadline/validator weakening | standing constraints | exact |

## Documentation-authority mapping

| Information | Old mixed location | New sole owner |
|---|---|---|
| Stable repository entry point | ROADMAP plus ledger prose | `ROADMAP.md` |
| Current base, active lanes, owners, gates, blockers, order | historical ledger | `docs/delivery/current.md` |
| Full chronology and recovery details | historical ledger | unchanged historical ledger |
| Product acceptance | ROADMAP/phase/track/task duplicates | canonical phase/track/task packet; reconciliation must split mixed claims rather than mass-check |
| Implementation facts | scattered roadmap evidence | `ARCHITECTURE.md` / `E2E_ARCHITECTURE.md` when next substantively updated |
| Parallel-delivery rules | `docs/parallel-delivery.md` plus volatile lane table | durable rules remain there; volatile wave state moves to `current.md` in a later bounded cleanup if needed |

## Explicit non-claims in this candidate

- F2a is not complete or integrated.
- `docs/delivery/current.md` is not canonical before atomic integration and
  target-session acknowledgement.
- The candidate does not close or reclassify any phase checkbox.
- The candidate does not prove product E2E, performance, packaging, or release
  acceptance.
- The copied Shutdown packet is durable planning input only; no shutdown
  implementation or missing `e2e_shutdown.rs` is claimed.
- Worktree cleanup is not performed here.

## Final cutover recheck

Before the Orchestrator integrates this lane:

- [ ] Re-read the complete latest old ledger, not only this preparation copy.
- [ ] Record final old-ledger and ROADMAP blob hashes.
- [ ] Verify full F2a integration SHA and no remaining F2a writer.
- [ ] Audit every active workflow and worktree; update current rows without
      losing any new queue item or decision added after `3bf31ee`.
- [x] Replace every pending metadata value in `current.md` with exact
      `0dc22fee16d737a077aa1b097e359e2f1d08a764` product/rollback identities. The cutover commit is
      intentionally self-resolving; verify its exact SHA after commit and put
      that SHA in the atomic session handoff rather than attempting a Git
      commit that contains its own identity.
- [ ] Verify all mapping rows against current task packets and implementation
      evidence; split ambiguous mixed outcomes rather than checking them.
- [ ] Run validator tests, worktree/index/HEAD audits, `just roadmap`, and
      `just verify`.
- [ ] Obtain independent fresh-operator readback.
- [ ] Send the exact atomic handoff through `tmux-pi-send` and obtain target
      Orchestrator acknowledgement before starting SHUTDOWN/B1/G1a.
