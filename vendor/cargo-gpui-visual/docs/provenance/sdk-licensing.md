# Licensing and third-party boundaries

## SDK license

SDK-owned source, documentation, build tooling, examples and tests are licensed
under the **Apache License, Version 2.0**, unless a file explicitly states
otherwise. The full license is in [LICENSE](../LICENSE); attribution and source
origins are summarized in [NOTICE](../NOTICE).

This license permits commercial use and proprietary applications, subject to
its terms. It is not a promise that every dependency in a particular build has
the same terms. Third-party code, copied upstream material in patches, bundled
assets and downloaded dependencies retain their own licenses and notices.

`publish = false` remains in the Cargo manifests. It disables Cargo registry
publication; it is not a license restriction.

## Approved touch-test imports

Patrick explicitly approved Apache-2.0 for the SDK's imported touch tests in
response to the licensing plan ("2. ja apache-2.0"). The approval covers the six
SDK-local Hackerschau test copies listed in [touch-import.json](touch-import.json).
The `gpui-sdk-touch-tests` package now declares Apache-2.0, as do the other
SDK-owned packages.

The original Hackerschau commit, source paths, hashes and `UNLICENSED` source
declaration remain recorded as **historical provenance**. This decision does
not relicense Hackerschau as a whole, its unchanged source checkout, other
application code, or any upstream implementation included by a test harness.
The frozen consumer inventories must continue to describe their original
commits rather than retroactively adopting the SDK license.

## Approved visual-tooling extraction

Patrick explicitly approved the proposed Apache-2.0 license with "1. ja dann
nehmen wir das", following the recommendation to license the reusable extracted
CLI/report/helper and their associated tests and fixtures. This is a separate
approval from the earlier touch-test decision and covers these SDK-local paths:

- `tools/cargo-gpui-visual/`: CLI, embedded HTML-report implementation and CSS/JS
  assets, contract tests and package documentation.
- `crates/gpui-mobile-kit-visual/`: host renderer helper and package documentation.
- `tests/visual/`: SDK visual-test harness, generated cover fixture and reference
  images. These are SDK fixtures, not Hackerschau's application assets/baselines.

All three Cargo packages declare `license = "Apache-2.0"`. The CLI and helper
include the full Apache license and package-local NOTICE files retaining
`Copyright (c) 2025-present piparo.tech GmbH` and the original import revision.
The SDK test crate is covered by the root LICENSE/NOTICE.

[visual-import.json](visual-import.json) remains unchanged: its proprietary source
license, revision and hashes describe the **historical import**, not the current
SDK-copy license. This approval does not relicense Hackerschau as a whole, change
its checkout, or relicense any upstream dependency or separately licensed asset.
Previously sealed private archives and the consumer's pinned archive/hash remain
unchanged; they retain their recorded metadata and notices. This change does not
claim that those archive bytes have been replaced or revalidated.

Publication was explicitly deferred ("2. noch nicht"). All three packages remain
`publish = false`; no push, registry upload or release is authorized. Apache-2.0
itself is not restricted to private use: the restriction here is on agent actions
and the current unpublished delivery state, not an additional license term.
Transitive GPUI licensing findings below and native acceptance remain separate.
[Technical/permission audit](visual-completion-audit.md).

## Pinned third-party sources

Exact revisions and ordered patch hashes are in
[upstream.lock.toml](../upstream.lock.toml). This review concerns these pins,
not later upstream versions:

| Source | Revision | License treatment |
| --- | --- | --- |
| [Zed](https://github.com/zed-industries/zed) | `5688167d224b5eca54875d49afb8bfd73a07915a` | Per-crate licenses; GPUI, gpui_wgpu and sum_tree declare Apache-2.0. No repository-wide Apache assumption. |
| [gpui-mobile](https://github.com/itsbalamurali/gpui-mobile) | `1d3ec2a1d14a63b74d1f4269340441d4eeada27a` | Upstream offers `GPL-3.0-or-later OR AGPL-3.0-or-later OR Apache-2.0`; this SDK uses the Apache-2.0 option for that package. |
| [Taffy](https://github.com/DioxusLabs/taffy) | `599b1b7d7410d97e13bafc4c01fbe6d7a9cf060a` (0.9.0) | MIT; original `LICENSE.md` and copyright retained in the materialized source. Same source as the former registry dependency, with the hashed lazy-minimum patch. |
| [stacker](https://github.com/rust-lang/stacker) | `cdf19cf36cd0343a97c174ee6f7f8241f88b35f0` | Upstream offers `MIT OR Apache-2.0`; this SDK uses the Apache-2.0 option for that package. |
| Gradle wrapper | `v8.13.0` | Existing license and bundled attributions retained in [hosts/android/gradle/LICENSE](../hosts/android/gradle/LICENSE); [wrapper provenance](../hosts/android/README.md). |

Selecting an offered license is not relicensing upstream. It applies only to
the material covered by that offer, not to separately licensed dependencies or
assets. Existing upstream license files and copyright notices are unchanged.
`gpui_shared_string` and `gpui_util` omit Cargo license metadata but each contains
a Zed `LICENSE-APACHE`; absence of metadata is not absence of a license.

## Unresolved Zed tracing dependency

The SDK depends on GPUI. GPUI's pinned upstream manifests, not a direct SDK
logging dependency, introduce this path on both mobile target graphs:

```text
gpui → sum_tree → ztracing → zlog
                         → ztracing_macro
```

- `zlog` declares `GPL-3.0-or-later` and contains `LICENSE-GPL`. No alternative
  license grant was identified in that crate during this review.
- `ztracing` and `ztracing_macro` declare `GPL-3.0-or-later`, but both also contain
  `LICENSE-APACHE` and `LICENSE-AGPL` in the pinned upstream source. The Apache
  files include a Zed copyright/license notice. This discrepancy requires
  clarification; neither a confirmed Apache option nor an exclusively GPL
  grant is inferred from it here.
- `sum_tree` uses `ztracing::instrument`. Without `cfg(ztracing)`, this resolves
  to the procedural macro that returns its input unchanged. The `zlog` call in
  `ztracing::init` is gated by `cfg(ztracing)`. The upstream build script enables
  this cfg when `ZTRACING` or `ZTRACING_WITH_MEMORY` is set.

A resolved Cargo graph is not evidence of the exact code incorporated into a
particular binary. Build-time tools, generated code, enabled configurations
and linked library code need separate examination. This observation is **not**
a finding that the current builds have no copyleft obligations. No dependency
was removed, upstream manifest rewritten, or final-binary legal review completed
as part of this licensing change.

[License inventories](inventory/README.md) record the declarations without
silencing or overriding the unresolved findings. Recheck them with:

```sh
python3 scripts/inventory_licenses.py --check
cargo tree --manifest-path tests/platform-graph/Cargo.toml --frozen \
  --target aarch64-apple-ios-sim --edges normal,build --invert zlog
cargo tree --manifest-path tests/platform-graph/Cargo.toml --frozen \
  --target aarch64-linux-android --edges normal,build --invert zlog
```

## Publication and application distribution

Publishing this source repository and distributing an application binary are
separate reviews. GPL does not prohibit public source releases; applicable
license terms still have to be met. A README warning alone does not satisfy
those terms, and a dependency declaration does not automatically relicense
every file in this repository.

`.upstream/` is derived and ignored. A Git push does not include those full
checkouts, but it does include the tracked patches and imported sources in the
published history. Their provenance and applicable notices still matter.

Remaining work before publication or an unrestricted proprietary-app claim:

1. Clarify the tracing license discrepancy with upstream, or review a technical
   change that avoids the dependency; verify the relevant build graphs and
   artifacts before drawing conclusions about application licensing.
2. Review all material actually distributed, including patch contents, non-Cargo
   dependencies/assets and required notices. The root NOTICE is not a complete
   notice bundle for a compiled application.
3. Review the intended public files and Git history for private or sensitive
   information; complete clean-checkout verification and obtain explicit
   publication approval. No remote, push, release or app-store upload is
   authorized by this local licensing preparation.

This document records the project decision and technical evidence. It is not a
legal opinion or a completed distribution-compliance audit.
