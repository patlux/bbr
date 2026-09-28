//! Command result presentation mapping (DESIGN.md §11.5): the wire-facing
//! row shape, the stable group names, and the destructive flag. Split out
//! of `command_rank.rs` so the ranking orchestration stays focused — the B2
//! Saved Page group and the optional collection label (packet
//! `saved-pages-actions-command`, DESIGN.md §11.16 `type:meta`) live here
//! with the rest of the presentation contract.

use bbr_command::rank::{Activation, CommandResult, ResultGroup};

/// One published ranked row. The typed [`Activation`] stays app-local so the
/// activation path routes through existing controller/app methods; the
/// testkit snapshot mapping strips it onto the wire `CommandResultSnapshot`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandResultRow {
    pub key: String,
    pub group: String,
    pub title: String,
    pub subtitle: Option<String>,
    /// The row's destination URL for the palette's mono column.
    pub url: Option<String>,
    /// Optional flat collection label of a Saved Page result (B2 packet
    /// `saved-pages-actions-command`; DESIGN.md §11.16 `type:meta` label).
    pub collection: Option<String>,
    pub destructive: bool,
    pub activation: Activation,
}

/// The §11.5 group name on the wire (`snake_case`, stable for E2E asserts).
pub fn group_name(group: ResultGroup) -> &'static str {
    match group {
        ResultGroup::ExactTab => "exact_tab",
        ResultGroup::CurrentPinned => "current_pinned",
        ResultGroup::CurrentTab => "current_tab",
        ResultGroup::OtherSpace => "other_space",
        ResultGroup::Space => "space",
        ResultGroup::SavedPage => "saved_page",
        ResultGroup::History => "history",
        ResultGroup::Action => "action",
        ResultGroup::Fallback => "fallback",
    }
}

/// The registry action id of "Close Tab" — the only destructive command in
/// the v1 catalog (typed/destructive tint only, no confirmation).
pub const DESTRUCTIVE_ACTION_CLOSE_TAB: u64 = 2;

/// Map one portable ranked result onto the app row. Destructive actions
/// (the settled §11.5 destructive set) carry the destructive flag that
/// drives the error/destructive tint; no confirmation dialog is invented.
pub fn row_from_result(result: CommandResult) -> CommandResultRow {
    let destructive = matches!(
        &result.activation,
        Activation::RunAction { action }
            if *action == bbr_command::id::ActionId(DESTRUCTIVE_ACTION_CLOSE_TAB)
    );
    CommandResultRow {
        key: result.key,
        group: group_name(result.group).to_string(),
        title: result.title,
        subtitle: result.subtitle,
        url: result.url,
        collection: result.collection,
        destructive,
        activation: result.activation,
    }
}
