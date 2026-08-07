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
  Writer→Review→gate→Remediation→Recheck
- **Modell-Policy ab Steering:** alle Subagents in allen Rollen (Writer,
  Reviewer, Gate-Validatoren, Judges, Remediation, Recheck) ausschließlich
  explizit `model: "homelab-llm/deepseek-v4-flash:high"`; niemals `tier`
  oder Session-Default erben. `gpt-5.6-sol` bleibt exklusiv dem Orchestrator
  vorbehalten; Luna wird nicht verwendet. Bereits in-flight gestartete Calls
  dürfen mit ihrer bisherigen Route zu Ende laufen; jeder danach neu gestartete
  `agent()`-Call, jedes `resumeFromRunId` und U1/U2 folgt dieser Policy.
- Event-getrieben (followUp-Delivery), **kein** sleep/status-Polling
- Commits/Merges nur durch Orchestrator (serielles Kernel)
- Portable Lanes: **keine neuen Dependencies** (Cargo.lock-Merges trivial)
- GUI-E2E über Host-Lock (`with_gui_lock`) automatisch serialisiert
- U-Lanes branchen zwingend vom Post-Anchor-Integrations-SHA (arg)
- **UI-Recovery-Policy ab User-Steering:** Wenn der laufende U1-Writer abbricht
  oder U2-Remediation leer/rot endet, die Lane nicht erneut als monolithischen
  Writer starten. Stattdessen denselben erhaltenen Worktree mit 2–4
  sequenziellen, eng begrenzten DeepSeek-Writer-Steps fortsetzen, typischerweise
  Struktur → Verhalten → Tests → Gates. Jeder Step ist ein eigener `agent()`-Call
  mit kurzem Abschluss-/Handoff-Bericht als Input des nächsten Steps; jeder
  Writer-Prompt fordert explizit einen Abschlussbericht an. Alle Suchen bleiben
  repo-lokal per `rg` im Lane-Worktree; GPUI-Quellen höchstens unter
  `~/.cargo/git/checkouts`. Niemals `find /`, `find /nix` oder andere
  ungebundene Dateisystem-Suchen. Run-IDs und Step-Übergänge im Ledger erfassen.

## Lane-Status

Legende: ⬜ offen · 🟦 läuft · 🟨 geliefert/wartet auf Integration · ✅ integriert · ❌ blockiert

| # | Lane | Status | Run-ID | Branch/Worktree | Notiz |
|---|---|---|---|---|---|
| D0 | Delivery-Smoke (Dummy-Run beweist followUp) | ✅ | `bbr-d0-session-delivery-smoke-msiyp4bv-axc79v` | — | `DELIVERY_OK` automatisch zugestellt, 1.6 s |
| H0 | Tooling-Hotfix auf `main` (xcrun-Shim, flake-Hook, justfile Xcode 26.2, rusqlite `=0.38.0`, Lint-Fixes) | ✅ | — | main | Commits `b9bad16`+`c4ce31b`+`a50bb92`+`574750b`; **Lane-Base = `574750b`** |
| L0 | Anchor-Finish (kritischer Pfad) | ✅ | `bbr-mvp-l0-anchor-finish-msiyvhmf-fb9qjp` | `main` · Post-Anchor-SHA `f89a4799999d1d43f785da7c767a1df68759ee0b` | Anchor-Merge `45107a3` + Quit-Race-Remediation `f89a479`; verify/Clippy/vendor/Dense/kanonisches E2E grün |
| L1 | Packaging `.app` (Bundle, Plist, ad-hoc-Sign, Launch-Smoke) | ✅ Infrastruktur / ⬜ Final-Gate | `bbr-mvp-l1-packaging-msiyvhmh-43x8cf` | `main` · Merge `9b0afb7` | Infrastruktur Lane `5f36dc7` integriert: Bundle-/Plist-/Icon-/Codesign-/Smoke-Scripts, Recipes, Tests und Paketvertrag. Post-merge `package-test` 12/12, Py-Compile, vollständiges `just verify`, Roadmap grün. Finales `just package-verify` gegen fertigen Binary bleibt Schlussakt |
| L2 | `bbr-web` new-window-Contract (`target=_blank`/`window.open`) | ✅ | `bbr-mvp-l2-new-window-contract-msiyvhmj-uqwhk5` | `main` · Merge `6228efd0d24f1418477cf08f19477672a620a9bc` | User-gesteuert vorgezogen und seriell integriert. Lane `fb3ac58`; erwarteter additiver `bbr-web/lib.rs`-Hunk aufgelöst. Post-merge 54 Serde-Unit + 11 New-Window, macOS 1+8 Tests und beide Crate-Clippies grün; keine Dependencies/GUI-Gates |
| L3 | Persistence-Hydration portable (chrome-first, sichtbare Sessions) | ✅ | `bbr-mvp-l3-persistence-hydration-msiyvhmk-7oogla` + `bbr-mvp-l3-structure-remediation-msj0gjba-0m06xf` | `main` · Merge `d36d44cbcf8dbeceadbc8c60083932d539b79936` | User-gesteuert vorgezogen und seriell integriert. Lane `506f349`; additiver L5-Hunk korrigiert, Hydration-Tests test-gated. Post-merge 119 Browser-Tests, Crate-/Workspace-Clippy und Roadmap grün; max. 372 L |
| L4 | Prep: DESIGN §11.4-Revision + Task-Packets U1/U2 (Assertion-IDs, exklusive Pfade) | ✅ | `bbr-mvp-l4-ui-prep-docs-msiyvhml-qf9avh` | `main` · Merge `08004dc11d86ebf0beb29efb17b2d03f256eb60a` | DESIGN + U1/U2-Pakete integriert; Base-Arg `f89a479` eingetragen und Roadmap/Phase/Track promotet |
| L5 | Favorites+Folders Domain/Controller (`bbr-domain`/`bbr-browser`, inkl. New-Folder/New-Space-Kommandos) | ✅ | `bbr-mvp-l5-favorites-folders-msiyvhmm-7j1qvw` | `main` · Merge `e923716042f207ef94c4aa6a871e0c797a777e6e` | 67 Domain + 95 Browser-Tests, Crate-Clippy und Workspace-Check post-merge grün; U2-API integriert |
| L6 | Downloads-Capability-Contract (`bbr-web` portabel) | ✅ | `bbr-mvp-l6-downloads-contract-msiyvhmn-4s2ypx` | `main` · Merge `6912cab98b2ec5e170dabf3b806d6f2ea598452d` | User-gesteuert vorgezogen und seriell integriert. Lane `e9d2f50`; post-merge 38 serde Tests + Doctest und Crate-Clippy grün; Struktur max. 459 L; ausschließlich `bbr-web`, keine Dependencies |
| L7 | Find-in-page-Contract (`bbr-web` portabel) | ✅ | `bbr-mvp-l7-find-in-page-contract-msiyvhmo-wt6te6` | `main` · Merge `7ed1bed4e5a0010821d47550c5f93828904c5bc2` | User-gesteuert vorgezogen und seriell integriert. Lane `2cfba1e`; post-merge L6+L7 gemeinsam 53 Default / 54 Serde Tests + Doctest und Crate-Clippy grün; keine Dependencies |
| Portal | NativeAboveWeb/GpuiDeferred Architektur-Kernel | ✅ | `bbr-native-portal-host-msj9f75q-1k0tf0` | Lane `f915626` · `main` Merge `9cc8ad0b36f98f773973248ce5afc175f97277ff` | Review-UAF durch Box-Pinning behoben; native Child-Attachment/Resize/Hit-Test/Teardown bewiesen. Portal 3×3/3, Command 11/11, Sidebar 3/3, Lane `just e2e` ×2 und post-merge vollständiges `just e2e` grün; Startup 459,7 ms |
| U1 | Palette-UI: a) Cmd-T zentriert · b) Overlay am URL-Feld | 🟦 nächster Hauptpfad | `bbr-mvp-u1-bounded-recovery-msj3u8wd-qbue8s` | `/Users/patwoz/dev/Personal/macos/bbr-mvp-u1-palette` · `parallel/mvp-u1-palette` | Vollständiger Diff weiterhin erhalten. Portal-Blocker ist durch Merge `9cc8ad0` geschlossen; Worktree als Nächstes auf den integrierten Kernel aktualisieren und Palette/URL-Overlay über `PortalHost::NativeAboveWeb` migrieren, mit echter AppKit-Frontmost-/Pointer-/Focus-Evidenz |
| U2 | Sidebar-UI: a) Struktur+Footer · d) [+]-Menü · b) Fav-Grid · c) Folders-UI · Icon-Glyphs | 🟦 settled / Perf nach U1 | `bbr-mvp-u2-final-router-closeout-msjbkcws-4hjmzs` | `/Users/patwoz/dev/Personal/macos/bbr-mvp-u2-sidebar` · `parallel/mvp-u2-sidebar` | Finaler Recheck/Struktur und alle U2-Szenarien grün. Erster post-Portal-Warm-Lauf weiterhin unter hoher systemweiter Last (`6,90/7,21/7,84`, Spotlight ~94% CPU): ausschließlich Startup 512,3>500 ms; U2 p95 Favorites 0,018 ms, Folder 0,0045 ms, Plus 0,0115 ms. Kein Blind-Retry; nächster Perf-Lauf nach U1 auf finalem Kernel |
| U3 | Startup-First-Frame-Perf-Beweis | ⬜ | — | — | optional, nur bei Kapazität |
| P8a | Portable Permissions-Contract | 🟨 geliefert / wartet seriell | `bbr-p8a-permissions-contract-msjegrwu-af42jd` | `/Users/patwoz/dev/Personal/macos/bbr-p8a-permissions` · `parallel/p8a-permissions-contract` · Base `43ab542` | `READY_FOR_ORCHESTRATOR`, uncommitted. Exakt fünf Capabilities; profil-/origin-isolierte Policy, one-time/durable Decisions, Pending-/Resolve-/Cancel-/Supersede-/Stale-Lifecycle und object-safe Persistence-Hook. Default 79+11+1, Serde 82+11+1, beide Clippies, Fmt/Roadmap/Diff grün; keine Dependencies/Shared-Root-Drift, max. 452 L. Integration bleibt hinter U1→U2→Packaging zurückgestellt |
| P8b | Portable Web-Error-/Recovery-Contract | 🟨 geliefert / wartet seriell | `bbr-p8b-error-surface-contract-msjegrwv-u70nuq` | `/Users/patwoz/dev/Personal/macos/bbr-p8b-web-errors` · `parallel/p8b-error-surface-contract` · Base `43ab542` | `READY_FOR_ORCHESTRATOR`, uncommitted. Geschlossene TLS/Navigation/Offline/Crash-Taxonomie, legale Recovery-Matrix ohne TLS-Bypass, Incident-/Stale-Lifecycle und privacy-safe Origin-Kontext. Review-Fund zu `javascript:`/`data:`-Payload-Leak fokussiert behoben und reverifiziert. Default 94+11+1, Serde 99+11+1, beide Clippies/Fmt/Roadmap/Diff grün; keine Dependencies/Shared-Root-Drift, max. 494 L. Integration bleibt hinter U1→U2→Packaging zurückgestellt |
| P8c | Portable Auth-Challenges-Contract | 🟦 | `bbr-p8c-auth-challenges-contract-msjfa3hh-xqhshc` | `/Users/patwoz/dev/Personal/macos/bbr-p8c-auth-challenges` · `parallel/p8c-auth-challenges-contract` · Base `92a0bfb` | Exklusive `crates/bbr-web/src/auth_challenges/**` + Task-Ledger; HTTP Origin/Proxy Auth und Client-Cert Request/Response-Lifecycle, opaque sensitive handles, serde/security tests; DeepSeek, bounded sequenziell, `agentRetries: 1`, keine Dependencies/GUI-Gates/Agent-Commits |
| P8d | Portable File-Upload/Save-Panel-Contract | 🟨 geliefert / wartet seriell | `bbr-p8d-file-panels-contract-msjfa3hi-gaon2u` | `/Users/patwoz/dev/Personal/macos/bbr-p8d-file-panels` · `parallel/p8d-file-panels-contract` · Base `92a0bfb` | `READY_FOR_ORCHESTRATOR`, uncommitted. Typisierte Upload-/Save-Requests, Accept-/Selection-/Capture-/Replacement-Validierung, artgetrennte opaque u64-Tokens, Pending-/Selection-/Terminal-/Supersede-Lifecycle und object-safe `FilePanelPort`. Default 109+11+1, Serde 115+11+1, beide Clippies/Fmt/Roadmap/Diff grün; keine Dependencies/Shared-Root-Drift, alle Dateien <500 L. Native Panels/Pfade/Sandbox-Orchestrierung bleiben später; Integration hinter U1→U2→Packaging |
| P8e | Portable Per-Site-Settings/Zoom-Contract | 🟨 geliefert / wartet seriell | `bbr-p8e-site-settings-zoom-contract-msjfa3hj-t2il4d` | `/Users/patwoz/dev/Personal/macos/bbr-p8e-site-settings` · `parallel/p8e-site-settings-zoom-contract` · Base `92a0bfb` | `READY_FOR_ORCHESTRATOR`, uncommitted. Profilisolierter normalisierter Origin-Key, integerbasierter Zoom 25–400% in 25er Schritten ohne Float-Drift, sparse Overrides/effective values, object-safe Store-Port und versionierte Supersession-/Stale-Ack-Lifecycle. Default 120, Serde 133, beide Clippies/Fmt/Roadmap/Diff grün; keine Dependencies/Shared-Root-Drift, alle Dateien ≤500 L. Integration bleibt hinter U1→U2→Packaging zurückgestellt |
| P7 | Command-History-Provider | ❌ nicht gestartet | — | — | Veto: integriertes Persistence-Schema v1 besitzt keine History-Tabelle/API; `bbr-command` ist absichtlich provider-neutral. Echter Provider erfordert zuerst serielle Migration/DTO/API und danach Bridge/Manifest-Entscheidung; keine saubere exklusive Allowlist ohne App/U1/U2 oder Shared-Root-Flächen |

## Merge-Reihenfolge (seriell, Orchestrator)

```
H0 → (Lanes branchen) → L0-Anchor → L5 → L6 → L7 → L3 → L2 → L1-Infrastruktur → Portal → U1 → U2 → L1-finales-package-verify
```

Nach letzter Integration: einmal `just e2e`, frische `.app` + manueller Smoke.

## Checklisten

### D0 — Delivery-Smoke
- [x] Frühere Session: Dummy-Background-Run gestartet (`d0-delivery-smoke-msixi7dy-urgp2y`)
- [x] Frühere Session: followUp-Ergebnis automatisch angekommen (kein Polling)
- [x] Dedizierte sol-Orchestrator-Session: Dummy-Background-Run gestartet (`bbr-d0-session-delivery-smoke-msiyp4bv-axc79v`)
- [x] Dedizierte sol-Orchestrator-Session: followUp-Ergebnis automatisch angekommen (kein Polling)

### H0 — Tooling-Hotfix `main`
- [x] `scripts/xcode-shims/xcrun` + flake.nix-Hook übernommen
- [x] justfile-Default auf Xcode 26.2 (16.4 existiert nicht → `-lSystem`-Linkerfehler)
- [x] `rusqlite =0.38.0` + `cargo update -p rusqlite --precise 0.38.0`
- [x] `unused_assignments`-Lint-Fixes (6 Stellen, auch auf main rot)
- [x] `just verify` cold grün + testkit-Clippy grün auf main
- [x] Commits: `b9bad16` tooling · `c4ce31b` rusqlite · `a50bb92` lints · `574750b` ledger — **Lane-Base: `574750b`**

### L0 — Anchor-Finish
- [ ] E2E-Suiten: smoke, shell-split-geometry, shell-command grün
- [ ] Split-Geometry-Flake determinisiert (kein Order-Flake)
- [ ] Docs-Fakten korrigiert (Xcode 26.2-Pfad, MSRV-Claim, Task-Packet-Basis)
- [ ] einmal `just e2e` komplett grün
- [ ] Task-Packet-Checkboxen aktualisiert
- [x] Commit-Gruppen (Orchestrator) + Merge in main; Post-Anchor-Integrations-SHA: `f89a4799999d1d43f785da7c767a1df68759ee0b` *(Anchor-Merge `45107a3497bc12603f497ed964d39c37c083a940`, Integrationsremediation `f89a479`)*
- [x] Housekeeping: alter `pi-tmux-resume`-Wächter (Writer `019fdb6b`) gestoppt / nicht mehr vorhanden (Prozess- und tmux-Prüfung)

### L1 — Packaging
- [x] `just`-Recipe: Bundle-Struktur + Info.plist + Icon-Platzhalter
- [x] ad-hoc-Codesign; App startet per Doppelklick
- [x] Launch-Smoke dokumentiert

### L2 — new-window-Contract
- [x] portabler Contract in `bbr-web` (exklusives neues Modul)
- [x] macOS-Adapter-Implementierung
- [x] Fake-Engine + Contract-Tests; keine neuen Dependencies

### L3 — Hydration portable
- [x] chrome-first Restore-Policy im Controller
- [x] nur sichtbare/aktive Sessions hydratisieren (kein WKWebView-Massenstart)
- [x] Contract-Tests; Warm-Startup-Perf-Szenario vorbereitet

### L4 — Prep-Docs
- [x] DESIGN §11.4-Revision: Fav-Grid, Folders, Footer `[Downloads] [Spaces] [+]`, Palette-Platzierungen
- [x] Task-Packet U1 (Scope, Nicht-Ziele, Assertion-IDs, exklusive Pfade)
- [x] Task-Packet U2 (dito)

### L5 — Favs+Folders Domain
- [x] `bbr-domain`: Folder-Entity (session-lokal), Favorites-Ordnung, Invarianten
- [x] `bbr-browser`: Kommandos NewFolder/NewSpace/Fav-Toggle/Move-in-Folder
- [x] Fake-Engine/Contract-Tests grün; keine neuen Dependencies

### L6 — Downloads-Contract
- [x] portabler Contract (destination, progress, retry, failure) + Tests

### L7 — Find-in-page-Contract
- [x] portabler Contract (query, next/prev, count) + Tests

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
| 15:1x | ✅ H0: main cold-grün (verify + testkit-Clippy), 4 Commits, Lane-Base `574750b` |
| 15:1x | Entscheidung: Orchestrierung wechselt in dedizierte gpt-5.6-sol-Session (Option B); diese Datei = Handoff-Vertrag |
| sol-Kickoff | 🟦 D0 dieser Session gestartet (`bbr-d0-session-delivery-smoke-msiyp4bv-axc79v`); Turn endet ohne Polling, automatische Zustellung ausstehend |
| sol-Kickoff +1.6 s | ✅ D0: `DELIVERY_OK` automatisch zugestellt — Delivery-Kette dieser Session verifiziert |
| sol-Kickoff | Worktrees L1–L7 exakt von Lane-Base `574750b` erstellt; L0 bestehender dokumentierter Dirty-Worktree bestätigt |
| sol-Kickoff | 🟦 L0–L7 als acht Background-Workflows gestartet; je intern Writer→Review→Gate→ggf. Remediation→Recheck; keine Commits durch Subagents |
| User-Steering | Modell-Policy umgestellt: ab sofort jeder neue Subagent-Call explizit `homelab-llm/deepseek-v4-flash:high`; kein `tier`/Default, kein Luna, Sol nur Orchestrator; bereits in-flight Calls dürfen enden |
| User-Steering | L0–L7 an der jeweils aktuellen in-flight Phase pausiert, damit die alten Scripts keinen weiteren Call mit früherer Route starten; laufende Calls dürfen enden, Fortsetzungen werden mit expliziter neuer Modellroute resumed |
| User-Steering-Recovery | ❌ L3: in-flight Reviewer durch Pause abgebrochen (`Subagent was aborted`); kein Implementierungsurteil, Worktree erhalten |
| User-Steering-Recovery | 🟦 L3 unter gleicher Run-ID resumed; alle neuen Calls explizit `homelab-llm/deepseek-v4-flash:high`, kein `tier`/Default |
| User-Steering-Recovery | ❌ L2: in-flight Gate durch Pause abgebrochen (`Subagent was aborted`); kein Implementierungsurteil, Writer/Review/Worktree erhalten |
| User-Steering-Recovery | 🟦 L2 unter gleicher Run-ID ab Gate resumed; alle neuen Calls explizit `homelab-llm/deepseek-v4-flash:high`, kein `tier`/Default |
| User-Steering-Recovery | ❌ L6: in-flight Gate durch Pause abgebrochen (`Subagent was aborted`); kein Implementierungsurteil, Writer/Review/Worktree erhalten |
| User-Steering-Recovery | 🟦 L6 unter gleicher Run-ID ab Gate resumed; alle neuen Calls explizit `homelab-llm/deepseek-v4-flash:high`, kein `tier`/Default |
| User-Steering-Recovery | ❌ L5: in-flight Remediation durch Pause abgebrochen (`Subagent was aborted`); kein Implementierungsurteil, vorheriger und partieller Remediation-Stand im Worktree erhalten |
| User-Steering-Recovery | 🟦 L5 unter gleicher Run-ID mit frischer worktree-basierter Remediation resumed; alle neuen Calls explizit `homelab-llm/deepseek-v4-flash:high`, kein `tier`/Default |
| User-Steering-Recovery | ❌ L7: finaler in-flight Recheck durch Pause abgebrochen (`Subagent was aborted`); kein Implementierungsurteil, Writer/Review/Gate/Worktree erhalten |
| User-Steering-Recovery | 🟦 L7 unter gleicher Run-ID mit frischem finalen Recheck resumed; Call explizit `homelab-llm/deepseek-v4-flash:high`, kein `tier`/Default |
| User-Steering-Recovery | ❌ L1: finaler in-flight Recheck durch Pause abgebrochen (`Subagent was aborted`); kein Implementierungsurteil, Implementierung/Review/Gate/Remediation/Worktree erhalten |
| User-Steering-Recovery | 🟦 L1 unter gleicher Run-ID mit frischem finalen Packaging-Recheck resumed; Call explizit `homelab-llm/deepseek-v4-flash:high`, kein `tier`/Default |
| User-Steering-Recovery | ❌ L4: in-flight Gate durch Pause abgebrochen (`Subagent was aborted`); kein Implementierungsurteil, Writer/Review/Docs-Worktree erhalten |
| User-Steering-Recovery | 🟦 L4 unter gleicher Run-ID ab frischem Docs-Gate resumed; alle neuen Calls explizit `homelab-llm/deepseek-v4-flash:high`, kein `tier`/Default |
| User-Steering-Recovery | ❌ L0: in-flight Review durch Pause abgebrochen (`Subagent was aborted`); kein Implementierungsurteil, kritischer Writer-/Anchor-Worktree-Stand erhalten |
| User-Steering-Recovery | 🟦 L0 unter gleicher Run-ID ab frischer unabhängiger Review resumed; alle neuen Calls explizit `homelab-llm/deepseek-v4-flash:high`, kein `tier`/Default |
| Lane-Delivery | 🟨 L2 geliefert (`READY_FOR_ORCHESTRATOR`): portable + macOS new-window-Contracts, 19 neue Tests; fmt, serde, beide Crate-Suiten und Clippy `-D warnings` grün; Allowlist/Diff geprüft, keine neuen Dependencies; wartet gemäß Reihenfolge bis nach U2 |
| Lane-Delivery | 🟨 L7 geliefert (`READY_FOR_ORCHESTRATOR`): portabler Find-State-Contract mit 16 deterministischen Tests; Default+Serde Check/Test/Clippy `-D warnings` und Fmt grün; 3 Allowlist-Pfade, keine Dependencies; wartet gemäß Reihenfolge hinter L6 |
| Lane-Delivery | 🟨 L1 geliefert (`READY_FOR_ORCHESTRATOR`): `just package-verify` grün, 12 Packaging-Tests, byte-identische Wiederholungs-Bundles, gültiges ad-hoc Codesign und echter 5.1-s Launch-Smoke ohne Restprozess; Low-Risk Fixture-Abhängigkeit von PATH/Nix dokumentiert; Integration planmäßig zuletzt |
| Lane-Delivery | 🟨 L6 geliefert (`READY_FOR_ORCHESTRATOR`) nach konkreter Remediation: 6 veraltete `apply_event`-Aufrufe, Cancel-Erwartung, Retry-ID-Pfade, Doctest und fehlende MismatchedId/EmptyDestination-Coverage korrigiert; 37 lib + 1 doc / 38 serde Tests, Clippy `-D warnings`, Fmt und vollständiges `just verify` grün; keine Dependencies; Integration vor L7 |
| Lane-Delivery | 🟨 L5 geliefert (`READY_FOR_ORCHESTRATOR`) nach Recovery-Remediation: session-lokale Folder/Favorites-Invarianten und U2-fähige NewFolder/NewSpace/FavoriteToggle/MoveTabIntoFolder-API vollständig; 21 Domain + 20 Controller-Tests, 67/95 Crate-Gesamttests, Clippy `-D warnings`, Fmt und Workspace-Checks grün; 6 Allowlist-Pfade, keine Dependencies; wartet direkt hinter L0 |
| Lane-Delivery | 🟨 L4 geliefert (`READY_FOR_ORCHESTRATOR`) nach fokussierter Docs-Remediation: DESIGN §11.4 und vollständige U1/U2-Pakete mit disjunkten Pfaden, deterministischen Assertion-IDs, E2E/Perf-Gates und `<post-anchor integration SHA>`-Arg; exakt 3 Docs-Pfade, `check_roadmap.py` für 43 Dateien grün; bei L0-Landung in Roadmap/Phase/Track promoten und SHA einsetzen |
| Orchestrator-Gate | L3 funktional grün, aber noch nicht geliefert: `hydration_tests.rs` 1117 L und `hydration.rs` 589 L verletzen Repo-Regel „Dateien unter ~500 Zeilen“; enger behavior-preserving Split + unabhängiger Recheck gestartet (`bbr-mvp-l3-structure-remediation-msj0gjba-0m06xf`, DeepSeek-only) |
| Lane-Delivery | 🟨 L3 nach Orchestrator-Strukturgate geliefert (`READY_FOR_ORCHESTRATOR`): Implementation in 264/136/236-L-Module, Tests in 23/290/372/185/293-L-Module gesplittet; öffentliche API und exakt 22 Tests erhalten; 97 `bbr-browser`-Tests, Workspace-Test/Check, Clippy `-D warnings`, Fmt und Roadmap-Validator grün; keine Dependencies; Integration hinter L7 |
| Integration | L0 in vier logischen Gruppen committed und seriell nach `main` gemergt: `45107a3497bc12603f497ed964d39c37c083a940`; erwartete H0-Konflikte in `Cargo.lock`/`flake.nix` aus gemergtem Manifest bzw. xcrun-Invariante gelöst |
| Integration-Gate | L0 post-merge: `just verify` ✅, testkit-Clippy ✅, vendored gpui-component 110/110 ✅; kanonisches `just e2e` bis Dense grün, dann Dense-Keyboard `Quit` mit `failed to fill whole buffer`/EOF ❌; kein Blind-Retry, Artefakt-/Prozessdiagnose gestartet |
| Integration-Remediation | Root Cause: `Request::Quit` rief `cx.quit()` vor dem Gateway-Writer-Flush von `Response::Ok` auf; unter Load ~52 schloss der Prozess den Socket zuerst. Zweiphasen-Shutdown implementiert: Ok-Frame flush+Ack → `Quitting` → `cx.quit()`; Regression beweist Client-Antwort während blockiertem Shutdown-Handler. 19 Testkit-Tests, Clippy, `just verify`, testkit-Clippy und beide Dense-Suiten grün |
| Integration-Gate | ✅ L0 final: kanonisches `just e2e` auf sauberem SHA `f89a4799999d1d43f785da7c767a1df68759ee0b` vollständig grün; Post-Anchor-SHA für U1/U2 festgeschrieben |
| Housekeeping | ✅ Alter `pi-tmux-resume`-Wächter für Writer `019fdb6b` nicht mehr vorhanden; Prozess- und BBR-tmux-Pane-Prüfung ohne Treffer |
| Integration | ✅ L4-Dokumente integriert (`08004dc`); U1/U2-Pakete tragen Post-Anchor-Base `f89a479` und sind in ROADMAP, Phase 4/5 sowie UI-Track promotet |
| Integration | ✅ L5 seriell integriert (`e923716042f207ef94c4aa6a871e0c797a777e6e`); 67 Domain + 95 Browser-Tests, Clippy `-D warnings` und Workspace-Check post-merge grün; U2-Fundament bereit |
| UI-Kickoff | Impeccable-Kontextskript blockiert (`node` im BBR-Shell nicht verfügbar); kein Blind-Retry/imperative Installation. PRODUCT/DESIGN/U-Pakete sowie Operate/New-Work/Craft-Floor direkt geladen; etablierte BBR-Designwelt bleibt autoritativ |
| UI-Kickoff | 🟦 U1/U2 exakt von Post-Anchor-SHA `f89a4799999d1d43f785da7c767a1df68759ee0b` gebrancht, danach je orchestrator-eigener Foundation-Merge mit L4/L5; Workflows `bbr-mvp-u1-palette-ui-msj221m6-mo85dt` und `bbr-mvp-u2-sidebar-ui-msj221m7-fd4vuf` gestartet; alle Calls explizit DeepSeek-only |
| UI-Recovery | 🟦 U2-Writer lieferte beim ersten Transportversuch `AGENT_EMPTY_OUTPUT`; Workflow-Runtime startete den erlaubten zweiten/letzten Versuch automatisch. Statusprüfung: Writer aktiv, laufende Thinking-Aktivität, `error=0`; noch kein Lane-Blocker |
| UI-Recovery | 🟦 U1-Writer lieferte beim ersten Transportversuch ebenfalls `AGENT_EMPTY_OUTPUT`; erlaubter zweiter/letzter Versuch läuft automatisch. Statusprüfung: Writer aktiv, laufende Thinking-Aktivität, `error=0`; noch kein Lane-Blocker |
| UI-Recovery | U2 zweiter Writer ebenfalls `AGENT_EMPTY_OUTPUT` nach einem am Output-Limit abgeschnittenen Tool-Call; keine Änderungen, Stashes oder fremden Commits. Read-only Review und Gate: `BLOCKED — no implementation delivered`. Workflow ist vertragsgemäß in Remediation fortgeschritten; diese läuft aktiv als letzter lokaler Implementierungsweg vor Recheck |
| UI-Progress | U1 zweiter Writer aktiv; Worktree HEAD unverändert auf Foundation `7a98762`, aktuell vier erlaubte uncommitted Protokolländerungen (`benchmark.rs`, `mod.rs`, `request.rs`, `snapshot.rs`, +38/−1). Noch keine Produktions-/E2E-Implementierung; deshalb keine Lieferprognose als erreicht behandelt |
| UI-Recovery | U1 Idle-Warnung nach 360 s untersucht: kein bounded Gate, sondern ungebundene Quellensuche `find / -name styled.rs -path '*gpui*'`, >6 min aktiv. Ausschließlich verifizierten Such-Prozessbaum beendet, damit der Writer einen Fehler erhält und fortsetzen kann; Workflow nicht gestoppt, U1-Diff unverändert. Gleichartige bereits verschwundene U2-Suche nicht angefasst; fremder Xcode-Prozess ausdrücklich unberührt |
| UI-Steering | User-approved Analyse aus Session `019fdcb7`: DeepSeek-Monolith-Writer verlieren nach langen Toolketten häufig den Abschlussreport (`AGENT_EMPTY_OUTPUT`). Bedingter Recovery-Vertrag festgelegt: bei U1-Abbruch bzw. leerer/roter U2-Remediation neuer sequenzieller 2–4-Step-Workflow statt Monolith-Retry; jeder Step DeepSeek-only, eng begrenzt, mit explizitem Abschlussbericht und Handoff; Suchen ausschließlich repo-lokal per `rg` bzw. GPUI unter `~/.cargo/git/checkouts`, niemals ungebunden |
| UI-Recovery | U2-Trigger eingetreten: erster Remediation-Versuch `AGENT_EMPTY_OUTPUT`, Runtime begann Versuch 2/2. Orchestrator stoppte Run `bbr-mvp-u2-sidebar-ui-msj221m7-fd4vuf` gemäß Steering vor weiterer Monolith-Arbeit; Worktree nachweislich sauber auf `05f06cd`. Neuer Run `bbr-mvp-u2-bounded-recovery-msj364tl-1956zr`: drei kurze Writer-Steps mit explizitem Bericht/Handoff, danach unabhängiges Review/Gate, höchstens eine fokussierte Remediation und Recheck; `agentRetries: 0`, alle Calls DeepSeek-only, Suchgrenzen in jedem Writer-Prompt |
| UI-Recovery | U1-Trigger ebenfalls eingetreten: Writer und erster Remediation-Versuch endeten `AGENT_EMPTY_OUTPUT`, Runtime begann Remediation 2/2. Orchestrator stoppte Run `bbr-mvp-u1-palette-ui-msj221m6-mo85dt`; vier erlaubte, `git diff --check`-saubere Testkit-Seed-Dateien (+40/−1) auf Foundation `7a98762` erhalten. Neuer Run `bbr-mvp-u1-bounded-recovery-msj3u8wd-qbue8s` nutzt dieselbe bounded Step-/Handoff-/Such-Policy mit `agentRetries: 0` |
| Produktentscheidung | User bestätigt Grundmodell: Profil/Container = isolierte Browser-Identität und Owner von Cookies, Sessions, Credentials, Cache/DataStore; mehrere Spaces dürfen dasselbe Profil teilen und sind organisatorische Bereiche; ein Space gehört genau einem Profil, Tabs genau einem Space und leiten ihr Profil daraus ab. Folgerung für U2: Favorites gehören zum Profil und sind über alle dessen Spaces sichtbar, aber niemals profilübergreifend; Folders/Pinned/Regular Tabs bleiben spacegebunden. Noch explizit zu entscheiden: profilweiter Favorite als Ziel-Link mit space-lokaler Tab-Instanz (aktuelles L5-Modell) oder eine einzige profilweite laufende Tab/WebView-Instanz (würde die aktuelle `Tab belongs to one Space`-Architektur erweitern) |
| UI-Pause | Runtime meldete den beim Pause-Befehl abgebrochenen aktiven U2-Behavior-Agent als `Subagent was aborted`; dies ist kein neuer Lane-Fehler. Workflow bleibt `paused`, Structure `done`, `error=0`; Worktree auf `05f06cd` enthält den erhaltenen U2-Struktur-Diff (12 tracked + 8 untracked Pfade, +896/−50 tracked), `git diff --check` grün. Fortsetzung wartet ausschließlich auf Favorites-Scope-Bestätigung |
| Produktentscheidung | User korrigiert und bestätigt final: mehrere Profile/Container koexistieren in einem Browserfenster; Profile isolieren Website-Daten, sind aber keine Fenstergrenze. Favorites sind fensterweit über alle Spaces/Profile identisch sichtbar und zusätzliche Präsentationen konkreter Tabs. L5 remediated zu `TabId`-Identität; Snapshot exponiert aktuellen `TabId + SpaceId + ProfileId`; Aktivierung wechselt im selben Fenster zum owning Space/Container und selektiert exakt den Tab, ohne Kopie/Move; Close entfernt Tile, Move lässt Favorite der stabilen TabId folgen. 67 Domain + 97 Browser-Tests, Crate-Clippy und vollständiges `just verify` grün |
| Tooling | Kalter Workspace-Gate fand `-liconv`-Linkfehler: realer Apple-`cc` interpretiert Nix `NIX_LDFLAGS` nicht. `flake.nix` setzt auf Darwin deklarativ `LIBRARY_PATH=${pkgs.libiconv}/lib`; normaler `nix develop` + `just verify` danach grün |
| Integration | Favorites-Vertrag auf `main` korrigiert: Tooling `b604835`, Product/L5 `f2b246bade4c486e19010d00bf0a271137078654`; 67 Domain + 97 Browser-Tests, Crate-Clippy und vollständiges `just verify` grün. Pausierten U2-Run endgültig gestoppt; neuer U2-Foundation-Merge `7ce24492ef32b501c9f14c056d84f10f9d000951`, Run `bbr-mvp-u2-concrete-favorites-msj5aqjv-oqp6m8` gestartet |
| Pause | U1-Run `bbr-mvp-u1-bounded-recovery-msj3u8wd-qbue8s` beendet: 108 bbr + 20 testkit Tests und 16/16 Command-E2E grün; Ankerfehler und `palette.rs`-Split behoben. Review/Gate trotzdem `BLOCKED`: native WKWebView liegt als AppKit-Subview über dem GPUI-Metal-Layer; semantische Testkit-Seams erkennen die reale Unsichtbarkeit/Hit-Test-Grenze nicht. Benötigt Root-Anchor-/Child-Window-Composition oberhalb der WKWebView oder eine explizite DESIGN-Änderung. U1 nicht integriert, Worktree erhalten |
| Architecture investigation | `gpui-component` ContextMenu/Popover geprüft: `deferred(anchored(...)).with_priority(1)` ist bereits ein React-Portal-Äquivalent für Layout-/Paint-Escape und Focus/Dismiss innerhalb **desselben GPUI-Metal-Layers**; es überholt keine native WKWebView. Zielpfad: allgemeines BBR-`Portal` mit Host-Policy — `GpuiDeferred` für reine Chrome-Flächen, `NativeAboveWeb` für Palette/Overlay und andere Glas-übergreifende Flächen. GPUI-CE unterstützt transparente `WindowBackgroundAppearance`, `WindowKind::PopUp/Floating` und raw `NSView`-Zugriff; AppKit bietet `addChildWindow:ordered:NSWindowAbove`. U1 bleibt blockiert, bis der Portal-Host als eigenes geprüftes Architekturpaket existiert |
| Launch | User bestätigt den allgemeinen Portal-Pfad. Planungscommit `292a0b8`; isolierter Worktree `/Users/patwoz/dev/Personal/macos/bbr-native-portal-host`, Branch `parallel/native-portal-host`, Basis `292a0b802fb87e88c2e58b1b8fc1e6a386d7b7fa`. Background-Run `bbr-native-portal-host-msj9f75q-1k0tf0`: sequenziell Contract → Native Host → App Probe/Testkit → E2E/Evidence → unabhängiges Native-Review → ggf. fokussierte Remediation → Recheck; DeepSeek-only, `agentRetries: 0`, Agents committen nicht. U1 eingefroren, U2 isoliert; keine U1-Migration vor grünem Portal-Gate |
| Recovery | U2-Endrecheck fand macOS `sockaddr_un.sun_path` bei 104 B. Run `bbr-mvp-u2-socket-remediation-msj9gt2t-kft5le` verschob Shell-/Perf-Harnesses auf deterministische kurze `/tmp`-Wurzeln, ergänzte Worst-Case-Regression (65 B bei `u32::MAX`) und lieferte Sidebar/Organize/Perf/Kanonisch grün. Orchestrator akzeptiert funktional, aber nicht strukturell: u. a. `e2e_shell_sidebar.rs` 511→1190, `shell/snapshot.rs` 579→845, `app/benchmarks.rs` 404→562. Struktur-Run `bbr-mvp-u2-structure-closeout-msj9xwql-nlzjnm` parallel zum Portal-Run gestartet; keine Lane gestoppt |
| User-Steering | Integration vorgezogen: L6 → L7 sofort seriell, danach L3 sofern kein konkreter Gate-/GUI-Lock-Konflikt; L2 bleibt nach U2. Portal/U2 laufen isoliert weiter. L6-Pre-Merge-Prüfung fand zunächst `downloads.rs` 752 L und State-Tests 678 L; behavior-preserving in `downloads/{mod,machine,tests_*}` gesplittet, öffentliche API unverändert, größte Datei 459 L. Frische Evidenz: 37 lib, 1 doctest, 38 serde, Crate-Clippy `-D warnings`, vollständiges `just verify`, Diff/Allowlist grün |
| Integration | ✅ L6 Lane `e9d2f50a781a4c71ce235b8815fc8f7dab42b1fb` seriell als Merge `6912cab98b2ec5e170dabf3b806d6f2ea598452d` integriert; post-merge 38 serde Tests + Doctest und bbr-web-Clippy grün. L7-Pre-Merge-Gate danach frisch grün: Default+Serde je 26 Tests, beide Crate-Clippies, Fmt, `just verify`; nur additive `bbr-web`-Moduloberfläche neben L6 |
| Integration | ✅ L7 Lane `2cfba1e429964f4984f0969549c608bddbdca42b` seriell als Merge `7ed1bed4e5a0010821d47550c5f93828904c5bc2` integriert; Git kombinierte den additiven `lib.rs`-Hunk konfliktfrei. Post-merge L6+L7: 53 Default / 54 Serde Tests, Downloads-Doctest und bbr-web-Clippy grün. L3 benötigt keinen GUI-Lock; frisches Pre-Merge-Gate 97 Browser-Tests inkl. 22 Hydration, Crate-Clippy, `just verify`, max. 372 L |
| Integration | ✅ L3 Lane `506f349c15e1ef99fa7ed7c35b2a4898a7153dd1` als Merge `d36d44cbcf8dbeceadbc8c60083932d539b79936` integriert. Erwarteter `bbr-browser/src/lib.rs`-Konflikt war additiv, verlor bei erster mechanischer Auflösung jedoch `#[cfg(test)]` vor `hydration_tests`; exakt korrigiert in `0656836`. Post-merge: Workspace-Check, 119 Browser-Tests (Organization + Hydration), Browser-/Workspace-Clippy `-D warnings`, Roadmap und Diff grün |
| Roadmap | User-beauftragte Post-MVP-Entwürfe aus `/Users/patwoz/dev/Personal/macos/bbr-post-mvp-drafts.md` in fünf bounded Pakete überführt: profilgebundene Singleton-Favorite-Runtime (deferred), Find-Portal-Bar, BBR-Kontextmenüs über Web, Downloads-Sidebar-Panel, Portal-Härtung. Veraltete Draft-Aussage „MVP Option A = URL-Link“ nicht übernommen: kanonischer MVP-Vertrag bleibt window-globales konkretes `TabId` im owning Space; deferred Option ändert ausdrücklich die Laufzeit-/Space-Ownership |
| Gate | U2 final strukturell geschlossen: `dispatch_benchmark.rs` 454 L + `dispatch_benchmark_u2.rs` 82 L; Sidebar-Suite-Root 470 L + Support 53 L; Baseline-JSON-Newline restauriert. Frisch grün: `just verify`, Testkit-Clippy, `just e2e-shell-sidebar` 7/7. `just e2e-perf` lief bei Load 6,70/7,48/9,69 parallel zu Portal `e2e_shell_portal`; ausschließlich einmaliger Startup-Sample 574,6 ms > 500 ms, alle drei U2-Szenarien 30 Samples und p95 0,003–0,017 ms grün. Kein Blind-Retry/Commit; warmen Rerun erst ohne konkurrierende Portal-GUI-Evidenz |
| User-Steering | L2 und L1-Packaging-Infrastruktur während Portal-Remediation vorgezogen; finales `just package-verify` bleibt letzter Release-Gate. L2 frische 19 Contract-Tests/Clippy/Fmt grün; L1 `package-test` 12/12, Py-Compile/Fmt grün. U1-Prep abgelehnt: Portal-Review belegt deterministischen UAF bei Resize; aktive Remediation muss Lifetime, E2E und die übergroße Step-1-Datei `portal/state.rs` ändern. Vertrag daher noch nicht stabil genug für parallele U1-Adaption |
| Integration | ✅ L2 Lane `fb3ac58d653ecc45035cbece445fe53b4a2a5c36` seriell als Merge `6228efd0d24f1418477cf08f19477672a620a9bc` integriert; ausschließlich erwarteter additiver `bbr-web/src/lib.rs`-Konflikt. Post-merge: 54 Serde-Unit + 11 New-Window-Tests, Downloads-Doctest, macOS 1+8 Tests und beide Crate-Clippies `-D warnings` grün. L1-Infrastruktur ist nächster Kernel; Bundle-/Launch-Gesamtgate bleibt final |
| Integration | ✅ L1-Infrastruktur Lane `5f36dc7e735c503933b248a80465d7c3b77e23ac` als Merge `9b0afb7` integriert. Post-merge: `package-test` 12/12, Python-Compile, vollständiges `just verify`, Roadmap 51 Dateien grün. Kein finaler Bundle-Build/Launch-Smoke vor fertigem Binary; `just package-verify` bleibt Schlussakt |
| User-Steering | Neue portable Nebenlanes P8a/P8b genehmigt. Serieller Anchor `43ab542afd6a630608e801aa8e1c1ae32b8cbc3d` fügt nur `pub mod permissions; pub mod web_errors;`, leere kompilierbare Modulverzeichnisse und zwei bounded Task-Pakete hinzu; Default/Serde/Clippy/Roadmap grün. P8a Run `bbr-p8a-permissions-contract-msjegrwu-af42jd`, P8b Run `bbr-p8b-error-surface-contract-msjegrwv-u70nuq`, beide disjunkt vom selben SHA, DeepSeek explizit, bounded sequenziell, `agentRetries: 1`, Agents committen nicht. P7-History-Provider abgelehnt: Schema v1 hat keine History-Persistenz; direkte `bbr-command -> bbr-persistence`-Kopplung widerspricht Provider-Neutralität, ein echter Pfad braucht erst serielle Migration/API/Bridge-Entscheidung |
| Lane-Delivery | 🟨 P8a Permissions `READY_FOR_ORCHESTRATOR`: unabhängiger Review und finaler Read-only-Recheck ohne Findings. Allowlist exakt `crates/bbr-web/src/permissions/**` + Task-Paket; `lib.rs`, Manifeste, Lockfiles, native/App/Testkit/UI unverändert; kein unsafe/stub, max. 452 L. Frisch agent-verifiziert: Default 79 lib + 11 Integration + 1 Doctest, Serde 82 + 11 + 1, beide bbr-web-Clippies, Fmt, Roadmap und Diff grün. Advisory: terminale Request-IDs bleiben caller-prunable via `forget()`; späterer WebKit-Adapter muss Speicher begrenzen. Worktree uncommitted erhalten; keine Integration vor U1→U2→Packaging-Endsequenz |
| Lane-Delivery | 🟨 P8b Web Errors `READY_FOR_ORCHESTRATOR`: Review fand eine konkrete Privacy-Kante — authority-lose URLs wie `javascript:alert('secret')` oder `data:…` konnten ihren Payload als Display-Origin tragen. Remediation reduziert diese auf das kleingeschriebene Scheme (`javascript:`, `data:`, `mailto:`), whitelisted `about:blank` und ergänzt adversariale Regressionen; finaler Recheck grün. Allowlist exakt `crates/bbr-web/src/web_errors/**` + Task-Paket; `lib.rs`, Manifeste, Lockfiles und alle anderen Crates unverändert, max. 494 L. Frisch agent-verifiziert: Default 94 lib + 11 Integration + 1 Doctest, Serde 99 + 11 + 1, beide Clippies, Fmt, Roadmap und Diff grün. Native Mapping/UI/E2E bleiben später; Worktree uncommitted erhalten und keine Integration vor U1→U2→Packaging |
| Lane-Delivery | 🟨 P8e Site Settings/Zoom `READY_FOR_ORCHESTRATOR`: unabhängige Review/Recheck ohne Findings. Allowlist exakt `crates/bbr-web/src/site_settings/**` + Task-Paket; `lib.rs`, Manifeste, Lockfiles, Persistence/Native/App/Testkit unverändert; kein URL/PSL-Paket, kein raw-f32-State, alle Dateien ≤500 L (`wire.rs` exakt 500, test-only). Frisch agent-verifiziert: Default 120 und Serde 133 Tests, beide bbr-web-Clippies, Fmt, Roadmap und Diff grün; object-safe Store extern implementierbar, default 100% erzeugt keinen Record, Reset entfernt Override, stale/mismatched Acks mutieren nicht. Nicht-blockierende Adapter-Hinweise: eine gültige seltene IPv6-Kompressionsform wird konservativ abgelehnt; Ack-Version 0 wird als stale klassifiziert; fehlende Rollup-Rücklinks liegen außerhalb der Lane-Allowlist. Worktree uncommitted erhalten; keine Integration vor U1→U2→Packaging |
| Lane-Delivery | 🟨 P8d File Panels `READY_FOR_ORCHESTRATOR`: unabhängige Review/Recheck ohne Findings. Allowlist exakt `crates/bbr-web/src/file_panels/**` + Task-Paket; `lib.rs`, zentraler `WebEngine`, Manifeste, Lockfiles und alle anderen Crates unverändert, alle Dateien <500 L (max. 499). Opaque Upload-/Save-Tokens bleiben bare Integer auf dem Wire; strukturelle/Serde-Tests verhindern Pfadfelder und artfremde Tokenverwendung. Frisch agent-verifiziert: Default 109 lib + 11 Integration + 1 Doctest, Serde 115 + 11 + 1, beide Clippies, Fmt, Roadmap und Diff grün. Advisory: Coordinator-Tombstones wachsen bis eine spätere Adapter-Lane eine GC-/Eviction-Policy besitzt; Sandbox-/Bookmark-Lifetime bleibt ausdrücklich Adapter-owned. Worktree uncommitted erhalten; keine Integration vor U1→U2→Packaging |
| Integration | ✅ Portal-Lane `f915626fff584854723576309c7d4383fe1d13c0` seriell als Merge `9cc8ad0b36f98f773973248ce5afc175f97277ff` integriert. Einziger Konflikt in `bbr-web-macos/src/lib.rs` additiv mit L2-New-Window gelöst. Pre-merge: `just verify`, beide Clippies, Portal 3/3, Command 11/11, Sidebar 3/3; post-merge vollständiges `just e2e` grün, Startup 459,7 ms. UAF, Struktur-, Teardown- und AppKit-Evidence-Findings geschlossen; U1 entsperrt |
| User-Steering | Letzte neue Lane-Welle für heute: P8c Auth, P8d portable File Panels, P8e Site Settings/Zoom vom Portal-integrierten `main`; danach ausschließlich Portal→U1→U2→Package-Endsequenz. Serieller Anchor `92a0bfbe42364772b185247fc0f60ed92764a8b8` Default/Serde/Clippy/Roadmap grün. P8c Run `bbr-p8c-auth-challenges-contract-msjfa3hh-xqhshc`, P8d `bbr-p8d-file-panels-contract-msjfa3hi-gaon2u`, P8e `bbr-p8e-site-settings-zoom-contract-msjfa3hj-t2il4d`; drei saubere disjunkte Worktrees vom selben SHA. P8d-Schnitt akzeptiert, weil portable Requests/Completions und opaque Adapter-Tokens ohne Pfade/native Objekte formulierbar sind; zentraler `WebEngine`-Edit und native Panel-Orchestrierung bleiben später. DeepSeek explizit, bounded sequenziell, `agentRetries: 1`, keine Agent-Commits/Dependencies/GUI-Gates; Contract-Merges dürfen bis morgen warten |
| Gate | U2 post-Portal Warm-Perf einmal wie geplant ausgeführt, aber Host nicht ruhig: Load `6,90/7,21/7,84`, `corespotlightd` ~94% CPU, weitere headless Workflows aktiv. Ausschließlich Startup 512,3 ms > 500 ms; sämtliche bestehenden und U2-Interaktionen grün, darunter Favorites p95 0,018 ms, Folder 0,0045 ms, Plus-Menu 0,0115 ms. Kein zweiter Blind-Retry; U2 bleibt uncommitted/settled und erhält den nächsten Perf-Beweis erst nach U1 auf dem finalen Portal-Kernel |
| 15:09 | Alter Writer-Wächter (Session `019fdb6b`, Pane bbr:1.0) gestoppt — kein Doppel-Writer-Risiko |
| 15:09 | ✅ Handoff: sol-Orchestrator gestartet — tmux `tws_personal_macos_bbr-orchestrator`, Pi-Session `019fdc57-7afb-7f81`, gpt-5.6-sol:high. Ab jetzt einziger Orchestrator und Ledger-Owner; die fable-Session (`019fdba4`) zieht sich zurück |

## Kickoff-Anweisung für die sol-Orchestrator-Session

Du bist der einzige Orchestrator für diesen Plan. Arbeitsvertrag:

1. Lies diese Datei komplett; sie ist die einzige Wahrheit über den Stand.
2. Beweise zuerst die Delivery-Kette in DEINER Session: 1-Agent-Dummy-Run
   (Background, tier small, Antwort `DELIVERY_OK`). Kein Polling — Turn
   beenden, auf automatische Zustellung warten. Ergebnis hier eintragen.
3. Starte danach die Lanes L0–L7 als je EINEN Background-Workflow mit
   internen Phasen Writer→Review→gate→Remediation→Recheck
   (siehe workflow-authoring-Skill). Für **jede** Rolle und **jeden** neuen
   `agent()`-Call explizit
   `model: "homelab-llm/deepseek-v4-flash:high"` setzen; niemals `tier`
   oder den Session-Default erben. Das gilt auch für Validatoren/Judges,
   Remediation, Recheck, `resumeFromRunId` sowie U1/U2. `gpt-5.6-sol` ist
   ausschließlich der Orchestrator; Luna nicht verwenden. Bereits vor dem
   Steering in-flight gestartete Calls dürfen mit ihrer alten Route enden.
4. Jede Lane: eigener Worktree ab Base `574750b`, exklusive Pfad-Allowlist
   im Prompt, keine neuen Dependencies in portablen Lanes, keine Commits
   durch Subagents. L0 arbeitet im existierenden Worktree
   `/Users/patwoz/dev/Personal/macos/bbr-gpui-ce-components-anchor`
   (Uncommitted-Stand ist dokumentierter Ausgangszustand; die H0-Deltas
   liegen dort bereits identisch vor und dedupen beim Merge).
5. Bei jedem Statuswechsel: Lane-Tabelle + Status-Log hier aktualisieren
   (Run-ID beim Launch, 🟨 bei Delivery, ✅ mit SHA bei Integration).
6. Integration ausschließlich durch dich, seriell in der Merge-Reihenfolge
   oben; vor jedem Merge Evidenz prüfen (Gates der Lane), Commits in
   logischen Gruppen nach Repo-Commit-Disziplin (AGENTS.md).
7. Nach L0-Integration: U1+U2 launchen mit dem Integrations-SHA als arg.
8. E2E läuft nur über die `just`-Recipes (Host-GUI-Lock serialisiert).
9. Wenn ein Run hängt/failt: Ergebnis-Delivery abwarten bzw. Watchdog
   wirken lassen, dann `resumeFromRunId` mit korrigiertem Script; nie
   denselben Fehler blind wiederholen.
10. Blocker, die du nicht lösen kannst: in dieser Datei dokumentieren und
    den User fragen — nicht raten.

## Offene Punkte außerhalb BBR

- [ ] Infra-Repo `main` ahead 2 → Push (wartet auf User-Ok)
