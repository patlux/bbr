# Task packet — Blueprint D11: Site settings and permission prompts

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[complete browser blueprint](paper-browser-blueprint.md) ·
[DESIGN.md](../../../DESIGN.md) §11.5, §19.

Status: implemented 2026-09-30 (DESIGN §19); position 18 of 24. Paper boards: M-8 · Site settings, M-9 · Permission requests, L-6 · Settings — Privacy & site data, PR-14 · Permissions (file
“BBR — Browser Shell”).

## Outcome

One prompt per site anchored to the address, in-use device chips, and a site-settings popover with Allow / Ask / Block per permission, zoom and site data.

## Scope

- Implement the target as drawn on the boards above, light and dark, with
  the variants of page 40 that apply (increased contrast, density, Reduce
  Motion).
- Land the decision in DESIGN.md §11.5 in the same group and flip the
  D11 row of §19 to adopted.
- Expose the new surface's stable ids and typed snapshot evidence through
  the testkit.

## Non-goals

- Other blueprint deltas, even where a board shows them next to this one.
- Behaviour the boards mark as Native OS (drawn by macOS or WebKit).

## Parallel-lane contract

Not parallelized.

## Constraints

- Today on main: The permissions and site-settings contracts exist; no prompt, popover or in-use indicator.
- GPUI owns the chrome; floating surfaces over web content use the one
  native Portal; web content is never sampled or redrawn.
- Colours, type and spacing come from the palette and theme tokens that
  the Paper file mirrors; no new literal colours.
- No weakened tests, budgets, bars or acceptance criteria.

## Evidence

### Unit and contract

- [x] Unit/contract: `site_permissions` (per profile/origin/capability, file round trip and tolerance, origins), `app::permissions` (remembered decisions, prompt copy), `bbr-web-macos` `permission_requests` origin strings and `profile_data::record_matches`, registry catalog (action 19), termination contract with the permission table.

### Real-app E2E

- [x] `just e2e-permissions` (`e2e_permissions.rs`): prompt anchored under the address naming site and profile, Remember + Don't Allow writes `site-permissions.txt`, quiet refusal with `bbr-address-blocked`, site settings Allow/Ask/Block applied and written, outside press closes, remembered Allow grants without a prompt, queue of two, one-time answers not remembered, `bbr-address-capture` from capture state, Settings › Privacy list and Forget, ⌘T Site Settings… and Escape, Clear Site Data… removes the real WebKit cookie; settings, profiles, zoom, sidebar, command and gallery suites green.

### Performance

- [ ] Extend `just e2e-perf` with the prompt and site-settings presentation scenarios (deferred).

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-permissions
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

If Patrick rejects or changes the D11 proposal, or the target conflicts
with an existing contract, stop with the evidence, the conflict and the
decision needed, and update the §19 row instead of implementing.

## Delivery ledger

```text
Outcome:                Camera/microphone permission prompts through the WKUIDelegate, remembered per profile/origin/device in site-permissions.txt, in-use and blocked marks in the address, site-settings popover (permissions, zoom, Clear Site Data, All Site Settings), Settings › Privacy site list, usage descriptions in the bundle.
Commits:                see git log (feat/fix commit carrying this packet)
Functional evidence:    Unit/contract: `site_permissions` (per profile/origin/capability, file round trip and tolerance, origins), `app::permissions` (remembered decisions, prompt copy), `bbr-web-macos` `permission_requests` origin strings and `profile_data::record_matches`, registry catalog (action 19), termination contract with the permission table.
Real-app E2E:           `just e2e-permissions` (`e2e_permissions.rs`): prompt anchored under the address naming site and profile, Remember + Don't Allow writes `site-permissions.txt`, quiet refusal with `bbr-address-blocked`, site settings Allow/Ask/Block applied and written, outside press closes, remembered Allow grants without a prompt, queue of two, one-time answers not remembered, `bbr-address-capture` from capture state, Settings › Privacy list and Forget, ⌘T Site Settings… and Escape, Clear Site Data… removes the real WebKit cookie; settings, profiles, zoom, sidebar, command and gallery suites green.
Performance evidence:   unpassed — execution deferred (Patrick, 2026-09-27)
Roadmap/documentation updates: DESIGN §11.5 Permissions + Site settings, §11.18 Privacy, §19 D11 row; limits recorded.
Open blockers and non-goals: location, notifications, clipboard and pop-up rows are not drawn (WebKit asks for none of them on macOS); the native WKUIDelegate request and KVO capture path need a hand check on the bundled app (the unbundled E2E binary has no usage descriptions, so WebKit exposes no media devices, and a real grant would raise the macOS privacy prompt); performance scenarios deferred
Worktree state:         main, clean after commit
```
