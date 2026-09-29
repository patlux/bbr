//! Real-app drag-to-chip E2E (F2a layer, DESIGN.md §10.9/§11.3/§12).
//!
//! Proves the production tab drag onto a §11.3 space chip over real
//! WKWebViews: the concrete pressed tab (never implicitly the active tab),
//! the exact persistent destination chip through the real strip layout hit
//! geometry, the typed during-drag target + §12 destination-profile
//! companion on the snapshot and in the semantic tree, same-profile live
//! session preservation, cross-profile exactly-one recreation in the
//! destination store, source-selection repair, and the no-op contract
//! (own space, chip gaps, the fixed `[Downloads]`/`[+]` controls, and
//! Escape cancel). Every drag runs through the production request/handler
//! path (`BeginTabDrag`/`MoveTabDrag`/`EndTabDrag`/`CancelTabDrag`) — never
//! a direct controller mutation — and every commit goes through the single
//! existing tab-move authority. The footer `[Downloads][Space Strip][+]`
//! composition, the space theme tint, the tracked strip scroll, and the
//! downloads panel stay invariant across the drag. The shared harness lives
//! in `e2e_common`; focused helpers in `support`.

#![cfg(target_os = "macos")]

#[path = "../e2e_common/mod.rs"]
mod e2e_common;
#[path = "../e2e_semantics_common/mod.rs"]
mod e2e_semantics_common;

use e2e_common::*;
use support::*;

mod cross_profile;
mod invariants;
mod noops;
mod profile_feedback;
mod same_profile;
mod support;

/// The §8 normal-density row surface the press-anchored drag geometry
/// anchors within (`row() / 2` — the press sits inside the 36 pt surface;
/// slot moves step on the §8 pitch, which the drag-chip scenarios express
/// in absolute pointer deltas).
const ROW_H: f32 = 36.0;
