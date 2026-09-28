#![forbid(unsafe_code)]
#![deny(missing_debug_implementations)]

//! # bbr-command — portable command-bar core
//!
//! The pure, dependency-free core of the Phase-5 unified command bar: typed
//! input parsing, a provider-neutral owned corpus, a deterministic command
//! registry, and ranking across the Phase-5 priority groups.
//!
//! ## Pipeline
//!
//! ```text
//! text + Mode ──parse──▶ Query ──rank──▶ RankedResults
//!                                     ▲
//!                    Corpus + CommandRegistry + QueryContext
//! ```
//!
//! - [`parse::parse_destination`] classifies raw text as a navigation
//!   destination or a web search (see its module docs for the exact rules).
//! - [`rank::rank`] resolves a [`Query`] against the [`Corpus`] and
//!   [`CommandRegistry`] into typed [`CommandResult`]s ordered by the
//!   Phase-5 groups, with deterministic tie-breaking that never depends on
//!   hash-map iteration order.
//! - Results carry typed [`Activation`]s (select a tab or space, open history,
//!   run an action, navigate, search) plus a stable `key` string suitable for
//!   later snapshot/testkit identity.
//!
//! ## Boundaries
//!
//! This crate has no GPUI, AppKit/WebKit, SQLite, filesystem, socket, or
//! platform dependency and forbids `unsafe`. It never touches a render path
//! or global state, so callers may re-run ranking freely (for example behind
//! a later debounce/cancellation seam). The corpus is provider-neutral:
//! domain/persistence layers construct [`TabEntry`]/[`SpaceEntry`]/
//! [`ProfileEntry`]/[`HistoryEntry`] values; identity newtypes mirror the
//! domain/engine shapes and are mapped at the app-composition boundary.
//! Global search-engine settings, a validated custom `%s` template, and
//! deterministic search-URL rendering live in [`search_settings`].
//!
//! Real-app presentation E2E, keyboard focus, and command-bar GPUI are owned
//! by the later serial app-composition packet, not by this crate.
//!
//! # Example
//!
//! ```
//! use bbr_command::corpus::Corpus;
//! use bbr_command::id::{ProfileId, SpaceId, TabId};
//! use bbr_command::parse::{Mode, Query, QueryContext};
//! use bbr_command::rank::{rank, ResultGroup};
//! use bbr_command::registry::CommandRegistry;
//!
//! let profile = ProfileId(1);
//! let space = SpaceId(10);
//! let mut corpus = Corpus::new();
//! corpus.tabs.push(bbr_command::corpus::TabEntry::new(
//!     TabId(100),
//!     "BBR docs",
//!     "https://example.com/docs",
//!     space,
//!     profile,
//!     false,
//!     0,
//! ));
//! let registry = CommandRegistry::builtin();
//! let query = Query::new(Mode::Switcher, "bbr");
//! let context = QueryContext::with_current(space, profile);
//! let ranked = rank(&corpus, &registry, &query, &context, 16);
//! assert_eq!(ranked.results[0].group, ResultGroup::CurrentTab);
//! assert_eq!(ranked.results[0].key, "tab:100");
//! ```

pub mod corpus;
pub mod id;
pub mod parse;
pub mod rank;
pub mod registry;
pub mod search_settings;

mod collect;
mod recents;

pub use corpus::{Corpus, HistoryEntry, ProfileEntry, SavedPageEntry, SpaceEntry, TabEntry};
pub use id::{ActionId, BookmarkId, HistoryId, ProfileId, SpaceId, TabId};
pub use parse::{Destination, Mode, Query, QueryContext};
pub use rank::{
    rank, Activation, CommandResult, MatchQuality, RankKey, RankedResults, ResultGroup,
};
pub use registry::{Action, ActionCategory, CommandRegistry};
pub use search_settings::{
    engine::{CustomUrlTemplate, SearchEngine, TemplateError, CURATED},
    render::{percent_encode, render_search_url},
    settings::{SearchSettingsV1, SettingsScope, SettingsVersion},
    store::{LoadOutcome, ResetOutcome, SettingsStore, StoreOutcome},
};
