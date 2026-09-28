//! The in-process, non-network gallery E2E suite (packet
//! `component-gallery-mvp.md`): startup, tree keyboard operation, stable
//! `StoryId`s, mode/state switching, the every-story-renders sweep, the
//! real-pointer disclosure collapse/expand scenarios (finding 1), the
//! real Core Controls interaction scenarios (finding 2), the Tab Row
//! recorder scenarios, and the Group 4 sidebar-component recorder
//! scenarios (packet `component-gallery-browser-components.md`) — all
//! driven through the production methods and real pointer/keyboard
//! events, and asserted on the semantic snapshot. The harness uses GPUI's
//! established main-thread test support (`TestAppContext` +
//! `VisualTestContext` on `TestPlatform`), so the suite runs on the test
//! thread without a real window server. GUI-locked via `just e2e-gallery`;
//! never part of `just e2e`.
//!
//! macOS-only (the gallery app is macOS-gated for v1); the whole module is
//! `#[ignore]`d so a plain `cargo test --workspace` never runs the suite.

#![cfg(target_os = "macos")]

mod address;
mod auth;
mod command;
mod context_menu;
mod controls;
mod downloads;
mod error;
mod find;
mod fixture_theme;
mod group4;
mod library;
mod pointer;
mod sidebar;
mod space_config;
mod startup;
mod sweep;
mod tab_menu_framing;
mod tab_row;
mod tree_keyboard;
