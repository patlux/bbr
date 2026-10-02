# GPUI Visual migration — requirement-to-artifact audit

Contract: [visual tooling](../roadmap/tasks/visual-snapshot-tooling.md),
[Gallery coverage](../roadmap/tasks/gallery-visual-coverage.md), and active goal
`1790884867215-ea6dba478fa758`. Execution/ownership remains in
[Current delivery](../delivery/current.md).

Inspection base: `36eea50e6d54f041127ba641b4dedfbecc10955d` on `main`.
This is an **incomplete milestone audit**, not migration acceptance. Portable
contracts, source inspection and real runtime evidence are separate columns of
proof; neither historical native captures nor synthetic PNGs substitute for fresh
Tart captures. No milestone checkbox is completed by this document.

## Requirement mapping

Paths in the table are repository-relative identifiers under
`/Users/patwoz/dev/Personal/macos/bbr`, not commands to execute.

| Requirement | Existing artifact / verified evidence | Remaining proof or boundary |
|---|---|---|
| Standalone immutable licensed tool | `scripts/visual/tool-source.json`, unchanged 83-file `vendor/cargo-gpui-visual` export at `c2723cac924b98b8a2c8c170d580bd2e434b674a`; provenance and upstream contracts pass | Import is implemented; it does not prove migration parity |
| Separate Rust consumer, no product dependency migration | `scripts/visual/Cargo.toml` has its own workspace/lock and path dependency on the export; product manifest/lock unchanged from activation | Public capture integration is still missing |
| One renderer and one comparison/seal/acceptance authority | Existing Gallery/browser producers; consumer calls upstream APIs in `vm_capture.rs`, `review.rs`, `main.rs` | Actual producer-to-report route must be exercised; retained unreachable legacy host branch is not the final VM route |
| Preserve GPUI pin and production boundaries | Root manifest/lock still pin `568271c01c5a9d640d7b091e5bfd136b04035f28`; no activation-to-current diff in product crates, gpui-component or baseline stores | Future Gallery fixes must preserve feature/dependency/profile boundaries and avoid product rendering changes |
| Select before rendering, complete requested appearances/pages | `selected_request.rs`, `gallery_selection.rs`, `guest/selected.rs`; ordered numeric calls and exact request/return binding tested | Real config has no story catalog; no real pre-render selected pilot yet |
| Independent required coverage and visible invalid siblings | Historical 936 required Gallery page pairs plus 31 additive withheld rows; eight-image browser validator; Collection contracts reject omitted/extra/duplicate output | Historical Gallery plan covers 49 of 53 stories; supported effective states and page/framing requirements are not fully authored or verified |
| Supported / inapplicable / open state classification | Six linked source audits cover registry IDs 0–52; actual predicates and fixture gaps recorded | This is source-level classification, not an executable complete supported-state contract. Do not promote recorder callbacks, labels or canvas preferences into interaction/effective-state proof |
| Nested content and shadow framing | Current measured outer pagination and existing semantic menu-height tests | Nested scroll windows, aliases, clipped descendants/shadows and complete content remain unproven; source formulas are not measurements |
| Tokens / real Checkbox / multipage Sidebar Light/Dark pilot | Existing production capture seams and portable selected pipeline | No fresh real pilot. Checkbox/initial Radio need legitimate asset-source approval; checked animation readiness, pointer transitions and framing require real evidence |
| All eight browser images | Shared `browser_output.rs` validates exact names, markers, RGBA8 and 2160×2814 / 608×2814 sizes | Current VM capture of all eight is absent; pre-Paper references are not newly approved |
| Actual guest capture only | `runner::require_vm_execution` refuses normal capture; `guest-execute` requires admin/VirtualMac, fixed actions and source/executable binding | No public VM capture route activated and no real guest execution in this migration |
| Exact source/build identity | `payload.rs`, `prepare.rs`, source snapshots, Cargo executable hashes, installation checks; historical host-only preparations verified | Prior payloads are stale diagnostics. Prepare again on the exact approved capture tree; verify actual guest installed/executed bytes |
| Separate host build and guest rendering environments | `vm_environment.rs`, guest observations and pre/post receipt binding; synthetic drift tests | Actual guest OS/display/fonts/session facts and equality between complete runs not observed |
| Guest runtime without repair/download fallback | `guest-runtime.json`, `vm_runtime.py`, `vm_runtime.rs`; pinned host closure verified offline | Availability of the pinned closure in the golden guest is unknown. Failure must not trigger installation or a host fallback |
| Safe bootstrap, transfer and raw return | Frozen helper hashes, fresh installation, bounded flat archives, original payload/request/receipt binding; hostile-input and cross-language tests | Actual read-only share, SSH stream, guest paths and returned native images remain unproven |
| Bounded cancellation/process/VM cleanup | Immutable upstream process owner, `vm_supervision.rs`, concrete bound evidence plus independent host checks; local signal/descendant tests | Real normal/timeout/cancelled Tart resource absence still required. Mocked observers prove protocol, not teardown |
| Network/privacy/profile boundaries | No new network or real profile use; clone-only hardening and listener/authentication policies implemented with fixtures | Before any boot: Patrick's current approval of disposable private-NAT/key-only SSH; then verify effective auth/listeners and allowed/forbidden access. No bridge, forwarding or host GUI |
| Exact decoded RGBA8, no tolerance/mask/resampling | Existing validators and immutable exact comparator; byte-preserving transfer/renames; exact one-pixel regression | Two independent complete same-source/environment VM captures still absent |
| Stills Review Collections | Explicit Rust `compare` uses current config and canonical coverage; invalid sibling visibility and fresh output contracts pass | Real migrated reports not visually inspected; failed suites remain failed evidence |
| Partial reports never full coverage/acceptance | Selected schema-2 `TEILLAUF` through separate upstream report, no Collection mixing; direct acceptance rejected with/without adoption | Real selected route still needs capture proof; even all-story selection remains development-only |
| JSON discovery / dry-run / inspect | `read_only.rs`, `discovery.rs`, actual CLI tripwires and explicit-path contracts | Real story discovery remains honestly unsupported until independently authored catalog activation; inspect does not validate current integrity |
| Explicit `suite:case` acceptance | Existing Rust update route delegates to immutable Collection acceptance; replay, stale sources/reports and malformed selection tests | Real baseline/environment adoption requires Patrick's approval of concrete report/cases; no approval inferred from green tooling |
| Existing baselines preserved, no WebP/LFS migration | No baseline-store diff since activation; no conversion command introduced | Missing Gallery baseline store and changed browser references remain open |
| Stale/no-write and integrity failure tests | Consumer/upstream tests cover source, config/policy, image/report, environment, selection, return and cleanup drift | Preserve these on public capture/cutover; portable passes are not actual rendering evidence |
| Remove replaced Python only after parity | Python entrypoints/recipes intentionally preserved and documented as unsafe for host capture | Removal has not happened and must wait for real VM parity; no second permanent pipeline is accepted |
| Reuse Tart/E2E infrastructure | Visual transport composes shared `scripts/tart_vm.py`; cleanup hardening keeps E2E defaults | Real compatibility with the existing guest/E2E route still needs current runtime evidence |
| Actual PNG and HTML review | No migration native images/reports were produced; tests inspect synthetic report strings only | Open actual PNGs and static reports after capture using approved tooling; generated HTML alone is not visual review |
| Additional Command / Downloads / Settings / Library compositions | Existing product surfaces and source findings only | After stable migration, define separate bounded packets; establish legitimate production capture before implementing. No synthetic Portal/native page substitute |
| Selected interaction / animation sequences | Upstream timeline contracts preserved; consumer still-review route rejects timelines rather than hiding them | Later packets need legitimate deterministic production time/input. Separate timeline reports required until Collections support them |
| Current portable and functional verification | Current non-GUI gates below pass | Fresh affected Gallery/browser GUI suites and stable functional Tart aggregate remain absent |
| Performance remains deferred | No budget, baseline or product timing changed; compilation/test durations are tooling observations | No product or canonical host-performance pass claimed |
| Ownership, preservation and publication | One local writer, small local commits; stopped foreign worktree untouched, no push/subagent | Maintain these constraints through capture, remediation and cutover |

## Current verification surface

At the inspection base, the repository was clean and no matching visual/Tart
process was found. No real GUI process was launched by these checks.

- `/tmp/bbr-visual-review-visual-test.log`: `just visual-test` passes 151 Python
  tests plus one separately run integration (152 discovered with one skip),
  unchanged upstream contracts and 82 consumer tests plus signal fixtures.
- `/tmp/bbr-visual-review-focused-reviewed.log` and
  `/tmp/bbr-visual-review-clippy-reviewed.log`: final review negatives and strict
  consumer all-target Clippy pass. Formatting and roadmap also pass.
- `/tmp/bbr-visual-migration-verify.log`: current full `just verify` sequence
  passes. `/tmp/bbr-visual-verify-hza9sd_c.just` preserves the repository recipes
  and substitutes only `mbx` for compiler-invoking `cargo check`, `cargo test` and
  `cargo clippy`. No target override, GUI recipe or changed assertion. Includes
  all 49 shared script contracts (14.606s measured).
- `/tmp/bbr-visual-migration-feature-gates.log`: bounded strict Clippy passes for
  `bbr --features testkit --all-targets`, the browser visual-capture test target,
  and `bbr-gallery --features visual-capture --all-targets`. Gallery capture-feature
  ordinary tests pass **159**, with **48 GUI tests ignored**, not run on the host.
  Gallery test time 0.49s; compiler-reported elapsed 13.72s / 8.51s / 3.13s / 17.53s.
  The existing `block v0.1.6` future-incompatibility warning remains unsuppressed.
- Activation-to-inspection Git comparison has no changes to root Cargo inputs,
  product crates, `vendor/gpui-component` or `tests/visual`. This proves preservation
  of those tracked inputs, not renderer correctness or runtime isolation.

Required missing runtime gates: fresh pilot, affected real Gallery/browser capture
and semantic suites, stable functional `just e2e` order excluding deferred perf
plus Peek/Gallery, two full exact repeated captures and actual visual review.
Historical Portal integration and pre-migration native snapshots do not satisfy
these current migration gates. Later source changes require new capture seals.

## Next bounded work and blocked boundaries

1. The [bounded pilot packet](../roadmap/tasks/gallery-visual-pilot.md) now defines
   the required input journey, two-clock endpoint boundary and default Sidebar
   nested/paint observations. Next local work is its source-bound endpoint
   identity/rejection contract, before changing fixtures or declaring a real
   story catalog usable. Keep the 936 historical
   pairs and 31 blockers until replacement evidence is actually proven. No product
   renderer change or guessed page count is authorized by this audit.
2. Execution boundary: obtain approval for the disposable clone's private NAT,
   dynamic guest TCP 22 reachable from the host/allowed NAT peers, run-specific
   key-only SSH, no bridge/forwarding/public listener, clone/key teardown and
   verification of unintended access paths. Do not boot merely to inspect runtime.
3. Asset boundary: obtain the narrow legitimate Gallery production asset-source
   correction for the real Check SVG used by Checkbox and initial Radio. Empty
   assets or a test-only drawn substitute cannot prove the pilot.
4. After those prerequisites, activate only the bounded pilot route, verify actual
   pinned guest runtime and source identity, and diagnose observed failures before
   expanding. A missing runtime needs an explicit decision, never silent repair.
5. Real baseline adoption is a later, report-specific approval, separate from the
   network/asset approvals and from tooling success. Additional composition/sequence
   packets are sequenced after stable migration, not idle parallel scope.

Useful authorized local requirement work remains, so this audit does not justify
pausing the whole goal. It also establishes that completion cannot be inferred
from the amount of implemented infrastructure or portable green tests.
