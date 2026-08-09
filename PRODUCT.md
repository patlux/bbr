# Product

<!-- impeccable:product-schema 1 -->

## Platform

adaptive

## Stack

Rust application core and browser chrome using GPUI. macOS is the first production platform. Native web engines are accessed through platform adapters, beginning with WebKit/WKWebView on macOS. Development tooling is reproducible through Nix flakes and direnv.

## Users

The initial user is a keyboard-oriented macOS power user who keeps many web contexts open and wants fast switching without conventional horizontal-tab clutter. The architecture should remain suitable for a wider public desktop audience later.

## Product Purpose

Create a battery-efficient, high-performance desktop browser organized around vertical tabs, spaces, profiles, and a command bar. Success means it can become a reliable daily browser on macOS while keeping the product core portable to Windows and Linux.

## Positioning

The browser combines Arc/Zen-style organization with a native, low-overhead shell: GPUI renders the browser interface while each platform uses its efficient native web engine. Spaces organize tasks; profiles provide isolated browsing identities; the command bar makes navigation and switching keyboard-first.

## Operating Context

Users keep work, personal, and project contexts separated for long sessions. They switch spaces frequently, retain pinned destinations, open transient tabs, search existing tabs and history from one command surface, and place two pages side by side when comparing or transferring information.

The existing Swift reference implementation is at `/Users/patwoz/dev/Personal/macos/PWBrowser`. It demonstrates spaces, profile-isolated WKWebsiteDataStore instances, pinned and temporary tabs, history, a command palette, vertical sidebar navigation, gestures, and WKWebView reuse.

## Capabilities and Constraints

- macOS-first, with cross-platform domain models and application services from the start.
- Rust and GPUI own browser state, commands, chrome, sidebar, layout, and persistence boundaries.
- macOS browsing uses WKWebView/WebKit for native battery and performance characteristics.
- Platform web views remain native child views; GPUI must not re-render web content into textures or screenshots.
- Native engine APIs are hidden behind a narrow `WebEngine` adapter so Windows and Linux implementations can differ without leaking into product logic.
- Required product primitives: spaces, profiles, pinned tabs, vertical tabs/sidebar, command bar, and split view.
- A profile owns an isolated cookie/cache/credential website data store. Multiple profiles and their spaces coexist inside one browser window; a profile is a data/container boundary, not a window boundary. A space belongs to one profile. Tabs belong to one space.
- The product may reference Arc and Zen behaviors, but must use its own name, assets, copy, and visual identity.
- Exact Windows and Linux engine choices remain open until those platform phases begin.

## Evidence on Hand

- Working Swift/WebKit reference: `/Users/patwoz/dev/Personal/macos/PWBrowser`.
- Existing reference models: `Profile`, `Space`, and `Tab`.
- Existing reference behavior: profile-specific `WKWebsiteDataStore`, retained WKWebViews, bounded background loading, tab pinning/reordering, recently closed tabs, zoom persistence, history, command palette, and space gestures.
- No final product name, logo, public claims, implemented benchmark harness/baseline, or cross-platform implementation exists yet; initial performance budgets are now specified in the product contract.

## Interaction Contract

Detailed, canonical behavior for tab selection, close/reopen, spaces, splits,
keyboard focus, and unresolved UX decisions lives in
[docs/product/browser-interactions.md](docs/product/browser-interactions.md).
Implementation and UI changes must preserve that contract or update it
explicitly together with tests. The cross-phase inventory of ordinary browser
muscle-memory and reliability gaps lives in
[docs/product/daily-driver-capability-inventory.md](docs/product/daily-driver-capability-inventory.md);
each concrete capability still requires one bounded canonical task packet.

## Performance Contract

Perceived responsiveness is a product invariant from the first browser shell,
not a final optimization pass. Canonical startup, tab/space switching, sidebar,
and interaction budgets plus benchmark methodology live in
[docs/product/performance-contract.md](docs/product/performance-contract.md).
Implementation must distinguish BBR presentation latency from external network
or page-load latency.

## Product Principles

- Instant shell and selected-content presentation, continuously protected by real-app performance E2E.
- Native engine efficiency over engine ownership.
- Portable product logic; explicitly platform-native integration edges.
- Keyboard-first without making pointer workflows secondary.
- Preserve browsing context while controlling background resource use.
- Build daily-driver reliability in thin, end-to-end increments.

## Accessibility & Inclusion

Browser chrome must expose semantic roles, labels, focus order, keyboard operation, sufficient contrast, reduced-motion behavior, and platform accessibility APIs. Web content accessibility remains the responsibility of the native web engine, while focus transitions between GPUI chrome and the native web view must be predictable.
