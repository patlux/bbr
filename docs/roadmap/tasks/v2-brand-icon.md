# Task packet — v2 S9: Phosphor-b app icon

Parents: [v2 program](v2-visual-refresh-program.md) ·
[deterministic app packaging](macos-app-packaging.md) ·
[DESIGN.md](../../../DESIGN.md) §1, §7, §18.1 (D8).

## Outcome

The packaged `BBR.app` carries the “Phosphor b” icon (concept C): a lowercase
b on a dark continuous-corner tile whose bowl is the accent loading arc,
rendered deterministically for every `.icns` size, with a committed vector
master for design use.

## Scope

- Vector master `packaging/icon/bbr-icon.svg`.
- `scripts/package/gen_icon.py` renders the concept with pure math (no new
  dependency), preserving byte-determinism.
- Packaging tests and `just package-determinism` stay green.
- DESIGN §1/§7 name the icon; §18.6 row.

## Non-goals

- No animated Dock icon (D8); About-panel motion is out of scope.
- No notarization or Developer-ID signing (Phase 9).

## Parallel-lane contract

```text
Base condition:          S0 integrated
Depends on:              v2-contract; deterministic packaging lane
Provides/API:            AppIcon.icns content
Exclusive paths:         packaging/icon, scripts/package/gen_icon.py and its tests
Shared integration points: none
Integration order:       parallel-capable with S6
```

## Constraints

- Identical input → byte-identical icon (determinism contract).
- The mark stays legible at 16 pt.

## Evidence

### Unit and contract

- [x] Packaging script tests (icon sizes, determinism, render shape,
      16 pt size ladder, vector master geometry) — `just package-test` 14/14.

### Real-app E2E

- [x] Bounded package smoke launches the bundle (GUI lock): alive for the
      5.2 s observation, then terminated.

### Performance

- [x] Not applicable.

## Completion gates

```bash
just verify
just package-test
just package-determinism
just package-smoke
```

## Iteration policy

Render function first with a size-ladder check, then determinism, then smoke.

## Blocked stop condition

If packaging prerequisites (Xcode developer dir, codesign) are unavailable,
land the renderer and tests, mark the smoke deferred with the exact error.

## Delivery ledger

```text
Outcome:               "Phosphor b" rendered analytically (superellipse tile,
                        halo, stem, track, gradient arc with glow, SDF edges);
                        SVG master committed; placeholder retired.
Commits:               see git log (feat(package): render the Phosphor b …)
Functional evidence:   just verify ok; just package-test 14/14
Real-app E2E:          just package-smoke (bundle launched, 5.2 s alive)
Determinism:           just package (release build) + just
                        package-determinism: diff -r identical
Performance evidence:   not applicable
Deviations:            the glow is an analytic Gaussian falloff of the arc's
                        distance field rather than a blur pass (keeps the
                        renderer pure stdlib and ~3 s); the tile is an
                        n = 5 superellipse approximating Apple's continuous
                        corner.
Review:                serial self-review: renderer vs SVG master compared
                        side by side at 256 px; 16/32 px ladder inspected.
Worktree state:        clean after commit
```
