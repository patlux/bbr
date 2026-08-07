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
| L1 | Packaging `.app` (Bundle, Plist, ad-hoc-Sign, Launch-Smoke) | 🟨 | `bbr-mvp-l1-packaging-msiyvhmh-43x8cf` | `/Users/patwoz/dev/Personal/macos/bbr-mvp-l1-packaging` · `parallel/mvp-l1-packaging` | `READY_FOR_ORCHESTRATOR`; `just package-verify` grün, 12 Tests, deterministisches ad-hoc-signiertes Bundle + 5.1-s Launch-Smoke; Integration zuletzt |
| L2 | `bbr-web` new-window-Contract (`target=_blank`/`window.open`) | 🟨 | `bbr-mvp-l2-new-window-contract-msiyvhmj-uqwhk5` | `/Users/patwoz/dev/Personal/macos/bbr-mvp-l2-new-window` · `parallel/mvp-l2-new-window` | `READY_FOR_ORCHESTRATOR`; 19 neue Contract-Tests + serde/all-crate/clippy/fmt grün; 6 erlaubte Pfade, keine Dependency-Änderung; Integration nach U2 |
| L3 | Persistence-Hydration portable (chrome-first, sichtbare Sessions) | 🟨 | `bbr-mvp-l3-persistence-hydration-msiyvhmk-7oogla` + `bbr-mvp-l3-structure-remediation-msj0gjba-0m06xf` | `/Users/patwoz/dev/Personal/macos/bbr-mvp-l3-hydration` · `parallel/mvp-l3-hydration` | `READY_FOR_ORCHESTRATOR`; 22 Hydration-Tests, Workspace/Fmt/Clippy/Roadmap grün; Struktur-Split max. 372 L; Integration nach L7 |
| L4 | Prep: DESIGN §11.4-Revision + Task-Packets U1/U2 (Assertion-IDs, exklusive Pfade) | ✅ | `bbr-mvp-l4-ui-prep-docs-msiyvhml-qf9avh` | `main` · Merge `08004dc11d86ebf0beb29efb17b2d03f256eb60a` | DESIGN + U1/U2-Pakete integriert; Base-Arg `f89a479` eingetragen und Roadmap/Phase/Track promotet |
| L5 | Favorites+Folders Domain/Controller (`bbr-domain`/`bbr-browser`, inkl. New-Folder/New-Space-Kommandos) | ✅ | `bbr-mvp-l5-favorites-folders-msiyvhmm-7j1qvw` | `main` · Merge `e923716042f207ef94c4aa6a871e0c797a777e6e` | 67 Domain + 95 Browser-Tests, Crate-Clippy und Workspace-Check post-merge grün; U2-API integriert |
| L6 | Downloads-Capability-Contract (`bbr-web` portabel) | 🟨 | `bbr-mvp-l6-downloads-contract-msiyvhmn-4s2ypx` | `/Users/patwoz/dev/Personal/macos/bbr-mvp-l6-downloads` · `parallel/mvp-l6-downloads` | `READY_FOR_ORCHESTRATOR` nach Remediation; 37 lib + 1 doc / 38 serde Tests, Clippy/Fmt/`just verify` grün; 4 erlaubte Pfade, keine Dependencies |
| L7 | Find-in-page-Contract (`bbr-web` portabel) | 🟨 | `bbr-mvp-l7-find-in-page-contract-msiyvhmo-wt6te6` | `/Users/patwoz/dev/Personal/macos/bbr-mvp-l7-find` · `parallel/mvp-l7-find` | `READY_FOR_ORCHESTRATOR`; 16 Find-Tests + Default/Serde Check/Test/Clippy/Fmt grün; 3 erlaubte Pfade, keine Dependency-Änderung; Integration nach L6 |
| U1 | Palette-UI: a) Cmd-T zentriert · b) Overlay am URL-Feld | 🟦 | `bbr-mvp-u1-bounded-recovery-msj3u8wd-qbue8s` | `/Users/patwoz/dev/Personal/macos/bbr-mvp-u1-palette` · `parallel/mvp-u1-palette` | alter Monolith-Run `bbr-mvp-u1-palette-ui-msj221m6-mo85dt` bei erneut leerer Remediation gestoppt; vier erlaubte Testkit-Seed-Dateien (+40/−1) erhalten; bounded Struktur→Verhalten→Evidenz→Review/Gate→ggf. fokussierte Remediation→Recheck läuft |
| U2 | Sidebar-UI: a) Struktur+Footer · d) [+]-Menü · b) Fav-Grid · c) Folders-UI · Icon-Glyphs | 🟦 pausiert | `bbr-mvp-u2-bounded-recovery-msj364tl-1956zr` | `/Users/patwoz/dev/Personal/macos/bbr-mvp-u2-sidebar` · `parallel/mvp-u2-sidebar` | Structure geliefert, Behavior-Step am sicheren Übergang pausiert: L5-Favorites sind derzeit browser-session-global statt explizit profilgebunden; User-Entscheidung zur Space-/Profil-Scope-Regel ausstehend, U2-Diff erhalten |
| U3 | Startup-First-Frame-Perf-Beweis | ⬜ | — | — | optional, nur bei Kapazität |

## Merge-Reihenfolge (seriell, Orchestrator)

```
H0 → (Lanes branchen) → L0-Anchor → L5 → U1 → U2 → L2 → L6 → L7 → L3 → L1
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
| Produktentscheidung | U2 am Behavior-Übergang pausiert: L5 erfüllt Space-Unabhängigkeit durch einen einzigen session-lokalen Favoriten-Store, ist damit aber zugleich profilübergreifend global; `Favorite`/`FavoriteSnapshot` tragen keinen `ProfileId`, URL-Eindeutigkeit gilt browserweit. Empfohlener Vertrag zur Bestätigung: Favorites profilgebunden und innerhalb dieses Profils über alle Spaces geteilt; Folders/Pinned/Regular Tabs spacegebunden; Aktivierung öffnet/fokussiert im aktiven Space mit dessen Profil-Datastore und verwendet niemals eine Session eines anderen Profils |
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
