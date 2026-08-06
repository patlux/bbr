set shell := ["bash", "-euo", "pipefail", "-c"]

# Keep local verification finite. Override only for an intentionally longer run.
check_timeout := env_var_or_default("BBR_CHECK_TIMEOUT", "5m")
test_timeout := env_var_or_default("BBR_TEST_TIMEOUT", "5m")
e2e_timeout := env_var_or_default("BBR_E2E_TIMEOUT", "2m")
e2e_full_timeout := env_var_or_default("BBR_E2E_FULL_TIMEOUT", "15m")
run_timeout := env_var_or_default("BBR_RUN_TIMEOUT", "8h")
xcode_developer_dir := env_var_or_default("BBR_XCODE_DEVELOPER_DIR", "/Applications/Xcode-16.4.0.app/Contents/Developer")

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

e2e-smoke:
    DEVELOPER_DIR="{{xcode_developer_dir}}" timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_smoke -- --ignored --nocapture

e2e-tabs:
    DEVELOPER_DIR="{{xcode_developer_dir}}" timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_tabs -- --ignored --nocapture

e2e-profiles:
    DEVELOPER_DIR="{{xcode_developer_dir}}" timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_profiles -- --ignored --nocapture

e2e-shell-tabs:
    DEVELOPER_DIR="{{xcode_developer_dir}}" timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_tabs -- --ignored --nocapture

e2e-shell-spaces:
    DEVELOPER_DIR="{{xcode_developer_dir}}" timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_spaces -- --ignored --nocapture
    DEVELOPER_DIR="{{xcode_developer_dir}}" timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_spaces_keyboard -- --ignored --nocapture

e2e-shell-sidebar:
    DEVELOPER_DIR="{{xcode_developer_dir}}" timeout --kill-after=10s {{e2e_timeout}} cargo test -p bbr --features testkit --test e2e_shell_sidebar -- --ignored --nocapture

e2e-perf:
    DEVELOPER_DIR="{{xcode_developer_dir}}" timeout --kill-after=10s {{e2e_timeout}} cargo test --release -p bbr --features testkit --test e2e_performance -- --ignored --nocapture

# Full repeated canonical benchmark: COLD_STARTUP_SAMPLES cold launches to
# build the startup/page-ready distributions plus the 30-sample interaction
# matrix. Materially longer than the short gate; bounded by BBR_E2E_FULL_TIMEOUT.
e2e-perf-full:
    DEVELOPER_DIR="{{xcode_developer_dir}}" timeout --kill-after=10s {{e2e_full_timeout}} cargo test --release -p bbr --features testkit --test e2e_performance_full -- --ignored --nocapture

# Injected-delay self-test: proves the budget gate fails under a deterministic
# testkit-only delay and turns green again after removal without budget change.
e2e-perf-inject:
    DEVELOPER_DIR="{{xcode_developer_dir}}" timeout --kill-after=10s {{e2e_full_timeout}} cargo test --release -p bbr --features testkit --test e2e_performance_inject -- --ignored --nocapture

# GUI suites must stay sequential: parallel AppKit window resizing can make
# one suite observe another suite's screen-clamped geometry.
e2e: e2e-smoke e2e-tabs e2e-profiles e2e-shell-tabs e2e-shell-spaces e2e-shell-sidebar e2e-perf

verify: fmt check test clippy roadmap
