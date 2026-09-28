# Task packet — Blueprint D03: Address control states

Parents: [Phase 4](../phases/04-ui-shell.md) ·
[complete browser blueprint](paper-browser-blueprint.md) ·
[DESIGN.md](../../../DESIGN.md) §11.5 §17.13, §19.

Status: implemented 2026-09-29 (DESIGN §19); position 5 of 24. Paper boards: S-1 · Address control, M-8 · Site settings (file
“BBR — Browser Shell”).

## Outcome

Not-secure glyph for http, internal pages as short name + glyph (Library, Settings, History, Welcome), local files by file name, and a leading site-identity trigger that opens site settings.

## Scope

- Implement the target as drawn on the boards above, light and dark, with
  the variants of page 40 that apply (increased contrast, density, Reduce
  Motion).
- Land the decision in DESIGN.md §11.5 §17.13 in the same group and flip the
  D03 row of §19 to adopted.
- Expose the new surface's stable ids and typed snapshot evidence through
  the testkit.

## Non-goals

- Other blueprint deltas, even where a board shows them next to this one.
- Behaviour the boards mark as Native OS (drawn by macOS or WebKit).

## Parallel-lane contract

Not parallelized.

## Constraints

- Today on main: The control shows the host only (internal pages unnamed, §17.13 open).
- GPUI owns the chrome; floating surfaces over web content use the one
  native Portal; web content is never sampled or redrawn.
- Colours, type and spacing come from the palette and theme tokens that
  the Paper file mirrors; no new literal colours.
- No weakened tests, budgets, bars or acceptance criteria.

## Evidence

### Unit and contract

- [x] bbr-ui text::address_identity (https/loopback secure contexts, plain http not secure, bbr:// internal names, file names decoded without path, placeholders) and AddressMark mapping; Warning/Gear/Document §7 glyphs; gallery address families (not secure, Library, Settings, local file); semantics value (Not secure / BBR page / Local file).

### Real-app E2E

- [x] `just e2e-shell-zoom` — identity::address_marks_not_secure_sites_and_names_local_files over a real WKWebView (loopback fixture secure with no mark; http://bbr-not-secure.invalid shows the mark and 'Not secure'; a file:// fixture shows page2.html and 'Local file'); command, sidebar and gallery suites green. Internal pages are unit/gallery evidence until the first bbr:// page is served (D10/D15).

### Performance

- [ ] Performance scenario: none added — the mark is a static glyph derived from the URL on the existing address render path; measurement is deferred (Patrick, 2026-09-27).

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-zoom
just e2e-perf
just e2e
```

## Iteration policy

Start with the smallest production path of the first board frame, then add
states in the order the boards list them. Compare the running app with the
boards after each state (visual capture), fix the mechanism behind each
difference, and keep DESIGN.md and this packet in step.

## Blocked stop condition

If Patrick rejects or changes the D03 proposal, or the target conflicts
with an existing contract, stop with the evidence, the conflict and the
decision needed, and update the §19 row instead of implementing.

## Delivery ledger

```text
Outcome:                The address shows the not-secure mark for plain http, internal pages by short name + glyph, and local files by file name.
Commits:                see git log (feat/fix commit carrying this packet)
Functional evidence:    bbr-ui text::address_identity (https/loopback secure contexts, plain http not secure, bbr:// internal names, file names decoded without path, placeholders) and AddressMark mapping; Warning/Gear/Document §7 glyphs; gallery address families (not secure, Library, Settings, local file); semantics value (Not secure / BBR page / Local file).
Real-app E2E:           `just e2e-shell-zoom` — identity::address_marks_not_secure_sites_and_names_local_files over a real WKWebView (loopback fixture secure with no mark; http://bbr-not-secure.invalid shows the mark and 'Not secure'; a file:// fixture shows page2.html and 'Local file'); command, sidebar and gallery suites green. Internal pages are unit/gallery evidence until the first bbr:// page is served (D10/D15).
Performance evidence:   unpassed — execution deferred (Patrick, 2026-09-27)
Roadmap/documentation updates: DESIGN §11.5 content + identity mark, §17.13 resolved, §19 D03 implemented
Open blockers and non-goals: none
Worktree state:         main, clean after commit
```
