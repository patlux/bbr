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
run_timeout := env_var_or_default("BBR_RUN_TIMEOUT", "8h")
xcode_developer_dir := env_var_or_default("BBR_XCODE_DEVELOPER_DIR", "/Applications/Xcode-26.2.0.app/Contents/Developer")
gui_gate := "python3 scripts/with_gui_lock.py --"

# Launch the development build. Override BBR_RUN_TIMEOUT for longer sessions.
run:
    DEVELOPER_DIR="{{xcode_developer_dir}}" timeout --kill-after=10s {{run_timeout}} cargo run -p bbr

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

script-tests:
    timeout --kill-after=10s {{test_timeout}} python3 -m unittest scripts/test_with_gui_lock.py -v
    timeout --kill-after=10s {{check_timeout}} python3 -m py_compile scripts/check_roadmap.py scripts/with_gui_lock.py scripts/test_with_gui_lock.py

e2e-smoke:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_smoke -- --ignored --nocapture

e2e-tabs:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_tabs -- --ignored --nocapture

e2e-profiles:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_profiles -- --ignored --nocapture

e2e-shell-tabs:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_tabs -- --ignored --nocapture

e2e-shell-spaces:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_spaces -- --ignored --nocapture
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_spaces_keyboard -- --ignored --nocapture
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_spaces_hint -- --ignored --nocapture

e2e-shell-sidebar:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_sidebar -- --ignored --nocapture

e2e-shell-split-geometry:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_split_geometry -- --ignored --nocapture

e2e-shell-organize:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_organize -- --ignored --nocapture

e2e-shell-hover:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_semantics -- --ignored --nocapture
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_hover -- --ignored --nocapture
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_semantic_branches -- --ignored --nocapture

e2e-shell-command:
    DEVELOPER_DIR="{{xcode_developer_dir}}" {{gui_gate}} timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_command -- --ignored --nocapture

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

# GUI suites must stay sequential: parallel AppKit window resizing can make
# one suite observe another suite's screen-clamped geometry.
e2e: e2e-smoke e2e-tabs e2e-profiles e2e-shell-tabs e2e-shell-spaces e2e-shell-sidebar e2e-shell-split-geometry e2e-shell-organize e2e-shell-hover e2e-shell-command e2e-shell-portal e2e-shell-find e2e-shell-zoom e2e-shell-context-menu e2e-shell-dense e2e-shell-downloads e2e-perf

verify: fmt check test clippy roadmap script-tests

# --- Packaging lane (L1): deterministic .app bundle (additive lane recipes) ---
package_timeout := env_var_or_default("BBR_PACKAGE_TIMEOUT", "12m")
package_smoke_timeout := env_var_or_default("BBR_PACKAGE_SMOKE_TIMEOUT", "90s")

# Non-GUI contract tests for the packaging scripts (icon, bundle, smoke probe).
package-test:
    timeout --kill-after=10s {{test_timeout}} python3 -m unittest scripts/tests/test_packaging.py -v

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
