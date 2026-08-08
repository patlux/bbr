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
| P0 | **Navigation ohne Schema zeigt eine leere Seite** | ✅ Merge `5009be5` | Von Patrick am gepackten `.app` reproduziert: `https://golem.de` lädt, `golem.de` bleibt weiß. WebKit ist nachweislich gesund (`DidFirstMeaningfulPaint`, nicht occluded), `normalize_url` und `looks_like_domain` sind beide korrekt — die Ursache liegt dazwischen und wird empirisch bestimmt |
| P0b | **Kein Test beweist je, dass Seiteninhalt ankommt** | ✅ Merge `5009be5` | Genau diese Lücke hat den Bug durchgelassen: alle Suiten prüfen Geometrie, Zustand und unsere eigene Sichtbarkeits-Buchführung — alles grün, während der Nutzer Weiß sieht |
| 1 | Manuelle Smoke-Prüfung des `.app` | ⬜ Patrick | `/Users/patwoz/dev/Personal/macos/bbr-mvp-smoke-checklist.md`. Sinnvoll erst nach P0 |
| 2 | **URL-Leiste wird Domain-Control, Palette legt sich darüber** | ⬜ **nächste Lane nach P0** | Patricks erste konkrete Stelle zu „die UI passt nicht". Die Sidebar-Adresse ist kein Eingabefeld mehr, sondern ein Button, der nur die Domain zeigt — randlose transluzente Pill, kein Border, **kein Fokusring**, Trailing-Slot für spätere Action-Icons. Texteingabe nur noch in der Palette, die sich **über** die Leiste legt statt darunter aufzuklappen, mit voller URL vorselektiert. Der alte Feld-Fokus mit Teal-Ring entfällt ersatzlos; betroffene `e2e-shell-command`-Fälle werden umgeschrieben. DESIGN §11.4/§11.5 revidiert, Paket [`url-address-control-palette.md`](../roadmap/tasks/url-address-control-palette.md), offene Details §17.13/§17.14 |
| 2b | Restliche UI-Anmutung nach S1 | ⬜ | Patrick-Befund „die UI passt nicht", noch ohne Detail. Braucht Screenshot oder konkrete Stelle, sonst raten wir |
| 3 | Downloads-Nachzug aus den Entscheidungen | ⬜ | History-Persistenz, Clear-History, Cancelled-Zeile mit Neustart und Löschen |
| 4 | Schema v2 ohne Migration | ✅ Merge `bb54841` (App-Verdrahtung in P7) | Versionskennung, bei Abweichung Store verwerfen. Trägt Space-Appearance, Settings, Downloads-History und P7 |
| 5 | Settings-/Search-Contract integrieren | ✅ Merge `ddd09cd` | Fertig im Worktree `bbr-settings-search-contract`, wartet nur auf die Host-Empfehlung oben |
| 6 | Space-Appearance-Contract integrieren | ✅ Merge `9f53f44` | Fertig im Worktree `bbr-space-appearance-contract`, wartet auf Palette/Emoji-Kalibrierung |
| 7 | R1 Footer-Space-Strip + Appearance-UI + Swipe | ⬜ | Der größte sichtbare Slice. Paket: `docs/roadmap/tasks/space-footer-strip-appearance-ui.md` |
| 8 | P7 Command-History-Provider | ⬜ **entblockt** | Das Veto beruhte allein auf dem fehlenden History-Schema. Mit „keine Migrationen" entfällt der Grund |
| 9 | P8b Error-Surfaces | ✅ Merge `a70e9a3` | TLS, Navigationsfehler, Offline, Renderer-Crash. Contract gemergt, Flächen fehlen |
| 10 | P8c Auth-Adapter | 🟦 F1e | Contract gemergt, natives Sheet fehlt |
| 11 | P8e Zoom-Wiring | 🟢 fertig im Worktree, wartet auf Menu-Hälfte | Cmd-Plus/Minus/0 auf die aktive WebView |
| 12 | Kontextmenüs über Web | ⚠️ Remediation läuft | Zweiter echter Nutzer des Portal-Hosts |
| 13 | External-URL nativ | ✅ Merge `e5dab25` | `Info.plist`-Registrierung, Delegate, Adapter. Portable Hälfte gemergt (`14f1093`) |
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
| 12:05 | ✅ **F1b nach Remediation gemergt (`a70e9a3`)**. Alle acht Befunde geschlossen — und Nr. 6 auf die richtige Art: `shell/snapshot/tests/u2.rs` ist **byte-identisch zur Basis**, die zwei neuen Flags wanderten mit Defaults in eine Eingabe-Struktur, statt jeden bestehenden Aufrufer umzuschreiben. Die stille Perf-Falle ist zu: `error_surface_present` ist geseedet (50/100 warm), gemessen p50 0,40 ms / p95 0,74 ms über 30 Samples, mit Enforcement-Assertion. `e2e-shell-errors` 10/10 über echte Fehlerpfade inkl. Redirect-Loop, abgeschnittener 500 und Renderer-Crash mit Reload.

**Integrationsarbeit des Orchestrators** (Lane-Base `a24321d`, `main` fünf Merges weiter): 12 Naht-Dateien, 23 Konfliktstellen. Alle waren Registry-Ergänzungen — beide Seiten fügten Kanal, Request, Snapshot-Feld oder Suite hinzu — also „beide behalten", **außer** der aggregierten `e2e`-Liste und einem Fehler-Match-Arm, die eine echte **Vereinigung** brauchten statt Nebeneinanderstellung. Mein pauschales Auto-Resolve zerlegte dabei drei Blöcke, deren Konfliktregion mitten im Rumpf endete (Handler-Blöcke in `testkit/mod.rs` und `window_seam.rs`, Struct-Ende in `protocol/snapshot.rs`) und verdoppelte eine `pub use`-Liste — von Hand repariert, danach kompilierte alles.

**Echtes Designsignal statt Formalismus**: Danach hatte `spawn_event_pump` **acht Argumente**, weil diese Lane und die External-URL-Lane je einen Receiver ergänzt hatten. Statt den Clippy-Lint stillzulegen, sind die Empfänger jetzt `EventPumpChannels` — jede native Fähigkeit, die die App erreicht, trägt hier eine Quelle nach; die Liste wäre sonst unbegrenzt gewachsen und jede Ergänzung hätte dieselbe Signatur angefasst. Gates nach Merge: `just verify` grün, `e2e-shell-errors` 10/10, `e2e-external-url` 5/5, `e2e-shell-command` 18/18 |
| 11:20 | ⚠️ **F1d sauber geteilt: Zoom fertig, Menüs nicht.** Reviewer-Verdikt REVIEW_FINDINGS, aber mit klarer Trennlinie. **Zoom ist belegt**: Contract beidseitig zitiert (MIN 25 / DEFAULT 100 / MAX 400 / STEP 25), keine erfundene Leiter, Defaults legen **nie** einen Record an (`PersistKind::Remove`), `e2e-shell-zoom` 4/4, Budgets auf der bestehenden Warm-Klasse **geseedet** samt Enforcement-Test. **Menüs brechen die Gates**: `fmt` 17+ Diffs, beide Clippy-Gates rot, und `context_menu/tests.rs:60` schattet den `items()`-Helper mit einer lokalen Bindung (E0618) — wodurch **alle** Bin-Unit-Tests nicht kompilieren, auch die 13/13 Zoom-Tests, die der Writer als Beleg angeführt hatte. Dazu fehlen die Pflicht-E2E-Suite samt Rezept und die Perf-Seed-Einträge.

**Der ernsteste Befund ist Nr. 5**, weil er nicht laut scheitert: Die Probe `context_menu_open` existiert, ist aber in `baseline.rs` und `performance-baseline.json` nicht geseedet — und `evaluate` **überspringt ungeseedete Kinds still**. Die Messung sähe vorhanden aus und bewiese nichts. Genau diese Falle hatte die Zoom-Hälfte für sich selbst geschlossen. Remediation `context-menus-remediation-msk3uuy9-wbwbiv` gestartet, mit der Auflage, den Seed **und** die Enforcement-Assertion nachzuziehen |
| 11:20 | ⚠️ **Startup-Budget beobachtet, nicht angefasst**: `just e2e-perf` riss zusätzlich am **bestehenden** Startup-Deckel — p95 506 ms, dann 544 ms gegen 500 ms, Einzel-Cold-Sample unter vier bauenden Lanes. Queue-Punkt 15 hatte zuletzt 389 ms gemessen. Nicht diesem Slice zurechenbar; **Budget bleibt unverändert**, braucht aber eine Messung auf ruhigem Host, bevor daraus eine Aussage wird |
| 11:05 | 📌 **Merge-Reihenfolge der vier laufenden Lanes festgelegt** (vorab, nicht unter Konfliktdruck). Gemessen: alle vier berühren dieselben **acht Naht-Dateien** — `app.rs`, `app/interaction.rs`, `app/testkit/snapshot.rs`, `shell/snapshot.rs`, `testkit/dispatch.rs`, `testkit/snapshot.rs`, `bbr-testkit/protocol/request.rs`, `protocol/snapshot.rs`. Das ist strukturell unvermeidlich, weil sich jede sichtbare Funktion dort registriert, und war heute zweimal rein additiv auflösbar. **Inhaltlich** überschneiden sich nur Address-Control und F1d in `app/palette.rs` — genau der Datei, deren Bedeutung Address-Control umbaut.

**Entscheidung: Address-Control wird zuerst gemergt, sobald sie abnahmefähig ist.** Begründung: Sie ist ein Architekturwechsel (eine Adresse, ein Editierzustand), kein Feature-Zuwachs. Wer zuletzt merged, trägt die Konfliktarbeit — und diese Arbeit darf nicht darin bestehen, den Architekturwechsel um bereits gemergte Annahmen herumzubiegen. F1d, F1b und F1e passen sich an die neue Basis an, nicht umgekehrt. Reihenfolge danach nach Konfliktkosten: F1b (eigene Fehlerflächen), F1e (Portal/Auth), F1d (`palette.rs`) zuletzt |
| 10:52 | ✅ **Beide wartenden Contracts integriert** — Space-Appearance `9f53f44`, Settings/Search `ddd09cd`.

**Patrick-Entscheidung 1 (Settings-Host) — und Korrektur meiner eigenen Empfehlung.** Ich hatte `bbr-web` empfohlen; das ist überholt. Schema v2 (`bb54841`, 20 Minuten vorher gemergt) bringt bereits eine `settings`-Tabelle mit `search_engine`/`custom_search_template` **und der Invariante genau eine Zeile**, gesetzt beim Anlegen des Stores. Damit implementiert **`bbr-persistence` den `SettingsStore`-Port**, die App verdrahtet ihn beim Start. Folgen: `bbr-command` bleibt provider-neutral und ohne Datenbank testbar; es gibt **einen** Ort für persistenten Zustand statt zweier widerspruchsfähiger; „Settings fehlen" wird durch die Einzelzeilen-Invariante ein unmöglicher Zustand statt eines Sonderfalls für jeden Aufrufer. Festgehalten: **Standard-Suchmaschine ist in v1 global, nicht pro Profil** — pro Profil bräuchte eine eigene Schema-Entscheidung.

**Patrick-Entscheidung 2 (Space-Appearance) — angenommen, aber der gelieferte Entwurf schlägt meinen Vorschlag.** Statt meiner acht frei erfundenen Töne liefert der Contract neun, deren erste sechs sich aus den **bestehenden Profil-Farbtönen** ableiten (`from_profile_hue`). Das vermeidet ein zweites, unverbundenes Farbvokabular neben dem, das BBR schon spricht — übernommen wie geliefert. Identität ist gültig by construction: Emoji nur aus der kuratierten Allowlist, Glyph nur aus bekannten Icons, ein nicht darstellbarer Space lässt sich damit gar nicht erst repräsentieren |
| 10:52 | ⚠️ **Perf-Flake unter Fremdlast, nicht aufgeweicht**: `bbr-command` `first_correct_result_below_50ms_for_500_tabs_and_10k_history` riss einmal mit `max=412 ms` bei `p50=4,3` / `p95=17,8`. Drei Wiederholungen sauber, `just verify` grün. Ursache ist Scheduling-Stall durch vier parallel bauende Lanes. **Die 50-ms-Decke bleibt unangetastet** — an einem Budget, das nur unter fremder Last reisst, wird nicht gedreht; die Konsequenz ist begrenzte Last während der Gates. Gehört zu Queue-Punkt 16 |
| 10:34 | 🟦 **F1b-Remediation gestartet** `error-surfaces-remediation-msk3dfn9-2zujhx`. Der Prompt trägt alle acht Befunde wörtlich samt Compiler-Ausgaben, mit klarer Reihenfolge: **Befund 6 zuerst** (die gebrochenen, fremden u2-Tests), erst danach die Substanz. Ausdrückliches Verbot, Modul-Deklarationen zu löschen oder Szenarien zu stubben, um `fmt` grün zu bekommen — die fehlenden Suiten sind der Inhalt des Slices, nicht sein Hindernis |
| 10:31 | ℹ️ Address-Control-Lane: `AGENT_EMPTY_OUTPUT`, Retry 2/2. Kein Datenverlust — der Worktree trägt bereits **17 Dateien, +1026/-119**, der Retry setzt darin fort. Muster bestätigt sich: leere Ausgaben treffen lange Agenten mit schweren Builds, nicht fehlerhafte Arbeit |
| 10:26 | ✅ **Schema v2 gemergt (`bb54841`)**. Der erhaltene Stand war laut Writer **substanziell fertig und tragfähig**; der Neustart hat ihn fortgesetzt, nicht ersetzt — genau das war der Zweck des Prompts. Kein Migrationspfad, kein Upgrade-Zweig, kein Alt-Format-Fallback überlebt (rg-geprüft). Die Sicherheitseigenschaft ist **beidseitig** getestet: `mismatched_stamp_discards_and_is_immediately_rewritable`, `unstamped_legacy_store_is_discarded` und `newer_stamp_is_refused_read_only_and_discarded_on_write` gegen `schema_v2_fixture_restores_without_writes` — eine passende Version verwirft nichts und schreibt nicht einmal. **Orchestrator-Nachprüfung der einen geänderten Erwartung**: `sql_statements` 517 / `inserted_rows` 508 sind im Diff **hinzugefügte** Assertions ohne entferntes Gegenstück, und die Arithmetik stimmt (9 Deletes + 508 Inserts; 508 = 1 Profil + 1 Space + 500 Tabs + 6 Einzelzeilen). Keine Abschwächung. Gates: `just verify` grün, `cargo test -p bbr-persistence` 13/13 |
| 10:26 | ℹ️ **Grenze offen und bewusst**: `bbr-persistence` ist noch von keinem Crate abhängig — das Schema existiert, die App benutzt es noch nicht. Verdrahtung in den App-Start ist Teil von **P7 History**, erst dort bekommt die `StoreLoad`-Disposition ihre Log-Fläche. Die `517/508`-Assertions laufen nur im release-only Benchmark (`--ignored`), also im Perf-Gate, nicht im Debug-Lauf |
| 10:12 | 🟦 **F1a Schema v2 neu gestartet** `schema-v2-resume-msk308yj-j08bnf`. Grund der Unterbrechung war **kein Fehler der Lane, sondern meiner**: `pause` beendet einen Subagenten, statt ihn einzufrieren. Der Worktree ist unberührt (20 Dateien, +1006/-573 gegen `a24321d`), der Prompt verlangt daher ausdrücklich, den Stand **zuerst zu lesen und fortzusetzen** — nicht neu zu schreiben und nichts zu verwerfen, das der Agent nicht versteht. Last bei 7, ein Slot war frei |
| 10:05 | 🟦 **Address-Control-Lane gestartet** `address-control-palette-msk2ivh9-ssykwg` (Worktree `bbr-address-control`, Base `1ca7470`) — Patricks erste UI-Priorität, direkt nach P0 wie vereinbart. Der Reviewer-Prompt verlangt ausdrücklich, **zuerst** Kompilierbarkeit und die Existenz jedes behaupteten Tests zu prüfen, weil heute bereits eine Lane Erfolg für einen Baum meldete, der nicht baute |
| 10:00 | ⚠️ **F1b nicht abnahmefähig**: Der Implement-Agent lieferte nach seinem Retry leere Ausgabe, der Reviewer prüfte einen halben Baum. Befunde: die E2E-Suite deklariert `mod recovery`/`mod surface`, beide Dateien fehlen; `error_surface_tests.rs` hat 11 öffnende gegen 12 schließende Klammern; `attach`/`ClickErrorAction`/`ClickErrorDismiss` existieren im Testkit nicht; kein `e2e-shell-errors`-Rezept; `visible_ids_u2` bekam zwei Parameter mehr und bricht damit **bestehende** u2-Snapshot-Tests; `fmt`, beide Clippy-Gates und `just verify` rot. Worktree erhalten, Remediation sobald ein GUI-Slot frei ist |
| 09:52 | ✅ **F1c gemergt (`e5dab25`)** — BBR nimmt jetzt Links entgegen, die andere Programme übergeben. Deklariert werden **nur** die Schemata, die der Contract akzeptiert; ein beanspruchtes und dann abgelehntes Schema hätte BBR zum schwarzen Loch für Links gemacht. Cold-Start-Links laufen über einen Kanal, der **vor** dem Callback existiert — der Fall, der am ehesten verloren geht und am wenigsten auffällt. Default-Browser-Status wird nie beim Start angefordert, nur auf ausdrückliche Nutzeraktion. Merge-Konflikt in `app.rs` war rein additiv (diese Lane und P0 ergänzten je ein Feld) — beide behalten. Gates nach Merge: `just verify`, `e2e-external-url` 5/5, `e2e-shell-command` 18/18 |
| 09:31 | ✅ **P0 gemergt (`5009be5`)**. Ursache: der **NewTab-Pfad reichte die Eingabe roh durch**, während Cmd-L normalisierte. `NSURL::URLWithString("golem.de")` ergibt eine relative URL ohne Schema; WebKit lehnte mit Fehler 101 „The URL can't be shown" ab, bevor eine Navigation begann — daher weiß, ohne sichtbaren Fehler. Fix an der **gemeinsamen Dispatch-Grenze**, nicht im NewTab-Zweig, damit der nächste Aufrufer nicht dieselbe Falle wiederentdeckt. Neu und dauerhaft: eine **Content-Probe**, die echten WebKit-DOM-Zustand liest — eine Seite, die nie lädt, kann nicht mehr als grün durchgehen. Gates: `just verify`, `e2e-shell-command` 18/18 inkl. Regressionstest, `e2e-tabs`, `e2e-smoke`, `package-verify` — alle grün. Neues Bundle-SHA256 `fe8b7c39c0ad99609447a166c78645fe635b06dbb4b01ed072b3138380eed94a` |
| 09:31 | ⚠️ Erster Gate-Lauf starb mit **SIGKILL** im Compiler — Speicherdruck durch drei parallele Lanes, kein Codefehler. Wiederholung mit `CARGO_BUILD_JOBS=4` grün. Merkposten: Integrations-Gates brauchen bei laufendem Fanout eine begrenzte Job-Zahl |
| 09:35 | ✅ **URL-Bar-Redesign in Contract überführt**: DESIGN §11.4 (Palette überdeckt die Adresse) und §11.5 (aus „Command bar" wird „Address control") revidiert, Paket angelegt, Phase 04 und UI-Track verlinkt, zwei offene Details als §17.13/§17.14 registriert. **Scheduling-Entscheidung**: P0 zuerst, weil beide denselben Eingabe-Commit-Pfad anfassen und ein Redesign auf kaputter Navigation nicht abnehmbar wäre; danach dieser Slice als eigene Lane mit Territorium Address-Control/Palette-Anchoring/Command-E2E — disjunkt zu F1b–F1e |
| 09:05 | 🟦 **Fanout-Welle F1 gestartet** — fünf Lanes parallel in isolierten Worktrees von `a24321d`, disjunkte Territorien, Gate-Phasen serialisieren über den hostweiten GUI-Lock des `justfile`: (P0) `schemeless-navigation-bug-msk0mwh9-7rnn94`; (F1a) `schema-v2-no-migration-msk0xahq-nw0hzi` — Persistenz/Domain; (F1b) `p8b-error-surfaces-ui-msk0xwvl-getw0m` — Fehlerflächen; (F1c) `external-url-native-msk0yi9s-lex0ro` — native Registrierung + Info.plist; (F1d) `p8e-zoom-and-context-menus-msk0z7o3-m1hhs3` — Zoom-Wiring plus Kontextmenüs über den **bestehenden** Portal-Host; (F1e) `p8c-auth-and-portal-hardening-msk0zz5r-ysqu6s` — Auth-Sheet plus Portal-Härtung gegen Vollbild/Space/Display/Minimieren/Schließen |
| 08:55 | ✅ Neue Delivery-Datei eröffnet; offene Punkte aus dem MVP-Plan übernommen, Patricks Entscheidungen eingetragen |
