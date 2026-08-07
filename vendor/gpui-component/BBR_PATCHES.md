# BBR vendored port of `gpui-component 0.5.1`

## Why this vendor exists

BBR pins GPUI to the GPUI-CE commit
`568271c01c5a9d640d7b091e5bfd136b04035f28` (see the workspace
`Cargo.toml` and `docs/roadmap/tasks/gpui-ce-components-anchor.md`).
The crates.io `gpui-component 0.5.1` sources do not compile against that
pin (72 errors: focus-arity deltas, the `Corner`→`Anchor` rename, removed
`gpui::Timer`, `Bounds::from_corner_and_size`, `TextStyleRefinement` and
`Corners::all(bool)` bound changes, `BoxShadow.inset`, `Line::paint`, …),
and upstream `main` (0.5.2-dev) requires post-pin GPUI APIs
(`flex_grow_1`, `restrict_scroll_to_axis`, container queries, …).

This directory is the exact crates.io `gpui-component 0.5.1` source with
only mechanical API-compatibility deltas applied so it builds against the
pinned GPUI-CE. There is **no redesign, no feature removal, and no
behavior weakening**: every module, struct, trait, and method that
existed upstream still exists with the same semantics. The optional
`webview` feature (Wry) stays disabled; BBR keeps its native WKWebViews.

## Provenance

- Upstream crate: `gpui-component 0.5.1` (crates.io), Apache-2.0.
- Upstream source checksum (crate tarball):
  `d021d46b4088d3d93a57ccdf443da85695a77272108caca2f6fe5369f584966a`
- GPUI-CE pin this port compiles against:
  `568271c01c5a9d640d7b091e5bfd136b04035f28` (gpui-ce/gpui-ce).
- Staging: pristine crate extracted under `/tmp`, `Cargo.lock` and
  `.cargo_vcs_info.json` removed, deltas applied, then copied here.
  `LICENSE-APACHE` is the upstream license file, unchanged.

The workspace wires this directory through:

```toml
gpui-component = { version = "=0.5.1", default-features = false }
[patch.crates-io]
gpui-component = { path = "vendor/gpui-component" }
```

so cargo still resolves one `gpui-component 0.5.1` whose `gpui = "0.2.2"`
requirement is redirected (via the workspace `[patch.crates-io] gpui`) to
the same GPUI-CE git source as the direct `gpui` dependency — one gpui
type world.

## Mechanical-delta inventory

All changes are compile-compatibility fixes for the GPUI-CE pin. Each
class below preserves the original behavior.

### Focus APIs (`Window::focus`, `FocusHandle::focus` gained `&mut App`)

- `src/input/state.rs`, `src/input/otp_input.rs`, `src/input/search.rs`,
  `src/time/date_picker.rs`, `src/popover.rs`, `src/root.rs`,
  `src/select.rs`, `src/list/list.rs`, `src/menu/context_menu.rs`,
  `src/menu/dropdown_menu.rs`, `src/menu/app_menu_bar.rs`,
  `src/menu/popup_menu.rs`, `src/dock/tab_panel.rs`: append the `cx`
  argument to every `focus_handle.focus(window)` /
  `window.focus(handle)` call.
- `src/root.rs`: the `on_action_tab` / `on_action_tab_prev` handlers and
  `focus_back` rename the `_: &mut App` parameters to `cx` and pass them
  to `window.focus_next(cx)` / `window.focus_prev(cx)` /
  `window.focus(&handle, cx)`.
- `src/input/otp_input.rs`: `focus` and the mouse-down handler rename
  their `_: &mut Context<Self>` parameters to `cx`.
- `src/input/search.rs`: the four `read(cx).focus_handle.focus(window,
  cx)` sites clone the `FocusHandle` first so the immutable `read` borrow
  ends before the mutable `cx` borrow (`focus` takes `&mut App`).
- `src/menu/popup_menu.rs`: `Anchor::other_side_corner_along(axis)` is
  replaced by the four-corner `flip_vertical_corner` helper (see the
  `Anchor` section below).

### `Anchor` (renamed `Corner`, gained center variants)

- `\bCorner\b` → `Anchor` across the crate (`styled.rs`, `dialog.rs`,
  `popover.rs`, `color_picker.rs`, `menu/*`, `date_picker.rs`, …).
- `src/popover.rs`, `src/color_picker.rs`, `src/menu/popup_menu.rs` keep the
  exact upstream 0.5.1 four-corner contract and nothing more. GPUI-CE's new
  `Anchor` center variants (`TopCenter`, `BottomCenter`, `LeftCenter`,
  `RightCenter`) are rejected loudly (`panic!`) instead of being assigned an
  invented geometry or keyboard meaning:
  - `popover::resolved_corner` and `color_picker::resolved_corner` reproduce
    the upstream `Corner` flip (`TopLeft→BottomLeft`, `TopRight→BottomRight`,
    `BottomLeft→TopLeft`, `BottomRight→TopRight`) via a private
    `flip_vertical_corner` helper; center variants panic.
  - The popover/color-picker top/bottom margin classification matches keep
    only the four corner arms; center variants panic.
  - `popup_menu::parent_side` keeps the upstream `Corner::TopLeft|BottomLeft
    → Side::Left` / `TopRight|BottomRight → Side::Right` classification via a
    private `corner_side` helper; center variants panic (no invented keyboard
    side). `update_submenu_menu_anchor` routes its corner-only submenu flip
    through `flip_vertical_corner`, exactly reproducing upstream
    `other_side_corner_along(Axis::Vertical)` for the only anchors that path
    ever produces (`TopLeft`/`TopRight`).
  - The earlier draft used GPUI-CE `Anchor::other_side_along(Axis::Vertical)`
    and added center-variant arms to the margin/side matches; that invented
    semantics (e.g. `LeftCenter`/`RightCenter` staying put under a vertical
    flip, `TopCenter` getting a top margin, `LeftCenter` getting a keyboard
    side). This was removed by remediation in favor of the strict
    four-corner contract above.

### Removed `gpui::Timer` (use `BackgroundExecutor::timer -> Task<()>`)

- `src/input/blink_cursor.rs`: `Timer::after(d).await` →
  `cx.background_executor().timer(d).await`; the infallible GPUI-CE
  `AsyncApp::update` no longer needs `.ok()`.
- `src/scroll/scrollbar.rs`: the idle-timer spawn awaits
  `cx.background_executor().timer(next_delay)` (mirrors `img.rs` in the
  pin).
- `src/text/text_view.rs`: `UpdateFuture` stored a `gpui::Timer`
  (`Timer::never()` + `set_after` + `poll_next`). It now stores a
  `gpui::BackgroundExecutor` plus an `Option<Task<()>>` that is re-armed
  on each change and polled via `Pin::new(timer).poll(cx)`; a completed
  task is dropped before the parse/send. Same debounce semantics.

### Geometry / style deltas

- `Bounds::from_corner_and_size(corner, origin, size)` removed:
  `src/scroll/scrollbar.rs` thumb bounds are built with `Bounds::new`
  after shifting the origin for the `TopRight`/`BottomLeft` anchors
  (x − width / y − height), identical to the removed helper.
- GPUI-CE `max_offset()` returns `Point<Pixels>` (was `Size`):
  `src/scroll/scrollbar.rs` `ScrollbarHandle::content_size` converts with
  the compiler-suggested `.into()` conversions (pointwise identical).
- `track_scroll` now takes `&UniformListScrollHandle`:
  `src/table/state.rs`, `src/tree.rs` borrow the clone.
- `BoxShadow` gained `inset: bool`: `src/styled.rs`,
  `src/window_border.rs`, `src/dialog.rs` set `inset: false`.
- `Corners::all` now requires `T: Add + Half` (bool no longer
  satisfies it): `src/button/button.rs` and `src/button/button_group.rs`
  build `Corners<bool>` struct literals instead.
- `Style.text` became a `#[refineable]` field, so
  `StyleRefinement.text` is a `TextStyleRefinement` (not `Option`):
  `src/slider.rs` reads `.text.color` directly instead of
  `.and_then(|text| text.color)`.
- `Line::paint` gained `align`/`align_width`:
  `src/input/element.rs`, `src/input/text_wrapper.rs` pass
  `TextAlign::default(), None` (the values the crates.io `Line::paint`
  used internally).

### Import hygiene (warning-free under `-D warnings`)

- Removed unused `PixelsExt` imports in `src/chart/*`,
  `src/plot/*`, `src/virtual_list.rs`, `src/styled.rs`,
  `src/input/element.rs`, `src/resizable/mod.rs`, `src/webview.rs`.
- Removed the now-unused `Anchor` import in `src/scroll/scrollbar.rs`.

## What is NOT patched

- No module, public API, feature flag, or behavior is removed or
  redesigned. `default-features = false` is the same dependency
  declaration BBR always used; the `webview`/`wry` feature is untouched
  and simply not enabled.
- No GPUI-CE code is copied in; the pin remains a git dependency patched
  from crates.io.

## Focused unit tests (BBR addition)

`src/popover.rs`, `src/color_picker.rs`, and `src/menu/popup_menu.rs` each
carry a `#[cfg(test)]` module proving the four-corner mappings equal the
upstream 0.5.1 contract and that every center variant is rejected loudly
(including the submenu keyboard side in `popup_menu`). To run them from the
BBR workspace, the vendored `Cargo.toml` declares an empty `[workspace]`
table (standalone root) plus its own `[patch.crates-io] gpui` pointing at the
same GPUI-CE pin, so the test build stays on one gpui type world:

```bash
cargo test --manifest-path vendor/gpui-component/Cargo.toml --lib
```

When the BBR workspace builds, that table is inert (only the BBR root
manifest's patches apply). The test run generates a `Cargo.lock` and
`target/` inside `vendor/gpui-component/`. `target/` is ignored
workspace-wide (`**/target/`); the standalone `Cargo.lock` is a harmless
byproduct — the workspace `Cargo.lock` at the repo root governs every
workspace build, and this standalone lock never participates in it. BBR
excludes that generated lockfile from the vendored source commit through
the vendor-local `.gitignore`, preventing isolated test runs from creating
tracked lockfile drift.

## Verification (as of this ledger)

- `cargo check -p bbr` and `cargo check -p bbr --features testkit
  --all-targets` are green on the pin.
- `cargo clippy -p bbr --features testkit --all-targets -- -D warnings`
  is green.
- `cargo tree -e features -p gpui-component` shows no `webview`/`wry`.
- Real-app E2E (smoke, profiles, command, sidebar, split geometry,
  tabs/spaces suites) runs through this vendored component under the
  `Root` composition.
