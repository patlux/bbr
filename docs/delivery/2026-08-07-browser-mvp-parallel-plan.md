# 2026-08-07 — Browser-MVP: Parallel-Delivery-Plan & Live-Status

Orchestrierungs-Ledger für den MVP-Abend. **Kein kanonisches Roadmap-Dokument** —
kanonische Checkboxen bleiben in `ROADMAP.md`/Phasen/Tracks/Task-Packets
(siehe [docs/roadmap/README.md](../roadmap/README.md)). Diese Datei hält den
Live-Stand der Orchestrierung und wird bei jedem Lane-Statuswechsel
aktualisiert (Launch, Delivery, Gate, Integration).

**Recovery-Hinweis:** Nach Session-Crash/Kompaktierung: diese Datei lesen,
dann `workflow_control list` → laufende Runs prüfen → ggf. `resumeFromRunId`.
Runs sind persistent; fertige Agents kommen aus dem Cache.

## Ziel (bis heute Abend)

MVP laut Goal 2026-08-07 10:45 CEST:

- GPUI-CE-Anchor integriert (Pin `568271c0`, `gpui-component =0.5.1`, `Root`
  ohne WKWebView-Ersatz)
- Command-Palette: zentriert via **Cmd-T** + Overlay am Sidebar-URL-Feld bei
  Fokus (Cmd-L/Klick)
- Zed-inspirierte Sidebar: Fav-Grid → Pinned → Folders → Tabs, Footer exakt
  `[Downloads] [Spaces] [+]`, Plus öffnet echtes Kontextmenü
  (New Space / New Folder)
- Startbare macOS `.app`
- Stretch: Persistence-Hydration, `target=_blank` → neuer Tab

## Regeln

- Jede Lane = 1 Background-Workflow, interne Phasen
  Writer→Review→gate→Remediation→Recheck; deepseek-Writer, luna-Reviewer
- Event-getrieben (followUp-Delivery), **kein** sleep/status-Polling
- Commits/Merges nur durch Orchestrator (serielles Kernel)
- Portable Lanes: **keine neuen Dependencies** (Cargo.lock-Merges trivial)
- GUI-E2E über Host-Lock (`with_gui_lock`) automatisch serialisiert
- U-Lanes branchen zwingend vom Post-Anchor-Integrations-SHA (arg)

## Lane-Status

Legende: ⬜ offen · 🟦 läuft · 🟨 geliefert/wartet auf Integration · ✅ integriert · ❌ blockiert

| # | Lane | Status | Run-ID | Branch/Worktree | Notiz |
|---|---|---|---|---|---|
| D0 | Delivery-Smoke (Dummy-Run beweist followUp) | ✅ | `d0-delivery-smoke-msixi7dy-urgp2y` | — | `DELIVERY_OK` automatisch zugestellt, 1.7 s |
| H0 | Tooling-Hotfix auf `main` (xcrun-Shim, flake-Hook, rusqlite `=0.38.0`, Lint-Fixes) | ⬜ | — | main | Orchestrator direkt, ~15 min; **alle Lanes branchen danach** |
| L0 | Anchor-Finish (kritischer Pfad) | ⬜ | — | `bbr-gpui-ce-components-anchor` | verify ✅, testkit-Clippy ✅; offen: E2E-Suiten, `just e2e`, Split-Geometry-Flake, Docs-Fakten |
| L1 | Packaging `.app` (Bundle, Plist, ad-hoc-Sign, Launch-Smoke) | ⬜ | — | — | MVP-Pflicht |
| L2 | `bbr-web` new-window-Contract (`target=_blank`/`window.open`) | ⬜ | — | — | Stretch; Fundament Link-Preview/Peek |
| L3 | Persistence-Hydration portable (chrome-first, sichtbare Sessions) | ⬜ | — | — | Stretch; 2 Phase-7-Items + Warm-Startup-Perf |
| L4 | Prep: DESIGN §11.4-Revision + Task-Packets U1/U2 (Assertion-IDs, exklusive Pfade) | ⬜ | — | — | nur Docs |
| L5 | Favorites+Folders Domain/Controller (`bbr-domain`/`bbr-browser`, inkl. New-Folder/New-Space-Kommandos) | ⬜ | — | — | gpui-frei; Fundament U2b/U2c/U2d |
| L6 | Downloads-Capability-Contract (`bbr-web` portabel) | ⬜ | — | — | füttert Footer-`[Downloads]` |
| L7 | Find-in-page-Contract (`bbr-web` portabel) | ⬜ | — | — | optional |
| U1 | Palette-UI: a) Cmd-T zentriert · b) Overlay am URL-Feld | ⬜ | — | — | **erst nach L0-Integration** (SHA-arg) |
| U2 | Sidebar-UI: a) Struktur+Footer · d) [+]-Menü · b) Fav-Grid · c) Folders-UI · Icon-Glyphs | ⬜ | — | — | **erst nach L0-Integration**; braucht L5 integriert |
| U3 | Startup-First-Frame-Perf-Beweis | ⬜ | — | — | optional, nur bei Kapazität |

## Merge-Reihenfolge (seriell, Orchestrator)

```
H0 → (Lanes branchen) → L0-Anchor → L5 → U1 → U2 → L2 → L6 → L7 → L3 → L1
```

Nach letzter Integration: einmal `just e2e`, frische `.app` + manueller Smoke.

## Checklisten

### D0 — Delivery-Smoke
- [x] Dummy-Background-Run gestartet (`d0-delivery-smoke-msixi7dy-urgp2y`)
- [x] followUp-Ergebnis automatisch in Session angekommen (kein Polling)

### H0 — Tooling-Hotfix `main`
- [ ] `scripts/xcode-shims/xcrun` + flake.nix-Hook übernommen
- [ ] `rusqlite =0.38.0` + `cargo update -p rusqlite --precise 0.38.0`
- [ ] `unused_assignments`-Lint-Fixes falls auf main rot
- [ ] `just verify` cold grün auf main
- [ ] Commit(s) auf main, SHA hier notiert: `________`

### L0 — Anchor-Finish
- [ ] E2E-Suiten: smoke, shell-split-geometry, shell-command grün
- [ ] Split-Geometry-Flake determinisiert (kein Order-Flake)
- [ ] Docs-Fakten korrigiert (Xcode 26.2-Pfad, MSRV-Claim, Task-Packet-Basis)
- [ ] einmal `just e2e` komplett grün
- [ ] Task-Packet-Checkboxen aktualisiert
- [ ] Commit-Gruppen (Orchestrator) + Merge in main; Integrations-SHA: `________`
- [ ] Housekeeping: alter `pi-tmux-resume`-Wächter (Writer `019fdb6b`) gestoppt

### L1 — Packaging
- [ ] `just`-Recipe: Bundle-Struktur + Info.plist + Icon-Platzhalter
- [ ] ad-hoc-Codesign; App startet per Doppelklick
- [ ] Launch-Smoke dokumentiert

### L2 — new-window-Contract
- [ ] portabler Contract in `bbr-web` (exklusives neues Modul)
- [ ] macOS-Adapter-Implementierung
- [ ] Fake-Engine + Contract-Tests; keine neuen Dependencies

### L3 — Hydration portable
- [ ] chrome-first Restore-Policy im Controller
- [ ] nur sichtbare/aktive Sessions hydratisieren (kein WKWebView-Massenstart)
- [ ] Contract-Tests; Warm-Startup-Perf-Szenario vorbereitet

### L4 — Prep-Docs
- [ ] DESIGN §11.4-Revision: Fav-Grid, Folders, Footer `[Downloads] [Spaces] [+]`, Palette-Platzierungen
- [ ] Task-Packet U1 (Scope, Nicht-Ziele, Assertion-IDs, exklusive Pfade)
- [ ] Task-Packet U2 (dito)

### L5 — Favs+Folders Domain
- [ ] `bbr-domain`: Folder-Entity (session-lokal), Favorites-Ordnung, Invarianten
- [ ] `bbr-browser`: Kommandos NewFolder/NewSpace/Fav-Toggle/Move-in-Folder
- [ ] Fake-Engine/Contract-Tests grün; keine neuen Dependencies

### L6 — Downloads-Contract
- [ ] portabler Contract (destination, progress, retry, failure) + Tests

### L7 — Find-in-page-Contract
- [ ] portabler Contract (query, next/prev, count) + Tests

### U1 — Palette-UI
- [ ] a) Cmd-T → zentrierte Palette (elev:2, Opacity `dur:fine`, Results `[icon][title][url][hint]`)
- [ ] b) Fokus aufs Sidebar-Command-Field → Overlay am Feld, gleiche Result-Engine
- [ ] E2E `e2e-shell-command` erweitert + `palette_open`-Perf-Szenario

### U2 — Sidebar-UI
- [ ] a) Regionen Fav-Grid → Pinned → Folders → Tabs; Footer ersetzt Statuszeile
- [ ] d) [+] öffnet echtes Kontextmenü → New Space / New Folder (Controller-Aktionen)
- [ ] b) Fav-Grid (Icon-Tiles)
- [ ] c) Folders-UI (auf/zu, Tabs enthalten)
- [ ] Icon-Glyphs (icon-system.md-Mitnahme)
- [ ] E2E `e2e-shell-sidebar` erweitert + Perf-Szenarien

### Abschluss
- [ ] Alle Integrationen seriell in Reihenfolge oben
- [ ] Einmal `just e2e` grün
- [ ] `.app` frisch gebaut + manueller Smoke
- [ ] Roadmap/Task-Packets aktualisiert (`just roadmap` grün)
- [ ] Delivery-Ledger je Slice im Task-Packet

## Status-Log

| Zeit (CEST) | Ereignis |
|---|---|
| ~13:45–14:20 | Anchor-Worktree: Metal-Toolchain installiert (host), xcrun-Shim + flake-Hook, rusqlite `=0.38.0`, 6× Lint-Fix → `just verify` + testkit-Clippy grün (uncommitted) |
| 14:4x | Plan erstellt, diese Datei angelegt; warte auf Go für D0→H0→Lane-Start |
| 14:5x | D0 Delivery-Smoke gestartet (`d0-delivery-smoke-msixi7dy-urgp2y`); Turn beendet, kein Polling |
| 14:5x | ✅ D0: `DELIVERY_OK` automatisch zugestellt (1.7 s, $0.002) — Event-Kette verifiziert |

## Offene Punkte außerhalb BBR

- [ ] Infra-Repo `main` ahead 2 → Push (wartet auf User-Ok)
