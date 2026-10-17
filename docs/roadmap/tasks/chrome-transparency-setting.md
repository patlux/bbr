# Task packet — Sidebar transparency setting

Parents: [Phase 4](../phases/04-ui-shell.md) · [UI design system](../tracks/ui-design-system.md) · [Desktop glass](chrome-desktop-glass.md) · [DESIGN](../../../DESIGN.md) §2.

## Outcome

Settings › Appearance has a “Sidebar transparency” slider. Moving it changes,
live in every browser window, how much of the native desktop material shows
through the sidebar and the frame around the content card. The value persists
across relaunches. Websites and WKWebViews stay opaque.

Patrick requested it on 2026-10-17: „füg bitte in den einstellungen einen
slider hinzu bei appearance mit der der nutzer live die stärke der transparenz
einstellen können“.

## Scope

- A `chrome_transparency` preference (0–100 %, default 50 %) in the existing
  `preferences.txt`; unknown or out-of-range values keep the default.
- One mapping from percent to tint coverage: 50 % is the calibrated glass
  (55 % light / 66 % dark coverage), 0 % turns glass off (opaque rail, native
  material hidden), 100 % shows twice the default's material share (10 % light
  / 32 % dark coverage).
- A Settings slider control in `bbr-ui` (press, drag anywhere in the window,
  arrow keys / Home / End while it is the active control), its value label and
  a detail line naming the current state.
- The shared preference re-presents every browser window at once.
- Testkit `SettingsInput::Slide`, `SettingsSnapshot::active_slider`, real-app
  E2E and the `chrome_transparency_change` performance scenario.

## Non-goals

- No change to WKWebView opacity, page content, Settings/Gallery window paint
  or floating surfaces.
- No per-space or per-window transparency.
- No override of Reduce Transparency or Increase Contrast: both still force the
  opaque rail, whatever the slider says.
- No native-composite proof (owned by the [glass packet](chrome-desktop-glass.md)).

## Parallel-lane contract

Not parallelized. One local writer; no delegation or sibling-worktree changes.

## Constraints

- The value applies at once (no Save button), like every Settings row.
- Up to the default, primary labels keep WCAG AA 4.5:1 over any raw desktop.
  Above it, label contrast depends on the wallpaper: the user's explicit
  choice, stated in the row's detail line.
- A drag writes the preferences file only when the stepped value changes.
- Do not weaken functional or performance checks; the near-opaque regression
  ceiling still applies at the default.

## Evidence

### Unit and contract

- [x] Percent → coverage scale: opaque at 0, calibrated at 50, monotonic, clamped; AA at every value up to the default.
- [x] Pointer position → stepped percent mapping; preference round trip and tolerant parsing.
- [x] Probe commits only from a render of the armed coverage and material state.

### Real-app E2E

- [x] Slider press updates the current and another browser window live; keys
      step 5 %, End/Home jump; 0 % hides the material; Reduce Transparency still
      wins; the file stores the value and a relaunch restores it.

### Performance

- [x] `chrome_transparency_change` starts at the slider's production event and
      commits from the browser-window frame that painted the armed coverage
      (50/100 ms warm class). The optimized executables compile.
- [ ] Execute it after Patrick's 2026-09-27 performance-execution deferral
      lifts; no performance run or pass is claimed.

### Visual

- [x] Official `just visual-check` of the changed Settings › Appearance stills;
      review required, no baseline acceptance.
- [ ] Patrick confirms the live effect and the range on his Mac.

## Completion gates

```bash
just verify
mbx clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-vm e2e-shell-settings e2e-windows e2e-shell-sidebar
just visual-check
```

Use the established hardened Tart wrapper for VM suites. Performance runs stay
deferred; only compilation of the optimized scenario is required now.

## Delivery sequence

1. Preference, mapping and slider control with lower-level tests.
2. Live multi-window E2E and the performance scenario.
3. Local review (no independent reviewer claimed), affected Tart suites and the
   official visual check.

## Iteration policy

Diagnose failures from typed Settings and native-material snapshots before
changing paint. Keep the website and content card unchanged.

## Blocked stop condition

Stop with the exact failing check, preserved worktree and the decision needed
if the slider cannot apply live without a second preference authority, or if
the Tart profile cannot run.

## Delivery ledger

```text
Outcome: Settings › Appearance › Window › Sidebar transparency (0–100 %,
  default 50 %) changes the chrome glass live in every browser window and
  persists; Reduce Transparency / Increase Contrast still win.
Commits: the local implementation commit after b09ee0e0 (this ledger).
Functional evidence: full mbx-routed `just verify` 2,169 tests; strict testkit
  all-target Clippy; new unit tests for the scale, pointer mapping, preference
  parsing and the probe commit gate.
Real-app E2E: hardened Tart e2e-windows 10 (incl. the new transparency test),
  e2e-shell-settings 1, e2e-shell-sidebar 24 on the final source:
  /tmp/bbr-e2e-vm/run-20261017-141821-85622 (cleanup verified). The same suites
  also passed before the final module split: run-20261017-135522-83166.
Performance evidence: `chrome_transparency_change` scenario, 50/100 ms budget,
  seed baseline entry and full-suite sample check; optimized executables
  compile. Not executed (deferral since 2026-09-27).
Visual: final batch reports/visual/batches/1792239668826525000-11638 (162
  images, zero capture errors, no environment mismatch). Exactly the six
  Appearance-pane stills (pane, contrast menu, motion menu × light/dark)
  differ from the glass-correction batch; all other PNGs are byte-identical.
  The first slider batch 1792238571399515000-72989 showed an almost invisible
  unfilled track in dark mode; one fix (`border.strong`) and one confirmation
  batch 1792238962861571000-27114 followed; the final batch repeats it
  byte-for-byte. Review required; no baseline accepted.
Retained failures: two `just verify` attempts failed only the untouched
  `bbr-command` 50 ms benchmark under heavy host load (p95 53.8 ms at load
  ~51; max 141.3 ms at load ~190: /tmp/bbr-glass-correction-verify.log,
  /tmp/bbr-transparency-verify3.log). Unchanged reruns pass; budget unchanged,
  no causation claimed. The first verify also failed roadmap validation until
  the new packet was marked intent-to-add.
Roadmap/documentation updates: this packet, Phase 4, UI track, ROADMAP,
  DESIGN §2, ARCHITECTURE, E2E_ARCHITECTURE, performance contract, delivery.
Open blockers and non-goals: Patrick's live check of effect and range;
  performance execution; native-composite proof (glass packet).
Worktree state: clean after the commit; nothing pushed.

Timing: not instrumented; no estimates claimed.
```
