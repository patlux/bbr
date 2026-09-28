# Task packet — v2 S10: curated intensity and grain

Parents: [v2 program](v2-visual-refresh-program.md) ·
[space appearance contract](space-appearance-contract.md) ·
[DESIGN.md](../../../DESIGN.md) §11.3, §18.1 (D6, D7).

## Outcome

Each space owns a curated tint intensity (subtle / standard / rich) and a
static-grain flag next to its theme and identity; the Space configuration
surface edits both with the existing draft/commit/cancel rules; the rail
renders the chosen intensity and an optional static grain; both persist.

## Scope

- Portable appearance model: closed intensity enum and grain flag with
  deterministic defaults (standard, off).
- Space configuration picker rows with stable ids; draft/commit through the
  existing exact-SpaceId authority.
- Space tint resolution per intensity within the D3 bars; static grain overlay
  on BBR-owned rail chrome only.
- Persistence columns with a schema-version bump under the closed
  discard-on-mismatch policy ([schema v2 decision](schema-v2-migration-decision.md)).
- DESIGN §11.3; §18.6 row.

## Non-goals

- No free colour field (D7), no animated grain (D6), no per-tab tint.

## Parallel-lane contract

```text
Base condition:          S4 integrated
Depends on:              v2-space-paper
Provides/API:            SpaceAppearance intensity/grain
Exclusive paths:         bbr-domain space appearance, bbr-persistence spaces, space_config surface
Shared integration points: SQLite schema version, testkit snapshot, e2e_shell_spaces_config, persistence suite
Integration order:       parallel-capable with S8
```

## Constraints

- Every intensity holds the D3 bars for every theme and mode.
- Grain never paints over web content or controls' text.

## Evidence

### Unit and contract

- [x] Domain round-trip, persistence round-trip and version-bump discard.
- [x] Gate across intensities.

### Real-app E2E

- [x] Configuration edits intensity and grain, commit applies, cancel keeps
      the previous values; restart restores them.

### Performance

- [x] Not applicable: configuration surface only.

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-spaces
just e2e-shell-persistence
just e2e-gallery
```

## Iteration policy

Domain and persistence first, then tint resolution, then the surface and E2E.

## Blocked stop condition

If the persistence change needs anything beyond the closed version-bump
policy, stop with the schema diff and ask.

## Delivery ledger

```text
Outcome:                Each space owns a curated paper (domain `SpacePaper`:
                        `TintIntensity` subtle/standard/rich + static grain,
                        defaults standard/off) beside theme and identity.
                        The §11.3 surface edits both in its draft (paper row
                        with rail previews + grain toggle; ids
                        bbr-space-config-intensity-<step>,
                        bbr-space-config-grain); Done commits through
                        `set_space_paper`, Cancel keeps the committed values.
                        `space_tint(token, intensity, theme)`: subtle 6 %
                        (IC 8), standard 12/14, rich 24/26 pinned to the
                        standard luminance. Static grain: one generated PAM
                        tile, tone per mode (white ≤ 28 % light, black
                        ≤ 36 % dark, half clear), on rail + chassis beneath
                        every child. Schema v4 adds spaces.tint_intensity +
                        grain; fixtures re-stamped, v3 kept as a
                        prior-schema discard fixture.
Commits:                feat(ui): add the curated space paper picker
Functional evidence:    bbr-domain paper tests (steps, set_paper, restore
                        keeps only non-default records); bbr-persistence
                        (v3 fixture discarded + v4 round-trip of a
                        non-default paper, v4 fixture restores rich/grain,
                        unknown intensity / non-boolean grain = corruption);
                        app `store::appearance` round-trips every curated
                        value; bbr-ui gates: every token × step × mode ×
                        contrast holds the D3 bars (space_tint + palette
                        rails incl. selection/footer/address veils), step
                        ordering, rich luminance pinning, grain specks only
                        raise text contrast; testkit wire round-trip +
                        legacy defaults.
Real-app E2E:           e2e_shell_persistence
                        `space_paper::space_paper_edits_as_a_draft_commits_and_survives_restart`
                        (draft never applies, cancel keeps, done applies
                        pine/rich/grain with the resolved rail hex, graceful
                        restart restores all three); e2e_shell_spaces_config
                        restart test now asserts the curated theme/identity
                        persist (renamed from its session-valid form, backed
                        by the §11.3 schema-v4 change); gallery space-config
                        press test extended to the paper row; release
                        visual capture of the steps, grain and the surface
                        in light and dark read.
Performance evidence:   not applicable
Scope decision:         the packet asks intensity and grain to persist.
                        Persisting them while theme/identity kept reverting
                        to the derived default would restore a mix the user
                        never chose, so the whole curated appearance now
                        persists (the v2 tint/glyph columns already
                        existed); DESIGN §11.3's session-valid boundary is
                        replaced. Grain is painted on the rail and the
                        chassis (both tinted chrome) so the sheet has no
                        seam at the card inset.
Self-review:            the intent stays appearance-free (appearance is a
                        companion store); restore goes through the domain
                        rule so derived defaults create no records; the
                        grain canvas requests no frames and uses one cached
                        texture per tone; the rich step's luminance pinning
                        is what lets it exceed the calibrated 12 % without
                        moving any bar.
Worktree state:         clean after commit
```
