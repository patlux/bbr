# Standalone extraction

## Contract

Patrick approved extracting the portable tool into `Personal/gpui-visual` with
“doch bitte auslagern”. Local repository only; no remote or publication. This
is the first bounded slice of goal `1790607331142-4a53bba10bf7a8`, not completion
of the Rust-first consumer migration.

Source: GPUI Mobile Kit commit
`7263178ffcb9921990aadd34bfe8d6918426cbc7`, path `tools/cargo-gpui-visual/`.
Imported via immutable Git archive, inspected in `/tmp` before applying. Original
package file hashes are recorded in [provenance/extraction.json](provenance/extraction.json).
SDK source history is preserved in its repository; this is a new extraction
commit, not rewritten SDK history or a claim to have imported that full history.

## Boundary

Move the CLI/library, original embedded report, contract tests, Cargo lock,
Apache LICENSE and NOTICE. Keep package/binary name `cargo-gpui-visual`, version
0.1.1, API, schemas, exit codes and dependencies unchanged. No source/asset/test
behavioral changes in this extraction. No new plugin or renderer framework.

The GPUI-bound `gpui-mobile-kit-visual` helper and eight production cover fixtures
stay in the SDK. Consumer GPUI pins, native code, baselines and ownership remain
unchanged. New captures, multi-suite report and Rust orchestration/API work are
subsequent slices, not implied by passing portable tests.

Development tooling uses the same Rust 1.98.1 and Nix input revisions as the SDK,
with only host components (no mobile std targets). No SDK path, materialization,
GPU, mobile SDK or Python is needed by the portable package.

## Consumption

Until separately authorized publication, consumers import reviewed immutable
Git exports with source revision and complete hashes. The SDK has replaced its
owned `tools/cargo-gpui-visual` tree with a pinned `vendor/gpui-visual` export.
That directory is dependency source, not a second development authority. Changes
land here first; update the consumer import in a separately verified commit.
There is no build-time sibling checkout override or dependency on an absolute
local path. Other consumers keep their current pinned tools until their own
migration/ownership gates pass.

Source-policy path changes require fresh captures. Never rewrite old capture
seals, baseline environments or references to make the move appear green.

## Gates

- All imported Rust, original review assets, existing tests, Cargo manifest/lock
  and LICENSE byte-identical to the immutable SDK source.
- Standalone `mbx test --locked --offline`, strict all-target Clippy and rustfmt.
- Embedded original report/selection/security behavior covered by the unchanged
  38 contract tests; portable metadata resolves no GPUI or SDK path dependencies.
- SDK pinned import byte/hash verification, portable tests and read-only doctor;
  renderer/fixture/baseline/upstream bytes unchanged. Old seals stay invalid on
  changed source policy; no acceptance or new native evidence claimed.
- Record commands and measured outcomes before committing; no remote configured.

## License provenance

The existing Apache-2.0 grant travels with the extracted files, not a new license
for Hackerschau or GPUI dependencies. [sdk-notice.txt](provenance/sdk-notice.txt),
[sdk-licensing.md](provenance/sdk-licensing.md) and
[hackerschau-import.json](provenance/hackerschau-import.json) are immutable
historical records copied from the source revision. Relative links inside the
archived SDK licensing record retain their original SDK meaning. Its broader
SDK/helper/distribution findings are not portable CLI dependency claims.

## Evidence

Fresh standalone gates on 2026-09-28, with selected Xcode
`/Applications/Xcode-27.0.0-Beta.6.app/Contents/Developer` and no SDK wrapper:

- `timeout --kill-after=10s 300s scripts/with-rust-toolchain.sh mbx test --locked --offline`:
  **38/38**, zero failures/ignored; measured 12.503s total, 1.66s tests.
- Same bounded wrapper, `mbx clippy --locked --offline --all-targets -- -D warnings`:
  pass, measured 4.084s.
- 120s bounded wrapper, `cargo fmt -- --check`: pass, measured 1.033s.
- `cargo metadata --locked --offline --filter-platform aarch64-apple-darwin --format-version=1`:
  69 resolved host nodes; no GPUI or external SDK path dependency. Unfiltered
  metadata initially failed on uncached Windows-only `anstyle-wincon`; no fetch,
  lock update or cross-platform verification claim. The host graph/build passed.
- All 14 original files accounted for; Rust, assets, tests, manifest/lock and
  LICENSE byte-identical. Only README/NOTICE adapted; original NOTICE archived.
  Current README/extraction links, wrapper shell syntax and staged whitespace pass.
- No remote configured. No native/GUI capture, baseline write or acceptance.

Logs: `/tmp/gpui-visual-extract-{tests,clippy,fmt}.log`; host metadata
`/tmp/gpui-visual-extract-metadata.json`.
Historical pre-extraction baseline remains `/tmp/gpui-visual-rust-api-before.log`.

### SDK consumer handoff

Standalone implementation commit: `94f4437981ed2e29112e8cf7af71362b9e591e32`.
SDK import commit: `2f6203ab36c1bb4af85db4e69c2ecbbdac42f59c`.
The SDK pins that exact standalone commit, not later documentation/development.
All 26 exported tracked files match it; `vendor/gpui-visual-source.json` records
revision and complete SHA-256 payload. The former owned tools tree is removed.

Fresh SDK-wrapper gates, `timeout --kill-after=10s` (300s test/Clippy, 120s others):

- `mbx test --manifest-path vendor/gpui-visual/Cargo.toml --locked --offline`:
  **38/38**, zero failures/ignored, measured 7.672s total and 1.80s tests.
- `mbx clippy --manifest-path vendor/gpui-visual/Cargo.toml --locked --offline --all-targets -- -D warnings`:
  pass, measured 2.232s.
- `cargo fmt --manifest-path vendor/gpui-visual/Cargo.toml -- --check`: pass.
- `mbx run --manifest-path vendor/gpui-visual/Cargo.toml --locked --offline -- doctor`:
  pass, 175 source inputs, no producer/device/server started.
- Same run prefix, `validate --run reports/visual/helper-final-b`: expected exit 2,
  `Capture source inventory changed; capture again`. Old manifests untouched.
- Exact byte checks against SDK source revision cover helper, fixtures/baselines,
  root Cargo manifest/lock, upstream lock and patches. Links and diff checks pass.

Logs `/tmp/gpui-visual-sdk-import-{tests,clippy,fmt,doctor,stale}.log`. BBR,
Hackerschau and Respawn were not written. No baseline/environment adoption, native
capture, GPU use or public release. This closes extraction only, not R1–R7.
