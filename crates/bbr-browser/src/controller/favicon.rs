//! Favicon store read accessors on the [`super::BrowserController`] (task
//! packet `real-favicon-pipeline.md`). The controller owns the one bounded
//! per-profile, host-keyed favicon record; every presentation — tab rows,
//! Favorites tiles, folder items — derives from this same store through
//! these read-only accessors. Split from `controller.rs` so the gateway
//! module stays under the ~500-line budget.

use bbr_web::ProfileId;

use super::BrowserController;

impl BrowserController {
    /// The resolved favicon image bytes for `(profile, host)` from the
    /// bounded per-profile store, if cached (task packet
    /// `real-favicon-pipeline.md`). The presentation layer decodes these
    /// once per record; a repeated host renders instantly and never
    /// refetches per tab.
    pub fn favicon_bytes(&self, profile: ProfileId, host: &str) -> Option<&[u8]> {
        self.favicons.image_for(profile, host)
    }

    /// The store's monotonic change counter (the presentation layer
    /// compares it to refresh its decoded mirror).
    pub fn favicon_version(&self) -> u64 {
        self.favicons.version()
    }

    /// The number of favicon records cached for `profile` (bounded-store
    /// evidence for contract tests).
    pub fn favicon_entry_count(&self, profile: ProfileId) -> usize {
        self.favicons.entry_count(profile)
    }

    /// All cached records of `profile` in deterministic insertion order
    /// (bounded-store evidence).
    pub fn favicon_entries(&self, profile: ProfileId) -> Vec<(String, Vec<u8>)> {
        self.favicons.entries_for(profile)
    }

    /// Every cached favicon record across all profiles, as
    /// `(profile, host, bytes)` (the presentation layer's decoded-image
    /// mirror source — task packet `real-favicon-pipeline.md`). The mirror
    /// rebuilds when [`Self::favicon_version`] changes, so every
    /// presentation (tab rows, Favorites tiles, folder items) decodes each
    /// acquired record exactly once.
    pub fn favicon_entries_all(&self) -> Vec<(ProfileId, String, Vec<u8>)> {
        self.favicons.entries_all()
    }

    /// The profiles holding favicon records, in id order.
    pub fn favicon_profiles(&self) -> Vec<ProfileId> {
        self.favicons.profiles()
    }

    /// Seed the favicon records the app kept on disk for `profile` (task
    /// packet `favicon-disk-cache`). Records of a profile that no longer
    /// exists are ignored, so a stale cache never resurrects one.
    pub fn restore_favicons(&mut self, profile: ProfileId, entries: Vec<(String, Vec<u8>)>) {
        if self.browser.profile(profile).is_some() {
            self.favicons.restore(profile, entries);
        }
    }

    /// Drop every favicon record of `profile` (Clear Browsing Data's
    /// cached images and files; a deleted profile drops its own).
    pub fn forget_favicons(&mut self, profile: ProfileId) {
        self.favicons.forget_profile(profile);
    }
}
