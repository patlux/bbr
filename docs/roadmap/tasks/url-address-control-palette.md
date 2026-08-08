# Task — Address control and palette over the address

**Status:** planned · **Owner:** UI shell · **Track:**
[ui-design-system](../tracks/ui-design-system.md) · **Phase:**
[04-ui-shell](../phases/04-ui-shell.md)

## Outcome

The sidebar address stops being an editable field and becomes a display
control that shows the domain. All text entry moves into the palette, which
opens **over** the address control instead of dropping below it, with the full
URL preselected.

## Why

BBR currently has two places that can hold an address in an editing state: the
sidebar field and the palette. Two editing surfaces mean two focus
appearances, two cursors and two answers to "what is the current address" —
and the sidebar field's teal focus ring is the visible symptom Patrick reacted
to. Collapsing entry into one surface removes the class of problem rather than
restyling it, and it frees the sidebar to show the one thing a user actually
scans for: which site am I on.

The reference behavior is Zen's: a quiet translucent pill carrying the domain
and a few action glyphs, and a search panel that takes the pill's place when
invoked.

## Scope

- The address control per DESIGN §11.5: domain-only content, borderless
  translucent pill, hover wash, trailing accessory slot with reserved
  geometry, button semantics, no border and no focus ring at any time.
- The anchored palette presentation per DESIGN §11.4: covers the address
  control, top and leading edges flush with it, width clamped 360–`win:palette`,
  full URL preselected in its own input.
- Rewriting the affected `e2e-shell-command` scenarios onto the new contract.
- The §11.3 profile hint and §12 chip presentation follow the surface that is
  actually visible.

## Non-goals

- The centered Cmd-T palette. It ships as delivered by U1 and only shares the
  input component and result engine.
- Accessory icons beyond the reserved slot. Permissions, reader and addon
  glyphs are follow-up slices on top of the slot geometry (DESIGN §17.14).
- History results in the palette. That waits on the history provider.
- Any change to the result ranking engine.

## Constraints

- **No backward compatibility.** The field-focus state, its teal ring and its
  editable text are removed outright, not deprecated.
- Geometry comes from the settled S1 scale table; invent no sizes.
- The translucent fill is a token pair driven by theme and space tint — never
  a hardcoded color, and never a bordered field style.
- Exactly one command surface open at a time (DESIGN §11.4) remains true.
- The address control must never accept keyboard text and must hold no
  editing state, cursor or selection.
- Opening or closing the palette must not change tab, session, profile or
  space identity, and must not disturb the native web view.
- Existing palette performance budgets (`palette_open`, `command_bar_*`)
  remain unchanged and must stay green.

## Evidence

- [ ] Component tests: domain derivation (scheme, path, query, `www.`
      elision, IDN, port, internal pages), ellipsis when an accessory is
      present, hover wash on both chrome polarities.
- [ ] Real-app E2E: activation by pointer and by `Cmd-L` opens the palette
      over the control; the control is not visible behind it; the full URL is
      present and selected; one keystroke replaces it; Enter navigates;
      Escape restores the previous state; focus returns to its origin.
- [ ] Real-app E2E: the address control never receives typed text, and no
      focus ring appears on it in any state.
- [ ] Rewritten `e2e-shell-command` scenarios pass on the new contract with no
      assertion weakened — a scenario that typed into the sidebar field now
      drives activation plus palette input.
- [ ] Existing palette perf scenarios green at unchanged budgets; a new
      scenario only if a genuinely new user-visible interaction appears, and
      then on an existing budget class.
- [ ] `just verify`, the affected suites, and the canonical `just e2e` at the
      end of the slice.

## Blocked stop condition

If the anchored overlay cannot cover the address control without either
overlapping the native web view incorrectly or violating the single-open-surface
rule, stop and report the measured geometry conflict rather than falling back
to a dropdown below the control.

## Open decisions

- DESIGN §17.13 — what internal pages show in the address control.
- DESIGN §17.14 — what the accessory slot carries in v1.
