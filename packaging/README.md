# BBR packaging lane (L1) — deterministic `.app` bundle

Owned by the prepared Phase 9 release-safety lane
[docs/roadmap/tasks/macos-app-packaging.md](../docs/roadmap/tasks/macos-app-packaging.md).
This lane deliberately does **not** integrate with the later GPUI anchor or
any app code: it consumes the already-built `bbr` Mach-O binary generically.

## What is produced

```text
<output>/BBR.app/
  Contents/Info.plist              token-filled from Info.plist.template
  Contents/PkgInfo                 APPL????
  Contents/MacOS/bbr               the release binary, mode 0755
  Contents/Resources/AppIcon.icns  "Phosphor b" (scripts/package/gen_icon.py; master packaging/icon/bbr-icon.svg)
  Contents/_CodeSignature/         ad-hoc signature (unless --skip-codesign)
```

## Determinism

Identical input binary → byte-identical bundle:

- fixed, ordered file writes; stale outputs wiped first;
- one fixed mtime for every file and directory we write
  (`SOURCE_DATE_EPOCH`, default `2020-01-01T00:00:00Z`);
- the icon is rendered from pure integer/float math (no randomness, no
  timestamps, fixed zlib level);
- ad-hoc signing uses `codesign --force --sign - --timestamp=none`; after
  signing, every file and directory in the bundle (including the executable
  and directories codesign touches) is pinned back to the fixed epoch.

`just package-determinism` repackages the same binary and `diff -r`s the two
trees as living evidence.

## Placeholders (confirmed in Phase 9, not invented here)

- `CFBundleIdentifier` default `dev.patwoz.bbr` — no final bundle id exists yet.
- `LSMinimumSystemVersion` `13.0` — the repo pins no deployment target yet.
- Notarization/Developer-ID signing remain Phase 9 work. `AppIcon.icns` is
  the adopted "Phosphor b" product icon (DESIGN §18.1 D8), rendered from the
  vector master's geometry.

## Boundaries

- No `Cargo.toml`/`Cargo.lock`/app-code changes; no new dependencies.
- All recipes bounded (`just package-verify`, or run the parts individually:
  `package-test`, `package`, `package-determinism`, `package-smoke`).
- The launch smoke is a bounded real-app launch through the bundle executable
  with `--bbr-hidden`, wrapped by the existing host-wide GUI gate
  (`scripts/with_gui_lock.py`); the process group is always reaped.
- Packaging is macOS-only by nature (Mach-O check, codesign); the scripts
  degrade cleanly elsewhere for the non-codesign unit paths.
