set shell := ["bash", "-euo", "pipefail", "-c"]

# Keep local verification finite. Override only for an intentionally longer run.
check_timeout := env_var_or_default("BBR_CHECK_TIMEOUT", "5m")
test_timeout := env_var_or_default("BBR_TEST_TIMEOUT", "5m")
e2e_timeout := env_var_or_default("BBR_E2E_TIMEOUT", "2m")
e2e_full_timeout := env_var_or_default("BBR_E2E_FULL_TIMEOUT", "15m")
# The dense suite seeds hundreds of real WKWebView sessions through the
# production controller open path (each transient session is destroyed before
# the next open), so it legitimately needs more than the 2-minute interaction
# budget; the per-wait deadlines inside the suite stay bounded.
e2e_dense_timeout := env_var_or_default("BBR_E2E_DENSE_TIMEOUT", "8m")
# The web-content navigation-gestures suite drives nine real-app scenarios
# (each spawns the real app, settles the window, and runs real WKWebView
# history navigations), so it needs a larger bounded budget than the
# 2-minute interaction default; the per-wait deadlines inside the suite stay
# bounded.
e2e_gestures_timeout := env_var_or_default("BBR_E2E_GESTURES_TIMEOUT", "8m")
run_timeout := env_var_or_default("BBR_RUN_TIMEOUT", "8h")
xcode_developer_dir := env_var_or_default("BBR_XCODE_DEVELOPER_DIR", "/Applications/Xcode-27.0.0-Beta.6.app/Contents/Developer")
# Select the entire Apple toolchain, not only xcrun's developer directory:
# Nix shells can otherwise retain an older SDKROOT and compiler/linker.
export DEVELOPER_DIR := xcode_developer_dir
export SDKROOT := shell('DEVELOPER_DIR="$1" /usr/bin/xcrun --sdk macosx --show-sdk-path', xcode_developer_dir)
export CC := shell('DEVELOPER_DIR="$1" /usr/bin/xcrun --sdk macosx --find clang', xcode_developer_dir)
export CXX := shell('DEVELOPER_DIR="$1" /usr/bin/xcrun --sdk macosx --find clang++', xcode_developer_dir)
export CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER := CC
export CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER := CC
gui_gate := "python3 scripts/with_gui_lock.py --"

# Launch the development build as a real macOS app through mbx-managed build
# storage. Cargo hands the built binary to the dev_app.py runner, which wraps
# it in target/debug/BBR.app (bundle identity, Phosphor-b Dock icon, BBR menu
# bar; a bare binary gets none of these) and execs the bundle executable, so
# logs and Ctrl-C stay in this terminal. Extra arguments reach the app:
# `just run --bbr-initial-url https://example.com`. Override BBR_RUN_TIMEOUT
# for longer sessions. Keep mbx in the terminal foreground: its interactive
# startup otherwise receives a job-control stop from the TTY.
dev_runner := "python3 " + justfile_directory() + "/scripts/package/dev_app.py --exec"
run *args:
    DEVELOPER_DIR="{{xcode_developer_dir}}" CARGO_TARGET_AARCH64_APPLE_DARWIN_RUNNER="{{dev_runner}}" CARGO_TARGET_X86_64_APPLE_DARWIN_RUNNER="{{dev_runner}}" timeout --foreground --kill-after=10s {{run_timeout}} mbx run -p bbr {{ if args == "" { "" } else { "-- " + args } }}

fmt:
    timeout --kill-after=10s {{check_timeout}} cargo fmt --all -- --check

check:
    DEVELOPER_DIR="{{xcode_developer_dir}}" timeout --kill-after=10s {{check_timeout}} cargo check --workspace --all-targets

test:
    DEVELOPER_DIR="{{xcode_developer_dir}}" timeout --kill-after=10s {{test_timeout}} cargo test --workspace

clippy:
    DEVELOPER_DIR="{{xcode_developer_dir}}" timeout --kill-after=10s {{check_timeout}} cargo clippy --workspace --all-targets -- -D warnings

roadmap:
    timeout --kill-after=10s {{check_timeout}} python3 scripts/check_roadmap.py

ui-deps:
    timeout --kill-after=10s {{check_timeout}} python3 scripts/check_ui_dependencies.py

script-tests:
    timeout --kill-after=10s {{test_timeout}} python3 -m unittest scripts/test_with_gui_lock.py scripts/test_check_roadmap.py scripts/test_check_ui_dependencies.py scripts/test_xcode_environment.py -v
    timeout --kill-after=10s {{check_timeout}} python3 -m py_compile scripts/check_roadmap.py scripts/roadmap_validation_support.py scripts/test_check_roadmap.py scripts/check_ui_dependencies.py scripts/with_gui_lock.py scripts/test_with_gui_lock.py scripts/test_check_ui_dependencies.py scripts/test_xcode_environment.py scripts/e2e_vm.py scripts/tart_vm.py

e2e-smoke:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_smoke -- --ignored --nocapture

e2e-shutdown:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shutdown -- --ignored --nocapture

e2e-tabs:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_tabs -- --ignored --nocapture

e2e-profiles:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_profiles -- --ignored --nocapture
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_profile_management -- --ignored --nocapture

e2e-shell-tabs:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_tabs -- --ignored --nocapture

e2e-shell-spaces:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_spaces -- --ignored --nocapture
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_spaces_keyboard -- --ignored --nocapture
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_spaces_hint -- --ignored --nocapture
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_spaces_config -- --ignored --nocapture
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_menus -- --ignored --nocapture

e2e-shell-sidebar:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_sidebar -- --ignored --nocapture

e2e-shell-favicons:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_favicons -- --ignored --nocapture

e2e-shell-titlebar-navigation:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_titlebar_navigation -- --ignored --nocapture

e2e-shell-split-geometry:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_split_geometry -- --ignored --nocapture

e2e-shell-organize:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_organize -- --ignored --nocapture
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_drag_chip -- --ignored --nocapture
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_folder_pins -- --ignored --nocapture
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_folder_rename -- --ignored --nocapture
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_folder_menu -- --ignored --nocapture
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_drag_offers -- --ignored --nocapture
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_closed_pins -- --ignored --nocapture
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_drag_back -- --ignored --nocapture

# Tab/Favorite context menus (packet `tab-favorite-context-menus`,
# DESIGN.md §11.13): the production right-click row/favorite seams open one
# menu bound to the exact invoking TabId; open/traverse/submenu/dismiss
# never mutate the source; keyboard/pointer placement and focus semantics;
# folder/space submenu traversal with same-profile WebViewId preservation
# and cross-profile recreation; B2 bookmark parity; the Favorite menu
# resolving the same concrete TabId; supported close/reopen/pin/suspend/
# ordering/copy rows and the omission of unsupported rows (Duplicate Tab,
# Share, split, media, rename/icon, new-window, bulk-close scopes).
e2e-shell-tab-menu:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_tab_menu -- --ignored --nocapture

e2e-shell-hover:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_semantics -- --ignored --nocapture
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_hover -- --ignored --nocapture
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_semantic_branches -- --ignored --nocapture

# Blueprint D16: the Settings window (⌘,), its rows applied at once, the
# preferences file and a relaunch.
e2e-shell-settings:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_settings -- --ignored --nocapture

# Blueprint D10: History (⌘Y) and Clear Browsing Data (⌘⇧⌫).
e2e-history:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_history -- --ignored --nocapture

# Blueprint D11: permission prompts, device marks and site settings.
e2e-permissions:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_permissions -- --ignored --nocapture

# Blueprint D12: the client-certificate sheet.
e2e-client-certificates:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_client_certificates -- --ignored --nocapture

# Blueprint D13: blocked pop-ups, links to other apps, links from other
# apps.
e2e-link-destinations:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_link_destinations -- --ignored --nocapture

# Blueprint D15: the first run (welcome, import, first space).
e2e-first-run:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_first_run -- --ignored --nocapture

# Blueprint D23: split groups.
e2e-split:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_split -- --ignored --nocapture

e2e-windows:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_windows -- --ignored --nocapture

# Reality gaps (item F): persistent website-data stores per profile.
e2e-web-data:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_web_data -- --ignored --nocapture

# Reality gaps (item C): real AppKit events into a shown key window.
e2e-real-input:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_real_input -- --ignored --nocapture

# Blueprint D24: the sidebar media card.
e2e-media:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_media -- --ignored --nocapture

e2e-shell-command:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_command -- --ignored --nocapture

# Saved Pages (B2 packet `saved-pages-actions-command`): Cmd-D toggle
# (idempotent save/remove, no duplicates), the deterministic flush, the
# graceful-restart restore of committed records, the Palette ranking of the
# global Saved Page rows, and the exactly-one-fresh-tab activation in the
# currently active Space/Profile over real WKWebViews.
e2e-shell-saved-pages:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_saved_pages -- --ignored --nocapture

# Saved Pages Library (B3 packet `saved-pages-library`): the integrated
# Cmd-Shift-L entry and Escape dismissal, deterministic search/sort,
# multi-select, inline pointer rename draft confirm/cancel, the destructive
# remove confirmation, the flat collection lifecycle, the selection toolbar
# and collection action surface through their pointer paths, the production
# scroll-wheel path, restart persistence, tab/record independence,
# exactly-one-fresh-tab row activation, and the 1k bounded
# window/scroll/selection/long/missing fallbacks over real
# WKWebViews.
e2e-shell-saved-pages-library:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_saved_pages_library -- --ignored --nocapture

# Productive new-window request seam (NW-SEAM): a real script-created
# window.open reaches the production WKUI delegate and is blocked/reported;
# typed user-activation injection enters the exact app method the native pump
# calls and opens one regular tab in the source Space/Profile.
e2e-shell-new-window:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_new_window -- --ignored --nocapture

# Background link tabs (DD-LINK packet `background-link-tabs`): the explicit
# Command-primary / middle-click background disposition opens one regular
# child tab immediately after the exact opener while the opener stays
# selected, visible, and web-focused; repeated children group in click order
# before unrelated tabs; child activation reuses its already-created session
# and Back-at-root returns to the exact opener; stale sources and profile
# mismatches reject without mutation. Real WKWebViews load deterministic
# background-link fixtures.
e2e-background-links:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_background_links -- --ignored --nocapture

# Web-content navigation gestures (N1 packet
# `web-content-navigation-gestures`): committed logical Back/Forward gestures
# over real WKWebView history navigate the exact pane; Back at a settled
# regular child-tab root closes only that child and presents the exact opener
# (warm reuse, child-history-first, split dissolution, cross-profile
# activation, failure states); the no-op matrix (disabled direction, no/stale
# opener, provisional load, stale view) and the Cmd-W normal-close regression.
e2e-web-navigation-gestures:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_gestures_timeout}} cargo test -p bbr --features testkit --test e2e_web_navigation_gestures -- --ignored --nocapture
# Link Preview / Peek (PEEK): an eligible user-initiated target=_blank
# request opens a transient source-profile preview above the warm origin
# (same WebViewId/URL/page state) without creating a regular tab; Escape /
# close restores the exact warm origin; navigation routes to the preview;
# Open as Tab adopts the SAME preview session without reload; one preview
# per pane replaces; exclusions fall back to ordinary tab; script requests
# stay blocked; crash teardown cleans up.
e2e-peek:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_peek -- --ignored --nocapture

# Persistence + warm-restore (Phase 7, warm-restore slice): chrome-first
# restore of repository-written browser state (identities before sessions,
# visible-set-only hydration, deferred session-less tabs), truthful restore
# disposition evidence, window/sidebar restore, graceful-restart committed-
# state recovery, and force-quit/crash-style recovery of exactly the
# committed repository state over real process lifecycles.
e2e-shell-persistence:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_persistence -- --ignored --nocapture
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_recovery -- --ignored --nocapture

e2e-shell-portal:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_portal -- --ignored --nocapture

e2e-shell-find:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_find -- --ignored --nocapture

# Per-site zoom wiring (queue item 11, part A): the Cmd-Plus/Cmd-Minus/
# Cmd-0 ladder, clamps, reset, same-site/cross-site persistence, and
# tab-switch re-activation over real WKWebViews.
e2e-shell-zoom:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_zoom -- --ignored --nocapture

# BBR context menus over web content (task packet
# `bbr-context-menus-over-web`): pointer + keyboard navigation, genuine
# Back/Forward enablement from real canGoBack()/canGoForward() events,
# dismissal without session mutation, the native bridge diagnostics, and
# resize re-pinning with no orphan, over real WKWebViews.
e2e-shell-context-menu:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_context_menu -- --ignored --nocapture

e2e-shell-dense:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_dense_timeout}} cargo test -p bbr --features testkit --test e2e_shell_dense -- --ignored --nocapture
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_dense_timeout}} cargo test -p bbr --features testkit --test e2e_shell_dense_edge -- --ignored --nocapture

# Downloads sidebar panel (task packet `downloads-sidebar-panel`): the
# anchored popover over real WKWebView downloads — progress (known and
# unknown length), success/path, failure, cancel, retry, zero bytes, bounded
# 6-row scroll, dismissal, identity preservation, and Reveal-in-Finder
# request semantics.
e2e-shell-downloads:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_downloads -- --ignored --nocapture

# External-URL open delivery (queue item 13): delivered links open a tab in
# the frontmost eligible window through the merged contract — identity
# preservation, profile/space routing, burst dedup, rejected schemes, and the
# real cold-start Apple-Event delivery.
e2e-external-url:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_external_url -- --ignored --nocapture
# Web-error surface (task packet `error-surface-contract`): real
# navigations against the deterministic failure fixtures (TLS vs plaintext,
# closed port, redirect loop, truncated 500, plain 500, flaky route), the
# classified-incident seam for the offline class, the production
# recovery/dismiss seams, supersession on re-failure, and the crash-panel
# Reload — identity preserved throughout.
e2e-shell-errors:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_errors -- --ignored --nocapture

# Renderer isolation / unresponsive tabs: real busy/heavy WebKit probes,
# chrome actions while hung, Wait/Reload/Close recovery, stale events,
# dialog/beforeunload boundaries, and productive typed crash recovery.
e2e-shell-responsiveness:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_dense_timeout}} cargo test -p bbr --features testkit --test e2e_shell_responsiveness -- --ignored --nocapture
# Credential sheet (queue item 10): the conventional macOS credential sheet
# in the one `NativeAboveWeb` portal window over a real WKWebView — realm/
# host presentation, typed masked editing, submit/cancel through the merged
# auth contract, redaction (credentials never in any snapshot/log/payload),
# per-web-view challenge identity, cancel-fails-the-request, and the
# wrong-credentials rechallenge path.
e2e-auth:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_auth -- --ignored --nocapture

e2e-perf:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test --release -p bbr --features testkit --test e2e_performance -- --ignored --nocapture

# Full repeated canonical benchmark: COLD_STARTUP_SAMPLES cold launches to
# build the startup/page-ready distributions plus the 30-sample interaction
# matrix. Materially longer than the short gate; bounded by BBR_E2E_FULL_TIMEOUT.
e2e-perf-full:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_full_timeout}} cargo test --release -p bbr --features testkit --test e2e_performance_full -- --ignored --nocapture

# Injected-delay self-test: proves the budget gate fails under a deterministic
# testkit-only delay and turns green again after removal without budget change.
e2e-perf-inject:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_full_timeout}} cargo test --release -p bbr --features testkit --test e2e_performance_inject -- --ignored --nocapture

# --- Gallery lane (G1b): dev-only component browser, never in `just e2e` ---
# The gallery is a separate native GPUI app (crates/bbr-gallery) with an
# in-process, non-network E2E suite. Both recipes are GUI-locked and
# dev-only; the gallery is never packaged into the .app and never joins
# the canonical `just e2e`.
gallery:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{run_timeout}} cargo run -p bbr-gallery

e2e-gallery:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr-gallery --bin bbr-gallery -- --ignored --nocapture

# Windowless Metal probe, not a sealed comparator/baseline run. Optional story
# selector derives from the registry. Builds with mbx; retains the host GPU lock.
gallery-capture story="all":
    timeout --kill-after=10s {{visual_timeout}} python3 scripts/visual/gallery.py --story {{quote(story)}}

# Run E2E recipes in a throwaway headless Tart macOS VM, so no test window
# takes the host's focus: `just e2e-vm e2e-shell-organize`, `just e2e-vm e2e`.
# Builds on the host, runs in the clone, deletes it (scripts/e2e_vm.py).
e2e-vm +recipes:
    timeout --kill-after=30s {{run_timeout}} python3 scripts/e2e_vm.py {{recipes}}

# GUI suites must stay sequential: parallel AppKit window resizing can make
# one suite observe another suite's screen-clamped geometry.
e2e: e2e-smoke e2e-shutdown e2e-tabs e2e-profiles e2e-shell-tabs e2e-shell-spaces e2e-shell-sidebar e2e-shell-favicons e2e-shell-titlebar-navigation e2e-shell-split-geometry e2e-shell-organize e2e-shell-hover e2e-shell-command e2e-shell-settings e2e-history e2e-permissions e2e-client-certificates e2e-link-destinations e2e-first-run e2e-media e2e-split e2e-windows e2e-real-input e2e-web-data e2e-shell-saved-pages e2e-shell-saved-pages-library e2e-shell-new-window e2e-background-links e2e-shell-persistence e2e-shell-portal e2e-shell-find e2e-shell-dense e2e-shell-downloads e2e-shell-errors e2e-shell-responsiveness e2e-external-url e2e-shell-zoom e2e-shell-context-menu e2e-shell-tab-menu e2e-web-navigation-gestures e2e-auth e2e-perf

verify: fmt check test clippy roadmap ui-deps script-tests

# --- Exact production-chrome snapshots (separate from functional E2E) ---
visual_timeout := env_var_or_default("BBR_VISUAL_TIMEOUT", "60m")

# Portable upstream contracts plus the small BBR integration adapter.
visual-test:
    timeout --kill-after=10s {{test_timeout}} python3 -m unittest discover -s scripts/visual -p 'test_*.py' -v
    timeout --kill-after=10s {{test_timeout}} mbx test --locked --manifest-path vendor/cargo-gpui-visual/Cargo.toml
    timeout --kill-after=10s {{test_timeout}} mbx test --locked --offline --manifest-path scripts/visual/Cargo.toml
    timeout --kill-after=10s {{visual_timeout}} python3 scripts/visual/test_update_integration.py -v

# Host builds + fresh external payload only; never boots Tart or executes native producers.
visual-prepare suite destination:
    timeout --kill-after=30s {{visual_timeout}} mbx run --locked --offline --manifest-path scripts/visual/Cargo.toml -- prepare {{quote(suite)}} {{quote(destination)}}

# All current snapshots: contracts, offscreen Gallery and native browser chrome.
# Both reports are produced even when one suite requires visual review.
visual: visual-test
    timeout --kill-after=10s {{visual_timeout}} python3 scripts/visual/batch.py

# Explicit reviewed selections from the printed batch; never implicit latest/all.
[positional-arguments]
visual-update batch +cases:
    timeout --kill-after=10s {{visual_timeout}} python3 scripts/visual/update.py "$@"

# Capture builds via mbx, then takes the GUI lock only for the real app suite.
# First run exits 1 until its explicitly reviewed cases have baselines.
visual-check:
    timeout --kill-after=10s {{visual_timeout}} python3 scripts/visual/tool.py check

visual-compare run:
    timeout --kill-after=10s {{check_timeout}} python3 scripts/visual/tool.py compare --run {{quote(run)}}

# Deliberately no accept-all; each argument is one reviewed case id.
[positional-arguments]
visual-accept run +cases:
    timeout --kill-after=10s {{check_timeout}} python3 scripts/visual/accept.py "$@"

visual-doctor:
    timeout --kill-after=10s {{check_timeout}} python3 scripts/visual/tool.py doctor

# --- Packaging lane (L1): deterministic .app bundle (additive lane recipes) ---
package_timeout := env_var_or_default("BBR_PACKAGE_TIMEOUT", "12m")
package_smoke_timeout := env_var_or_default("BBR_PACKAGE_SMOKE_TIMEOUT", "90s")

# Non-GUI contract tests for the packaging scripts (icon, bundle, smoke probe).
package-test:
    timeout --kill-after=10s {{test_timeout}} python3 -m unittest scripts/tests/test_packaging.py scripts/tests/test_dev_app.py -v

# Build the release binary once, then assemble + ad-hoc sign the .app bundle.
package:
    DEVELOPER_DIR="{{xcode_developer_dir}}" timeout --kill-after=10s {{package_timeout}} cargo build --release -p bbr
    timeout --kill-after=10s {{package_timeout}} python3 scripts/package/build_app.py \
        --binary target/release/bbr --output target/bbr-app/BBR.app

# Repackage the same binary into a second tree and diff: byte-identical output
# proves deterministic packaging.
package-determinism:
    timeout --kill-after=10s {{package_timeout}} python3 scripts/package/build_app.py \
        --binary target/release/bbr --output target/bbr-app-verify/BBR.app
    diff -r target/bbr-app/BBR.app target/bbr-app-verify/BBR.app
    rm -rf target/bbr-app-verify

# Bounded launch smoke: the packaged app must stay alive, then be terminated.
# GUI-gated: it briefly opens a (non-focused, --bbr-hidden) GPUI window.
package-smoke:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{package_smoke_timeout}} \
        python3 scripts/package/smoke_launch.py --app target/bbr-app/BBR.app \
        --observe 5 --stderr target/bbr-smoke/app-stderr.txt

# Full packaging lane gate: unit contracts + bundle + determinism + real smoke.
package-verify: package package-test package-determinism package-smoke
