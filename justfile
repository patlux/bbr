set shell := ["bash", "-euo", "pipefail", "-c"]

# Keep local verification finite. Override only for an intentionally longer run.
check_timeout := env_var_or_default("BBR_CHECK_TIMEOUT", "5m")
test_timeout := env_var_or_default("BBR_TEST_TIMEOUT", "5m")
e2e_timeout := env_var_or_default("BBR_E2E_TIMEOUT", "2m")
xcode_developer_dir := env_var_or_default("BBR_XCODE_DEVELOPER_DIR", "/Applications/Xcode-16.4.0.app/Contents/Developer")

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

e2e-perf:
    DEVELOPER_DIR="{{xcode_developer_dir}}" timeout --kill-after=10s {{e2e_timeout}} cargo test --release -p bbr --features testkit --test e2e_performance -- --ignored --nocapture

e2e-perf-full: e2e-perf

# GUI suites must stay sequential: parallel AppKit window resizing can make
# one suite observe another suite's screen-clamped geometry.
e2e: e2e-smoke e2e-tabs e2e-profiles e2e-perf

verify: fmt check test clippy roadmap
