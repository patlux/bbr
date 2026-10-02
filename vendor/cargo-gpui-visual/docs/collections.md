# Review collections

## R3a bounded contract

Extend the original report to show multiple independently sealed suites in one
HTML document. Reuse its exact comparison, controls, embedded script/CSP and
styling; no iframe, replacement UI or combined baseline/environment authority.
A collection is a new explicit output directory, never an implicit latest run.

Each suite supplies an explicit ID, label, repository, sealed run, baseline root
and repository-relative catalog. A page set has a stable variant-aware identity
and must contain every numbered page once in authored order. Paths are resolved
once; all run/baseline/output
roots must be pairwise disjoint. Suite IDs namespace image assets, anchors and
radio groups. Duplicate case IDs across suites are valid.

The catalog is a consumer-owned source input, not the producer's emitted inventory.
It lists ordered snapshot bases, groups, optional sequences/moments/pages and
required/withheld/unsupported states. Both appearances remain required for each
required base. Missing required, unexpected and withheld captures stay visible as
failures; unsupported entries remain explicit non-coverage, never an image pass.
The exact catalog file must appear in the capture's source seal. Metadata cannot
be attached to historical images without a fresh capture.

A suite validation failure is reported without suppressing valid sibling suites.
Original source seals, image hashes, exact RGBA8 checks, baseline metadata and
per-suite environments still govern comparison. No baseline or capture writes.

R3a publishes collection evidence only after the complete report exists. It does
not publish single-suite acceptance seals in its asset directories and does not
provide collection acceptance. R3b must bind exact `suite:case` selection to this
explicit collection and revalidate every selected authority before writing. No
collection may be accepted through an unreviewed per-suite back door.

Verification uses disposable repository/capture/baseline fixtures: duplicate case
IDs, source-bound catalog, missing/extra/withheld/unsupported coverage, groups and
ordered pages/sequences, invalid sibling suites, independent environments,
path/ID collisions, escaping, untouched original controls/CSP, and no-write/error
behavior. Locked/offline tests, strict all-target Clippy and rustfmt precede the
owned local commit. Browser checks use existing managed Chrome only; generated
fixture pixels are report tests, not product/native capture evidence.

No consumer import, production capture, baseline adoption, independent review,
new dependency, native/GPU execution or consumer ownership change belongs to R3a.
If report/browser verification is unavailable, record it without fabricating a
visual pass; keep missing migration/acceptance work explicitly open.

## R3b bounded contract

Add `accept-collection --report <explicit-directory> --case <suite:case>` with
optional repeated `--adopt-suite <id>`. No unqualified IDs, wildcard, implicit
latest, recapture or cross-suite environment flag. Every selected suite must have
valid coverage and an error-free reviewed comparison. An invalid unselected suite
may remain visible without authorizing any of its references.

Publish a report-file hash seal last; acceptance validates the exact HTML, copied
images, collection authority document and file inventory. Hold the collection and
selected suite report locks, prepare all selected payloads using the existing
single-suite acceptance rules, acquire every selected baseline lock and revalidate
before any reference write. A known invalid later selection must not partially
accept an earlier suite. Hashes protect integrity, not human authentication.

There is no cross-filesystem transaction: once reference writes start, publish a
create-new attempt journal; update per-suite completion and retain failures.
A collection with any attempt journal cannot be replayed. Regenerate an explicit
collection from still-valid captures to review remaining work, never repair/reseal
old evidence or recapture during acceptance. Interrupted per-file writes remain
subject to baseline metadata mismatch, as before.

Prove exact routing, duplicate/unknown/wildcard refusal, environment isolation,
all-selected preflight, changed catalog/source/capture/baseline/report/asset/seal
refusal, lock conflicts and partial-write evidence in disposable fixtures. Preserve
all old single-suite tests. Run full locked/offline tests, strict Clippy/fmt, review
the owned diff, and commit only with exact evidence. UI design and consumer imports
remain outside R3b; R2/R4–R7 stay open.

## Explicit withheld acknowledgement follow-up (2026-10-02)

BBR's reviewed 28-image collection has valid image/comparison evidence but an
explicit `remaining-app-surfaces` withheld requirement. The strict R3b route
correctly refuses it; this follow-up separates reference acceptance from coverage
completion without deleting or downgrading that requirement.

Add opt-in `accept-collection --acknowledge-withheld <suite:base>` (repeatable)
and an additive Rust API. The existing API/default remains strict. Acknowledgements
must equal the entire withheld set of exactly the selected/adopted suites; reject
unknown, duplicate, malformed, wildcard, required/unsupported or unselected-suite
IDs. They grant no image selection or environment adoption. Missing required,
unexpected/non-required captures, selected-development runs, unseen timelines,
comparison errors, stale sources, locks and all integrity failures still refuse
before writes. Derive the allowance from typed sealed catalog state, never by
matching an error prefix. Record exact acknowledgements in the attempt journal;
never change the sealed report, catalog, coverage verdict or original pixels.

Verify negative-first CLI/API disposable-store contracts, all-selected preflight,
unchanged strict API behavior, explicit-only environment adoption, retained failed
attempts and a fresh comparison which remains coverage-failing after every image
matches. Run complete locked/offline tests, strict all-target Clippy and fmt before
an owned local commit. No dependency/report UI/renderer changes or other consumer
writes. BBR may then use the verified CLI on its already-approved exact report
while sources are still valid, followed by a separately verified immutable import
and fresh Tart screenshot evidence. New/changed images are not approved by that
historical approval. Stop at an unresolved test/integrity/runtime failure; no
manual copying, stale resealing or per-suite back door.

### Usage and verification

`accept-collection --report <exact-reviewed-directory> --case browser:<case>
--adopt-suite browser --acknowledge-withheld browser:remaining-app-surfaces`
is explicit acceptance of that case/environment only, not a coverage pass. Repeat
`--case` for every approved image and `--acknowledge-withheld` for every withheld
base of the selected authorities. The Rust API is
`collection::accept_acknowledging_withheld`; `collection::accept` stays strict.
Opt-in attempt journals use schema 2 and retain `acknowledged_withheld` even after
partial failure. Ordinary journals stay schema 1. Sealed report bytes never change.

Standalone regression first failed on the absent CLI flag. All 28 collection tests
then pass, including five parameterized acknowledgement contracts: exact image
adoption with still-failing fresh coverage, malformed/incomplete acknowledgement,
structural/comparison failures, stale authorities/locks and explicit environment
adoption. The private interruption contract tests schema-1 and schema-2 journals
at both suite and partial-file boundaries. Full locked/offline suite: **183 passing
results** (180 top-level plus three child fixtures), no failures/ignored. Strict
all-target Clippy and rustfmt pass through the pinned mbx wrapper with hard process
group deadlines; serial test execution retains original deadlines/assertions.
Evidence: `/private/tmp/gpui-visual-withheld/{red,collection,tests,clippy,fmt}.log`.
Self-review only; no independent agent. Original report assets, comparator, source
seals, codecs, dependency graph and licenses are unchanged. These are disposable
fixtures, not BBR/native acceptance or screenshot evidence.

## R3a evidence

Implemented in the standalone authority only. `collection::{Input, Suite, compare}`
and CLI `review-collection` reuse the exact existing comparator through an internal
asset-only route, and the original report document/viewer/CSP through namespaced
suite presentation. Original JS bytes unchanged. CSS adds coverage status colors
and collection-only narrow-screen layout; single-suite document layout retained.
Baseline authority paths, individual capture hashes/environments, groups, sequences,
moments, pages and all coverage dispositions are visible. Paths/IDs are preflighted
before output; valid suites are revalidated after the full collection completes.

Fresh bounded wrapper gates:

- `mbx test --locked --offline`: **82/82**, 12.91s measured total, including the
  previous 70 entries and 12 collection tests; zero ignored/failures.
- `mbx clippy --locked --offline --all-targets -- -D warnings`: pass, 1.09s total.
- `cargo fmt -- --check`: pass, 0.53s total; `git diff --check` pass.
- New API tests first failed at the absent collection import, not a claimed
  runtime regression. All 38 original contracts pass unchanged. An intermediate
  strict-Clippy failure identified two unnecessary owned path comparisons; fixed
  without allowances. Page validation rejects omissions, duplicates, reversed order
  and inconsistent totals in the independent catalog.

Managed `chrome-patwoz`0.38.1 attached to the existing Personal Chrome at
127.0.0.1:9222, one task-owned tab, no server/new browser/profile. Disposable fixture
only: eight generated20×12 RGBA images, two independent environments, two one-pixel
changes and one suite-only environment mismatch. DOM checks proved36 unique IDs,
10 valid anchors, two distinct radio groups,14 decoded eager images, original CSP
script activation, keyboard swipe, onion, row-mark crop/reset, global mode/crop,
mixed state and finding filter. First control check incorrectly assumed2-up despite
an existing saved preference; inspected actual checked modes, then checked
independence against that initial state. Global preference writes were intercepted
in-page during the successful check; user storage was not cleared or exported.

Screenshots inspected at1652×942 and390×844. Narrow viewport initially overflowed
(693px document/390px viewport) because of inherited fixed sidebar/control layout.
A collection-only responsive adjustment fixed this (390/390, full-width main,
static navigation/controls); desktop retained the original design. No browser
errors in the final check. Task tab closed; retained fixtures are test artifacts,
not product screenshots. Initial mechanical detector returned no findings; it is
not an accessibility certification, and no native/UI/product coverage is claimed.

Logs `/tmp/gpui-visual-collection-{red,first,second,clippy,final-tests,final-clippy,final-fmt}.log`;
DOM result `/tmp/gpui-visual-collection-browser-check.json`; final screenshots
`/tmp/gpui-visual-collection-{desktop,mobile}-final.png`. Browser fixture roots and
commands are recorded in `/tmp/gpui-visual-collection-browser-fixture{,-final}.log`.

R3a commit `2d3db96a24ea9ee50efb18cfa1213734a8448b31` does not implement acceptance
or import consumers. The subsequent R3b evidence below covers portable acceptance.
The source-bound catalog protects against accidental inventory omission; authoring
truthful requirements and production scenarios remains consumer-owned.

## R3b evidence

`collection::accept` / `accept-collection` route exact selections from the explicit
reviewed authority document. The collection seal covers HTML, every displayed
asset, source-bound catalog metadata and complete report file inventory. Root and
selected suite report locks prevent cooperating asset replacement. All selected
payloads use the extracted original acceptance policy and are prepared before any
reference writes; payload bytes are themselves rehashed. Existing pending files,
invalid selections, source/capture/baseline changes and lock conflicts fail before
payload mutation. Baseline-lock acquisition may create empty directories, not
reference metadata/images. Per-suite explicit adoption preserves the original
byte-identical unselected-reference requirement.

Attempt evidence is create-new and synced before writes; completed authorities
are recorded after each suite. Private deterministic write-boundary tests interrupt
after the first suite, then separately fail the second suite after a PNG write but
before metadata. Both retain failed attempts naming only the completed first suite,
refuse replay and preserve partial-current-suite invalidity on a fresh comparison.
No cross-filesystem atomicity, hostile same-user protection, escaped processes or
power-loss durability claim. Tests do not kill a live production acceptance.

Fresh bounded pinned-wrapper gates:

- `mbx test --locked --offline`: **92/92**,16.79s total.21 collection integration
  entries, one private interruption test, original38 contracts and prior32 entries.
- `mbx clippy --locked --offline --all-targets -- -D warnings`: pass,1.36s total.
- `cargo fmt -- --check`: pass,0.49s; `git diff --check` pass.
- Mutation removing both collection seal checks fails the unchanged-report test
  on altered `index.html`; exact seal checks restored and full suite green.
- CLI tests refuse no selection, global adoption and a replacement suite input,
  then accept only the selected disposable suite. No producers are invoked.
- UI/document/comparator/dependency/licensing/provenance bytes unchanged from R3a;
  its browser evidence is retained, not claimed as a fresh R3b browser run.

Logs `/tmp/gpui-visual-collection-accept-{first,tests,clippy,interruption,seal-red,final-tests,final-clippy,final-fmt}.log`.
No consumer import, product capture or real baseline/environment adoption. Portable
R3 is proven for these contracts; consumer group/sequence reconciliation, large
Gallery adoption and exact production captures remain R4–R6.
