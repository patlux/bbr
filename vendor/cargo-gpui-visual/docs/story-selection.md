# Individual-story development runs

## Scope

Requested from Hackerschau discussion `01a0ebf9-f5c4-73a3`: developers must be able
to discover stories and run just the relevant ones. This implements the **shared
portable protocol**, not a Hackerschau/BBR/SDK producer migration. Consumers must
import an immutable export and implement selection in their own renderer adapter.
No consumer checkout, GPUI pin, production pixels or actual baseline is changed.

Selection happens before the producer renders. No image-only post-filter, wildcard,
substring match, persisted default filter or silent full-capture fallback. All
selected story requirements remain mandatory; incomplete pages, appearances or
timelines fail closed. This is development feedback, never full-suite evidence.

## CLI

```sh
cargo gpui-visual list --repo /path/to/consumer
cargo gpui-visual check --repo /path/to/consumer --story tokens \
  --report reports/visual/tokens-review
cargo gpui-visual check --repo /path/to/consumer \
  --story opening --story tokens --report reports/visual/selected-review
cargo gpui-visual compare --repo /path/to/consumer \
  --run reports/visual/runs/explicit-id --report reports/visual/selected-review
```

`list` returns the full validated catalog as JSON, in authored order, with no
build/producer/report/baseline write. It verifies that the catalog belongs to the
source policy. It can discover requirements without running the renderer; listing
does not prove that the producer implements the protocol.

`--story` accepts repeatable exact IDs only on `check`. Unknown, duplicate and
empty IDs fail. Selection is normalized into authored order. Omit the flag for a
full run. Every selected check/comparison needs an explicit `--report`; choose a
separate directory, not the full-suite report. A unique run is still created by
default; existing run directories are never overwritten.

Exit 0 means exact agreement **inside the selected scope**, 1 review required,
2 invalid input/producer failure. Terminal and HTML prominently label TEILLAUF,
name the scope and block acceptance suggestions. Machine readers must inspect the
schema and `selection`, not interpret `passed` or exit 0 as full-suite success.

Host/GPU/device authorization, host gate and compatibility locks are unchanged.
The example commands alone do not grant native capture permission.

## Independent consumer catalog

Add a canonical repository-relative JSON path before `[source_policy]`:

```toml
story_catalog = "tests/visual/stories.json"

[source_policy]
scopes = ["src", "tests/visual", "visual.toml"]
suffixes = [".rs", ".toml", ".json"]
```

Keep the consumer's existing scopes/suffixes as well. Catalog requirements must
come from the consumer's authored story authority, not from the just-emitted
inventory or capture output. A catalog may be generated from that independent
registry before capture, but changes must be sealed like other inputs.

```json
{
  "schema": 1,
  "stories": [
    {
      "id": "tokens",
      "label": "Tokens",
      "cases": ["tokens-page-01", "tokens-page-02"]
    },
    {
      "id": "opening",
      "label": "Opening animation",
      "sequences": [
        {"base": "sequence-opening-60hz", "hz": 60, "ticks": 181},
        {"base": "sequence-opening-120hz", "hz": 120, "ticks": 361}
      ]
    }
  ]
}
```

Each still/page base requires `--light` **and** `--dark`. Enumerate every supported
state/page; selecting a story cannot select an individual page or appearance.
Timeline bases start with `sequence-`; still/page bases cannot. IDs use lowercase
ASCII letters/digits/hyphens starting with a letter/digit. IDs and capture units
are unique; each story needs a nonempty label and at least one requirement.
Timelines require 60/120 Hz and 1–7200 total ticks per appearance (drawn **and**
held ticks), preserving the existing sequence-validator limit. A story may own
multiple timelines and stills. Unknown JSON fields are rejected.

This catalog is a development-selection mapping, not a replacement for the
collection's richer required/withheld/unsupported coverage authority. Full-suite
coverage and missing/withheld capabilities remain consumer responsibilities.

## External raw producer

Supported with `environment_command` configured; legacy self-sealing producers
are explicitly refused for selection. GPUI Visual resolves the catalog, source
hash and requested requirements before invoking the producer. It supplies:

- `GPUI_VISUAL_OUTPUT`: the fresh run path, as before.
- `GPUI_VISUAL_SELECTION`: JSON `selection::Selection` for a selected run only.
  It contains `catalog`, `catalog_hash` (SHA-256), and resolved `stories` in
  authored order. Every story contains its complete `cases` and `sequences`.

The producer must parse/validate this JSON, use it to construct only the selected
render schedule **before rendering**, and preserve its normal readiness, asset,
semantic and complete-page/timeline checks. Do not simply echo a request after
rendering everything or derive requirements from output. Unknown/unimplemented
stories must error instead of invoking the full schedule.

After successful output, write the exact deserialized selection to
`$GPUI_VISUAL_OUTPUT/selection.json`. Then return normally. The existing PNGs,
`inventory.json` and optional complete `sequences.json` remain mandatory. The
portable tool requires the receipt to match, including when all catalog stories
were explicitly selected, and rejects omitted/extra cases or timelines and
changed cadence/tick counts. The producer cannot publish `capture.json` itself.

Without selection, the CLI removes an inherited `GPUI_VISUAL_SELECTION`, including
from environment probes. A full capture must not emit a selection receipt. This
prevents stale shell state from silently narrowing the default run. The tool can
verify requested/output scope, not prove how much hidden rendering a consumer
performed: consumer integration must test that unselected stories are not invoked.

## Rust/local producer

```rust,ignore
let selected = config.select(repo, &["tokens".into()])?;
capture::capture_local(paths, &selected, cancelled,
    |output| {
        // Consumer-owned implementation: plan from this selection BEFORE rendering.
        produce_selected(output, selected.selection.as_ref())?;
        if let Some(scope) = &selected.selection {
            store::write_json(&output.join("selection.json"), scope)?;
        }
        Ok(())
    },
    || environment_probe(),
)?;
```

`Config::select` returns a clone, leaving the normal full configuration unchanged.
`selection` is runtime-only and cannot be set as an implicit TOML default.
`capture_local` does not modify global environment variables. The closure reads
`selected.selection`; external-command callers receive the same serialized type.
Use existing portable process/gate APIs for subprocess ownership and cleanup.
No additional renderer abstraction, dependency or nested supervisor is introduced.

## Seals, comparisons and compatibility

Full captures/reviews remain schema 1 with no `selection`; historical full runs
still deserialize. Selected captures **and reviews use schema 2**, and embed the
resolved selection including the catalog hash. Existing schema-1 tool versions
reject these rather than silently ignoring scope and accepting partial evidence.
Schema 2 without selection (or schema 1 with it) is invalid. Old consumers may
need to add `selection: None` to explicit Rust `Manifest`/`Review` literals when
updating their import.

Source and image integrity checks still apply; edited catalog or source inputs
require a fresh capture. Unselected references are excluded only from the
missing-candidate comparison check/presentation. The **entire baseline store** is
still integrity-checked and recorded; corruption or environment drift is not
hidden by selection. Exact same-size decoded RGBA8 and precision rules do not
change. Selected complete journeys retain metadata/tick/pixel comparisons.

A selected report is never accepted into baselines, including via environment
adoption. Collections refuse it as full-suite evidence. Even selecting all
catalog stories stays a development run. Make a fresh ordinary full capture and
review that for any separately authorized baseline acceptance. Do not remove
scope fields, downgrade schemas or reseal existing images to bypass this boundary.

## Verification

Disposable Git repositories and synthetic PNGs exercise the actual CLI producer,
local callback path and report. They prove protocol behavior, not native capture
speed or application rendering. The dedicated `tests/selection.rs` and
`tests/selection/adversarial.rs` cover:

- read-only list, exact/unknown/duplicate/empty selection and authored order;
- unsupported catalog/legacy producer, stale catalog/source binding and forged scope;
- absent/mismatched receipts, ignored selection, extra/missing pages/appearances/ticks;
- complete selected timelines, old-schema rejection and inherited-env clearing;
- full baseline integrity, default full-run removal errors and explicit report paths;
- no acceptance/adoption or full-suite collection evidence, no baseline writes;
- original gallery groups, complete pairs/pages, timelines and partial verdicts.

Optional `GPUI_SELECTION_REVIEW_OUTPUT=<fresh-directory>` on test
`adversarial::partial_review_fixture_preserves_pairs_pages_groups_and_full_sequences`
retains a synthetic original-UI report for browser checks; no application fixture
or reference image is synthesized.
