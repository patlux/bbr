"""The blueprint's planned artboards (code, page, name, mark, DESIGN refs,
owning packet) and the coverage matrix rows that map to them."""

PAGES = {
    "00": "p-9-0", "10": "p-A-0", "11": "p-B-0", "20": "p-C-0", "21": "p-D-0",
    "22": "p-E-0", "23": "p-F-0", "24": "p-G-0", "25": "p-H-0", "26": "p-I-0",
    "30": "p-J-0", "40": "p-K-0", "90": "p-L-0",
}

S, C, N, O = "Shipped", "Change", "New · proposal", "Native OS"

# code: (page, title, mark, design refs, packet)
BOARDS = {
    # 00 index
    "I-1": ("00", "Read me first", S, "all", "paper-browser-blueprint"),
    "I-2": ("00", "Coverage matrix", S, "all", "paper-browser-blueprint"),
    # 10 foundations
    "F-1": ("10", "Colour — neutrals, accent, status", S, "§3 §18.2", "v2-tokens"),
    "F-2": ("10", "Colour — space paper, grain, monograms", S, "§3.5 §7.2 §11.3 §12", "v2-theme-picker"),
    "F-3": ("10", "Type & density", S, "§4 §8 §18.3", "global-ui-scale-density"),
    "F-4": ("10", "Space, radius, elevation, materials", S, "§2 §5 §18.2 §18.3", "v2-tokens"),
    "F-5": ("10", "Icons & identity glyphs", S, "§7", "icon-system"),
    "F-6": ("10", "Motion tokens", S, "§6 §18.5", "v2-motion-core"),
    # 11 building blocks
    "B-1": ("11", "Buttons & keys", S, "§10 §11.8 §17.2", "ui-contract-conformance"),
    "B-2": ("11", "Fields, toggles & focus", C, "§9 §11.3 §11.5", "ui-contract-conformance"),
    "B-3": ("11", "Rail rows — §10 states", S, "§10 §11.1 §11.2 §11.11", "sidebar-reference-polish"),
    "B-4": ("11", "Tiles, chips & badges", S, "§7.2 §11.3 §11.5 §11.10", "favorites-drag-offers"),
    "B-5": ("11", "Menu & result items", S, "§11.4 §11.13 §18.4", "tab-favorite-context-menus"),
    "B-6": ("11", "Floating shells", S, "§18.4 §11.12 §11.13", "v2-overlays"),
    "B-7": ("11", "Feedback — progress, attention, banners", C, "§10.5 §10.6 §18.5", "v2-motion-core"),
    # 20 window & shell
    "W-1": ("20", "Shell — standard", S, "§1 §2 §11.4", "visual-shell-reference"),
    "W-2": ("20", "Shell — space tint", S, "§11.3 §18.2", "v2-paper-alignment"),
    "W-3": ("20", "Shell — sidebar hidden ⌘B & hover reveal", C, "§2 §11.4 §17", "daily-driver-sidebar-main"),
    "W-4": ("20", "Shell — narrow & wide windows", C, "§5 §8", "global-ui-scale-density"),
    "W-5": ("20", "Shell — full screen", N, "§2", "paper-browser-blueprint"),
    "W-6": ("20", "Shell — page loading", S, "§10.5 §18.5 D5", "v2-motion-core"),
    "W-7": ("20", "Shell — titlebar navigation states", S, "§11.14", "titlebar-navigation-controls"),
    "W-8": ("20", "Shell — dense list (500 tabs)", S, "§8 §11.6", "dense-list-slice"),
    "W-9": ("20", "Shell — several windows", C, "§2 §11.4", "new-window-production-seam"),
    # 21 sidebar
    "S-1": ("21", "Address control", S, "§11.5 §17.13 §18.5", "address-field-space-identity"),
    "S-2": ("21", "Favorites grid", S, "§11.10 §18.5", "favorites-drag-offers"),
    "S-3": ("21", "Space heading & pinned tabs", S, "§11.2 §11.3", "sidebar-reference-polish"),
    "S-4": ("21", "Folders", S, "§11.11", "folder-context-menu"),
    "S-5": ("21", "Tab rows in context", S, "§10 §11.1 §11.9", "sidebar-reference-polish"),
    "S-6": ("21", "New Tab row & staged tab", S, "§11.4", "browser-mvp-sidebar-ui"),
    "S-7": ("21", "Footer strip", S, "§11.3 §11.4", "space-footer-strip-appearance-ui"),
    "S-8": ("21", "Media card", N, "§11.15", "sidebar-media-control"),
    "S-9": ("21", "Drag in flight", S, "§10.9 §11.2 §11.10 §11.11", "favorites-drag-offers"),
    "S-10": ("21", "Rail width & resize", S, "§5 §8", "daily-driver-sidebar-main"),
    # 22 spaces & profiles
    "P-1": ("22", "Space switching", S, "§11.3 §18.5 D4", "space-footer-strip-appearance-ui"),
    "P-2": ("22", "Space configuration", S, "§11.3 D7", "v2-theme-picker"),
    "P-3": ("22", "New space", S, "§11.3", "space-creation-appearance-ui"),
    "P-4": ("22", "Space menu & delete", C, "§11.3 §11.13", "space-creation-appearance-ui"),
    "P-5": ("22", "Profile identity in chrome", S, "§12", "cross-context-profile-hint"),
    "P-6": ("22", "Profiles — create & edit", N, "§12", "paper-browser-blueprint"),
    "P-7": ("22", "Move a tab across profiles", C, "§12 PRODUCT", "browser-controller"),
    # 23 command
    "C-1": ("23", "⌘T — empty with recents", S, "§11.4 §18.4", "browser-mvp-palette-ui"),
    "C-2": ("23", "⌘T — results by kind", S, "§11.4 §11.16 §18.4", "v2-paper-alignment"),
    "C-3": ("23", "⌘T — no match & web search", S, "§11.4", "general-search-settings-contract"),
    "C-4": ("23", "⌘T — actions", C, "§11.4", "command-composition"),
    "C-5": ("23", "⌘T — tab in another space or profile", S, "§11.4 §12", "cross-context-profile-hint"),
    "C-6": ("23", "⌘L — edit address", S, "§11.4 §11.5 §18.4", "url-address-control-palette"),
    # 24 content & glass
    "G-1": ("24", "Glass — before attach & blank", S, "§2 §13", "seated-content-card"),
    "G-2": ("24", "Glass — empty space", S, "§11.8 D10", "v2-empty-states"),
    "G-3": ("24", "Glass — crashed page", S, "§11.8 D10", "v2-empty-states"),
    "G-4": ("24", "Glass — unresponsive page", S, "§11.8a", "renderer-isolation-unresponsive-tabs"),
    "G-5": ("24", "Glass — load errors", S, "§11.8 §10.7", "error-surface-contract"),
    "G-6": ("24", "Find in page", S, "§11.12", "find-in-page-portal-bar"),
    "G-7": ("24", "Page zoom", C, "§11.5", "site-settings-zoom-contract"),
    "G-8": ("24", "Peek preview", S, "§11.17", "link-preview-peek"),
    "G-9": ("24", "Split view — two panes", C, "§11.7", "split-app-composition"),
    "G-10": ("24", "Split view — create, resize, compact", C, "§11.7", "split-app-composition"),
    # 25 menus & panels
    "M-1": ("25", "Tab menu", S, "§11.13", "tab-favorite-context-menus"),
    "M-2": ("25", "Favorite & folder menus", S, "§11.13 §11.11", "folder-context-menu"),
    "M-3": ("25", "Page menu & web menus", S, "§11.13", "page-background-context-menu"),
    "M-4": ("25", "Downloads panel", S, "§11.12", "downloads-sidebar-panel"),
    "M-5": ("25", "Saved Pages Library", S, "§11.16", "saved-pages-library"),
    "M-6": ("25", "History", N, "phase 8", "paper-browser-blueprint"),
    "M-7": ("25", "Clear browsing data", N, "phase 8", "paper-browser-blueprint"),
    "M-8": ("25", "Site settings", N, "§11.5 phase 8", "site-settings-zoom-contract"),
    "M-9": ("25", "Permission requests", N, "phase 8", "permissions-contract"),
    "M-10": ("25", "Sign-in challenges", S, "phase 8", "auth-challenges-contract"),
    "M-11": ("25", "Client certificates", N, "phase 8", "auth-challenges-contract"),
    "M-12": ("25", "Open in another app", C, "phase 8", "external-url-open-contract"),
    "M-13": ("25", "Pop-ups & new windows", C, "phase 8", "new-window-production-seam"),
    "M-14": ("25", "File upload & save", O, "phase 8", "file-panels-contract"),
    # 26 system & lifecycle
    "L-1": ("26", "First run — welcome", N, "phase 9", "paper-browser-blueprint"),
    "L-2": ("26", "First run — import, default browser, first space", N, "phase 7 phase 9", "paper-browser-blueprint"),
    "L-3": ("26", "Settings — General", N, "phase 8", "general-search-settings-contract"),
    "L-4": ("26", "Settings — Profiles & spaces", N, "§12", "paper-browser-blueprint"),
    "L-5": ("26", "Settings — Search", N, "phase 8", "general-search-settings-contract"),
    "L-6": ("26", "Settings — Privacy & site data", N, "phase 8", "site-settings-zoom-contract"),
    "L-7": ("26", "Settings — Downloads", N, "§11.12", "downloads-sidebar-panel"),
    "L-8": ("26", "Settings — Appearance", N, "§3.4 §6 §8", "v2-system-prefs"),
    "L-9": ("26", "Settings — Shortcuts", N, "§9", "paper-browser-blueprint"),
    "L-10": ("26", "Session restore", S, "phase 7", "persistence-hydration"),
    "L-11": ("26", "Recovery & migration errors", N, "phase 7", "persistence-sqlite-contract"),
    "L-12": ("26", "About & updates", N, "phase 9", "macos-app-packaging"),
    "L-13": ("26", "Menu bar & Dock", O, "phase 8", "paper-browser-blueprint"),
    "L-14": ("26", "Windows — new, close, quit", C, "phase 8", "process-lifecycle-shutdown"),
    # 40 variants & motion
    "V-1": ("40", "Increased contrast", S, "§3.4", "v2-system-prefs"),
    "V-2": ("40", "Space paper matrix", S, "§11.3 D7", "v2-theme-picker"),
    "V-3": ("40", "Density S · N · XL", S, "§8", "global-ui-scale-density"),
    "V-4": ("40", "Reduce Motion", S, "§6 §18.5", "v2-system-prefs"),
    "V-5": ("40", "Motion — moments", S, "§18.5", "v2-motion-moments"),
    "V-6": ("40", "Motion — transitions", C, "§6 §18.5", "v2-motion-core"),
    # 90 decisions & delta
    "D-1": ("90", "Open decisions", N, "§17", "paper-browser-blueprint"),
    "D-2": ("90", "Delta — Paper vs code", C, "all", "paper-browser-blueprint"),
}

PROCESSES = [
    "First run & onboarding", "New tab from ⌘T", "Address, back, forward, reload",
    "Open links — foreground, background, Peek, external, pop-up",
    "Tab life — pin, folder, favorite, close, reopen, suspend",
    "Organize — folders & drag rules", "Spaces — create, style, switch, delete",
    "Profiles — create, assign, move a tab", "Split view — create, focus, resize, close",
    "Downloads — progress, done, reveal, retry", "Find in page", "Save a page & Library",
    "Errors & recovery", "Permissions", "Sign-in challenges", "Per-site zoom",
    "Media in a background tab", "Quit, relaunch, crash recovery",
    "Keyboard focus — chrome ↔ page", "Links from other apps", "Windows — new, move, close",
    "History & clear data", "Change settings",
]
PROCESS_MARKS = "N S S C S S C C C S S S S N S C N S C C C N N".split()
for index, (title, m) in enumerate(zip(PROCESSES, PROCESS_MARKS), start=1):
    BOARDS[f"PR-{index:02d}"] = ("30", title, {"S": S, "C": C, "N": N}[m], "flow", "paper-browser-blueprint")


def board_name(code):
    page, title, *_ = BOARDS[code]
    return f"{code} · {title}"
