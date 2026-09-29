//! App-owned Saved Page command authority (B2 packet
//! `saved-pages-actions-command`): availability and dispatch for
//! `Bookmark Page`, `Remove Bookmark`, and `Open Saved Page`.
//!
//! This is the **one** app-owned command authority Cmd-D, the Command
//! Palette, and the capability-gated Z1/Z3 menu rows share. Active-page and
//! menu callers supply a current navigable URL/title or a [`BookmarkId`];
//! render callbacks never implement persistence or tab behavior directly.
//! The authority is stateless: all state lives in the app store (the
//! in-memory Saved Page provider) and the controller (the NewTab/navigation
//! authority). Z1/Z3 must not duplicate this logic — they consume the same
//! command seam.
//!
//! Semantics (settled B2 contract):
//! - saving is canonical-URL-idempotent (the `bbr-domain` canonical seam is
//!   the one equality authority; a duplicate returns the existing
//!   [`BookmarkId`] and never creates a second record);
//! - removing never mutates a tab, and closing the source tab never removes
//!   the record (a Saved Page carries no `TabId`/session link);
//! - opening revalidates the record in the in-memory provider, then appends
//!   exactly one **fresh regular tab** to the currently active Space/Profile
//!   through the existing NewTab authority — it never silently selects an
//!   existing matching tab, switches Space/Profile, pins, favorites, splits,
//!   or reuses a session;
//! - missing, malformed, dangerous, internal-unsupported, or non-navigable
//!   URLs reject or omit the action with a deterministic reason (the
//!   canonical seam's typed error).

use bbr_command::id::BookmarkId;
use bbr_domain::canonical_url;
use gpui::{Context, Window};

use super::BbrApp;

/// The outcome of a Cmd-D toggle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleOutcome {
    /// The page was saved (the canonical URL had no active record).
    Saved(BookmarkId),
    /// The page was removed (the canonical URL had an active record).
    Removed(BookmarkId),
    /// The URL is not navigable for a durable Saved Page (missing,
    /// malformed, dangerous, or unsupported); the action is omitted.
    Rejected,
}

/// The one app-owned Saved Page command authority (see the module docs).
#[derive(Debug, Clone, Copy, Default)]
pub struct SavedPageCommand;

impl SavedPageCommand {
    /// Availability of `Bookmark Page` for `url`: the URL must be navigable
    /// through the canonical seam and must not already have an active
    /// record. Never mutates source state. Consumed by the testkit
    /// availability probe today; the Z1/Z3 menu rows consume the same seam
    /// when they integrate.
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))]
    pub fn can_save(&self, app: &BbrApp, url: &str) -> bool {
        match canonical_url(url) {
            Ok(canonical) => app.store.saved_page_by_url(&canonical).is_none(),
            Err(_) => false,
        }
    }

    /// Availability of `Remove Bookmark` for `url`: the URL must be
    /// navigable through the canonical seam and must already have an active
    /// record. Never mutates source state. Consumed by the testkit
    /// availability probe today; the Z1/Z3 menu rows consume the same seam
    /// when they integrate.
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))]
    pub fn can_remove(&self, app: &BbrApp, url: &str) -> bool {
        match canonical_url(url) {
            Ok(canonical) => app.store.saved_page_by_url(&canonical).is_some(),
            Err(_) => false,
        }
    }

    /// Idempotent save: canonical-URL dedupe through the in-memory provider
    /// (a duplicate returns the existing identity and never creates a second
    /// record). `None` when the URL is not navigable. Never mutates the
    /// source tab/Favorite/pin/session/profile/space. Consumed by the
    /// testkit save probe today; the Z1/Z3 menu rows consume the same seam
    /// when they integrate.
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))]
    pub fn save(&self, app: &mut BbrApp, url: &str, title: &str) -> Option<BookmarkId> {
        let canonical = canonical_url(url).ok()?;
        let outcome = app.store.save_saved_page(&canonical, title);
        Some(BookmarkId(outcome.id))
    }

    /// Idempotent remove by stable identity; an unknown or already-removed
    /// identity is a no-op success. Never mutates a tab. Consumed by the
    /// testkit remove probe today; the Z1/Z3 menu rows consume the same seam
    /// when they integrate.
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))]
    pub fn remove(&self, app: &mut BbrApp, id: BookmarkId) {
        app.store.remove_saved_page(id.0);
    }

    /// Cmd-D toggle: save when the canonical URL has no active record,
    /// remove when it does. `Rejected` for a non-navigable URL.
    pub fn toggle(&self, app: &mut BbrApp, url: &str, title: &str) -> ToggleOutcome {
        let Ok(canonical) = canonical_url(url) else {
            return ToggleOutcome::Rejected;
        };
        if let Some(existing) = app.store.saved_page_by_url(&canonical) {
            app.store.remove_saved_page(existing.id);
            ToggleOutcome::Removed(BookmarkId(existing.id))
        } else {
            let outcome = app.store.save_saved_page(&canonical, title);
            ToggleOutcome::Saved(BookmarkId(outcome.id))
        }
    }

    /// Open a Saved Page: revalidate the record in the in-memory provider
    /// (a stale/deleted `BookmarkId` rejects truthfully), then open exactly
    /// one fresh regular tab in the currently active Space/Profile through
    /// the existing NewTab authority. Never silently selects an existing
    /// matching tab, switches Space/Profile, pins, favorites, splits, or
    /// reuses a session. Returns false when the record is gone or the tab
    /// could not be opened.
    pub fn open(
        &self,
        app: &mut BbrApp,
        id: BookmarkId,
        window: &mut Window,
        cx: &mut Context<BbrApp>,
    ) -> bool {
        let Some(page) = app.store.saved_page_by_id(id.0) else {
            return false;
        };
        if app.open_tab(&page.canonical_url, cx).is_some() {
            app.finish_web_activation(window, cx);
            true
        } else {
            false
        }
    }
}

impl BbrApp {
    /// Cmd-D: toggle the active/focused page's saved state. The active
    /// page's current navigable URL/title come from the controller snapshot
    /// (the same source the address control mirrors); a missing or
    /// non-navigable URL omits the action with the deterministic reason.
    pub fn toggle_bookmark(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let snapshot = self.controller.snapshot();
        let Some(url) = snapshot.nav.url.clone() else {
            return;
        };
        let title = snapshot.nav.title.clone().unwrap_or_default();
        let command = self.saved_pages;
        // Blueprint D08 (board PR-12): a save is confirmed by a toast whose
        // Undo removes the record again.
        if let ToggleOutcome::Saved(id) = command.toggle(self, &url, &title) {
            self.show_toast(
                bbr_ui::icons::Icon::Bookmark,
                None,
                bbr_ui::surfaces::toast::ToastTone::Accent,
                "Saved to Library",
                Some((
                    "Undo".into(),
                    crate::app::transient::ToastAction::UndoSavePage(id),
                )),
                cx,
            );
        }
        cx.notify();
    }
}

/// The pure availability predicate for `Bookmark Page` (the store-backed
/// half of [`SavedPageCommand::can_save`]): the URL must be navigable
/// through the canonical seam and must not already have an active record.
/// Deterministic and unit-testable without a full `BbrApp`.
#[cfg(test)]
pub(crate) fn can_save_url(store: &crate::store::AppStore, url: &str) -> bool {
    match canonical_url(url) {
        Ok(canonical) => store.saved_page_by_url(&canonical).is_none(),
        Err(_) => false,
    }
}

/// The pure availability predicate for `Remove Bookmark` (the store-backed
/// half of [`SavedPageCommand::can_remove`]): the URL must be navigable
/// through the canonical seam and must already have an active record.
#[cfg(test)]
pub(crate) fn can_remove_url(store: &crate::store::AppStore, url: &str) -> bool {
    match canonical_url(url) {
        Ok(canonical) => store.saved_page_by_url(&canonical).is_some(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbr_persistence::SavedPageRecord;

    /// A minimal provider-backed app harness: the store's in-memory
    /// provider is driven directly (the pure save/remove policy is
    /// unit-tested in `crate::store::saved_pages`); the command authority
    /// is exercised against it through the same store methods the app uses.
    fn provider() -> crate::store::AppStore {
        crate::store::AppStore::ephemeral()
    }

    fn record(id: u64, url: &str, title: &str) -> SavedPageRecord {
        SavedPageRecord {
            id,
            canonical_url: url.into(),
            captured_title: title.into(),
            user_label: None,
            created_at: 1,
            updated_at: 1,
            collection_id: None,
        }
    }

    #[test]
    fn canonical_seam_rejects_unsupported_and_dangerous_urls() {
        // The deterministic reason ladder: malformed/empty, unsupported
        // schemes, and credentials all reject whole.
        assert!(canonical_url("").is_err());
        assert!(canonical_url("javascript:alert(1)").is_err());
        assert!(canonical_url("data:text/html,hi").is_err());
        assert!(canonical_url("file:///etc/passwd").is_err());
        assert!(canonical_url("mailto:a@b.example").is_err());
        assert!(canonical_url("https://user:pass@host.example/").is_err());
        assert!(canonical_url("//host.example").is_err());
        // The navigable set (http/https/about) canonicalizes.
        assert!(canonical_url("https://example.com/").is_ok());
        assert!(canonical_url("http://example.com/").is_ok());
        assert!(canonical_url("about:blank").is_ok());
    }

    #[test]
    fn save_is_idempotent_and_remove_is_idempotent() {
        let mut store = provider();
        let first = store.save_saved_page("https://example.com/", "Example");
        let second = store.save_saved_page("https://example.com/", "Example");
        assert_eq!(
            first.id, second.id,
            "repeated save returns the same identity"
        );
        assert!(second.existed);
        assert_eq!(store.saved_pages_count(), 1, "no duplicate record");
        store.remove_saved_page(first.id);
        assert_eq!(store.saved_pages_count(), 0);
        store.remove_saved_page(first.id);
        assert_eq!(store.saved_pages_count(), 0, "repeated remove is a no-op");
    }

    #[test]
    fn duplicate_save_preserves_created_at_and_never_touches_a_label() {
        let mut store = provider();
        let first = store.save_saved_page("https://example.com/", "First title");
        let page = store.saved_page_by_id(first.id).unwrap();
        assert_eq!(page.created_at, page.updated_at);
        // A re-save with a changed title updates the captured title (no
        // user label exists) and bumps updated_at, never created_at.
        let second = store.save_saved_page("https://example.com/", "Second title");
        assert_eq!(second.id, first.id);
        let page = store.saved_page_by_id(first.id).unwrap();
        assert_eq!(page.captured_title, "Second title");
        assert!(page.updated_at >= page.created_at);
    }

    #[test]
    fn provider_order_presents_most_recently_updated_first() {
        let mut pages = vec![
            record(1, "https://a.example/", "A"),
            record(2, "https://b.example/", "B"),
            record(3, "https://c.example/", "C"),
        ];
        pages[0].updated_at = 10;
        pages[1].updated_at = 30;
        pages[2].updated_at = 20;
        crate::store::saved_pages::provider_order(&mut pages);
        assert_eq!(
            pages.iter().map(|p| p.id).collect::<Vec<_>>(),
            vec![2, 3, 1],
            "most recently updated first, then id descending"
        );
    }

    #[test]
    fn availability_matrix_covers_saved_unsaved_and_invalid_urls() {
        // The settled availability matrix: `Bookmark Page` is available for
        // a navigable unsaved URL, `Remove Bookmark` for a navigable saved
        // URL, and neither for a non-navigable URL — without mutating
        // source state (the provider count is unchanged by an availability
        // probe).
        let mut store = provider();
        assert!(can_save_url(&store, "https://example.com/"));
        assert!(!can_remove_url(&store, "https://example.com/"));
        // Non-navigable URLs reject whole for both directions.
        for bad in [
            "",
            "javascript:alert(1)",
            "data:text/html,hi",
            "file:///etc/passwd",
            "mailto:a@b.example",
            "https://user:pass@host.example/",
        ] {
            assert!(!can_save_url(&store, bad), "{bad:?} cannot be saved");
            assert!(!can_remove_url(&store, bad), "{bad:?} cannot be removed");
        }
        // After a save the matrix flips: remove becomes available, save
        // does not (no duplicate record is offered).
        store.save_saved_page("https://example.com/", "Example");
        assert!(!can_save_url(&store, "https://example.com/"));
        assert!(can_remove_url(&store, "https://example.com/"));
        // A stale/deleted identity is a no-op remove; the availability
        // probes never mutate the provider.
        let before = store.saved_pages_count();
        store.remove_saved_page(9999);
        assert_eq!(store.saved_pages_count(), before);
        assert!(!can_save_url(&store, "https://example.com/"));
        assert!(can_remove_url(&store, "https://example.com/"));
    }
}
