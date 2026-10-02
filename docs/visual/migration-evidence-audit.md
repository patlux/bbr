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

## Runtime addendum (2026-10-02)

The earlier table and blocked-boundary narrative describe the original inspection
and first two attempts. They are superseded only for the semantic prerequisite by
`/private/tmp/bbr-visual-runtime-fix/attempt9`: the existing supervised route now
passes49/49 Gallery scenarios, including the Checkbox target observer, and returns
fully validated original-source/build results after independently verified teardown.
The pinned51-path runtime is exported from already verified host bytes and imported
offline into the disposable clone, with no host/Golden mutation or downloads.
Guest console,444 font hashes, actual CoreGraphics geometry and Metal device are
observed before/after and equal. Real failure-path, cancellation and successful
semantic-run cleanup are proven; image-capture timeout cleanup is not inferred.
A final freshly staged `attempt10` under the same evidence root repeats49/49,
original identity validation and independent cleanup with all1,513 current source
hashes matching. Timings:5.38s tests /119.54s caller, report-only. Neither run
produces images. Current visual contracts/strict Clippy pass, but full non-GUI
`verify` now fails at the unchanged Command benchmark under host load; the isolated
benchmark pass does not erase those aggregate failures. Exact logs and p95 values
are recorded in Current delivery, superseding the old verification surface below.

The concrete runtime-absence blocker is resolved. The observer no longer lacks its
semantic callback gate and may be committed with its dependent49-case catalog and
route after local verification. This is not image capture or migration parity:
Check SVG/input/two-clock readiness, Sidebar nested/paint framing, browser/full
Gallery captures, exact fresh repeats, PNG/HTML review, stable functional aggregate,
public capture activation and Python removal remain open. No baseline or environment
adoption follows from semantic success. Detailed failed attempts and current next
work are recorded in Current delivery; historical no-install/approval stops are not
new user prohibitions.

## Static Tokens report addendum (2026-10-02)

After semantic checkpoint `299dc644`, Patrick authorized the first partial Tokens
report. The existing internal supervised capture → upstream seal → compare route
now produces twelve actual 2400×2000 RGBA8 guest images: all six authored Tokens
cases in Light/Dark. The configured story catalog is deliberately Tokens-only,
with no default selection and no claim to full Gallery coverage. All 936 historical
required pairs and 31 withheld requirements remain unchanged. Public capture and
interactive endpoint guards remain closed.

Report: `/Users/patwoz/dev/Personal/macos/bbr/reports/visual/tokens-pilot-20261002-a/index.html`.
Run: `/Users/patwoz/dev/Personal/macos/bbr/reports/visual/gallery-runs/tokens-pilot-20261002-a`.
Evidence: `/private/tmp/bbr-visual-tokens-pilot/attempt1`, including original payload,
request-bound transport/cleanup, and independent source/image/resource recheck.
All 1,515 source hashes and twelve image/report-copy hashes match; the schema-2
review has twelve missing baselines, zero validation errors and no environment
mismatch. Its `selected-development` scope forbids adoption. No baseline changed.

All twelve original PNGs and the rendered HTML were visually inspected through
file reads and existing managed Chrome. Material swatches, labels and elevation
text are legible without observed truncation. Elevation rows are descriptions,
not painted shadow samples; complete paint-framing is still unproven. The report
shows TEILLAUF and blocked adoption; all twelve unique image sources load. No exact
second capture, Checkbox journey, Sidebar framing, full Gallery/browser coverage
or stable-migration acceptance follows. Clone/key/process/disk/SSH absence and both
unheld gates were independently rechecked. Measured caller duration 286.428s is
report-only tooling evidence.

Focused catalog/discovery tests, serial full visual contracts, strict consumer
Clippy and formatting pass. The initial default-concurrency visual test run failed
in two unchanged upstream 250ms process fixtures with absent PID files during high
host load; the serial pass retains every deadline/assertion and does not erase
those failures. The prior aggregate Command benchmark failures remain separately
open. Current delivery records exact logs and next work. The remaining table and
blocked narrative retain the original inspection chronology; these two addenda
supersede their obsolete no-runtime/no-image/no-catalog statements, not their
unmet full-coverage, interaction, repeat or acceptance requirements.

## Requirement mapping at the original inspection base

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
| Tokens / real Checkbox / multipage Sidebar Light/Dark pilot | Existing production capture seams, portable selected pipeline and typed schema-2 Checkbox still requirements; endpoint-bearing execution/review fails closed | No fresh real pilot. Checkbox/initial Radio need legitimate asset-source approval; checked animation readiness, pointer transitions and framing require real evidence |
| All eight browser images | Shared `browser_output.rs` validates exact names, markers, RGBA8 and 2160×2814 / 608×2814 sizes | Current VM capture of all eight is absent; pre-Paper references are not newly approved |
| Actual guest capture only | `runner::require_vm_execution` refuses normal capture; `guest-execute` requires admin/VirtualMac, fixed actions and source/executable binding; two approved headless clone attempts recorded below | No public VM capture route activated; bootstrap runtime failure precedes any Gallery execution |
| Exact source/build identity | `payload.rs`, `prepare.rs`, source snapshots, Cargo executable hashes, installation checks; historical host-only preparations verified | Prior payloads are stale diagnostics. Prepare again on the exact approved capture tree; verify actual guest installed/executed bytes |
| Separate host build and guest rendering environments | `vm_environment.rs`, guest observations and pre/post receipt binding; synthetic drift tests | Actual guest OS/display/fonts/session facts and equality between complete runs not observed |
| Guest runtime without repair/download fallback | `guest-runtime.json`, `vm_runtime.py`, `vm_runtime.rs`; pinned host closure verified offline | Actual approved retry proves pinned libiconv is absent in the golden guest; no installation or host fallback occurred |
| Safe bootstrap, transfer and raw return | Frozen helper hashes, fresh installation, bounded flat archives, original payload/request/receipt binding; hostile-input and cross-language tests | Actual read-only share, SSH stream, guest paths and returned native images remain unproven |
| Bounded cancellation/process/VM cleanup | Immutable upstream process owner, `vm_supervision.rs`, concrete bound evidence plus independent host checks; local signal/descendant tests | Two real failed attempts prove request-bound resource absence and independent host recheck; successful execution and timeout/cancellation teardown still required |
| Network/privacy/profile boundaries | Patrick approved the bounded route; actual retry proves effective key-only auth, TCP22 listener/access and TCP88/5900 rejection; both disposable clones removed | No bridge, forwarding, public listener, host GUI or real profile use; preserve these boundaries on future approved runs |
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
   nested/paint observations. The source-bound endpoint identity/rejection contract
   is implemented without activating a real plan/catalog or native execution.
   A bounded capture-only target observer is now an uncommitted candidate with
   portable rejection tests and a compiled, ignored guest scenario; its current
   evidence and exact files remain in Current delivery. Pure tests do not verify
   the callback. Host-only `prepare gallery-semantic` now reuses source-bound
   staging/runtime checks for an exact feature-enabled release test harness and
   standalone consumer; it executes neither. Its distinct payload purpose never
   becomes an image suite. The guest-only fixed action and exact 49-case
   result validator are portable-tested, not guest runtime evidence: they bind
   original requirement/executable hashes, list before running, and require exact
   successful libtest output with actual zero exit status. Portable-tested guest
   composition now preserves the two exact report-only sweep files outside the
   installed tree before its unchanged strict verification. The distinct pure host
   return validator reuses byte/receipt guards and retains original host identities;
   it does not produce capture evidence. The shared-supervisor/frozen-bootstrap
   integration now passes synthetic complete-chain tests, retaining both leases
   through independent cleanup checks. No public CLI or actual VM call is enabled
   by this local evidence. Actual host-only preparation and syntax-only bootstrap
   inspection passed at `/private/tmp/bbr-visual-semantic-route-prepare.Mw6A35`;
   later source/documentation edits require another fresh staging before execution.
   All these candidates stay uncommitted with the observer dependency. The generic E2E runner
   attempts dependency realization and must not run unchanged under this goal.
   Real callback/input/paint still needs the approved guest, and further input
   wiring must not conceal that missing gate. Keep the 936 historical
   pairs and 31 blockers until replacement evidence is actually proven. No product
   renderer change or guessed page count is authorized by this audit.
2. Execution boundary: Patrick approved the described private-NAT/key-only SSH
   semantic run on 2026-10-02. Two real attempts and independent cleanup are now
   recorded in Current delivery; the retry passed effective auth/listener/access
   checks but stopped at absent pinned libiconv before Gallery execution. Network
   approval is not the remaining blocker; no-install guest runtime is.
3. Asset boundary: obtain the narrow legitimate Gallery production asset-source
   correction for the real Check SVG used by Checkbox and initial Radio. Empty
   assets or a test-only drawn substitute cannot prove the pilot.
4. After those prerequisites, activate only the bounded pilot route, verify actual
   pinned guest runtime and source identity, and diagnose observed failures before
   expanding. A missing runtime needs an explicit decision, never silent repair.
5. Real baseline adoption is a later, report-specific approval, separate from the
   network/asset approvals and from tooling success. Additional composition/sequence
   packets are sequenced after stable migration, not idle parallel scope.

The subsequent semantic-route checkpoint in Current delivery completes the
bounded local prerequisite: portable action, strict diagnostics, receipt/return,
shared supervision and host-only preparation are verified. Its actual guest gate
is now the next required step. Repeating audits/builds adds no runtime proof;
additional input/framing work is deferred until that observer gate, and later
compositions/parity removal depend on it. The later approved runs at
`/private/tmp/bbr-visual-semantic-approved.09DN2I` supersede the network-approval
blocker. The first uncovered a legitimate OpenSSH repeated-field parser defect;
a negative-first repair passed portable tests and the actual fresh retry audit.
That retry failed because the pinned guest library
`/nix/store/jspv3c5l2zx4kiwzhq0zgxcwp34cqifz-libiconv-115.100.1` is absent.
Both clones and keys were removed with independently rechecked evidence. No
Gallery scenario ran and no dependency was installed. Preserve the uncommitted
observer/dependent candidates; migration remains incomplete at the concrete
runtime boundary. The isolated parser repair may be committed separately because
its actual guest audit passed and it has no observer dependency.
