# Task packet — Reference-faithful production shell

Parents: [Phase 4](../phases/04-ui-shell.md) · [UI design-system track](../tracks/ui-design-system.md) · [Daily-driver Sidebar main](daily-driver-sidebar-main.md) · [titlebar navigation](titlebar-navigation-controls.md).

Visual authority: Patrick's 2× production-shell reference at `/Users/patwoz/Library/Application Support/CleanShot/media/media_aio6G8pltD/Screenshot_2026-08-12_12-16-59@2x.png`.

## Outcome

BBR presents a credible light macOS browser shell whose expanded Sidebar matches the reference's material, hierarchy, scale, density and titlebar integration while preserving BBR's real browser model and native WKWebView composition. The light, softly tinted rail extends behind the native traffic-light safe area; the active Space name is the primary heading; navigation and address display are compact; Favorites, tabs, folders and the bottom Downloads/Space/Plus strip use the reference's visual rhythm; no standalone BBR brand block remains.

## Scope

- Make the reference the canonical visual authority for the expanded production Sidebar and its unified titlebar/navigation row.
- Use a light, matte, softly tinted rail and reference-calibrated selection, control, Favorite and footer surfaces.
- Integrate GPUI chrome behind the public native transparent/full-size titlebar while deriving traffic-light-safe geometry from the native layout guide.
- Refine the production `bbr-ui` Sidebar composition, density tokens, Favorite tiles, tab rows, status visibility and footer surface.
- Preserve real active-Space, tab, Favorite and folder content; empty regions remain truthful rather than being populated in production for appearance.
- Extend typed semantics and Gallery fixtures only as needed for the changed production composition.
- Add deterministic testkit-driven visual evidence using the real app path. Prefer native window capture when the host grants Screen Recording; otherwise capture the running production GPUI window's exact Metal scene through GPUI's public `test-support` renderer path and pair it with native window/titlebar geometry diagnostics. The fixture may create real test tabs, Favorites and folders through existing production commands.
- Capture and inspect both a representative populated fixture and an ordinary current-data launch at 2×.

## Non-goals

- No fake production tabs, folders, Favorites, favicons, media sessions or capabilities.
- No second Sidebar renderer, Gallery-only substitute, page screenshot/texture composition or sampled site tint.
- No Split work, Folder-menu command semantics, Shift-link semantics, packaging, accessibility closeout, startup optimization or unrelated browser capability.
- No media control without a truthful public capability authority.
- No private WebKit SPI, hardcoded traffic-light coordinates, pixel-perfect test assertion or weakened test/performance contract.

## Constraints

- `TabId`, `SpaceId`, `ProfileId`, profile isolation, exact native session ownership and the WKWebView boundary remain unchanged.
- Existing accessibility ids, keyboard/pointer routes, dense-list bounds and performance scenario code/budgets remain intact. New visible controls require matching typed semantics.
- The production renderer comes from `bbr-ui`; Gallery and test fixtures consume it rather than copying it.
- The ordinary-data screenshot must show the actual state available to a fresh isolated profile. The representative screenshot must identify every testkit mutation used to populate the same production path. A direct-renderer fallback is chrome-only: it must not fabricate AppKit traffic lights or WKWebView pixels, and the ledger must state that boundary.
- Content-dependent differences from the reference are documented, not fabricated.
- One isolated writer owns `/Users/patwoz/dev/Personal/macos/bbr-visual-shell`; only the Orchestrator commits and integrates.

## Visual ledger

| Reference region | Production target | Acceptance classification |
|---|---|---|
| Native traffic lights over full-height rail | public transparent/full-size macOS titlebar; native safe-area geometry | window chrome/material |
| Compact top controls | Sidebar toggle, Back, Forward and Reload from existing authorities | hierarchy/geometry |
| Address display | existing non-editable address control under navigation | geometry/state styling |
| Favorite tiles | real Favorite-backed `TabId` tiles at reference-calibrated size | hierarchy/density |
| Space name | exact active Space title as the main list heading | hierarchy/typography |
| Selected and resting tab rows | real tabs with existing favicon/monogram, close and state authorities | density/state styling |
| Folder rows | real session-local folders only when present | data-dependent |
| Bottom control strip | Downloads, real Space identities and Plus on a quiet footer surface | geometry/material |
| Media card/control | omitted unless an existing eligible public capability supplies it | capability-dependent |

## Evidence

### Unit and contract

- [x] Light palette meets unchanged normal/increased-contrast contracts.
- [x] Normal-density geometry matches the settled reference scale while small/large/dense invariants remain coherent.
- [x] Active Space heading, conditional status, New Tab row and all changed visible ids derive from the production model.
- [x] Empty regions and representative populated regions preserve exact browser ownership and deterministic ordering.
- [x] Public native titlebar configuration remains full-size and traffic-light-safe without private APIs.

### Real-app visual evidence

- [x] Optimized 2× representative capture uses only deterministic fixture pages and existing production/testkit mutations; its exact app/window identity and setup are recorded. Screen Recording was unavailable, so the accepted artifact is the exact production GPUI Metal scene plus the same run's native parent-window/safe-titlebar diagnostics.
- [x] Optimized 2× ordinary current-data capture uses a fresh isolated profile with no fabricated content under the same direct-renderer evidence mode.
- [x] Both captures were inspected directly against the reference region by region and are recorded in this ledger, including truthful data/capability differences.

### Real-app behavioral evidence

- [x] Sidebar, Titlebar, Tabs, Spaces, Organize, Hover, Dense and Gallery suites pass under the repository GUI lock: Sidebar 12/12, Titlebar 5/5, Tabs 3/3, Spaces 20/20, Organize 6/6, Hover/semantics 3/3, Dense 3/3 and Gallery 44/44, executed serially from a fresh ordinary-`testkit` target.
- [x] Formatting, affected warning-denied Clippy, unit/contract tests, UI-dependency, scripts and diff checks pass; the roadmap control graph is proven for the candidate through the temp-index-staged invocation, which is the only truthful form while the packet is still uncommitted.
- [x] Performance code, guards, budgets and sample requirements compile unchanged; execution remains unexecuted/unpassed under the active override.

## Completion gates

```bash
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo test -p bbr-ui -p bbr-browser
CARGO_BUILD_JOBS=4 cargo test -p bbr --features testkit --bins
CARGO_BUILD_JOBS=4 cargo clippy -p bbr-ui --all-targets -- -D warnings
CARGO_BUILD_JOBS=4 cargo clippy -p bbr --features testkit --all-targets -- -D warnings
CARGO_BUILD_JOBS=4 cargo clippy -p bbr-gallery --all-targets -- -D warnings
just ui-deps
just e2e-shell-sidebar
just e2e-shell-titlebar-navigation
just e2e-shell-tabs
just e2e-shell-spaces
just e2e-shell-organize
just e2e-shell-hover
just e2e-shell-dense
just e2e-gallery
just roadmap   # for the uncommitted candidate, run check_roadmap.py against a temp-index copy that stages every candidate path; plain `just roadmap` only passes after the Orchestrator commit, because the script validates links against tracked files
just script-tests
git diff --check
```

Performance execution, startup, canonical aggregate `just e2e`, packaging and accessibility closeout are intentionally not completion gates for this slice.

## Delivery sequence

1. Commit this visual authority with the production cut rather than leaving the old dark/anatomy-only interpretation authoritative.
2. Establish light rail and public full-size titlebar composition.
3. Refine top controls/address, Space hierarchy, Favorites/tabs/folders and footer in descending visual impact.
4. Populate a deterministic representative app state through existing production commands and capture both required app states at 2×.
5. Compare by region, remediate the highest-impact mismatch, then freeze the exact candidate.
6. Independent read-only visual/product/architecture review; one bounded remediation; focused recheck; serial integration onto local `main`.

## Iteration policy

At each checkpoint, compare the newest real-app capture with the reference and classify mismatches as window chrome/material, hierarchy/content visibility, geometry/density, typography/iconography, state styling, or data/capability-dependent. Fix the smallest production authority responsible for the highest-impact mismatch, then re-run the focused semantic/geometry gate before another capture.

## Blocked stop condition

Stop fail-closed if public titlebar APIs cannot produce a traffic-light-safe full-height rail, if required visual parity would require fabricated production data/capabilities, if a reviewer cannot bind a verdict to the exact candidate, if the macOS GUI host is unreliable, or if a required gate fails without a defensible bounded fix. Preserve exact commit/tree/status fingerprints, captures, comparison findings, passed/unpassed gates, the preserved dirty-worktree fingerprints and the smallest user input needed.

## Delivery ledger

```text
Outcome: frozen pre-review visual candidate replaces the hardcoded dark/anatomy-only shell with the light reference-authoritative production rail while preserving real controller/session/profile ownership and native WKWebView composition.
Candidate binding: uncommitted sole-writer candidate from main@fa18ee54c781d37647560f14d004b6d11edad19b; 63 paths after the review remediation (60 modified plus 3 new files); real index remains base tree f9e8cc6dc60f9d245951e0d81cc79487cf47657c. The exact NUL-sorted path hash, temporary-index tree, binary-diff SHA and status hash are rebound externally after this ledger settles because a Git tree cannot truthfully contain its own hash.
Reference: /Users/patwoz/Library/Application Support/CleanShot/media/media_aio6G8pltD/Screenshot_2026-08-12_12-16-59@2x.png; 608×2814; SHA-256 d94ef99a1aa25b89aeff14c83e28d45646d6a899c5f5cc13bdb9f3e20cd62364.
Optimized evidence binary: target/release/bbr built from the final candidate with the explicit additive `visual-capture` feature; arm64; SHA-256 8d0b3818d86f8a00b8e90e8e3640ed6dca4685de53045055fb6c10ee72e9762b. Ordinary `testkit` E2E keeps the production executor because GPUI renderer support is no longer unified into it. The superseded pre-isolation evidence binary was a8c8ab90b0567f0a769cddaa226c26cdd586ed01fbbccda43863fd1c2ddbafd1.
Representative capture: /tmp/bbr-visual-final-captures-20260812T202249/representative.rail.png; 608×2814; SHA-256 0acb421eb1809bc507e384ca63a55b7f9654a6d0894e10f8dc885bf39dbd62dc. Full 2160×2814 GPUI scene 1ddfb9b594e9834c73bf57c684aa725b7443a6c96e6b5798f966fe5acb2d3b0f; typed snapshot a9ac6b8fccb4f0ba0e5353c9e1b91b82b2d360f794fd1489a3e798943285999b; run log 8251d55f04151cb7b3a73ec67f12ace6475e0427c1f0a70d15e361ca20862e08 (both captures). Production-path setup: four fixture-backed native WKWebView tabs, all four favorited through ToggleFavoriteTab, tab 0 pinned and selected through existing requests, one real empty collapsed folder through the Plus menu, web-focus rest state. Asserted snapshot state: light theme, visible rail at 300 pt, exact active-Space title and New Tab entry, four real tabs, four real favorites, one pinned tab, one truthful empty collapsed folder, positive native parent window number, 1080×1407 pt at backing scale 2.
Ordinary-data capture: /tmp/bbr-visual-final-captures-20260812T202249/ordinary.rail.png; 608×2814; SHA-256 1480c0a3534f207ec4ef0c4457a8fead7d468ad891648fc436e22bb249a34d48. Full GPUI scene 37deb8ac5d705d5de2fb078190cd3a38c18bfcd6fd082ea524b0ba2808a704a8; typed snapshot fd0429185cd2f40e989704fbcdee589caf208b11d291c2139129909afe638805. Fresh isolated profile: one real native tab, zero Favorites, zero folders, same 1080×1407/300/2× geometry and web-focus rest state. The superseded pre-isolation captures were 585394895ecd7ea206d24c25d816321c3a084382b20d344f17b78fd31d8773a0 and 2764611e5cb5561c2199be0cc37140b852bc2247905cacbf9f555ed47947f783; the final build reproduces the same reviewed composition.
Evidence mode: /usr/sbin/screencapture failed even for a display probe ('could not create image from display'), proving the current agent process lacks Screen Recording authority. The accepted public fallback is GPUI's test-support Window::render_to_image on the running production window's exact Metal scene. It is available only through the explicit additive `visual-capture` feature (which includes the authenticated local testkit gateway), restricted to one direct /tmp/*.png path, and absent from default and ordinary-`testkit` builds. It truthfully captures GPUI chrome but does not synthesize native AppKit traffic-light or WKWebView child pixels; those boundaries are separately proven by the typed native diagnostics above.
Region-by-region comparison:
- Window chrome/material: light rail is full-height at #E0E0E0 and the public transparent/full-size titlebar preserves the measured native safe area. GPUI-only artifact omits native traffic-light pixels; no white titlebar strip or private coordinate override exists. The reference's own top strip and its left diamond/pill cluster belong to the captured reference application's window chrome, not to a BBR-designed region, and are deliberately not reproduced.
- Titlebar/navigation/address: Sidebar/Back/Forward/Reload occupy the top safe-area row; the address display is reference-calibrated to a 4 pt rail inset and quiet #D4D4D4 surface. Disabled Back/Forward and enabled Reload derive from real nav state. The reference's trailing '140%' accessory inside the address pill is capability-dependent: BBR's live zoom level is not surfaced in this shell region, so the slot stays empty rather than showing an invented value.
- Favorites: four real TabId-backed destinations render as 64×44 pt #D2D2D2 tiles with deterministic existing favicon-monogram fallback. Tile order/activation retains exact TabId/SpaceId/ProfileId authority.
- Hierarchy/content: exact active Space title 'Default' is the primary heading; no BBR brand block or generic visible Favorites/Pinned/Tabs/Folders label remains. One exact pinned active tab, one real empty collapsed folder, New Tab and regular real tabs follow in production order.
- Rows/state: 36 pt rows on a continuous 36 pt grid, 14 pt system text, 18 pt favicon/monogram slots, 4 pt inset #EEF2F9 selected surface and 600-weight selected title. Known deliberate density deviation: the reference uses roughly a 40 pt pitch (36 pt surfaces separated by about 4 pt gaps) while the shipped rail places 36 pt rows back to back; the tighter grid is retained because the §11 dense-list bounds, hit targets and Dense E2E geometry are proven against it, and the gap remains an open follow-up rather than an undisclosed mismatch. No decorative rest-state accent rail; focus/drag/status keep the Phosphor authority. Close/pin reveal only on hover/press.
- Footer: 36 pt #F3F8FE strip implementing the DESIGN §11.4 anatomy `[Downloads] [scrollable Spaces strip] [+]` with glyph-only Downloads and Plus and the exact real Space identity rendered as validated emoji or canonical two-letter BBR glyph badge; full labels remain semantic names. This is anatomy conformance, not pixel equivalence: the reference footer shows a different control cluster (tall diamond and small glyph near the left third, one wide filled pill at the right edge, large empty gap), whereas BBR's chip follows its own Space authority immediately after Downloads.
Content/capability-dependent differences: no media card because no truthful public media-session capability exists; fixture pages have no real site favicons so the existing deterministic monogram fallback appears; the representative fixture has four real tabs and one real folder rather than Patrick's personal long data set; ordinary fresh-profile state truthfully has one tab and no Favorites/folders. No fake data or capability was added.
Unit/contract evidence: /tmp/bbr-visual-freeze-check-20260812T141614 — format, bbr-ui 139/139, app bins, bbr-testkit, warning-denied bbr-ui/bbr/bbr-gallery Clippy, default-feature bbr all-target check, ui-deps and diff-check green. Earlier direct-capture path/wire/default-build gates: /tmp/bbr-direct-texture-capture-check-20260812T135153. Non-GUI candidate pass: /tmp/bbr-visual-shell-nongui-20260812T133606 plus corrected focused recheck /tmp/bbr-visual-shell-nongui-recheck-20260812T133743.
Real-app E2E evidence: every affected suite passed serially under the repository GUI lock from fresh ordinary-`testkit` target `/tmp/bbr-visual-target-sidebar-green-20260812T144934`. Sidebar 12/12 — `/tmp/bbr-visual-sidebar-green-20260812T144934/e2e-shell-sidebar.log`, SHA-256 8ea3e3d5b16a73dad521eb7a2d426ddc88776794d071097667371c27a8741734. Titlebar 5/5 — `/tmp/bbr-visual-titlebar-20260812T145213/e2e-shell-titlebar-navigation.log`, 4741e613d9690946ece2d560d0fa6cb1c6a5ea720e19c671adb75f935085a3cb. Tabs 3/3 — `/tmp/bbr-visual-tabs-20260812T145235/e2e-shell-tabs.log`, 5589c221fdd22ccae87b068ff3bb8ca3102877299e88b2b5b81056343db8fb5e. Spaces 20/20 (9+4+2+5) — `/tmp/bbr-visual-spaces-green-20260812T145647/e2e-shell-spaces.log`, d9a62cfbb4601aa4f75d48234ab4e7dc80f7e1b1b235d237c5223198ee4ae2dc. Organize 6/6 — `/tmp/bbr-visual-organize-20260812T145740/e2e-shell-organize.log`, b79e642edf03e8e588e9fa281cd2ef87b0861e3292fe079ec390c2d5271e7458. Hover/semantics 3/3 — `/tmp/bbr-visual-hover-20260812T145800/e2e-shell-hover.log`, 1659413e181c2455252e67b37214b874f4970fb501c9ac8538c18f7320b647c7. Dense 3/3 including the 500-tab/50-session and edge scenarios — `/tmp/bbr-visual-dense-20260812T145822/e2e-shell-dense.log`, 3991f03cda4be14fa24f3e580694caae3d37ad2c88ed64b24dcfdf8701011ed9. Gallery 44/44 — `/tmp/bbr-visual-gallery-20260812T145915/e2e-gallery.log`, ef3483a5937e88b31b71829afc289ee1e9a6734e3b3c781e0911225d7ab9f37c. Both `visual-capture` evidence tests passed under the same lock from isolated target `/tmp/bbr-visual-capture-target-20260812T202249`. Persistence 5/5 was added after the review remediation touched the portable recovery default — `/tmp/bbr-visual-persistence-20260812T210033/e2e-shell-persistence.log`, SHA-256 c20854e065181e749c0ebb9da9f3b0e10d0d03403d75964802647e3ab5e475d0; the real-app restore path already required the 300 pt rail. No deadline, budget or assertion was weakened for any suite.
Independent review: three independent read-only reviewers (visual/product, architecture/tests, docs/evidence) plus one synthesizer examined the exact frozen candidate and reproduced its binding byte-for-byte. Verdict REVIEW_FINDINGS with sixteen findings, none blocking and none major: eight ledger/packet truth corrections (path count, derived 780 pt prose, unreproducible untracked fingerprint, roadmap-gate form, row-pitch deviation, footer anatomy-vs-pixel claim, zoom accessory, reference top chrome), one DESIGN §11.4 region-order contradiction, six stale 280 pt comments/fixtures and one dead binding. Verified non-findings include exact palette values, no private WebKit SPI/KVC/PID heuristics, GPUI-chrome-only composition with untouched web crates, fail-closed `visual-capture` isolation, unchanged deadlines/budgets, independently recomputed contrast, truthful capture/binary/log hashes and all files under the size guidance.
Remediation/recheck: two screenshot-led local remediation passes completed before freeze: Favorite surfaces/folder-label/rest-state/footer/address corrections, then representative fixture hierarchy. The first final Sidebar gate exposed stale 280 pt, standalone-brand, permanent-status and technical-section assertions; they now assert the 300 pt rail default, the active Space/New Tab hierarchy, the absent brand block, conditional loading/recovery status and the unchanged 64 pt content floor. The complementary 780 pt native content width is derived prose (1080 − 300); the suites assert the real native slot rectangle and its strictly positive floor rather than that literal. A second causal finding showed the initial capture wiring globally enabled GPUI `test-support`, changing every ordinary testkit executor, while dense quit could continue processing 1,537 buffered WebKit callbacks after `Quitting`. Capture is now isolated behind `visual-capture`, and the testkit pump exits its observation loop only after the real `cx.quit()` path and response are committed; native teardown/persistence/process ownership remain unchanged. Base control was green, diagnostic exit improved to 444 ms without deadline changes, fresh-target Dense passed, full Sidebar passed 12/12, and warning-denied ordinary/capture Clippy plus bbr-testkit 29/29 are green.
Review remediation: every finding was addressed causally rather than by softening a claim. The ledger now records 58 paths, marks 780 pt as derived prose, rebinds the untracked fingerprint to the named `git ls-files -o --exclude-standard -z` serialization, states the temp-index form of the roadmap gate, discloses the 36 pt-vs-40 pt row pitch as a deliberate retained grid, reframes the footer as §11.4 anatomy conformance instead of pixel equivalence, and classifies the reference zoom accessory and reference top chrome as capability/foreign-chrome differences. DESIGN §11.4 now describes the shipped and reference-faithful order (address → Favorites → active Space title → Pinned/Folders/New Tab/Tabs) including truthful omission of an empty Favorites region. The six stale 280 pt references now state 300 pt, and the dead regular-count binding is removed; no assertion, budget or deadline changed. A wider sweep for the same root cause found one substantive residue the reviewers had not reached: `bbr_persistence::SidebarState::default()` still recovered an invalid persisted rail to 280 pt, so a corrupt window row would restore a width that is no longer the product default. The portable default and its sidebar-contract test now pin 300 pt with an explicit reference to the `bbr_ui` authority; the 200..=400 pt validation range, the typed recovery warning and the untouched-browser-data guarantee are unchanged.
Integration: pending focused recheck of the remediation and serial local-main integration.
Preserved dirty-worktree audit: initial and latest fingerprints of /Users/patwoz/dev/Personal/macos/bbr remain exact — tree/index e043f90c9726518faa48f4c3c53fecaf36465d2c; status `git status --porcelain=v1 -z | shasum -a 256` = 16ad4ea18f20bd1e43f0f8112a8a913479afdeafabd97dd9b41e720057fa4503; worktree diff eb70caaf56cdd3a4dfe77353aeeb9e6c573b2b679c81c40f96a44711c35e1988; cached diff e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855. The untracked-file fingerprint is rebound to one explicitly named serialization because the original value was recorded without it: `git ls-files -o --exclude-standard -z | shasum -a 256` = 2c853bbf6653e5fe04d2ceaaccd51eee3ab90a03cafb449fab33f0c8c1775a8e over exactly 3 untracked paths (LC_ALL=C sorting yields the same value). The earlier ced8ddd3… value is superseded as unreproducible and must not be used for future comparison.
Deferred/unpassed gates: performance execution; startup; canonical just e2e; packaging; accessibility closeout; Split; Folder menu; Shift-link; media capability.
```
