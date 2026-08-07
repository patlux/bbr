# Task packet — GPUI-CE and gpui-component window-root anchor

Parents: [Phase 4](../phases/04-ui-shell.md) · [UI design-system track](../tracks/ui-design-system.md) · [Accessibility/input track](../tracks/accessibility.md) · [Product contract](../../product/browser-interactions.md)

## Outcome

BBR runs its existing production browser shell and real native WKWebViews on one exactly pinned GPUI-CE type world, with `gpui-component 0.5.1` initialized and every browser window rooted in `gpui_component::Root` around a separately addressable `Entity<BbrApp>`. Existing browser behavior, testkit control, focus choreography, semantic snapshots, and presentation measurements remain correct. This anchor is the clean shared foundation for later native command input, floating palette windows, sidebar component composition, and packaging lanes.

## Scope

- Pin `gpui` and `gpui_platform` to GPUI-CE commit `568271c01c5a9d640d7b091e5bfd136b04035f28` and patch crates.io `gpui` to that same source.
- Pin `gpui-component` to `=0.5.1` with its optional `webview` feature disabled, regenerate `Cargo.lock`, and prove one `gpui` source/type world.
- Establish and document the exact supported Rust and macOS/Xcode floors from dependency metadata and fresh builds; update the workspace/tooling facts rather than preserving stale claims.
- Start through `gpui_platform::application()`, call `gpui_component::init(cx)` before component use, and make `gpui_component::Root` the first view in the browser window.
- Preserve a separate `Entity<BbrApp>` and migrate global actions, async event/ranking paths, testkit dispatch, snapshots, focus sinks, and presentation hooks from direct `WindowHandle<BbrApp>` assumptions to the Root-window plus app-entity contract.
- Port only current-main GPUI API deltas required by the exact pin, including current async `update` call sites, focus APIs, and styling changes. Use the old spike only as read-only evidence; do not copy its lockfile or apply it wholesale.
- Update architecture, E2E architecture, roadmap, and this delivery ledger with implemented facts and measured evidence.

## Non-goals

- Native command-field input, IME acceptance, the centered/global or URL-anchored palette presentation, palette overlay-window policy, and sidebar redesign.
- The sidebar footer, Downloads/Spaces/Add menu, New Folder model, app packaging, persistence hydration, WebKit popup/new-window routing, or production split composition.
- Enabling `gpui-component`'s `webview` feature, replacing BBR's native WKWebViews, visually redesigning BBR, or importing the stale GPUI-CE spike lockfile/API assumptions.
- Canonical performance certification without a qualified dedicated display-attached runner.

## Parallel-lane contract

Not parallelized. This task owns the serial integration kernel.

```text
Base commit: af8983261245ae2dd9076a2314e360ce90fbec8b
Depends on: integrated command composition, split geometry contract, portable split core, clean main
Provides/API: one pinned GPUI-CE type world; gpui-component initialization; WindowHandle<Root> + Entity<BbrApp> composition seam
Exclusive paths: root Cargo.toml/Cargo.lock; crates/bbr-app/Cargo.toml; crates/bbr-app/src/main.rs; current Root/app-handle migration call sites; task/architecture/evidence docs
Shared integration points: workspace dependencies and lockfile; application startup; window root; global actions; async app updates; testkit dispatch/window handle; canonical E2E recipes/toolchain facts
Integration order: first; all palette, sidebar, and packaging writers branch only from its reviewed green integration commit
```

## Constraints

- One exclusive writer in one dedicated worktree. The writer may modify only the declared current-main anchor paths and any additional exact call site proven necessary for compilation; it must report every touched path.
- The writer must not commit, push, merge, rebase, amend, stash, reset, clean, change branches, edit other worktrees, or weaken tests/budgets/validators. The parent owns review, commits, and integration.
- Mutation-capable implementation and bounded remediation use `homelab-llm/deepseek-v4-flash:high` without silent model substitution. Architecture decisions, task-packet changes, reviews, gates, and integration remain parent-owned.
- `/Users/patwoz/dev/Personal/macos/bbr-url-input` and `/Users/patwoz/dev/Personal/macos/bbr-split-app-composition` remain untouched and preserved.
- `gpui-component` supplies standard component behavior; BBR retains its design tokens, geometric icons, browser orchestration, profile isolation, dense virtualization, semantic contract, and native child-view geometry.
- `gpui-component`'s `webview` feature stays disabled. No screenshot, texture, Wry, or replacement web-content composition.
- Keep the testkit feature local Unix-socket-only, authenticated, and absent from ordinary release builds. Do not create or broaden network/service exposure.
- Do not preserve compatibility with crates.io GPUI or duplicate Root/application paths. There must be one source and one current contract.
- Every failed command must lead to root-cause inspection before another attempt. All long-running commands stay bounded and all GUI commands use the host-wide lock.

## Evidence

### Dependency/toolchain contract

- [ ] `Cargo.lock` is freshly generated from this branch and contains the exact GPUI-CE revision plus `gpui-component 0.5.1`.
- [ ] `cargo tree -d` and inverse dependency queries prove exactly one `gpui` package/source in the resolved graph.
- [ ] Direct `gpui` and `gpui_platform` dependencies resolve to the exact same GPUI-CE revision; crates.io `gpui` consumers are patched to it.
- [ ] `cargo tree -e features -p gpui-component` proves its `webview` feature and Wry dependency are absent.
- [ ] The minimum Rust claim is derived from current dependency manifests and tested with the declared toolchain where feasible; unsupported floors remain explicit.
- [ ] macOS/Xcode tooling facts match an installed, successful bounded build path rather than the absent Xcode 16.4 path.

### Unit and contract

- [ ] Root/window/app-entity ownership has focused tests or compile-time contract coverage for actions, async updates, focus sinks, testkit dispatch, and snapshots.
- [ ] Existing affected `bbr-app`, command, browser, domain, testkit, web, and macOS adapter tests remain green.
- [ ] `cargo fmt --all -- --check`, `git diff --check`, `just roadmap`, `just verify`, and testkit Clippy are green.

### Real-app E2E

- [ ] Fresh smoke proves startup, real WKWebView hosting, navigation, resize, `Cmd-L`, Escape, and clean shutdown through the Root contract.
- [ ] Profiles prove cookie/data-store isolation and session recreation/reuse remain unchanged.
- [ ] Command E2E proves current command ranking/activation/focus behavior survives the Root migration.
- [ ] Sidebar and split-geometry regressions prove native slot geometry remains keyed and correct.
- [ ] Tabs, spaces, organization, hover/semantics, and dense suites remain green in the final canonical sequential gate.
- [ ] Typed snapshots/semantics continue to expose the production browser state; no synthetic Root-only proxy substitutes for app evidence.

### Performance

- [ ] Optimized report-only performance runs through production Root/app-entity paths and preserves all existing fast-but-wrong guards.
- [ ] Startup and changed action/command presentation samples remain within local absolute ceilings; canonical certification remains explicitly open.

### Review

- [ ] One independent fresh-context read-only review maps the packet to the actual diff/evidence and reports no unresolved blocker/high/medium correctness, architecture, native-boundary, test, or scope finding.
- [ ] If concrete findings exist, one bounded DeepSeek V4 Flash remediation is followed by a focused independent recheck.

## Completion gates

Use the installed supported Xcode path recorded by the slice; never bypass the host-wide GUI lock.

```bash
cargo fmt --all -- --check
git diff --check
cargo tree -d
cargo tree -i gpui
cargo tree -e features -p gpui-component
just roadmap
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-smoke
just e2e-profiles
just e2e-shell-command
just e2e-shell-sidebar
just e2e-shell-split-geometry
just e2e-perf
just e2e
```

Add any focused checks required by compilation/API migrations. The final `just e2e` runs once after the complete anchor is stable.

## Delivery sequence

1. Parent records the clean base, task packet, path ownership, toolchain facts, and dedicated worktree.
2. One DeepSeek V4 Flash writer ports current `main`, regenerates the lockfile, adds focused evidence, and runs targeted checks without committing.
3. Parent inspects the complete diff/status and runs required gates.
4. One independent read-only reviewer maps requirements to actual source/tests/evidence.
5. Only concrete findings receive one bounded DeepSeek V4 Flash remediation, then a focused recheck.
6. Parent commits the coherent reviewed anchor and delivery ledger on its lane, fast-forwards `main` serially, and re-runs affected integration gates before any fan-out.

## Iteration policy

Start with dependency/source unification and a compiling empty Root wrapper, then migrate one ownership boundary at a time: startup/root creation, actions, event pumps/ranking, testkit dispatch, focus/snapshot/performance. After each failure, inspect the exact compiler/test mechanism and apply the smallest current-API fix; do not transplant stale spike code or add dual compatibility. Use targeted non-GUI checks first, then affected real-app suites, then one canonical sequential gate when stable.

## Blocked stop condition

If no defensible path remains, stop with:

- exact worktree, branch, HEAD, status, touched paths, and owner;
- dependency graph and toolchain evidence gathered;
- compiler/test/runtime failures and approaches attempted;
- the specific GPUI-CE/gpui-component/native-focus/toolchain incompatibility;
- preserved changes and residual risks;
- the smallest upstream patch, environment change, product decision, or user input needed to continue.

## Delivery ledger

```text
Outcome: pending
Commits: pending
Functional evidence: pending
Real-app E2E: pending
Performance evidence: pending
Roadmap/documentation updates: packet activated from main af898326
Open blockers and non-goals: native input, palette surfaces, sidebar redesign, packaging, persistence, popup routing, split composition excluded
Worktree state: pending dedicated worktree

Planning/read-only: started 2026-08-07; exact duration pending
Implementation: pending
Test execution: pending
Review: pending
Remediation: pending
Waiting/retries: pending
Total elapsed: pending
Human intervention: footer/menu product correction and explicit DeepSeek V4 Flash writer preference supplied
```
