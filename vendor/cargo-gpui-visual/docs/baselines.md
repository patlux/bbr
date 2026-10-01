# Baseline provenance storage

Baseline metadata schema 2 stores each case's `image_hash` and `capture_hash`,
then stores the historical source map once under `captures[capture_hash].sources`.
The existing environment and ordered sequence receipts stay in the same document.
A capture is removed only when no reference case uses it. Captures with identical
sources but different capture hashes remain distinct historical records.

`baseline::Baseline::load` reads schema 1 (inline per-case sources) and schema 2
without changing files. Comparison uses this reader; explicit acceptance writes
schema 2, including untouched cases' original provenance. Loading rejects missing,
unreferenced or conflicting capture records, invalid source paths/hashes, unknown
fields and invalid sequence inventories. Ordinary source values are SHA-256;
`@git` entries retain Git SHA-1 or SHA-256 object IDs from the source collector.
No current source inventory is substituted for historical provenance.

Old tool versions do not understand schema 2. Upgrade and verify all baseline
readers before migrating a consumer. Capture/review/agent schemas are independent
and unchanged by this baseline format.

## Explicit metadata-only migration

Use the consumer's verified pinned executable. Supply the exact SHA-256 of its
existing `baseline.json`; there is deliberately no configured-store fallback:

```sh
cargo gpui-visual compact-baselines --baselines tests/visual/baselines \
  --expected-hash <baseline-json-sha256> --dry-run --json
# After reviewing the dry-run and authorizing this metadata write:
cargo gpui-visual compact-baselines --baselines tests/visual/baselines \
  --expected-hash <same-baseline-json-sha256> --json
```

The operation does not load configuration, inspect current source files, execute a
producer or alter any PNG, environment or sequence. It verifies reference hashes,
complete PNG inventory and lossless RGBA8 decoding. Unknown metadata is rejected
rather than discarded. Existing acceptance locks, pending writes, symlinks and
input/store drift fail closed. A write rechecks the store under the existing
cooperative acceptance lock and atomically replaces only `baseline.json`.
`--dry-run` takes no lock and writes nothing; its output is observational, not a
reservation. Repeating on the canonical schema-2 file is a no-op. This is not a
hostile-writer/filesystem-transaction guarantee.

JSON reports old/new byte counts and hashes, case/capture counts, `changed`,
`written`, and an empty `accepted` list. Success is **not visual acceptance**.
Changing the metadata invalidates previously generated baseline-state seals:
regenerate the report before any separately approved image acceptance. Source or
tooling changes still require fresh captures; never reseal historical images.

## Reference formats

A store keeps its references as PNG (default) or as exact lossless WebP. Captures
are always PNG; only acceptance and explicit conversion write references.

- **Schema.** PNG stores stay schema 2 and keep the exact capture bytes, so older
  exports read them unchanged. A WebP store is schema 3: `image_format: "webp"`
  and, per case, `pixel_hash` next to `image_hash` (SHA-256 of the file). Schema 3
  requires both fields together (a converted-back PNG store keeps schema 3 with
  `image_format: "png"`). Old exports do not read schema 3: upgrade every reader
  of a store before converting it.
- **Pixel fingerprint.** `codec::pixel_hash` is SHA-256 over a fixed tag, width,
  height and every RGBA8 sample of the decoded image. It is computed by this
  library from decoded pixels at acceptance or conversion, never taken from a
  producer.
- **Writing.** Acceptance (`accept`, `accept --all`, `accept-collection`) decodes the
  capture PNG, encodes lossless WebP (vendored libwebp 1.6.0, lossless preset 8,
  `exact`), decodes the bytes back and writes only if dimensions and every sample
  are identical. The store's declared format always governs; `baseline_format` in
  `visual.toml` (or `accept::accept_with_format`) only chooses the format of a
  *new* store, and a configured format that differs from an existing store is
  refused until the store is converted.
- **Reading.** References are only read strictly: 8-bit PNG, or the exact container
  this library writes (`RIFF`/`WEBP` with one `VP8L` lossless chunk). Lossy (`VP8 `),
  extended (`VP8X`), alpha-plane, animated, truncated or trailing data is rejected.
- **Comparison.** For an intact reference (file digest equals `image_hash`), equal
  file bytes or an equal pixel fingerprint means identical pixels: the reference is
  not decoded (its container is still checked) and the report writes the same
  findings and diff image as a full comparison. Otherwise the reference is decoded,
  its recorded fingerprint verified, and every pixel compared exactly. Report
  assets stay PNG (`expected.png` is the reference or its identical candidate), so
  report, `inspect` and agent output are unchanged.
- **Trust boundary.** The fingerprint path trusts the `baseline.json` binding of
  `image_hash` and `pixel_hash` written by acceptance/conversion (covered by the
  review's baseline-state seal). A manual edit of a reference plus its `image_hash`
  is outside the protocol; `compact-baselines --dry-run` decodes every reference
  and verifies every recorded fingerprint.

### Explicit format conversion

```sh
cargo gpui-visual convert-baselines --baselines tests/visual/baselines \
  --format webp --expected-hash <baseline-json-sha256> --dry-run --json
# After reviewing the dry-run (byte savings) and authorizing the store rewrite:
cargo gpui-visual convert-baselines --baselines tests/visual/baselines \
  --format webp --expected-hash <same-baseline-json-sha256> --json
```

Like compaction it takes no configuration, inspects no current sources and runs no
producer. It refuses locks, pending writes, symlinks, metadata/store drift and an
invalid inventory (including stray files of the other format). Every reference is
decoded, its recorded fingerprint (if any) checked, re-encoded and proven
pixel-identical, all before any write. Environment, captures, capture hashes and
sequences are retained. A write takes the acceptance lock, rechecks the store,
writes each new reference atomically, then `baseline.json`, and only then removes
the old-format files. If interrupted, the retained evidence is explicit (pending
files, or leftover old-format files reported as a metadata/file mismatch); inspect
it before retrying. Converting an already converted store is a no-op; converting
back to PNG is exact as well. Regenerate reports after a write.

Measured on Hackerschau's 3,306 references (16 cores): PNG 271.7 MB of unique
files → 59.7 MB lossless WebP (22%) in 50.7 s, zero pixel differences. Preset 9
saved 0.3 points at ~12x the time; pure-Rust `image-webp` encoding reached only
47%, AVIF lossless 69-84% and JPEG XL (effort 10) 18% at ~6x the encode and decode
time of WebP preset 9, without default browser support. `image-webp` decodes libwebp's output exactly
and fast, so only encoding needs the C library (`libwebp-sys`, MIT; statically
built from vendored libwebp BSD-3 sources with `cc`, no bindgen/libclang).

## Verification

Disposable contracts cover legacy reads, shared storage and size growth, exact
image/environment/timeline/history preservation, selective acceptance/pruning,
Git object IDs, dry-run/no-op, stale review rejection, malformed/conflicting/
missing/orphan provenance, locks, pending writes, hash drift and invalid/extra
images. CLI contracts exercise explicit arguments, human/JSON output and config-
independent operation. No reference fixture is automatically accepted in a real
consumer.

Reference-format contracts: exact/deterministic codec round trips, fingerprint
coverage of every sample and the dimensions, rejection of lossy/extended/truncated/
trailing/foreign files; conversion dry-run/write/idempotence/back-conversion with
retained environment/history; findings and diff images identical to the PNG path;
WebP acceptance with exact change detection and PNG report assets; `accept --all`,
sequences and a native-layer collection suite in WebP; tampered or lossy references
failing closed; locks, pending writes, hash drift and stray files; schema-3 field
rules; PNG stores staying schema 2 with exact bytes; configured new-store format
and mismatch refusal; CLI conversion.
