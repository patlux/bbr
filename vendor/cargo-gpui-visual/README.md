# GPUI Visual

Standalone home of the `cargo-gpui-visual` library and CLI, extracted from GPUI
Mobile Kit. This repository is the sole portable-tool development authority;
SDKs and apps consume immutable versions and keep their own GPUI pins/renderers.
See [extraction and provenance](docs/extraction.md) and the active
[Rust-first migration](docs/migration.md).

Portable Rust CLI: exact RGBA comparison, static HTML review, sealed-source validation
and selective acceptance. No Python/Pillow runtime, GPU, GPUI dependency, web server,
network access or browser launch is needed for compare/accept. Git is required for
source inventory. The consumer's capture command owns its renderer and build tools.

## For coding agents and consumer developers

Use the consumer's pinned entrypoint; no MCP server or extra runtime is needed.
Read the [short consumer guide](skills/gpui-visual/references/consumer.md), the
[bundled skill](skills/gpui-visual/SKILL.md), and the
[JSON/inspection/dry-run contract](docs/agent-interface.md).

```sh
cargo gpui-visual skill                  # instructions matching this binary
cargo gpui-visual doctor --json          # read-only prerequisites, no producer
cargo gpui-visual list --json            # exact authored story IDs
cargo gpui-visual check --story tokens --report reports/visual/dev-001 --dry-run --json
# Execute only with the consumer's required native permissions and gate options:
cargo gpui-visual check --story tokens --report reports/visual/dev-001 --json
cargo gpui-visual inspect --report reports/visual/dev-001 --case tokens-page-01--dark --json
```

JSON stdout stays separate from producer stderr. Responses include scope, typed
errors, exact artifact paths and observational phase timings. `inspect` loads only
requested findings/ticks and labels stored evidence as unvalidated. Selected runs
are development-only, never baseline-acceptance or independent full-suite evidence.
No command installs a skill, changes agent configuration or grants native permission.

## Local use (not published)

```sh
# On this machine use mbx for compiler-invoking commands.
scripts/with-rust-toolchain.sh mbx run --locked --offline --release -- \
  check --repo /path/to/consumer
# The produced executable supports Cargo's subcommand dispatch:
cargo-gpui-visual gpui-visual doctor
```

After a separately approved publication, the intended interface is
`cargo install cargo-gpui-visual --locked` followed by `cargo gpui-visual check`.
This package is currently `publish = false`; no registry release is claimed.
It is an independent Cargo root so installing/building the CLI does not resolve GPUI
or require an SDK checkout. A `.crate` archive includes the report assets.

`visual.toml` (schema 1) configures argv, not shell text:

```toml
schema = 1
capture = ["mbx", "test", "--locked", "--test", "visual", "--", "--ignored"]
environment_command = ["my-environment-probe"] # prints JSON
baselines = "tests/visual/baselines"
report = "reports/visual/latest"
runs = "reports/visual/runs"
[source_policy]
scopes = ["src", "tests/visual", "Cargo.toml", "Cargo.lock", "visual.toml"]
suffixes = [".rs", ".toml", ".lock", ".png", ".svg"]
```

The producer gets `GPUI_VISUAL_OUTPUT`, writes PNGs and `inventory.json` (a list of
IDs). PNG channels must have at most 8-bit precision (including palette/grayscale
images decoded losslessly to RGBA8); 16-bit candidates and references are rejected
as invalid input, never silently downsampled. Each ID is `<case>--light` or `<case>--dark`; both appearances are required.
The CLI probes environment and sources before/after and seals `capture.json`.
Alternatively an existing producer can seal schema 1 itself; omit
`environment_command`. Its source inventory must equal the configured policy.
`{run}` in capture argv is substituted with the absolute run path.

```sh
cargo gpui-visual check                 # fresh unique run, never overwrite
cargo gpui-visual compare --run <run>
cargo gpui-visual accept --run <run> --case example--light --case example--dark
cargo gpui-visual accept --run <run> --all   # everything this review flags; asks first
cargo gpui-visual doctor                # read-only config/source preflight
```

Compare and accept also work without configuration with explicit `--repo`, `--run`,
`--baselines`, `--report`. `--adopt-environment` requires a reviewed mismatch; every
unselected existing reference must be byte-identical. `--all` is an explicit bulk
selection, never a default: it derives the list from the report's sealed review
(changed and new stills, complete changed or new sequences; never unchanged cases or
single journey frames), prints every ID on stderr and asks on an interactive terminal.
Without a terminal only `--yes` confirms it. The list is derived again under the
report lock and must be unchanged; then every ordinary acceptance check applies
(sources, images, capture/baseline state, full capture, environment). No tolerance or
mask option. Exit 0 = exact match/explicit successful operation; 1 = review required;
2 = invalid capture/configuration. Reports and baseline PNGs are ordinary files.
No command commits or pushes.

Environment comparison can explicitly separate rendering from run provenance using
an exact, reversible versioned partition. Legacy observations remain strict until
an authorized exact-hash metadata migration; before/after capture identity is
always complete. See [environment contracts](docs/environments.md).

Baseline schema 2 stores historical sources once per capture instead of per frame.
Legacy schema 1 remains readable without writes; explicit acceptance writes schema 2.
For an existing store, `compact-baselines --baselines <store> --expected-hash <sha256>
--dry-run` previews a lossless metadata-only migration. Upgrade consumer readers first;
this does not accept images and makes old baseline-state review seals stale.
See [baseline provenance and migration](docs/baselines.md).

References can be stored as **exact lossless WebP** (about a fifth of the capture
PNG bytes on UI screenshots). Captures stay PNG. Opt in explicitly per store with
`convert-baselines --baselines <store> --format webp --expected-hash <sha256>
[--dry-run]`, or set `baseline_format = "webp"` in `visual.toml` for a new store.
Every written reference is decoded back and proven identical in every RGBA8
sample; schema 3 records a library-computed pixel fingerprint per case, which
comparison uses to skip decoding identical references. PNG stores are unchanged
(schema 2, exact capture bytes). See [reference formats](docs/baselines.md#reference-formats).

Source discovery includes untracked, non-ignored relevant files and explicitly
selected submodule HEADs plus their working sources. Use narrow scopes: never
include secret directories. `.env*`, symlinks and escaping paths are refused or
excluded. Only the baseline directory resolved from `--baselines` or `visual.toml`
is excluded, independent of its name; other directories named `baselines` remain
ordinary inputs. `sources --policy <file>` and `validate --run <run>` use that same
resolution. Without configuration, pass `--baselines` when an exclusion is needed;
no implicit directory-name exclusion applies. Changing these inputs requires a
fresh capture, not resealing old images. Derived upstreams must be prepared and
validated before capture; include their canonical lock/patch inputs in the policy.

Starting with 0.1.1, `[source_policy].artifacts` can explicitly list narrow
repository-relative ignored package directories. Their complete file contents,
including newly added files, join the source seal. Missing paths, symlinks,
secret-name paths and artifact scopes overlapping the configured baseline tree
(in either direction) are refused. This is for independently
verified derived packages, not broad filesystem discovery. Changing such a package
after capture invalidates comparison and acceptance even though Git ignores it.

Hash manifests guard accidental/stale input changes; they do not authenticate a
human review. Acceptance validates all selected payloads first, uses a cooperative
lock and per-file atomic replacement. An interrupted multi-file write fails closed
through metadata mismatch; it is not a transactional filesystem guarantee.
Comparison and acceptance share a cooperative `.review.lock` in the report folder.
Replacing report images first invalidates the old HTML and acceptance seal; a new
`review.json` is written only after the complete report succeeds. After an ordinary
error, regenerate the report. After a killed process, confirm no comparison or
acceptance is active before removing its stale `.review.lock`, then regenerate.

## Run individual stories

Consumers that implement the explicit story-selection protocol can list and run
one or several exact story IDs without rendering unrelated stories:

```sh
cargo gpui-visual list --repo /path/to/consumer
cargo gpui-visual check --repo /path/to/consumer \
  --story tokens --story opening --report reports/visual/selected-review
```

`list` prints the source-authored JSON catalog without building or capturing.
`--story` is repeatable and check-only; unknown/duplicate IDs fail before capture.
No flag means the unchanged full run. Selection preserves complete Light/Dark
pairs, all authored pages and every tick of each selected timeline. The report
and terminal say **TEILLAUF**; an exact selection match is not full-suite evidence.
Selected runs cannot accept baselines or be used as a full-suite collection.
`check --story` and later `compare` of that run require an explicit `--report` so
they do not implicitly replace the normal report.

Requires `story_catalog` in `visual.toml`, sealed source requirements and producer
acknowledgement. Existing producers are **not** automatically compatible; neither
post-render filtering nor a silent full-capture fallback is provided. Native runs
still require their normal host/compatibility gates. See the
[producer protocol, schemas and Rust integration](docs/story-selection.md).

## Rust orchestration API

`config::Config::load` / `from_toml` expose the same strict schema-1 config as the
CLI. `capture::capture(repo, config, run, baselines, report)` invokes a fresh
producer and returns a validated `store::Manifest`. Relative paths resolve against
`repo`; explicit baseline/report paths include any caller overrides. Capture does
not compare, generate a report or accept references. Use the existing `compare`
and `accept` modules for those separately authorized operations.

The CLI uses this same implementation. `capture_with` accepts `Execution` limits
and a caller-owned cancellation flag; it does not install global signal handlers.
Defaults: 30 minutes per producer, 30 seconds per environment probe, 10 seconds
for cleanup and 1 MiB captured probe stdout. CLI `check` handles SIGINT/SIGTERM/SIGHUP
by cancelling and cleaning its owned process group before returning invalid-input
exit 2. `process::run` is also available for bounded Rust orchestration.

Unix execution preserves the child leader until group kill, then reaps it and
requires the group to disappear. Probe stdout uses a private anonymous tempfile,
not a pipe an orphan can keep open; output is size-checked before reading into
memory. Size checks may overshoot on disk between supervision ticks. Producers
must not daemonize/escape their group or install an external child reaper. SIGKILL
and machine crashes cannot execute cleanup. Non-Unix producer execution currently
fails closed; comparison/review/acceptance remain GPUI-independent. Only macOS
execution is verified in this slice.

For GUI/GPU/native consumers, `capture::capture_guarded` / CLI `check --host-gate`
hold the same-user host gate through cleanup, including offscreen Metal. An
unfinished durable lease blocks later capture after supervisor crash; explicit
hash-checked recovery is separate from capture/acceptance. Existing flocks can be
bridged with repeated `--compatibility-gate <absolute-path>` (never nest inside
the same legacy lock). See [host-gate contract and mixed-caller limitations](docs/host-gate.md).
The library's default `capture` remains unguarded for portable fixtures; consumer
native adapters must adopt the guarded route before native execution.

Source inventory Git subprocesses use the same bounded runner (30 seconds and 64 MiB
stdout per invocation, including nested submodules). `store::source_hashes_with`
accepts explicit limits/cancellation; capture passes its cancellation flag through.
The limit is per Git process, not a total filesystem/hash deadline. This currently
requires Unix; non-Unix source discovery fails closed, not a verified platform.
Producer inventories do not replace consumer-owned required-coverage checks. Failed runs
are retained, never overwritten or silently resealed.

## Multi-suite review collections

`review-collection --collection suites.json --report <new-explicit-directory>`
compares already sealed runs in the **original** review UI, with shared viewing
controls and namespaced suite assets/anchors. It never runs producers, chooses a
latest batch, writes baselines or adopts an environment. `collection::compare`
and `collection::Input::load` provide the same Rust path. Each suite retains its
repository, capture seal, environment and baseline store; a failed suite remains
visible without hiding valid siblings.

```json
{
  "schema": 1,
  "suites": [
    {
      "id": "gallery",
      "label": "Gallery",
      "repo": ".",
      "run": "reports/visual/runs/explicit-id",
      "baselines": "tests/visual/gallery-baselines",
      "catalog": "tests/visual/catalog.json"
    }
  ]
}
```

In a suite input file, repository paths resolve relative to that file; run and
baseline paths resolve relative to the repository. The Rust API resolves relative
repositories from the working directory. Catalogs are canonical repository-relative
paths. All capture, baseline and collection output roots must be disjoint.

Each catalog is an **independently authored requirement**, included in the
capture's source hashes, not copied from the producer's emitted inventory:

```json
{
  "schema": 1,
  "snapshots": [
    {
      "base": "panel-open",
      "label": "Open panel",
      "group": "Panels",
      "sequence": "Open and close",
      "moment": "Open",
      "state": "required"
    }
  ]
}
```

This requires `panel-open--light` and `panel-open--dark`. Catalog order is
preserved. Optional `page: {"set":"panel-standard", "number":1, "total":2}`
requires the complete ordered page set within that group/sequence/moment.
`withheld` and `unsupported` require a `reason`; withheld always fails coverage,
unsupported remains visible non-coverage. Missing required and unexpected or
non-required emitted cases fail. Omitting requirements from an emitted inventory
cannot turn the report green. Suite validation failures and environment drift
remain separate from image changes.

A new collection directory cannot be overwritten by the review command.
`collection-seal.json` is published last and hashes the exact HTML, copied image
assets, collection authority document and complete file inventory. Per-suite asset
directories contain no independent `review.json` acceptance seal.

```sh
cargo gpui-visual accept-collection --report <explicit-reviewed-directory> \
  --case gallery:panel-open--light --case browser:panel-open--dark
# Only after reviewing an actual mismatch in this exact suite:
cargo gpui-visual accept-collection --report <explicit-reviewed-directory> \
  --adopt-suite gallery
```

`collection::accept(report, selected, adopt_suites)` shares these semantics. Every
ID is exact; no wildcard, unqualified case, accept-all, implicit latest, new suite
input, global environment adoption or acceptance-time recapture. All selected
suites are preflighted using the same single-suite policy before payload writes,
then their baseline locks are acquired and inputs revalidated. Invalid unselected
suites remain visible and untouched. Selected suites with invalid coverage or
comparison errors cannot be accepted. Unselected references within an adopted
suite must remain byte-identical under the existing adoption rules.

Acceptance holds collection/selected-asset report locks and publishes an
`acceptance.json` attempt journal before writing references. It records completed
suites, success or partial failure. Any attempt blocks replay of that collection;
generate a new explicit review from still-valid captures for further selections.
This is **not** a cross-filesystem transaction. Interruptions can leave some suites
complete and others partially written; metadata mismatch and retained pending files
remain visible. Do not delete/reseal evidence to retry blindly. Report hashes
protect accidental integrity, not human authentication or hostile same-user writes.
See [collection contract and evidence](docs/collections.md).

## Review controls and output

The static report is a single scrollable review bench („Prüfstand“): a sticky
table of contents on the left, the stage in the middle and, beside every changed
or new case, a sticky inspector. Every entry stays in the page flow in this order:
findings, sequences, then all snapshots as grouped Light/Dark pair tiles that
open their comparison in place. Filters and keys are shortcuts only; nothing is
hidden by default. The report keeps group/sidebar counts, readable statuses,
height presets and the opt-in findings-only filter. All referenced stills
(including unchanged gallery entries) and sequences share 2-up, swipe (pointer and
keyboard), onion blending, diff and 1:1 controls; global mode/crop switches
include both. Missing sides keep the toolbar visible with comparison modes disabled
and an explanation; 1:1 and sequence time controls remain usable.

Changed rows are numbered boxes over their changed columns, read back from the
exact diff image; clicking one opens a 1:1 window on it. The inspector lists the
same rows with 1:1 Alt/Neu/Diff loupes. For valid full-run comparisons without an
environment mismatch it shows the same exact single-case `accept` command as the
terminal, with copy and a browser-local „gesehen“ note; selected runs, invalid
input, drift, collections and direct `report::write` calls show why no command is
offered. The report never accepts anything. Sequences show every tick as a colored
strip (same, differing, held, missing, no reference); thumbnails load only after
expansion. A missing reference or removed candidate tick has an explicit
placeholder, not an empty difference. Keys: `J`/`K` next/previous entry, `1`–`4`
view for all, `Z` 1:1 for all, `R` next row in the inspector, `C` copy the
command, `/` filter the table of contents. Below 900 px the sidebars stack above
and below the stage. No report server, web fonts or external assets are needed.

Optional consumer-defined gallery categories preserve domain ownership:

```toml
[[review.groups]]
title = "Components"
prefixes = ["component-"]
[[review.groups]]
title = "Settings"
prefixes = ["screen-settings"]
```

The first matching prefix group wins. Groups and cases are displayed alphabetically;
uncategorized cases fall back to their first ID prefix. A group count is the number
of Light/Dark pairs, while sidebar counts refer to individual cases. Empty titles,
empty prefix lists/prefixes and unknown review configuration fields are rejected.
Configuration without a `review` section remains supported.

Terminal output summarizes whole sequences rather than listing thousands of frame
findings. It prints at most 12 findings, links the report and offers a shell-quoted
single-case acceptance example only for valid full-run comparisons without an environment
mismatch. This is a suggestion requiring explicit visual approval, never an action.
Colors require a TTY and are disabled by `NO_COLOR`; full machine-readable evidence
remains in `review.json`.

### Sequence review

An optional `sequences.json` from a raw producer is sealed into `capture.json`.
Each journey has an ID such as `sequence-opening-60hz--light`, a 60/120-Hz virtual
cadence and an ordered receipt for every tick (including held frames). It must have
a matching appearance counterpart. Every drawn image belongs to its exact tick;
held ticks retain the preceding image identity. State/events/demand and pixels all
participate in comparison. Playback, stepping, scrubbing, first difference and
expected/actual metadata are review aids, **not native FPS measurements**. Filmstrip
thumbnails load only after expansion and retain browser lazy loading.

Accept whole sequence IDs, never frame IDs. Selected obsolete frames are removed;
unselected timelines and references remain protected. Removal of an entire journey
fails closed. New source edits invalidate older capture seals: use fresh runs and
fresh report directories, never rewrite provenance. `review.json`, the acceptance
seal, is still published last; an interrupted generation cannot be accepted.
Collections refuse suites containing timelines until they can present them: review
such a suite in its own report (`compare_with_options`/CLI `check`/`compare`).
Library consumers read the same optional `[review]` section via `config::Config`.

The `presentation::review_parity_fixture` contract test can retain a **synthetic**
report in a fresh `GPUI_REVIEW_FIXTURE_OUTPUT` directory for browser-control QA.
These generated color fields exercise comparison controls and missing sides, not
application rendering or approval evidence.

## License

**Apache-2.0**; see [LICENSE](LICENSE) and [NOTICE](NOTICE). Patrick explicitly
approved this SDK-local CLI/report extraction and its tests. The report assets
and adapted behavior originate in Hackerschau at
`b1528bd21a728da3a110074fb88b635a03f15c8c`; copyright and historical source-license
provenance are retained, not a blanket relicense of the application. Dependencies
retain their own licenses. Immutable approval/import records and the exact SDK
extraction revision are retained under [docs/provenance](docs/provenance/).
Publication remains deferred; `publish = false` is unchanged.

## Development

The standalone Nix toolchain pins Rust 1.98.1 without mobile targets or SDK
materialization. Use hard timeouts; mbx owns build storage:

```sh
timeout --kill-after=10s 300s scripts/with-rust-toolchain.sh mbx test --locked --offline
timeout --kill-after=10s 300s scripts/with-rust-toolchain.sh mbx clippy --locked --offline --all-targets -- -D warnings
timeout --kill-after=10s 120s scripts/with-rust-toolchain.sh cargo fmt -- --check
```

Portable contract tests use temporary repositories and disposable baselines. They
do not launch consumer apps, capture production pixels or authorize references.
