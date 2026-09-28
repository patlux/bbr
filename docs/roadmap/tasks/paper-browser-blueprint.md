# Task packet — Complete browser blueprint in Paper

Parents: [Phase 4](../phases/04-ui-shell.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[DESIGN.md](../../../DESIGN.md) · [PRODUCT.md](../../../PRODUCT.md) ·
[daily-driver inventory](../../product/daily-driver-capability-inventory.md).

Paper file: **BBR — Browser Shell** (`01KZY8FRMDSAYFJ4TJ9TYTV4AF`). The
pages that existed before 2026-09-28 are kept with an `Archiv ·` prefix.

## Outcome

Every surface, state and process of the browser — shipped, planned and not
yet packeted — is drawn in Paper as the target state, in light and dark,
with the necessary variants and a motion specification. A coverage matrix
proves that every DESIGN component and state, every UI-bearing roadmap item,
every daily-driver capability and every E2E suite maps to an artboard. The
differences between the drawings and the shipped code are recorded in a
delta ledger with one target packet each, and adopted target decisions are
written into DESIGN.md.

## Scope

- **W0 setup:** this packet; archive the old Paper pages; page 00 (read-me,
  legend, naming, coverage); page 10 foundations with the tokens synced from
  the code (`palette.rs`, `space_tint.rs`, `theme.rs`, `elevation.rs`); page
  11 building blocks every other board is composed from.
- **W1:** window & shell (20), sidebar (21), spaces & profiles (22).
- **W2:** command (23), content & glass (24).
- **W3:** menus & panels (25), including the new History, Clear data, Site
  settings, Permission, Sign-in/certificate, External-app and Pop-up
  surfaces.
- **W4:** system & lifecycle (26): first run, the Settings window, restore,
  recovery, About/update, menu bar, windows; split view on page 24.
- **W5:** the 23 process storyboards (30).
- **W6:** variant matrices where they matter (increased contrast, space
  paper, density, Reduce Motion) and the motion specification (40).
- **W7:** open decisions and the delta ledger (90); DESIGN.md updates for
  the adopted target decisions; one target task packet per delta; the final
  coverage check.

## Non-goals

- No product-code change; implementation follows through the delta packets.
- Native OS surfaces (file panels, WebKit element menus, system prompts,
  traffic lights, menu bar) are drawn only as labelled context.
- Web content is a neutral placeholder; no site screenshots.
- Increased contrast, tints, density and narrow windows are drawn only in
  the W6 matrices and where a surface changes shape, not for every board.

## Parallel-lane contract

Not parallelized. One writer works serially on one Paper file; no subagents.

## Constraints

- DESIGN.md stays the canonical contract. A board becomes authority only
  when its decision lands in DESIGN.md with a packet (W7).
- Paper colours, type and spacing use the file's tokens as CSS variables;
  the tokens match the code values (checked by script, see Evidence).
- Each board names its mark (Shipped · Change · New · proposal · Native OS),
  its DESIGN sections, the owning packet and the E2E assertion ids.
- UI copy and annotations are English, as in the product.
- The Paper font stand-ins are `System Sans-Serif` (SF Pro) for UI and
  JetBrains Mono for `SF Mono`, which Paper cannot load.

## Evidence

### Unit and contract

- [x] W0: old pages archived; index, foundations and building blocks drawn
      and screenshot-reviewed; Paper tokens match the code.
- [x] W1: shell, sidebar, spaces and profiles boards drawn and reviewed.
- [x] W2: command and content/glass boards drawn and reviewed.
- [x] W3: menus and panels boards drawn and reviewed.
- [x] W4: system, lifecycle and split-view boards drawn and reviewed.
- [x] W5: 23 process storyboards drawn and reviewed.
- [ ] W6: variant matrices and motion specification drawn and reviewed.
- [ ] W7: decisions and delta ledger drawn; DESIGN.md updated; one packet
      per delta; coverage check reports every row drawn.

### Real-app E2E

Not applicable: design-only packet. The E2E suites appear in the coverage
matrix so every exercised surface has a drawing.

### Performance

Not applicable: no runtime change.

## Completion gates

```bash
python3 scripts/paper_blueprint/verify_tokens.py    # Paper tokens = code values
python3 scripts/paper_blueprint/check_coverage.py   # every board and row drawn
just roadmap
```

The scripts need Paper Desktop with this file open (see
`scripts/paper_blueprint/README.md`); their output is recorded in the ledger
per wave.

## Iteration policy

Follow the wave order. Build each board from the page-11 blocks, one visual
group per write, screenshot it and fix spacing, type, contrast, alignment and
fit before moving on. Record the wave in the ledger and commit the
documentation before the next wave. Patrick reviews asynchronously; his
findings become remediation in the next wave.

## Blocked stop condition

If Paper or its MCP endpoint is unavailable, token writes fail, or a board
cannot be drawn without contradicting DESIGN.md in a way only Patrick can
decide, stop with the evidence, the attempted paths, the blocker and the
decision needed.

## Coverage matrix

Generated by `check_coverage.py --render` from the board inventory; "Drawn"
is read from the Paper file.

<!-- coverage:start -->
| Source | Item | Artboards | Drawn |
|---|---|---|---|
| DESIGN §2 | Composition, materials, content card, hairlines | F-4 · Space, radius, elevation, materials; W-1 · Shell — standard; G-1 · Glass — before attach & blank | yes |
| DESIGN §3 | Colour tokens — neutrals, accent, status, selection | F-1 · Colour — neutrals, accent, status | yes |
| DESIGN §3.4 | Increased-contrast mode | F-1 · Colour — neutrals, accent, status; V-1 · Increased contrast | planned |
| DESIGN §4 | Typography and truncation | F-3 · Type & density; S-5 · Tab rows in context | yes |
| DESIGN §5 | Spacing, radius, layout grid | F-4 · Space, radius, elevation, materials; S-10 · Rail width & resize | yes |
| DESIGN §6 | Motion principles and tokens | F-6 · Motion tokens; V-5 · Motion — moments; V-6 · Motion — transitions | planned |
| DESIGN §7 | Iconography, app icon, identity glyphs | F-5 · Icons & identity glyphs | yes |
| DESIGN §7.2 | Favicon tiles and monograms | F-2 · Colour — space paper, grain, monograms; B-4 · Tiles, chips & badges | yes |
| DESIGN §8 | Density S / N / XL | F-3 · Type & density; V-3 · Density S · N · XL; W-8 · Shell — dense list (500 tabs) | planned |
| DESIGN §9 | Focus | B-2 · Fields, toggles & focus; PR-19 · Keyboard focus — chrome ↔ page | yes |
| DESIGN §10.1 | State default | B-3 · Rail rows — §10 states | yes |
| DESIGN §10.2 | State hover | B-3 · Rail rows — §10 states; B-1 · Buttons & keys | yes |
| DESIGN §10.3 | State active/pressed | B-3 · Rail rows — §10 states; B-1 · Buttons & keys | yes |
| DESIGN §10.4 | State selected | B-3 · Rail rows — §10 states; S-5 · Tab rows in context | yes |
| DESIGN §10.5 | State loading | B-3 · Rail rows — §10 states; B-7 · Feedback — progress, attention, banners; W-6 · Shell — page loading | yes |
| DESIGN §10.6 | State attention | B-3 · Rail rows — §10 states; B-7 · Feedback — progress, attention, banners; S-5 · Tab rows in context | yes |
| DESIGN §10.7 | State error | B-3 · Rail rows — §10 states; G-5 · Glass — load errors | yes |
| DESIGN §10.8 | State disabled | B-1 · Buttons & keys; B-3 · Rail rows — §10 states; W-7 · Shell — titlebar navigation states | yes |
| DESIGN §10.9 | State dragging / drag-target | B-3 · Rail rows — §10 states; S-9 · Drag in flight | yes |
| DESIGN §10.10 | State focused (keyboard) | B-2 · Fields, toggles & focus; B-3 · Rail rows — §10 states | yes |
| DESIGN §10.11 | State unresponsive | B-3 · Rail rows — §10 states; G-4 · Glass — unresponsive page | yes |
| DESIGN §11.1 | Vertical tab row | B-3 · Rail rows — §10 states; S-5 · Tab rows in context | yes |
| DESIGN §11.2 | Pinned tab | S-3 · Space heading & pinned tabs; S-9 · Drag in flight | yes |
| DESIGN §11.3 | Space chips, profile context, configuration | S-7 · Footer strip; P-1 · Space switching; P-2 · Space configuration; P-3 · New space; P-4 · Space menu & delete | yes |
| DESIGN §11.4 | Sidebar and command surfaces | W-1 · Shell — standard; S-6 · New Tab row & staged tab; C-1 · ⌘T — empty with recents; C-2 · ⌘T — results by kind; C-6 · ⌘L — edit address | yes |
| DESIGN §11.5 | Address control | S-1 · Address control; G-7 · Page zoom | yes |
| DESIGN §11.6 | Dense list (500 tabs) | W-8 · Shell — dense list (500 tabs) | yes |
| DESIGN §11.7 | Split preview / split view | G-9 · Split view — two panes; G-10 · Split view — create, resize, compact; PR-09 · Split view — create, focus, resize, close | yes |
| DESIGN §11.8 | Empty, crash and load-error panes | G-2 · Glass — empty space; G-3 · Glass — crashed page; G-5 · Glass — load errors | yes |
| DESIGN §11.8a | Unresponsive renderer pane | G-4 · Glass — unresponsive page | yes |
| DESIGN §11.9 | Missing favicon and long titles | S-5 · Tab rows in context; B-4 · Tiles, chips & badges | yes |
| DESIGN §11.10 | Favorites grid | S-2 · Favorites grid; S-9 · Drag in flight | yes |
| DESIGN §11.11 | Folders | S-4 · Folders; M-2 · Favorite & folder menus | yes |
| DESIGN §11.12 | Downloads panel, find bar, sheets | M-4 · Downloads panel; G-6 · Find in page | yes |
| DESIGN §11.13 | Context-menu language | M-1 · Tab menu; M-2 · Favorite & folder menus; M-3 · Page menu & web menus; P-4 · Space menu & delete | yes |
| DESIGN §11.14 | Titlebar navigation controls | W-7 · Shell — titlebar navigation states | yes |
| DESIGN §11.15 | Sidebar media control | S-8 · Media card; PR-17 · Media in a background tab | yes |
| DESIGN §11.16 | Saved Pages result and Library | C-2 · ⌘T — results by kind; M-5 · Saved Pages Library; PR-12 · Save a page & Library | yes |
| DESIGN §11.17 | Peek preview bar | G-8 · Peek preview; PR-04 · Open links — foreground, background, Peek, external, pop-up | yes |
| DESIGN §12 | Profile identity | P-5 · Profile identity in chrome; P-6 · Profiles — create & edit; P-7 · Move a tab across profiles | yes |
| DESIGN §13 | Native web-content boundary | G-1 · Glass — before attach & blank; M-3 · Page menu & web menus; M-14 · File upload & save | yes |
| DESIGN §17.2 | Tooltips | B-6 · Floating shells | yes |
| DESIGN §17.13 | Internal page names in the address | S-1 · Address control | yes |
| DESIGN §18.2 | v2 materials, colour, elevation | F-1 · Colour — neutrals, accent, status; F-4 · Space, radius, elevation, materials; W-2 · Shell — space tint | yes |
| DESIGN §18.3 | v2 radius and type | F-3 · Type & density; F-4 · Space, radius, elevation, materials | yes |
| DESIGN §18.4 | v2 floating surfaces | B-6 · Floating shells; C-2 · ⌘T — results by kind; C-6 · ⌘L — edit address; M-1 · Tab menu | yes |
| DESIGN §18.5 | v2 motion moments | V-5 · Motion — moments; V-4 · Reduce Motion | planned |
| Phase 4 | Startup presents sidebar and selected tab | L-10 · Session restore; PR-18 · Quit, relaunch, crash recovery | yes |
| Phase 4 | Empty space, first launch, many tabs, long titles, missing favicons, crashed pages | G-2 · Glass — empty space; L-1 · First run — welcome; W-8 · Shell — dense list (500 tabs); S-5 · Tab rows in context; G-3 · Glass — crashed page | yes |
| Phase 4 | Collapsible, adjustable sidebar | W-3 · Shell — sidebar hidden ⌘B & hover reveal; S-10 · Rail width & resize | yes |
| Phase 4 | Drag reorder and move between spaces | S-9 · Drag in flight; PR-06 · Organize — folders & drag rules | yes |
| Phase 6 | Split view: create, show two, divider, focus, route, close, dissolve, compact | G-9 · Split view — two panes; G-10 · Split view — create, resize, compact; PR-09 · Split view — create, focus, resize, close | yes |
| Phase 7 | First-run, empty, migration-error and recovery UI | L-1 · First run — welcome; L-2 · First run — import, default browser, first space; L-11 · Recovery & migration errors | yes |
| Phase 7 | Chrome-first restore | L-10 · Session restore; PR-18 · Quit, relaunch, crash recovery | yes |
| Phase 8 | Downloads | M-4 · Downloads panel; L-7 · Settings — Downloads; PR-10 · Downloads — progress, done, reveal, retry | yes |
| Phase 8 | Find in page | G-6 · Find in page; PR-11 · Find in page | yes |
| Phase 8 | Camera, microphone, location, notification, clipboard permissions | M-9 · Permission requests; M-8 · Site settings; PR-14 · Permissions | yes |
| Phase 8 | HTTP/proxy authentication and client certificates | M-10 · Sign-in challenges; M-11 · Client certificates; PR-15 · Sign-in challenges | yes |
| Phase 8 | Upload/save panels | M-14 · File upload & save | yes |
| Phase 8 | Web errors and recovery | G-5 · Glass — load errors; PR-13 · Errors & recovery | yes |
| Phase 8 | History UI and clear-data controls | M-6 · History; M-7 · Clear browsing data; PR-22 · History & clear data | yes |
| Phase 8 | Saved Pages | M-5 · Saved Pages Library; C-2 · ⌘T — results by kind; PR-12 · Save a page & Library | yes |
| Phase 8 | Default browser and external URLs | M-12 · Open in another app; L-2 · First run — import, default browser, first space; PR-20 · Links from other apps | yes |
| Phase 8 | Native menus, window lifecycle, app updates | L-13 · Menu bar & Dock; L-14 · Windows — new, close, quit; L-12 · About & updates; W-9 · Shell — several windows; PR-21 · Windows — new, move, close | yes |
| Phase 8 | Media session control | S-8 · Media card; PR-17 · Media in a background tab | yes |
| Phase 8 | Per-site settings and zoom | M-8 · Site settings; G-7 · Page zoom; L-6 · Settings — Privacy & site data; PR-16 · Per-site zoom | yes |
| Phase 8 | Pop-up / new-window policy | M-13 · Pop-ups & new windows; PR-04 · Open links — foreground, background, Peek, external, pop-up | yes |
| Phase 8 | Renderer hang: responsive chrome | G-4 · Glass — unresponsive page; PR-13 · Errors & recovery | yes |
| Phase 8 | Empty, loading, error, cancellation, destructive states | G-1 · Glass — before attach & blank; G-2 · Glass — empty space; G-5 · Glass — load errors; P-4 · Space menu & delete; M-7 · Clear browsing data; B-7 · Feedback — progress, attention, banners | yes |
| Phase 8 | Portal host (floating surfaces above web) | B-6 · Floating shells; G-6 · Find in page; G-8 · Peek preview | yes |
| Phase 9 | Light, dark, IC, narrow, wide, empty, dense, loading, error variants | V-1 · Increased contrast; V-2 · Space paper matrix; V-3 · Density S · N · XL; W-4 · Shell — narrow & wide windows; W-8 · Shell — dense list (500 tabs) | planned |
| Phase 9 | Purposeful transitions, Reduce Motion | V-4 · Reduce Motion; V-5 · Motion — moments; V-6 · Motion — transitions | planned |
| Phase 9 | Onboarding and first-run activation | L-1 · First run — welcome; L-2 · First run — import, default browser, first space; PR-01 · First run & onboarding | yes |
| Phase 9 | Keyboard and VoiceOver for every action | B-2 · Fields, toggles & focus; PR-19 · Keyboard focus — chrome ↔ page; L-9 · Settings — Shortcuts | yes |
| Phase 9 | Settings surface (search, privacy, appearance, shortcuts) | L-3 · Settings — General; L-4 · Settings — Profiles & spaces; L-5 · Settings — Search; L-6 · Settings — Privacy & site data; L-7 · Settings — Downloads; L-8 · Settings — Appearance; L-9 · Settings — Shortcuts; PR-23 · Change settings | yes |
| Inventory | Trackpad back/forward over web content | W-7 · Shell — titlebar navigation states; PR-03 · Address, back, forward, reload | yes |
| Inventory | Child-tab Back-at-root returns to opener | PR-03 · Address, back, forward, reload | yes |
| Inventory | Link-created tab vs Peek vs external vs blocked | G-8 · Peek preview; M-12 · Open in another app; M-13 · Pop-ups & new windows; PR-04 · Open links — foreground, background, Peek, external, pop-up | yes |
| Inventory | Foreground/background link opening | PR-04 · Open links — foreground, background, Peek, external, pop-up | yes |
| Inventory | Titlebar and shortcut back/forward/reload | W-7 · Shell — titlebar navigation states; PR-03 · Address, back, forward, reload | yes |
| Inventory | Reopen recently closed tabs | PR-05 · Tab life — pin, folder, favorite, close, reopen, suspend; M-6 · History | yes |
| Inventory | Downloads, find, errors, auth, permissions, file panels | M-4 · Downloads panel; G-6 · Find in page; G-5 · Glass — load errors; M-10 · Sign-in challenges; M-9 · Permission requests; M-14 · File upload & save | yes |
| Inventory | Saved Pages | M-5 · Saved Pages Library; PR-12 · Save a page & Library | yes |
| Inventory | Native context menus | M-1 · Tab menu; M-2 · Favorite & folder menus; M-3 · Page menu & web menus | yes |
| Inventory | Split-view daily use | G-9 · Split view — two panes; G-10 · Split view — create, resize, compact; PR-09 · Split view — create, focus, resize, close | yes |
| Inventory | External URLs / default browser | M-12 · Open in another app; PR-20 · Links from other apps | yes |
| Inventory | Per-site zoom/settings | G-7 · Page zoom; M-8 · Site settings; PR-16 · Per-site zoom | yes |
| Inventory | Session restore and crash recovery | L-10 · Session restore; L-11 · Recovery & migration errors; PR-18 · Quit, relaunch, crash recovery | yes |
| E2E smoke/shutdown | Launch and quit | L-10 · Session restore; L-14 · Windows — new, close, quit | yes |
| E2E tabs/shell-tabs | Tab open/select/close/reopen | S-5 · Tab rows in context; PR-05 · Tab life — pin, folder, favorite, close, reopen, suspend | yes |
| E2E profiles | Profile-isolated sessions | P-5 · Profile identity in chrome; P-7 · Move a tab across profiles | yes |
| E2E shell-spaces | Spaces, strip, swipe, configuration | P-1 · Space switching; P-2 · Space configuration; P-3 · New space | yes |
| E2E shell-sidebar | Rail, favorites, lift, appearance | W-1 · Shell — standard; S-2 · Favorites grid; S-3 · Space heading & pinned tabs | yes |
| E2E shell-favicons | Favicon slots | S-5 · Tab rows in context; B-4 · Tiles, chips & badges | yes |
| E2E shell-titlebar-navigation | Titlebar controls | W-7 · Shell — titlebar navigation states | yes |
| E2E shell-split-geometry | Split geometry | G-9 · Split view — two panes | yes |
| E2E shell-organize | Folders and organized area | S-4 · Folders; PR-06 · Organize — folders & drag rules | yes |
| E2E shell-tab-menu | Tab menu | M-1 · Tab menu | yes |
| E2E shell-hover | Hover and press | B-3 · Rail rows — §10 states | yes |
| E2E shell-command | ⌘T/⌘L command surfaces | C-1 · ⌘T — empty with recents; C-2 · ⌘T — results by kind; C-3 · ⌘T — no match & web search; C-4 · ⌘T — actions; C-5 · ⌘T — tab in another space or profile; C-6 · ⌘L — edit address | yes |
| E2E shell-saved-pages(-library) | Saved Pages and Library | M-5 · Saved Pages Library; C-2 · ⌘T — results by kind | yes |
| E2E shell-new-window | New-window requests | M-13 · Pop-ups & new windows; W-9 · Shell — several windows | yes |
| E2E background-links | Background link tabs | PR-04 · Open links — foreground, background, Peek, external, pop-up | yes |
| E2E web-navigation-gestures | Navigation gestures | PR-03 · Address, back, forward, reload | yes |
| E2E peek | Peek preview | G-8 · Peek preview | yes |
| E2E shell-persistence | Restore, space paper, activation | L-10 · Session restore; P-2 · Space configuration | yes |
| E2E shell-portal | Portal host above web | B-6 · Floating shells | yes |
| E2E shell-find | Find bar | G-6 · Find in page | yes |
| E2E shell-zoom | Zoom chip | G-7 · Page zoom; S-1 · Address control | yes |
| E2E shell-context-menu | Page/favorite/folder menus | M-2 · Favorite & folder menus; M-3 · Page menu & web menus | yes |
| E2E shell-dense | Dense list | W-8 · Shell — dense list (500 tabs) | yes |
| E2E shell-downloads | Downloads panel | M-4 · Downloads panel | yes |
| E2E external-url | External URL opening | M-12 · Open in another app | yes |
| E2E shell-errors | Error surfaces and illustrations | G-2 · Glass — empty space; G-3 · Glass — crashed page; G-5 · Glass — load errors | yes |
| E2E shell-responsiveness | Unresponsive renderer | G-4 · Glass — unresponsive page | yes |
| E2E auth | Authentication sheet | M-10 · Sign-in challenges | yes |
| E2E gallery | Component Gallery stories | B-1 · Buttons & keys; B-2 · Fields, toggles & focus; B-3 · Rail rows — §10 states; B-4 · Tiles, chips & badges; B-5 · Menu & result items; B-6 · Floating shells; B-7 · Feedback — progress, attention, banners | yes |
| E2E perf | Presentation budgets (no visual surface) | V-6 · Motion — transitions | planned |

Board inventory (page · code · title · mark · DESIGN · packet):

| Page | Board | Mark | DESIGN | Packet | Drawn |
|---|---|---|---|---|---|
| 00 | I-1 · Read me first | Shipped | all | paper-browser-blueprint | yes |
| 00 | I-2 · Coverage matrix | Shipped | all | paper-browser-blueprint | yes |
| 10 | F-1 · Colour — neutrals, accent, status | Shipped | §3 §18.2 | [v2-tokens](v2-tokens.md) | yes |
| 10 | F-2 · Colour — space paper, grain, monograms | Shipped | §3.5 §7.2 §11.3 §12 | [v2-theme-picker](v2-theme-picker.md) | yes |
| 10 | F-3 · Type & density | Shipped | §4 §8 §18.3 | [global-ui-scale-density](global-ui-scale-density.md) | yes |
| 10 | F-4 · Space, radius, elevation, materials | Shipped | §2 §5 §18.2 §18.3 | [v2-tokens](v2-tokens.md) | yes |
| 10 | F-5 · Icons & identity glyphs | Shipped | §7 | [icon-system](icon-system.md) | yes |
| 10 | F-6 · Motion tokens | Shipped | §6 §18.5 | [v2-motion-core](v2-motion-core.md) | yes |
| 11 | B-1 · Buttons & keys | Shipped | §10 §11.8 §17.2 | [ui-contract-conformance](ui-contract-conformance.md) | yes |
| 11 | B-2 · Fields, toggles & focus | Change | §9 §11.3 §11.5 | [ui-contract-conformance](ui-contract-conformance.md) | yes |
| 11 | B-3 · Rail rows — §10 states | Shipped | §10 §11.1 §11.2 §11.11 | [sidebar-reference-polish](sidebar-reference-polish.md) | yes |
| 11 | B-4 · Tiles, chips & badges | Shipped | §7.2 §11.3 §11.5 §11.10 | [favorites-drag-offers](favorites-drag-offers.md) | yes |
| 11 | B-5 · Menu & result items | Shipped | §11.4 §11.13 §18.4 | [tab-favorite-context-menus](tab-favorite-context-menus.md) | yes |
| 11 | B-6 · Floating shells | Shipped | §18.4 §11.12 §11.13 | [v2-overlays](v2-overlays.md) | yes |
| 11 | B-7 · Feedback — progress, attention, banners | Change | §10.5 §10.6 §18.5 | [v2-motion-core](v2-motion-core.md) | yes |
| 20 | W-1 · Shell — standard | Shipped | §1 §2 §11.4 | [visual-shell-reference](visual-shell-reference.md) | yes |
| 20 | W-2 · Shell — space tint | Shipped | §11.3 §18.2 | [v2-paper-alignment](v2-paper-alignment.md) | yes |
| 20 | W-3 · Shell — sidebar hidden ⌘B & hover reveal | Change | §2 §11.4 §17 | [daily-driver-sidebar-main](daily-driver-sidebar-main.md) | yes |
| 20 | W-4 · Shell — narrow & wide windows | Change | §5 §8 | [global-ui-scale-density](global-ui-scale-density.md) | yes |
| 20 | W-5 · Shell — full screen | New · proposal | §2 | paper-browser-blueprint | yes |
| 20 | W-6 · Shell — page loading | Shipped | §10.5 §18.5 D5 | [v2-motion-core](v2-motion-core.md) | yes |
| 20 | W-7 · Shell — titlebar navigation states | Shipped | §11.14 | [titlebar-navigation-controls](titlebar-navigation-controls.md) | yes |
| 20 | W-8 · Shell — dense list (500 tabs) | Shipped | §8 §11.6 | [dense-list-slice](dense-list-slice.md) | yes |
| 20 | W-9 · Shell — several windows | Change | §2 §11.4 | [new-window-production-seam](new-window-production-seam.md) | yes |
| 21 | S-1 · Address control | Shipped | §11.5 §17.13 §18.5 | [address-field-space-identity](address-field-space-identity.md) | yes |
| 21 | S-2 · Favorites grid | Shipped | §11.10 §18.5 | [favorites-drag-offers](favorites-drag-offers.md) | yes |
| 21 | S-3 · Space heading & pinned tabs | Shipped | §11.2 §11.3 | [sidebar-reference-polish](sidebar-reference-polish.md) | yes |
| 21 | S-4 · Folders | Shipped | §11.11 | [folder-context-menu](folder-context-menu.md) | yes |
| 21 | S-5 · Tab rows in context | Shipped | §10 §11.1 §11.9 | [sidebar-reference-polish](sidebar-reference-polish.md) | yes |
| 21 | S-6 · New Tab row & staged tab | Shipped | §11.4 | [browser-mvp-sidebar-ui](browser-mvp-sidebar-ui.md) | yes |
| 21 | S-7 · Footer strip | Shipped | §11.3 §11.4 | [space-footer-strip-appearance-ui](space-footer-strip-appearance-ui.md) | yes |
| 21 | S-8 · Media card | New · proposal | §11.15 | [sidebar-media-control](sidebar-media-control.md) | yes |
| 21 | S-9 · Drag in flight | Shipped | §10.9 §11.2 §11.10 §11.11 | [favorites-drag-offers](favorites-drag-offers.md) | yes |
| 21 | S-10 · Rail width & resize | Shipped | §5 §8 | [daily-driver-sidebar-main](daily-driver-sidebar-main.md) | yes |
| 22 | P-1 · Space switching | Shipped | §11.3 §18.5 D4 | [space-footer-strip-appearance-ui](space-footer-strip-appearance-ui.md) | yes |
| 22 | P-2 · Space configuration | Shipped | §11.3 D7 | [v2-theme-picker](v2-theme-picker.md) | yes |
| 22 | P-3 · New space | Shipped | §11.3 | [space-creation-appearance-ui](space-creation-appearance-ui.md) | yes |
| 22 | P-4 · Space menu & delete | Change | §11.3 §11.13 | [space-creation-appearance-ui](space-creation-appearance-ui.md) | yes |
| 22 | P-5 · Profile identity in chrome | Shipped | §12 | [cross-context-profile-hint](cross-context-profile-hint.md) | yes |
| 22 | P-6 · Profiles — create & edit | New · proposal | §12 | paper-browser-blueprint | yes |
| 22 | P-7 · Move a tab across profiles | Change | §12 PRODUCT | [browser-controller](browser-controller.md) | yes |
| 23 | C-1 · ⌘T — empty with recents | Shipped | §11.4 §18.4 | [browser-mvp-palette-ui](browser-mvp-palette-ui.md) | yes |
| 23 | C-2 · ⌘T — results by kind | Shipped | §11.4 §11.16 §18.4 | [v2-paper-alignment](v2-paper-alignment.md) | yes |
| 23 | C-3 · ⌘T — no match & web search | Shipped | §11.4 | [general-search-settings-contract](general-search-settings-contract.md) | yes |
| 23 | C-4 · ⌘T — actions | Change | §11.4 | [command-composition](command-composition.md) | yes |
| 23 | C-5 · ⌘T — tab in another space or profile | Shipped | §11.4 §12 | [cross-context-profile-hint](cross-context-profile-hint.md) | yes |
| 23 | C-6 · ⌘L — edit address | Shipped | §11.4 §11.5 §18.4 | [url-address-control-palette](url-address-control-palette.md) | yes |
| 24 | G-1 · Glass — before attach & blank | Shipped | §2 §13 | [seated-content-card](seated-content-card.md) | yes |
| 24 | G-2 · Glass — empty space | Shipped | §11.8 D10 | [v2-empty-states](v2-empty-states.md) | yes |
| 24 | G-3 · Glass — crashed page | Shipped | §11.8 D10 | [v2-empty-states](v2-empty-states.md) | yes |
| 24 | G-4 · Glass — unresponsive page | Shipped | §11.8a | [renderer-isolation-unresponsive-tabs](renderer-isolation-unresponsive-tabs.md) | yes |
| 24 | G-5 · Glass — load errors | Shipped | §11.8 §10.7 | [error-surface-contract](error-surface-contract.md) | yes |
| 24 | G-6 · Find in page | Shipped | §11.12 | [find-in-page-portal-bar](find-in-page-portal-bar.md) | yes |
| 24 | G-7 · Page zoom | Change | §11.5 | [site-settings-zoom-contract](site-settings-zoom-contract.md) | yes |
| 24 | G-8 · Peek preview | Shipped | §11.17 | [link-preview-peek](link-preview-peek.md) | yes |
| 24 | G-9 · Split view — two panes | Change | §11.7 | [split-app-composition](split-app-composition.md) | yes |
| 24 | G-10 · Split view — create, resize, compact | Change | §11.7 | [split-app-composition](split-app-composition.md) | yes |
| 25 | M-1 · Tab menu | Shipped | §11.13 | [tab-favorite-context-menus](tab-favorite-context-menus.md) | yes |
| 25 | M-2 · Favorite & folder menus | Shipped | §11.13 §11.11 | [folder-context-menu](folder-context-menu.md) | yes |
| 25 | M-3 · Page menu & web menus | Shipped | §11.13 | [page-background-context-menu](page-background-context-menu.md) | yes |
| 25 | M-4 · Downloads panel | Shipped | §11.12 | [downloads-sidebar-panel](downloads-sidebar-panel.md) | yes |
| 25 | M-5 · Saved Pages Library | Shipped | §11.16 | [saved-pages-library](saved-pages-library.md) | yes |
| 25 | M-6 · History | New · proposal | phase 8 | paper-browser-blueprint | yes |
| 25 | M-7 · Clear browsing data | New · proposal | phase 8 | paper-browser-blueprint | yes |
| 25 | M-8 · Site settings | New · proposal | §11.5 phase 8 | [site-settings-zoom-contract](site-settings-zoom-contract.md) | yes |
| 25 | M-9 · Permission requests | New · proposal | phase 8 | [permissions-contract](permissions-contract.md) | yes |
| 25 | M-10 · Sign-in challenges | Shipped | phase 8 | [auth-challenges-contract](auth-challenges-contract.md) | yes |
| 25 | M-11 · Client certificates | New · proposal | phase 8 | [auth-challenges-contract](auth-challenges-contract.md) | yes |
| 25 | M-12 · Open in another app | Change | phase 8 | [external-url-open-contract](external-url-open-contract.md) | yes |
| 25 | M-13 · Pop-ups & new windows | Change | phase 8 | [new-window-production-seam](new-window-production-seam.md) | yes |
| 25 | M-14 · File upload & save | Native OS | phase 8 | [file-panels-contract](file-panels-contract.md) | yes |
| 26 | L-1 · First run — welcome | New · proposal | phase 9 | paper-browser-blueprint | yes |
| 26 | L-2 · First run — import, default browser, first space | New · proposal | phase 7 phase 9 | paper-browser-blueprint | yes |
| 26 | L-3 · Settings — General | New · proposal | phase 8 | [general-search-settings-contract](general-search-settings-contract.md) | yes |
| 26 | L-4 · Settings — Profiles & spaces | New · proposal | §12 | paper-browser-blueprint | yes |
| 26 | L-5 · Settings — Search | New · proposal | phase 8 | [general-search-settings-contract](general-search-settings-contract.md) | yes |
| 26 | L-6 · Settings — Privacy & site data | New · proposal | phase 8 | [site-settings-zoom-contract](site-settings-zoom-contract.md) | yes |
| 26 | L-7 · Settings — Downloads | New · proposal | §11.12 | [downloads-sidebar-panel](downloads-sidebar-panel.md) | yes |
| 26 | L-8 · Settings — Appearance | New · proposal | §3.4 §6 §8 | [v2-system-prefs](v2-system-prefs.md) | yes |
| 26 | L-9 · Settings — Shortcuts | New · proposal | §9 | paper-browser-blueprint | yes |
| 26 | L-10 · Session restore | Shipped | phase 7 | [persistence-hydration](persistence-hydration.md) | yes |
| 26 | L-11 · Recovery & migration errors | New · proposal | phase 7 | [persistence-sqlite-contract](persistence-sqlite-contract.md) | yes |
| 26 | L-12 · About & updates | New · proposal | phase 9 | [macos-app-packaging](macos-app-packaging.md) | yes |
| 26 | L-13 · Menu bar & Dock | Native OS | phase 8 | paper-browser-blueprint | yes |
| 26 | L-14 · Windows — new, close, quit | Change | phase 8 | [process-lifecycle-shutdown](process-lifecycle-shutdown.md) | yes |
| 40 | V-1 · Increased contrast | Shipped | §3.4 | [v2-system-prefs](v2-system-prefs.md) | planned |
| 40 | V-2 · Space paper matrix | Shipped | §11.3 D7 | [v2-theme-picker](v2-theme-picker.md) | planned |
| 40 | V-3 · Density S · N · XL | Shipped | §8 | [global-ui-scale-density](global-ui-scale-density.md) | planned |
| 40 | V-4 · Reduce Motion | Shipped | §6 §18.5 | [v2-system-prefs](v2-system-prefs.md) | planned |
| 40 | V-5 · Motion — moments | Shipped | §18.5 | [v2-motion-moments](v2-motion-moments.md) | planned |
| 40 | V-6 · Motion — transitions | Change | §6 §18.5 | [v2-motion-core](v2-motion-core.md) | planned |
| 90 | D-1 · Open decisions | New · proposal | §17 | paper-browser-blueprint | planned |
| 90 | D-2 · Delta — Paper vs code | Change | all | paper-browser-blueprint | planned |
| 30 | PR-01 · First run & onboarding | New · proposal | flow | paper-browser-blueprint | yes |
| 30 | PR-02 · New tab from ⌘T | Shipped | flow | paper-browser-blueprint | yes |
| 30 | PR-03 · Address, back, forward, reload | Shipped | flow | paper-browser-blueprint | yes |
| 30 | PR-04 · Open links — foreground, background, Peek, external, pop-up | Change | flow | paper-browser-blueprint | yes |
| 30 | PR-05 · Tab life — pin, folder, favorite, close, reopen, suspend | Shipped | flow | paper-browser-blueprint | yes |
| 30 | PR-06 · Organize — folders & drag rules | Shipped | flow | paper-browser-blueprint | yes |
| 30 | PR-07 · Spaces — create, style, switch, delete | Change | flow | paper-browser-blueprint | yes |
| 30 | PR-08 · Profiles — create, assign, move a tab | Change | flow | paper-browser-blueprint | yes |
| 30 | PR-09 · Split view — create, focus, resize, close | Change | flow | paper-browser-blueprint | yes |
| 30 | PR-10 · Downloads — progress, done, reveal, retry | Shipped | flow | paper-browser-blueprint | yes |
| 30 | PR-11 · Find in page | Shipped | flow | paper-browser-blueprint | yes |
| 30 | PR-12 · Save a page & Library | Shipped | flow | paper-browser-blueprint | yes |
| 30 | PR-13 · Errors & recovery | Shipped | flow | paper-browser-blueprint | yes |
| 30 | PR-14 · Permissions | New · proposal | flow | paper-browser-blueprint | yes |
| 30 | PR-15 · Sign-in challenges | Shipped | flow | paper-browser-blueprint | yes |
| 30 | PR-16 · Per-site zoom | Change | flow | paper-browser-blueprint | yes |
| 30 | PR-17 · Media in a background tab | New · proposal | flow | paper-browser-blueprint | yes |
| 30 | PR-18 · Quit, relaunch, crash recovery | Shipped | flow | paper-browser-blueprint | yes |
| 30 | PR-19 · Keyboard focus — chrome ↔ page | Change | flow | paper-browser-blueprint | yes |
| 30 | PR-20 · Links from other apps | Change | flow | paper-browser-blueprint | yes |
| 30 | PR-21 · Windows — new, move, close | Change | flow | paper-browser-blueprint | yes |
| 30 | PR-22 · History & clear data | New · proposal | flow | paper-browser-blueprint | yes |
| 30 | PR-23 · Change settings | New · proposal | flow | paper-browser-blueprint | yes |
<!-- coverage:end -->

## Delivery ledger

```text
Outcome:
W0:                     2026-09-28. Eight old pages renamed "Archiv · …";
                        13 new pages (00–90). Tokens synced from the code:
                        7 values corrected (font-ui → System Sans-Serif,
                        dark footer 5 %, address border/accessory, lift
                        shadow, palette N 600, IC washes 8/12 %) and 104
                        created (54 luminance-pinned rails token × step ×
                        mode, monogram slates, space-only identity hues,
                        elevation/card shadows, status surfaces, IC
                        tint.accent/address veils, grain, palette S/XL,
                        overlay min). Boards I-1 read-me (marks, naming,
                        waves, page map), I-2 coverage matrix, F-1…F-6
                        foundations, B-1…B-7 building blocks — each
                        screenshot-reviewed; fixes: IC washes, veil alpha
                        labels, row pinned states per §10.5–§10.11 (loading
                        title secondary, lifering error/unresponsive,
                        92 % drag with border.strong), elevation row,
                        danger labels, banner placement (no chassis gap).
W1:                     2026-09-28. Page 20 W-1…W-9 (standard, tint,
                        ⌘B hidden + hover-reveal proposal, narrow/minimum/
                        wide, full screen, loading with D5 frames, titlebar
                        states, 500-tab list, two windows), page 21
                        S-1…S-10 (address states incl. §17.13 proposals,
                        Favorites incl. drag offers, heading/pinned incl.
                        ghost row and S tiles, folders, rows in context,
                        New Tab, footer strip incl. plus menu and drag
                        target, media card capability states, drag in
                        flight ×5 targets, rail widths/divider), page 22
                        P-1…P-7 (switch frames swipe + D4 fade, config
                        panel, new space, space menu + delete sheet,
                        profile identity, new-profile sheet, cross-profile
                        move). Review fixes: minimum-window rail formula,
                        loading title colour on the selected row, inactive
                        window lights, fragment caption alignment, strip
                        focus ring clipping, chip targets, token-only tint
                        cross-fade layer (no colour-mix), config panel
                        widths, drag label clipping.
W2:                     2026-09-28. Page 23 C-1…C-6 (⌘T empty with recents
                        proposal, results by kind, no match, > actions
                        proposal, cross-space/profile results + one-shot
                        hint, ⌘L overlay with the URL selected), page 24
                        G-1…G-8 (canvas before attach / blank / page, empty
                        space with the D10 cards, crash heartbeat,
                        unresponsive Wait/Reload/Close, offline / DNS /
                        certificate errors with the code's copy, find bar
                        incl. no match, zoom readout proposal, Peek incl.
                        loading and crashed). Review fixes: SVG
                        fill-/stroke-opacity is ignored by Paper (switched
                        to opacity), card min-width clipping, placeholder
                        max-width, state rows fitted to the board, the >
                        prefix text.
W3:                     2026-09-28. Page 25 M-1…M-14: tab menu with the
                        Move to Space submenu (shipped rows), favorite and
                        folder menus with the safe delete, page menu +
                        WebKit element menu (Native OS) + back-history menu
                        proposal, Downloads popover (active/history/empty),
                        Saved Pages Library, and the new surfaces: History
                        (Library anatomy), Clear Browsing Data sheet, site
                        settings popover, permission prompts (camera/mic,
                        location, clipboard, in-use and quietly blocked
                        chips), sign-in sheet (shipped, + wrong password,
                        proxy), client certificates, open-in-app sheet +
                        no-app banner, pop-up notice + sign-in pop-up
                        window, native file panel context. Review fix: the
                        site-identity trigger drawn in the address.
W4:                     2026-09-28. Page 26 L-1…L-14: welcome in the first
                        tab, three skippable first-run steps (import,
                        default browser, first space), the Settings window
                        with seven panes (General, Profiles, Search,
                        Privacy, Downloads, Appearance, Shortcuts), chrome-
                        first session restore with notice, partial /
                        unreadable / newer-schema recovery panes, About +
                        update ready, menu bar and Dock (Native OS), new /
                        closed window with reopen toast; page 24 G-9/G-10
                        split view (two panes with focus outline and rail
                        pair mark, drop zone, divider drag, compact rest).
                        Review fixes: duplicated New Profile label, the
                        Shortcuts window height, the > prefix text.
W5:                     2026-09-28. Page 30 PR-01…PR-23: 109 numbered
                        frames (4–6 per flow) with action → result captions
                        and timing chips, composed from the page-11 blocks
                        as single-write frames (compact rail content so
                        every change stays in view). Storyboards are drawn
                        in light and name the boards holding each
                        surface's dark variant (I-1 read-me). Marks per
                        flow: shipped, change or proposal. Review fixes:
                        one selection per list, the tinted Done frame, the
                        in-use device chip. Two Paper disconnects rolled
                        back four boards once; they were redrawn and every
                        storyboard's frame count re-read from the file.
W6:
W7:
Token check:            W0 verify_tokens.py: 126 checked, 0 mismatches.
Coverage check:
Commits:
Worktree state:
```
