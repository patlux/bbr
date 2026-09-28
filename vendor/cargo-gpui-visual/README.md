# cargo-gpui-visual

Portable Rust CLI: exact RGBA comparison, static HTML review, sealed-source validation
and selective acceptance. No Python/Pillow runtime, GPU, GPUI dependency, web server,
network access or browser launch is needed for compare/accept. Git is required for
source inventory. The consumer's capture command owns its renderer and build tools.

## Local use (not published)

```sh
# On this machine use mbx for compiler-invoking commands.
mbx run --manifest-path tools/cargo-gpui-visual/Cargo.toml --release -- check
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
capture = ["cargo", "test", "--locked", "--test", "visual", "--", "--ignored"]
environment_command = ["my-environment-probe"] # prints JSON
baselines = "tests/visual/baselines"
report = "reports/visual/latest"
runs = "reports/visual/runs"
[source_policy]
scopes = ["src", "tests/visual", "Cargo.toml", "Cargo.lock", "visual.toml"]
suffixes = [".rs", ".toml", ".lock", ".png", ".svg"]
```

The producer gets `GPUI_VISUAL_OUTPUT`, writes PNGs and `inventory.json` (a list of
IDs). Each ID is `<case>--light` or `<case>--dark`; both appearances are required.
The CLI probes environment and sources before/after and seals `capture.json`.
Alternatively an existing producer can seal schema 1 itself; omit
`environment_command`. Its source inventory must equal the configured policy.
`{run}` in capture argv is substituted with the absolute run path.

```sh
cargo gpui-visual check                 # fresh unique run, never overwrite
cargo gpui-visual compare --run <run>
cargo gpui-visual accept --run <run> --case example--light --case example--dark
cargo gpui-visual doctor                # read-only config/source preflight
```

Compare and accept also work without configuration with explicit `--repo`, `--run`,
`--baselines`, `--report`. `--adopt-environment` requires a reviewed mismatch; every
unselected existing reference must be byte-identical. No accept-all, tolerance or
mask option. Exit 0 = exact match/explicit successful operation; 1 = review required;
2 = invalid capture/configuration. Reports and baseline PNGs are ordinary files.
No command commits or pushes.

Source discovery includes untracked, non-ignored relevant files and explicitly
selected submodule HEADs plus their working sources. Use narrow scopes: never
include secret directories. `.env*`, symlinks and escaping paths are refused or
excluded; baseline paths are excluded. Derived upstreams must be prepared and
validated before capture; include their canonical lock/patch inputs in the policy.

Starting with 0.1.1, `[source_policy].artifacts` can explicitly list narrow
repository-relative ignored package directories. Their complete file contents,
including newly added files, join the source seal. Missing paths, symlinks,
secret-name paths and baseline scopes are refused. This is for independently
verified derived packages, not broad filesystem discovery. Changing such a package
after capture invalidates comparison and acceptance even though Git ignores it.

Hash manifests guard accidental/stale input changes; they do not authenticate a
human review. Acceptance validates all selected payloads first, uses a cooperative
lock and per-file atomic replacement. An interrupted multi-file write fails closed
through metadata mismatch; it is not a transactional filesystem guarantee.

## License

**Apache-2.0**; see [LICENSE](LICENSE) and [NOTICE](NOTICE). Patrick explicitly
approved this SDK-local CLI/report extraction and its tests. The report assets
and adapted behavior originate in Hackerschau at
`b1528bd21a728da3a110074fb88b635a03f15c8c`; copyright and historical source-license
provenance are retained, not a blanket relicense of the application. Dependencies
retain their own licenses. See `../../docs/licensing.md` and
`../../docs/visual-import.json` in the SDK checkout for the approval and import
hashes. Publication remains deferred; `publish = false` is unchanged.
