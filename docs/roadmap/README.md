# Roadmap maintenance

This directory turns [ROADMAP.md](../../ROADMAP.md) into a drill-down planning system.

## Hierarchy

```text
ROADMAP.md
  └── phases/<phase>.md
        └── tracks/<workstream>.md
              └── tasks/<implementation-packet>.md
```

- **Roadmap:** product-level sequence and current focus.
- **Phase:** independently valuable product outcome and phase acceptance.
- **Track:** one durable concern spanning one or more phases.
- **Task packet:** bounded implementation work that should fit one or a few logical commits.

## Canonical ownership

A concrete checkbox must have exactly one canonical owner, normally a track or task file. Higher-level checkboxes are roll-ups and are checked only when every required linked acceptance item is complete.

Do not copy the same detailed checklist into multiple files. Link to it.

## Checkbox rules

- `[x]` means implementation exists and verification evidence was run.
- `[ ]` means not yet proven, even if code is partially present.
- Add `(deferred)` only when explicitly moved to a later linked phase.
- Do not check an item merely because a lower-level API exists; check the user-visible or contract outcome stated by the item.
- When completing an item, update its canonical file in the same commit as the implementation whenever practical.
- A browser feature remains `[ ]` until automated real-app E2E coverage passes. Unit, fake-engine, and contract tests are additional evidence, not substitutes.

## Evidence

Prefer short evidence references next to completed groups:

```text
Evidence: `just verify`, `just e2e-smoke`, commit `abc1234`.
```

Avoid volatile timings unless performance is the acceptance criterion. Implemented-fact details belong in `ARCHITECTURE.md` or `E2E_ARCHITECTURE.md`, not in every roadmap file.

## Adding work

1. Place the item under the nearest existing track.
2. Create a task packet when implementation needs sequencing, explicit non-goals, or several commits.
3. Add a new track only for a durable concern.
4. Add a new phase only for a distinct user-visible or architectural milestone.
5. Link upward so every task is discoverable from `ROADMAP.md`.

## Regular review cadence

At the start or end of a meaningful implementation group:

1. Read `ROADMAP.md` and the current phase.
2. Update completed canonical checkboxes.
3. Promote the next task packet into **Current focus**.
4. Confirm phase acceptance still matches the product.
5. Run unit/contract tests plus automated real-app E2E for every affected browser feature before marking work complete.

`just roadmap` validates the hierarchy's local Markdown links. It is also part
of `just verify`, so moved or deleted roadmap detail files cannot silently leave
stale references behind.
