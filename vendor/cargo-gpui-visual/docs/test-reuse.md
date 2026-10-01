# Test infrastructure reuse at the current consumer pins

Decision checkpoint for migration goal `1790607331142-4a53bba10bf7a8`,
2026-09-28. This is a source/API evaluation, not native accessibility validation
or a successful kittest runtime integration. No dependency or consumer pin changed.

## Keep ordinary Rust and GPUI tests

The SDK now shares its `CoverScenario`, semantic `present` action, named moments,
measured generation and independent eight-case requirement between ordinary
`#[gpui::test]`/`#[test]` coverage and an ignored Metal capture candidate. SDK commit
`212f5ef3b894d99875cfc7112716095b3ed5c50e` passes seven non-GPU tests; both Metal
captures remain ignored pending host ownership and real pixel parity.

BBR already uses `TestAppContext`/`VisualTestContext` in
`crates/bbr-gallery/src/harness.rs`: production `gpui_component::Root` setup,
semantic snapshots, actual debug-selector bounds and pointer/keyboard dispatch.
Its browser has the existing real-app semantic testkit. Preserve those routes;
scenario extraction should reuse them rather than invent another runner.

- **libtest-mimic:** do not add now. Eight fixed SDK cases and BBR's existing
  registry/test harness do not establish a missing dynamic-test requirement.
  The source-bound catalog handles report coverage, not test execution.
- **nextest:** do not add or replace canonical gates. Test concurrency groups
  cannot replace a host-wide lease across independent commands and native tools.
- **Insta:** no image-baseline role. The current typed assertions are sufficient
  for this slice; adding a second snapshot store would duplicate authority.

These decisions can be revisited for a concrete demonstrated gap, not as a
prerequisite to the migration.

## kittest / AccessKit: no integration at these test seams

[kittest 0.4.0](https://docs.rs/kittest/0.4.0/kittest/) declares AccessKit `^0.24.0`
and accesskit_consumer `^0.35.0`. Its
[State implementation](https://docs.rs/kittest/0.4.0/src/kittest/state.rs.html)
requires an initial real `TreeUpdate` and subsequent per-frame updates. Version
compatibility alone cannot supply the tree or route actions back to the view.

### BBR: matching types, absent test-platform bridge

Rechecked BBR's Cargo declarations and lock: GPUI-CE
`568271c01c5a9d640d7b091e5bfd136b04035f28`, AccessKit0.24.1 and
accesskit_consumer0.35.0/0.38.0. The locally inspected GPUI checkout is that exact
commit; inspected files have no working diff.

At that revision:

- `crates/gpui/src/platform.rs:629` defines `A11yCallbacks`; `:768` supplies
  default **no-op** `PlatformWindow::a11y_init` and `a11y_tree_update` methods.
- `crates/gpui/src/platform/test/window.rs` neither overrides those methods nor
  stores callbacks/tree updates. Its available simulated-input API is the
  existing pointer/keyboard path, not an AccessKit adapter.
- `crates/gpui/src/window.rs:1333` starts accessibility inactive. The platform
  activation callback enables the real tree; drawing sends updates to
  `platform_window.a11y_tree_update` (`:2805`).
- Real actions enter the registered platform callback/channel and reach
  `handle_a11y_action` (`:5624`), which is `pub(crate)`. Public
  `on_a11y_action` registers listeners; it does not let a consumer inject a
  request into that dispatcher.
- `HeadlessAppContext` constructs the same `TestPlatform`; injecting a text
  system or renderer does not replace its accessibility adapter.

Therefore a consumer-only kittest query/action test cannot observe the real
Accessibility tree through the current ordinary GPUI test contexts. Hand-building
an AccessKit tree from BBR snapshots would fabricate a second semantics authority,
not prove production tree updates. An upstream test-platform bridge or a separately
owned native adapter would be a distinct prerequisite; neither is added here.

### SDK: no AccessKit tree at its GPUI pin

SDK's GPUI is Zed `5688167d224b5eca54875d49afb8bfd73a07915a`, with the canonical
SDK patches. Scoped search of the exact materialized GPUI crate, upstream mobile
source and SDK navigation-GPUI source found no `accesskit`, `a11y_init` or
`a11y_tree_update` integration. The cover itself is explicitly a paint-only
production overlay, with no accessibility target. Its measured-image fixture
cannot serve as a meaningful AccessKit action proof.

Do not force the SDK onto BBR's different GPUI graph or infer that its separate
UIKit/Android/native accessibility work is absent or verified from this finding.

**Decision:** retain existing semantic IDs/actions and GPUI tests. No kittest
dependency, fabricated tree, pin upgrade, cache patch or broad custom platform
implementation. The real-tree/action proof was not executed: the required public
test seam is absent. This is an evidence-backed non-adoption decision, not a claim
that GPUI-CE lacks native accessibility or that integration is universally impossible.
