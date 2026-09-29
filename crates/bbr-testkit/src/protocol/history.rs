//! Blueprint D10: History and Clear Browsing Data on the wire.

use serde::{Deserialize, Serialize};

/// The open History page as drawn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistorySnapshot {
    /// `all` or `closed`.
    pub view: String,
    /// `today`, `week` or `all`.
    pub range: String,
    pub search: String,
    pub closed_count: usize,
    pub groups: Vec<HistoryGroupSnapshot>,
    /// The keyboard row's id.
    pub selected: Option<u64>,
    /// The empty-state line, when nothing is listed.
    pub empty: Option<String>,
    /// The page holds the keyboard.
    pub focused: bool,
    pub clear: Option<ClearDataSnapshot>,
}

/// One day group (or "Recently closed").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryGroupSnapshot {
    pub title: String,
    pub rows: Vec<HistoryRowSnapshot>,
}

/// One row: the visit id (the tab id in Recently closed).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryRowSnapshot {
    pub id: u64,
    pub time: Option<String>,
    pub title: String,
    pub url: String,
}

/// The Clear Browsing Data sheet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClearDataSnapshot {
    pub profile: String,
    pub range: String,
    pub range_open: bool,
    pub options: Vec<ClearOptionSnapshot>,
    pub confirm_enabled: bool,
}

/// One option row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClearOptionSnapshot {
    pub label: String,
    pub checked: bool,
    pub enabled: bool,
}

/// One History interaction through the method the page's controls call
/// (keys go through `DispatchKeystroke`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistoryInput {
    /// 0 All history, 1 Recently closed.
    View {
        view: usize,
    },
    /// 0 Today, 1 Week, 2 All.
    Range {
        range: usize,
    },
    Open {
        id: u64,
    },
    Remove {
        id: u64,
    },
    ClearOpen,
    ClearRangeMenu,
    ClearRange {
        index: usize,
    },
    ClearToggle {
        index: usize,
    },
    ClearCancel,
    ClearConfirm,
}
