//! U2 sidebar contract budgets (DESIGN.md §11.4/§11.10/§11.11): the plus
//! menu open/close, the favorites tile add/remove, and the folder
//! disclosure each commit the changed rail on the next frame — the warm
//! next-frame presentation class, same as sidebar toggle and tab selection
//! (the packet: "warm 50/100 ms budget class"). Split from `super` so the
//! baseline module stays under the ~500-line budget.

use super::Budget;

/// The warm contract budgets of the three U2 sidebar scenarios.
pub(super) fn contract_budget(kind: &str) -> Option<Budget> {
    let b = match kind {
        "sidebar_plus_menu" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        "sidebar_favorites" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        "sidebar_folder_toggle" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        _ => return None,
    };
    Some(b)
}

/// The seeded baseline kinds of the three U2 sidebar scenarios.
pub(super) fn seed_kinds() -> [&'static str; 3] {
    [
        "sidebar_plus_menu",
        "sidebar_favorites",
        "sidebar_folder_toggle",
    ]
}
