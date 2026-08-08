# 2026-08-08 — Post-MVP-Delivery: Live-Status und Arbeitsliste

Diese Datei löst `2026-08-07-browser-mvp-parallel-plan.md` als **einzige
Quelle der Arbeitsliste und der Wahrheit** ab. Der Vorgänger bleibt als
Nacht-Protokoll des MVP-Closeouts erhalten und wird nicht mehr fortgeschrieben.

Der Ausgangsstand: das MVP ist geschlossen, `main` steht auf `78f3782`, das
gepackte `.app` trägt Binary-SHA256
`b42891013e564f3227cf3425c3bed1d0310c0293e7a041902124ddf352e2e8fa` (S1-Skala,
Find-in-page, Downloads-Panel).

## Arbeitsregeln (übernommen und weiterhin gültig)

- Diese Datei ist der alleinige Fortsetzungszeiger. Nach jedem Workflow, Gate,
  Merge und jeder Kompaktierung vollständig lesen und den nächsten offenen
  Punkt ausschließlich daraus bestimmen.
- Aufgaben kommen ausschließlich aus dieser Queue plus dokumentierten
  User-Entscheidungen. Neue Ideen werden erst als Queue-Eintrag dokumentiert,
  dann geplant.
- Nur der Orchestrator committet, merged und integriert; Merges strikt seriell
  und evidenzgeprüft. Agenten committen nie.
- Implementierung läuft in Background-Workflows in isolierten Worktrees vom
  letzten verifizierten `main`-SHA, mit disjunkten Pfad-Territorien.
  Subagenten immer explizit mit `homelab-llm/deepseek-v4-flash:high`.
- GUI-/E2E-/Perf-Gates sind hostexklusiv: immer nur eine Lane in ihrer
  Gate-Phase.
- Jede Gate-Liste enthält `cargo fmt --all -- --check`.
- **Testfehler werden durch schärfere Messung gelöst, nie durch Lockern.**
  Budgets, Sample Counts, Deadlines und Validatoren bleiben unangetastet.
- Blocker parken statt festhängen: Entscheidungsfragen mit Optionen,
  Empfehlung und Auswirkung hierher; technische Blocker mit Evidenz, Versuchen
  und kleinstem Entblocker. Danach sofort unabhängige Arbeit.

## Entscheidungen vom 2026-08-08 (Patrick, verbindlich)

| Thema | Entscheidung | Folge |
|---|---|---|
| Persistenz-Migrationen | **Keine Migrationen, keine Abwärtskompatibilität.** Wir sind in der Entwicklung; bei Schemaänderung werden die Daten vorher gelöscht | Entblockt Schema v2 **und** P7 History sofort. Statt Migrationspfaden genügt eine Versionskennung, die bei Abweichung den Store verwirft |
| Downloads §17.9 Zielordner | Still in den Standardordner (`Suggested`) | Kein Save-Panel im Weg; `Explicit` bleibt später bruchfrei nachrüstbar |
| Downloads §17.11 Badge | Zählt nur laufende Attempts | Zahl geht auf Null, wenn nichts mehr läuft |
| Downloads §17.10 Persistenz | **History persistiert** über Neustarts | Braucht Schema-Feld; wegen „keine Migrationen" trivial |
| Downloads §17.12 Öffnen | **Nur Reveal-in-Finder.** Kein Auto-Open, kein Doppelklick-Open | Bleibt bei der freigegebenen v1-Form |
| Downloads §17.8 Cancelled-Zeile | Bietet **Neustart** und **Löschen** | Abgebrochenes ist kein Sackgassen-Eintrag mehr |
| Downloads §17.7 Clear-History | Folgt aus der Persistenz-Entscheidung: Clear leert die **Controller-History**, nicht nur die sichtbare Liste | Sonst wäre gelöschter Verlauf nach Neustart zurück |

## Offene Entscheidungen mit Empfehlung

| Punkt | Frage | Empfehlung |
|---|---|---|
| Settings-Contract-Host | Wohnt der Settings-/Suchmaschinen-Contract in `bbr-web` oder `bbr-command`? | **`bbr-web`.** Settings sind persistenznah und werden vom Store geladen und geschrieben; `bbr-command` ist bewusst provider-neutral und darf keine Produktkonfiguration besitzen, sonst wird die Palette zum zweiten Zustandshalter |
| Space-Appearance-Palette | Welche Farben und welches Emoji-Set? | Siehe Vorschlag unten |

### Vorschlag Space-Appearance (zur Kalibrierung)

- **Palette:** acht kuratierte Töne statt freier Farbwahl, weil freie Wahl in
  einer getönten Chrome zuverlässig unlesbar wird. Vorschlag entlang der
  bestehenden BBR-Tokens: Graphit (neutral, Default), Indigo, Türkis, Moos,
  Bernstein, Terrakotta, Magenta, Schiefer. Jeder Ton existiert als
  Light-/Dark-Paar mit geprüftem Kontrast gegen `type:title` und `type:meta`.
- **Wirkung:** Tönung ausschließlich auf Chrome-Flächen (Sidebar-Hintergrund,
  aktive Zeile, Footer-Chip), nie auf Text, nie auf der Web-Fläche. Die
  Sättigung bleibt niedrig genug, dass zwei benachbarte Spaces unterscheidbar
  sind, ohne dass die Oberfläche bunt wirkt.
- **Emoji:** kuratiertes Set statt freiem Picker, damit Chips optisch
  konsistent bleiben. Vorschlag: rund 40 Glyphen in fünf Gruppen — Arbeit,
  Lernen, Privat, Medien, Werkzeuge. Freitext-Emoji explizit nicht in v1.
- **Default:** neuer Space bekommt Graphit und den nächsten unbenutzten Glyph
  seiner Gruppe, nie eine Zufallsfarbe.

## Queue

Legende: ⬜ offen · 🟦 läuft · 🟨 wartet auf Integration · ✅ erledigt · ❌ blockiert

| # | Punkt | Status | Notiz |
|---|---|---|---|
| P0 | **Navigation ohne Schema zeigt eine leere Seite** | 🟦 `schemeless-navigation-bug-msk0mwh9-7rnn94` | Von Patrick am gepackten `.app` reproduziert: `https://golem.de` lädt, `golem.de` bleibt weiß. WebKit ist nachweislich gesund (`DidFirstMeaningfulPaint`, nicht occluded), `normalize_url` und `looks_like_domain` sind beide korrekt — die Ursache liegt dazwischen und wird empirisch bestimmt |
| P0b | **Kein Test beweist je, dass Seiteninhalt ankommt** | 🟦 Teil derselben Lane | Genau diese Lücke hat den Bug durchgelassen: alle Suiten prüfen Geometrie, Zustand und unsere eigene Sichtbarkeits-Buchführung — alles grün, während der Nutzer Weiß sieht |
| 1 | Manuelle Smoke-Prüfung des `.app` | ⬜ Patrick | `/Users/patwoz/dev/Personal/macos/bbr-mvp-smoke-checklist.md`. Sinnvoll erst nach P0 |
| 2 | UI-Anmutung nach S1 stimmt nicht | ⬜ | Patrick-Befund „die UI passt nicht", noch ohne Detail. Braucht Screenshot oder konkrete Stelle, sonst raten wir |
| 3 | Downloads-Nachzug aus den Entscheidungen | ⬜ | History-Persistenz, Clear-History, Cancelled-Zeile mit Neustart und Löschen |
| 4 | Schema v2 ohne Migration | 🟦 F1a | Versionskennung, bei Abweichung Store verwerfen. Trägt Space-Appearance, Settings, Downloads-History und P7 |
| 5 | Settings-/Search-Contract integrieren | ⬜ | Fertig im Worktree `bbr-settings-search-contract`, wartet nur auf die Host-Empfehlung oben |
| 6 | Space-Appearance-Contract integrieren | ⬜ | Fertig im Worktree `bbr-space-appearance-contract`, wartet auf Palette/Emoji-Kalibrierung |
| 7 | R1 Footer-Space-Strip + Appearance-UI + Swipe | ⬜ | Der größte sichtbare Slice. Paket: `docs/roadmap/tasks/space-footer-strip-appearance-ui.md` |
| 8 | P7 Command-History-Provider | ⬜ **entblockt** | Das Veto beruhte allein auf dem fehlenden History-Schema. Mit „keine Migrationen" entfällt der Grund |
| 9 | P8b Error-Surfaces | 🟦 F1b | TLS, Navigationsfehler, Offline, Renderer-Crash. Contract gemergt, Flächen fehlen |
| 10 | P8c Auth-Adapter | 🟦 F1e | Contract gemergt, natives Sheet fehlt |
| 11 | P8e Zoom-Wiring | 🟦 F1d | Cmd-Plus/Minus/0 auf die aktive WebView |
| 12 | Kontextmenüs über Web | 🟦 F1d | Zweiter echter Nutzer des Portal-Hosts |
| 13 | External-URL nativ | 🟦 F1c | `Info.plist`-Registrierung, Delegate, Adapter. Portable Hälfte gemergt (`14f1093`) |
| 14 | Portal-Hardening | 🟦 F1e | Vollbild, Space-Wechsel, Monitorwechsel, Minimierung |
| 15 | U3 Startup-Evidenz | ⬜ | Persistierte Warm-Startup-Reihe; aktuell 389 ms bei 500 ms Decke |
| 16 | Perf-Lücken | ⬜ | Ohne Canonical-Runner-Evidenz, namentlich `command_bar_first_result` |
| 17 | Icon-System-Reste | ⬜ | Nach U2 offen geblieben |
| 18 | L0-Reste | ⬜ | Split-Geometry-Flake, Xcode-/MSRV-Checkboxen |
| 19 | Persistence-App-Composition / Warm-Restore | ⬜ | Fenster- und Tab-Wiederherstellung, ohne Split-View-Port |

## Status-Log

| Zeit | Eintrag |
|---|---|
| 08:03 | ✅ Downloads-Panel integriert (Merge `7cfdbd2`), kanonisches `just e2e` über 20 Suiten grün, `package-verify` grün, neues Bundle `b428910…` |
| 08:50 | ⚠️ Patrick meldet am gepackten `.app`: Web-Fläche bleibt weiß. Orchestrator-Messung: WebKit lädt und malt (`DidFirstMeaningfulPaint`, `window occluded 0`), also kein WebKit-Problem. Patrick grenzt selbst präzise ein: mit Schema lädt es, ohne Schema nicht |
| 08:55 | 🟦 Lane `schemeless-navigation-bug-msk0mwh9-7rnn94` gestartet (Worktree `bbr-schemeless-nav`, Base `78f3782`): erst fehlschlagender Regressionstest entlang des Produktionspfads, dann Fix, plus dauerhafte Evidenz, dass Seiteninhalt wirklich ankommt |
| 09:05 | 🟦 **Fanout-Welle F1 gestartet** — fünf Lanes parallel in isolierten Worktrees von `a24321d`, disjunkte Territorien, Gate-Phasen serialisieren über den hostweiten GUI-Lock des `justfile`: (P0) `schemeless-navigation-bug-msk0mwh9-7rnn94`; (F1a) `schema-v2-no-migration-msk0xahq-nw0hzi` — Persistenz/Domain; (F1b) `p8b-error-surfaces-ui-msk0xwvl-getw0m` — Fehlerflächen; (F1c) `external-url-native-msk0yi9s-lex0ro` — native Registrierung + Info.plist; (F1d) `p8e-zoom-and-context-menus-msk0z7o3-m1hhs3` — Zoom-Wiring plus Kontextmenüs über den **bestehenden** Portal-Host; (F1e) `p8c-auth-and-portal-hardening-msk0zz5r-ysqu6s` — Auth-Sheet plus Portal-Härtung gegen Vollbild/Space/Display/Minimieren/Schließen |
| 08:55 | ✅ Neue Delivery-Datei eröffnet; offene Punkte aus dem MVP-Plan übernommen, Patricks Entscheidungen eingetragen |
