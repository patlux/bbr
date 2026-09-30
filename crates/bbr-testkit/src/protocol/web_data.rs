//! Website-data stores (reality-gaps packet, item F): what WebKit keeps for
//! the app, read through `fetchAllDataStoreIdentifiers:`, and the removal
//! of every test run's stores.

use serde::{Deserialize, Serialize};

/// The latest store listing and test-store removal.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebDataStoresSnapshot {
    /// Every identifier-backed store of the app (lowercase UUIDs) as of the
    /// last `ListWebDataStores`; `None` while that listing runs.
    pub identifiers: Option<Vec<String>>,
    /// How many test-run stores the last `RemoveTestWebDataStores` removed;
    /// `None` while it runs or after it failed.
    pub removed: Option<usize>,
    /// The first error of the last removal.
    pub removal_error: Option<String>,
}
