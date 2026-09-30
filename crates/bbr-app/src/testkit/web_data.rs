//! Reality gaps (item F): the `ListWebDataStores` /
//! `RemoveTestWebDataStores` seam. WebKit answers asynchronously on the main
//! thread; the answer lands in a thread-local the snapshot reads, so the
//! E2E polls for it under a deadline.

use std::cell::RefCell;

use bbr_testkit::protocol::{Request, Response, WebDataStoresSnapshot};

thread_local! {
    static STORES: RefCell<WebDataStoresSnapshot> = RefCell::new(WebDataStoresSnapshot::default());
}

/// The latest listing and removal, for the snapshot.
pub(crate) fn current() -> WebDataStoresSnapshot {
    STORES.with(|stores| stores.borrow().clone())
}

/// Handle the two requests; `None` for every other request.
pub(crate) fn dispatch(req: &Request) -> Option<Response> {
    match req {
        Request::ListWebDataStores => {
            STORES.with(|stores| stores.borrow_mut().identifiers = None);
            #[cfg(target_os = "macos")]
            bbr_web_macos::test_stores::fetch_identifiers(|identifiers| {
                STORES.with(|stores| stores.borrow_mut().identifiers = Some(identifiers));
            });
            Some(Response::Ok)
        }
        Request::RemoveTestWebDataStores => {
            STORES.with(|stores| {
                let mut stores = stores.borrow_mut();
                stores.removed = None;
                stores.removal_error = None;
            });
            #[cfg(target_os = "macos")]
            bbr_web_macos::test_stores::remove_test_stores(|result| {
                STORES.with(|stores| {
                    let mut stores = stores.borrow_mut();
                    match result {
                        Ok(count) => stores.removed = Some(count),
                        Err(error) => stores.removal_error = Some(error),
                    }
                });
            });
            Some(Response::Ok)
        }
        _ => None,
    }
}
