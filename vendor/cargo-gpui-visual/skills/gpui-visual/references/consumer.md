# Consumer quickstart and integration

## Start with the project's existing entrypoint

Use an immutable reviewed export of GPUI Visual. This package is currently
`publish = false`; no published install is promised. The portable tool has no
GPUI dependency; the consumer owns its GPUI pin, renderer, production views,
fixtures, native readiness, independent coverage and baseline stores. Never edit
a vendored copy. Update upstream first, then import its exact revision with full
hash provenance in a separately authorized consumer commit.

Build the pinned executable once through the consumer's documented Rust/toolchain
wrapper. Reuse it for help/list/inspect; do not run a build wrapper for every
metadata request unnecessarily. With managed target garbage collection, retain a
private executable hard link/copy before waiting on a host lease, as the consumer
already does where applicable. This is retention, not a compiler-cache/target-dir
override. Never use a stale executable built from different sealed renderer inputs.

## New integration: minimal raw producer

```toml
schema = 1
capture = ["my-visual-producer", "produce"]
environment_command = ["my-visual-producer", "environment"]
baselines = "tests/visual/baselines"
report = "reports/visual/latest"
runs = "reports/visual/runs"
story_catalog = "tests/visual/stories.json"
[source_policy]
scopes = ["src", "tests/visual", "Cargo.toml", "Cargo.lock", "visual.toml"]
suffixes = [".rs", ".toml", ".lock", ".json", ".png", ".svg"]
```

Replace the example argv with the consumer's real route, not shell text. Existing
producer orchestration stays consumer-owned. The producer writes losslessly
RGBA8-decodable PNGs and `inventory.json` into `GPUI_VISUAL_OUTPUT`; it must not
self-publish `capture.json`. The environment command writes only its JSON object
to stdout, including the explicit `layer` (`gpui`, `ios` or `android`). Other probe
messages belong on stderr. GPUI Visual probes and seals before/after inputs.

Author a catalog from independent requirements, not the emitted inventory:

```json
{"schema":1,"stories":[
  {"id":"tokens","label":"Tokens","cases":["tokens-page-01","tokens-page-02"]},
  {"id":"opening","label":"Opening","sequences":[{"base":"sequence-opening-60hz","hz":60,"ticks":121}]}
]}
```

Each requirement includes Light and Dark. A selected producer parses
`GPUI_VISUAL_SELECTION`, plans only those stories **before** invoking renderers,
checks complete requirements and writes the exact selection to `selection.json`.
No request means full execution and no receipt. Reject unsupported selections.
Consumer tests must prove unselected render functions never run; the portable
validator can validate receipts/output, not hidden work. Source edits require a
fresh capture. Never remove selection fields to accept partial results.

Use `Config::select` for Rust/local producers and read `config.selection` in the
closure. `capture_local_timed` shares the existing seal implementation without
nesting supervisors. For example, in an already-authorized consumer function:

```rust,ignore
let selected = config.select(repo, &story_ids)?;
let mut timings = cargo_gpui_visual::timing::Timings::default();
// Keep existing bounded build and source-after-build validation here.
let executable = timings.measure("build_ms", || build_existing_harness())?;
let mut gate = timings.measure("host_gate_wait_ms", ||
    cargo_gpui_visual::gate::Gate::acquire(&gate_options, cancelled))?;
let result = cargo_gpui_visual::capture::capture_local_timed(
    paths, &selected, cancelled,
    |output| produce_with_existing_guarded_runner(
        &mut gate, &executable, output, selected.selection.as_ref()),
    || probe_environment(), &mut timings,
);
// Preserve existing poisoned-lease/CleanupFailure handling before finish.
// Only proven-clean cleanup may call gate.finish(); never clear it after a crash.
```

This is integration pseudocode, not a standalone renderer/framework or permission
to execute native work. Use the complete host-gate contract in the pinned source.

## Reports and diagnosis

`--json` wraps results in schema 1 (`agent::Response`). `check`/`compare` return
scope, logical-unit counts, environment flag and artifact paths. `inspect` returns
bounded stored summaries, exact PNG paths and one tick's state. It does not certify
source, baseline or report-asset integrity. `validate` checks sources and capture
images without running the producer. `compare` regenerates a report from an
explicit still-valid capture. Neither automatically accepts anything.

Collection review keeps separate suite source/environment/baseline authorities.
Inspect exact `suite:case` IDs. Invalid siblings remain visible; collection coverage
is not interchangeable with native acceptance. Collections currently refuse
sequence timelines and selected development runs. No global environment adoption.

`timings_ms` records wall time, not GPU time/FPS. `producer_ms` includes any build
hidden inside argv and group cleanup; probes accumulate; capture total includes
validation/sealing. Gate wait and report generation are separate where measured.
Phases overlap and missing values are not zero. To compare performance, measure
same-source warm/cold runs separately, counts/render invocations and output bytes;
never weaken coverage or exact comparison for faster green results.

## Consumer AGENTS.md snippet (review before adding)

> Use the project's pinned GPUI Visual entrypoint and bundled skill for visual
> work. Discover exact stories, then use selected development captures with an
> explicit separate report. Inspect only relevant image/tick details. Preserve
> native authorization, host gates, source seals and independent full coverage.
> Never accept baselines or adopt an environment without explicit visual approval.
> Report partial evidence as partial. Do not upgrade/import tooling automatically.

Adding this snippet or installing the skill into an agent directory is explicit
project setup, not an automatic effect of a visual command.
