# Delivery-control completion audit

This is the evidence checklist for the active delivery-control reconciliation
goal. A checked item requires direct artifact or command evidence; preparation
success is not cutover success. Re-run every item marked **final** against the
actual integrated commit before declaring completion.

## 1. Authority and document model

| Requirement | Artifact / evidence | State |
|---|---|---|
| `ROADMAP.md` is the sole stable entry point | `ROADMAP.md` defines the authority model and contains exactly one `Current delivery` Markdown pointer | prepared; final commit audit pending |
| Exactly one committed current-delivery pointer and marker | `scripts/check_roadmap.py::validate_current_delivery_pointer`; regression tests for zero/two pointers and marker ownership | prepared; worktree/index preparation audit green; HEAD final pending |
| Volatile runtime state has one owner | `docs/delivery/current.md`: metadata, status vocabulary, active lanes, worktree/owner/next gate, blockers, serial order | prepared and kept current during F2a; final post-F2a state pending |
| Durable documents contain only assigned long-lived authority | roadmap maintenance/template now require durable base conditions in packets and exact SHA/owner/worktree only in Current delivery; parallel-delivery kernel no longer calls root ROADMAP a runtime current-focus roll-up; activation-condition edits cover F2a/G1a/B1/B2/B3/Gallery/Z packets | prepared; final diff audit pending |
| `PLAN.md` is not an execution pointer | initial milestone renamed completed; ROADMAP named sole current entry | prepared |
| Historical ledger is preserved | `docs/delivery/2026-08-08-post-mvp-delivery.md` retained; candidate is conflict-free three-way refreshed through committed `main` `0dc22fee16d737a077aa1b097e359e2f1d08a764`, including the `02:10` F2a integration entry, and adds only cutover/archive framing plus integrated decision resolution | prepared; final latest-ledger hash/diff audit pending |
| No second delivery authority before cutover | Exact product/rollback SHAs are staged, but `current.md` remains a non-canonical candidate until committed HEAD audit, fresh-operator readback, and atomic handoff; target session confirms the old ledger remains canonical | directly proven |

## 2. Queue, decisions, TODOs, and acceptance preservation

| Requirement | Artifact / evidence | State |
|---|---|---|
| No open queue item is lost | `docs/delivery/delivery-control-equivalence.md` maps P0…20, B1–B3, Z1–Z5, G0–G3 and current blocker/program rows | prepared; final latest-ledger re-read pending |
| No dependency/integration-order rule is lost | `current.md` post-F2a wave and serial integration order; equivalence decision mapping | prepared; target-session final readback pending |
| No blocker is silently completed | `current.md` MANUAL-SMOKE, UI-2B, PORTAL-MINIMIZE, PERF-500-50, PERF-RUNNER, PERF-RESOURCES and phase/program rows | prepared; final audit pending |
| No acceptance criterion is silently removed | root-roll-up preservation table maps every removed root current-focus checkbox to surviving phase/track/task owner | prepared |
| No mass checkbox completion | staged checkbox delta shows only evidence-backed integrated items; mixed Phase-8/testing/native/UI items split into delivered and remaining rows | directly audited; independent final review pending |
| Historical decisions use later integrated resolution | old ledger marks Settings owner and nine-token SpaceTheme as later authoritative resolutions; current standing decisions match | prepared |
| G1a dependency contract matches source truth | `crates/bbr-domain/Cargo.toml`, `crates/bbr-domain/src/lib.rs`, `ARCHITECTURE.md`, `cargo tree -p bbr-domain`: direct `bbr-domain → bbr-web` exists for `ProfileId`; G1a requires exact direct dependencies `gpui` + `bbr-domain`, permits only that approved first-party closure, forbids direct `bbr-ui → bbr-web`, and avoids a duplicate hue/identity authority | prepared; implementation dependency preflight remains post-cutover |
| Product constraints unchanged | current standing decisions preserve no migrations/backcompat, Downloads behavior, Saved Pages boundary, one authority/writer, unchanged budgets/samples/deadlines | prepared; final diff audit pending |

## 3. Durability and discoverability validator

| Requirement | Artifact / evidence | State |
|---|---|---|
| All relevant first-party Markdown is scanned | validator enumerates every tracked `.md` except explicit external/build prefixes (`vendor`, `target`, `.git`, `.direnv`) | 89-file preparation audit green |
| Filesystem-only/untracked targets cannot satisfy links | Git path set is index/HEAD based; regression `test_link_to_untracked_file_fails_even_when_file_exists` | green |
| Git file mode is durable | tracked Markdown symlinks/non-regular modes and unstaged worktree symlink replacements are rejected instead of dereferenced | green |
| Worktree deletion cannot fall back to index | regression `test_deleted_tracked_markdown_fails_worktree_audit` | green |
| Worktree/index/HEAD can be audited separately | `--source worktree|index|head`; regression proves unstaged breakage is absent from index/HEAD but visible in worktree | green |
| Local fragments are durable | same-file and cross-file Markdown fragments resolve to real GitHub-style heading anchors; fenced examples are ignored | green for all current first-party fragments |
| Every task is reachable from ROADMAP | graph traversal from `ROADMAP.md`; candidate reaches all 52 task packets | green in worktree/index preparation audits |
| New/substantially revised task packets require current template | exact-digest grandfather list for eight legacy packets; any byte change loses exemption; required headings must occur exactly once outside fences | green; warnings name all legacy packets |
| Current pointer exposes operator-required data | exactly one marker; all required operational sections exactly once; fixed status vocabulary; unique IDs; non-empty active operator rows; exact old pointer and final base/rollback SHA shapes | green |
| Incomplete cutover cannot pass normal gates | validator requires exact reachable product/rollback SHAs and one current pointer; `--allow-pending-current` remains preparation-only and absent from Justfile | exact metadata staged; normal final gate pending |
| Validator regression tests run under `just verify` | `justfile` `script-tests` runs `test_with_gui_lock.py` plus `test_check_roadmap.py` and compiles the rule owner, Git/Markdown support module, tests, and GUI-lock scripts | 30 validator + 3 GUI-lock tests green; SHA-bearing final metadata must resolve to real commit objects reachable from HEAD, and rollback must contain the preserved ledger path |
| Packaging link is correct | `packaging/README.md` corrected from repository-escaping `../../docs/...` to `../docs/...` | preparation validator green |
| Shutdown packet is durable | source untracked file imported from SHA-256 `dc2fb80f7c4d3d6b30853df8e3bb43af4cf090951de0a0a69fa9fbab7de578c4`; exact diff changes only `Status`/`Base commit` metadata to durable Activation/Base condition; candidate SHA-256 `9924c00100447941cf86ad27962f3b9c23f9531ae9c1bef47a1224cf3e2ea249` | staged in isolated lane; final commit pending |

## 4. Scope, ownership, and live-session continuity

| Requirement | Artifact / evidence | State |
|---|---|---|
| Preparation occurs in isolated lane | `/Users/patwoz/dev/Personal/macos/bbr-delivery-control-reconcile`, branch `parallel/delivery-control-reconcile`, committed base `3bf31eec5559feca30ac41cf3b5339f920ab4414` | proven |
| No product/runtime/Cargo/baseline/GUI behavior changes | staged-path audit excludes `crates/**`, `Cargo.toml`, `Cargo.lock`, performance baseline; only docs, validator/test, Packaging README, and minimal Justfile script-test hook | proven; final staged audit pending |
| No competing commit/merge | lane remains uncommitted; only target Orchestrator may commit/merge/integrate | proven so far |
| Unrelated main dirt preserved | pre-lane fingerprint `cf47c216371ee9dbc73a692332f6182abc916e941b9baf3333c98f6d0fef7b37`; later main changes inspected without mutation. The separate dark-WebKit session settled and integrated its exact four-path scope as `2da0cae`; main returned to only the original protected Phase-09/research/Shutdown-planning dirt | ongoing; final before/after scope audit pending |
| Active F2a worktree is not mutated by this lane | all candidate edits under isolated docs lane; target session continues F2a | proven by separate worktree status/history |
| Target pane/session verified before every mutation | original preparation messages used verified pane `tws_personal_macos_bbr-orchestrator:2.0`; that pane was later reused by another Pi session. Read-only audit relocated the unchanged target UUID/footer `019fdc57-7afb-7f81` to `tws_personal_macos_bbr-orchestrator:3.0`. Any future handoff must re-run exact-pane/footer verification rather than trust either historical pane | proven; final handoff check pending |
| Every steering/handoff is atomic | `tmux-pi-send` JSON reports bracketed paste and one submit; target acknowledged continuity and candidate freeze | proven for preparation messages |
| Existing session keeps working | F2a product/docs integration is committed through `0dc22fee`; all historical F2a workflows are settled or explicitly obsolete; post-cutover SHUTDOWN/B1/G1a bundles remain preparation-only until acknowledgement | proven through committed old-ledger chronology and closeout; final post-cutover continuation pending |

## 5. Verification evidence

| Command / evidence | Coverage | State |
|---|---|---|
| `git diff --cached --check` | staged syntax/whitespace | green repeatedly |
| focused validator tests | 30 durability/control regressions; `check_roadmap.py` 499 lines plus focused `roadmap_validation_support.py` and `test_check_roadmap.py` below 500 lines; table construction is Python-3.9-compatible and retains the existing explicit row/header length guard. Direct host and `nix develop` reproduction both report Python 3.12.2, so the preflight's 3.9.6 environment claim is not a current blocker | green |
| `just script-tests` | validator + GUI-lock tests and py_compile | green |
| `scripts/check_roadmap.py --source worktree --allow-pending-current` | tracked worktree candidate graph | green with expected pending warning |
| `scripts/check_roadmap.py --source index --allow-pending-current` | staged durable candidate graph | green with expected pending warning |
| normal `scripts/check_roadmap.py` | fail-closed cutover metadata and authority graph | pending final run with exact staged SHAs |
| G1a dependency source audit | `cargo tree -p bbr-domain` shows only `bbr-domain → bbr-web`; domain source has the sole `bbr_web::ProfileId` re-export; `cargo tree -p gpui` proves a flat transitive native/serde blacklist would be invalid | green; packet/ARCHITECTURE contract corrected, implementation gate remains post-cutover |
| `just fmt`, `just check`, `just test`, `just clippy` | full non-roadmap project verification with unchanged product code | prior preparation green; final reconciled `just verify` on base `0dc22fe` pending below |
| pre-fail-closed `just verify` | complete original candidate before pending gate added | green; not accepted as final proxy |
| final `just roadmap` | complete cutover graph with exact metadata | **pending final gate** |
| final `just verify` | complete documentation/validator candidate on unchanged product base | **pending final gate** |
| final worktree/index/HEAD audits | same graph before/after commit from all three sources | worktree/index green; committed HEAD audit pending |
| preparation operator readback | local parser starting only at `ROADMAP.md` identified the sole target, both active remediations, exact worktrees/owners/next gates, and five blockers without historical-ledger reconstruction | green on candidate; not a substitute for the mandatory independent committed post-cutover readback |
| independent fresh-operator readback | active slice, contract, status, worktree/owner, next gate, blockers, order, rollback from the committed cutover | **pending committed cutover** |

## 6. Atomic cutover

All items below are mandatory and currently open:

- [ ] Complete and integrate all F2a layers; record exact verified post-F2a
      `main` SHA; no F2a writer remains.
- [ ] Re-read the complete latest historical ledger and inspect every active
      workflow/worktree/owner.
- [ ] Reconcile every new entry since preparation base `3bf31ee`; update the
      equivalence audit and current snapshot.
- [x] Replace all pending metadata with exact verified product base and
      pre-cutover rollback SHA `0dc22fee16d737a077aa1b097e359e2f1d08a764`.
- [ ] Run normal worktree and index validator audits, `just roadmap`, and
      `just verify` green.
- [ ] Only the target Orchestrator creates the single Docs/validator integration
      commit; verify its resulting exact SHA and HEAD audit.
- [ ] Obtain independent fresh-operator readback from the committed new entry
      point.
- [ ] Send one atomic `tmux-pi` cutover message containing old pointer, new
      pointer, cutover SHA, post-F2a base, status equivalence, full open queue,
      settled workflow/ownership state, serial order, and rollback pointer.
- [ ] Target session acknowledges those facts before SHUTDOWN/B1/G1a starts.
- [ ] Verify target session continues work from the new pointer and can state
      the next executable action without reconstructing the old ledger.

Until every checkbox in this section is proven, the active goal is not complete
and the old ledger remains canonical.
