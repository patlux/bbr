# Phase 10 — Cross-platform adapters

Parent: [Roadmap](../../../ROADMAP.md)

## Outcome

Portable product behavior runs over practical native engine and window adapters on Windows and Linux without weakening macOS quality.

## Checklist

- [ ] Confirm Windows engine/embedding strategy through a focused spike.
- [ ] Confirm Linux engine/embedding strategy through a focused spike.
- [ ] Implement `bbr-web-windows` and its native-view bridge.
- [ ] Implement `bbr-web-linux` and its native-view bridge.
- [ ] Keep domain, controller, command, persistence, and most GPUI components portable.
- [ ] Map native menus, credentials, downloads, permissions, file panels, drag/drop, and Accessibility per platform.
- [ ] Adapt shortcuts and platform conventions without promising identical engine behavior.
- [ ] Run the shared fake-engine/controller suites everywhere.
- [ ] Add platform-native integration and packaged-app smoke suites.
- [ ] Apply platform-specific UI adaptations while preserving BBR's visual identity.

## Phase exit

- [ ] Shared product behavior is consistent; platform-native expectations remain intact.
