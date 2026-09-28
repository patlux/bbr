//! Blueprint D14: the session history behind the Back/Forward history
//! menu, read from WebKit's own `WKBackForwardList` (never a BBR-side
//! mirror that could drift), and the one-step jump to an entry.

use bbr_web::{HistoryEntry, SessionHistory};
use objc2_web_kit::{WKBackForwardListItem, WKWebView};

/// The live view's back and forward lists, nearest entry first on both
/// sides (WebKit orders `backList` oldest first, so it is reversed).
pub(crate) fn read(web_view: &WKWebView) -> SessionHistory {
    // SAFETY: main-thread WebKit reads on a live view; the lists are
    // snapshots owned by the returned arrays.
    unsafe {
        let list = web_view.backForwardList();
        let back = list.backList();
        let forward = list.forwardList();
        let mut history = SessionHistory {
            back: (0..back.count())
                .map(|index| entry(&back.objectAtIndex(index)))
                .collect(),
            forward: (0..forward.count())
                .map(|index| entry(&forward.objectAtIndex(index)))
                .collect(),
        };
        history.back.reverse();
        history
    }
}

/// Jump `offset` entries (negative = back). An offset outside the list is
/// a no-op: WebKit returns no item for it.
pub(crate) fn go(web_view: &WKWebView, offset: i32) {
    // SAFETY: main-thread WebKit navigation on a live view with an item
    // taken from that view's own list.
    unsafe {
        let list = web_view.backForwardList();
        if let Some(item) = list.itemAtIndex(offset as isize) {
            let _ = web_view.goToBackForwardListItem(&item);
        }
    }
}

fn entry(item: &WKBackForwardListItem) -> HistoryEntry {
    // SAFETY: plain property reads on a list item.
    unsafe {
        HistoryEntry {
            url: item
                .URL()
                .absoluteString()
                .map(|url| url.to_string())
                .unwrap_or_default(),
            title: item
                .title()
                .map(|title| title.to_string())
                .unwrap_or_default(),
        }
    }
}
