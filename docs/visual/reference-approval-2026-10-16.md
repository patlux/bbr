# Browser reference approval — 2026-10-16

Patrick approved the final inspector-corrected report with:

> ok sieht alles gut aus. lass uns den stand nehmen

This accepts the exact browser images and reported environment below. It does
not complete uncaptured coverage, approve Gallery references or commit the
110 pre-existing unrelated working-tree entries.

## Approved authority

- Report: `/Users/patwoz/dev/Personal/macos/bbr/reports/visual/batches/1792166040944502000-85620/review/index.html`
- Capture: `/Users/patwoz/dev/Personal/macos/bbr/reports/visual/batches/1792166040944502000-85620/browser`
- Portable tool: immutable `c4a2b7fc4db88751150b0f4272e800f2e7e3ab6b` export, imported by `2e2b33b5`.
- Capture SHA-256: `42a5ed1d488935f358d1ed68f778abfcbcb46a38492db9ea5880e3b747e17daa`.
- Collection SHA-256: `f058c981b774dd1df3269e37dd6e979ab1b6f88588d75f4d8e7061d1a3853fdc`.
- Reviewed HTML SHA-256: `c7c3b9b5b3a11334d9cd8f06ee5eeb6c3ad2e50825b15c1281b6aa8b5be1a74e`.

The pinned `accept-collection` command selected all **156 changed/missing**
`browser:case` IDs from that exact collection: 66 replacements and 90 additions.
The six already-identical references and their original case/source records remain
unchanged. The store now has 162 references. No image format conversion, resizing,
masking, tolerance, manual metadata edit or acceptance-time recapture was used.

`--adopt-suite browser` explicitly adopts this report's environment. Its rendering
deltas are host-build macOS 27.0 → 27.0.1 and build 26A428 → 26A434. Portable-tool
revision and guest Metal registry instance remain separately classified provenance.
No other suite/environment was adopted.

The command acknowledged exactly these withheld IDs, without altering their
catalog entries or marking coverage complete:

- `browser:remaining-app-surfaces`
- `browser:history-remaining-states`
- `browser:settings-remaining-states`
- `browser:portal-surfaces`
- `browser:permission-surfaces`
- `browser:welcome-about`

The original report's new acceptance journal records state `complete`, completed
suite `browser`, all selected IDs, environment adoption and acknowledgements:
`/Users/patwoz/dev/Personal/macos/bbr/reports/visual/batches/1792166040944502000-85620/review/acceptance.json`.
Journal SHA-256: `fc865ba48f45170964f482ad613863fe3ed90b4bc1854fbacc8d1442f008348d`.
All 308 original sealed report files remain byte-identical; nothing was resealed.

Baseline metadata SHA-256:

- Before: `d9b23b6161cabf9b84525c3392d5f70c86a8a36fc6662783efc476112bef649a`
- After: `c3258b2f854a476b6a9c34cc4ad56460cef7c98e4617ba1df93abd9c0018cbd2`

## Verification after adoption

An official consumer comparison of the stored approved capture reports 162 exact
matches, no missing references, no rendering drift and zero comparison errors:
`/Users/patwoz/dev/Personal/macos/bbr/reports/visual/accepted-1792166040944502000-85620/index.html`.
This comparison did not execute a producer or probe the environment.

A subsequent **fresh** `just visual-check browser` used the hardened disposable
Tart route and produced:
`/Users/patwoz/dev/Personal/macos/bbr/reports/visual/batches/1792167950283367000-66451/review/index.html`.
Its capture SHA-256 is
`bd9729e47a05f76f6246dcada82b3236900db3fb3d37b07310cda064b086123a`.
All 162 PNG hashes equal both the approved capture and accepted references.
All 1,542 source hashes equal the approved capture and current working tree.
Both new reports have 488 valid sealed files, 162 exact results, zero comparison
errors and no rendering mismatch. Six withheld families and one unsupported state
remain explicit. Exit 1 means incomplete coverage, not a failed capture or pixel
comparison. This is not full application/native-state or performance acceptance.

Fresh bounded `just visual-test` passes with 342 reported Rust test entries plus
the recipe's Python checks. The pinned tool was built through locked/offline mbx;
its `validate` command confirmed the approved sources/images before acceptance.
All 94 vendor hashes validate. Separate read-only verification checks accepted
image/record identities, both new report seals, the unchanged original report,
and 742 preserved non-baseline dirty/capture/report files with their file modes.
Only the 156 selected PNGs and baseline metadata change in the reference store.

The new guest's cleanup observations prove clone/key removal, detached disk,
reaped owned process and unreachable SSH. The host gate is inactive with no
pending work or groups. Network audit reports SSH 22 reachable, ports 88 and
5900 refused. No host application GUI, listener, performance run or publication.

## Retained attempts and local evidence

The first acceptance invocation supplied generic `--repo`, which collection
commands reject because their sealed authorities already bind repositories.
It exited 2 before writing; all saved hashes/modes and absent journal were checked.
The corrected invocation omitted only that flag and completed successfully.
The first post-adoption comparison could not start because its previously present
managed-target executable was absent. A bounded unchanged consumer rebuild restored
the executable; the ordinary comparison then completed. No cause is asserted for
the target disappearance, and neither failure changed acceptance policy.

Backup, exact CLI arguments/selections, command responses, build/test/capture logs
and the read-only verifier are retained under:
`/tmp/bbr-reference-approval-1792166040944502000-85620/`.
The summary is `/tmp/bbr-reference-approval-1792166040944502000-85620/verification.json`.
This was one local writer's verification, not an independent review. Unrelated
implementation remains dirty and is not silently included in the reference commit.
