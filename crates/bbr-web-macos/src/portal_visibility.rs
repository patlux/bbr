//! Parent-owned Portal visibility, evaluated on the deferred AppKit turn.
//! The retained intent is shared with queued work: dismissal/drop supersedes
//! a previously queued show without retaining a raw pointer to NativePortal.

use objc2::MainThreadMarker;
use objc2_app_kit::{NSApplication, NSWindow, NSWindowOrderingMode};
use std::cell::Cell;

#[derive(Default)]
pub(crate) struct VisibilityIntent {
    pub requested: Cell<bool>,
    pub pending: Cell<bool>,
}

#[derive(Clone, Copy)]
struct Visibility {
    requested: bool,
    attached: bool,
    parent_visible: bool,
    parent_key: bool,
    parent_minimized: bool,
    parent_on_space: bool,
    app_active: bool,
    app_hidden: bool,
}

impl Visibility {
    fn should_show(self) -> bool {
        self.requested
            && self.attached
            && self.parent_visible
            && self.parent_key
            && !self.parent_minimized
            && self.parent_on_space
            && self.app_active
            && !self.app_hidden
    }
}

pub(crate) fn apply(
    mtm: MainThreadMarker,
    portal: &NSWindow,
    parent: &NSWindow,
    intent: &VisibilityIntent,
) {
    intent.pending.set(false);
    let app = NSApplication::sharedApplication(mtm);
    let facts = Visibility {
        requested: intent.requested.get(),
        // AppKit drops the native child relation on orderOut. The retained
        // owner survives hiding; refuse attachment to any different parent.
        attached: portal.parentWindow().is_none_or(|owner| &*owner == parent),
        parent_visible: parent.isVisible(),
        parent_key: parent.isKeyWindow(),
        parent_minimized: parent.isMiniaturized(),
        parent_on_space: parent.isOnActiveSpace(),
        app_active: app.isActive(),
        app_hidden: app.isHidden(),
    };
    if facts.should_show() {
        if portal.level() != parent.level() {
            portal.setLevel(parent.level());
        }
        if portal.parentWindow().is_none() {
            // SAFETY: both windows are retained by the deferred work, on
            // the main thread, and the child has no other parent.
            unsafe { parent.addChildWindow_ordered(portal, NSWindowOrderingMode::Above) };
        }
        if !portal.isVisible() {
            // Never orderFront: re-show only relative to this parent, without
            // lifting a background browser above another app/window.
            portal.orderWindow_relativeTo(NSWindowOrderingMode::Above, parent.windowNumber());
        }
    } else if portal.isVisible() {
        portal.orderOut(None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn foreground() -> Visibility {
        Visibility {
            requested: true,
            attached: true,
            parent_visible: true,
            parent_key: true,
            parent_minimized: false,
            parent_on_space: true,
            app_active: true,
            app_hidden: false,
        }
    }

    #[test]
    fn every_parent_lifecycle_precondition_is_required() {
        let ready = foreground();
        assert!(ready.should_show());
        for blocked in [
            Visibility {
                requested: false,
                ..ready
            },
            Visibility {
                attached: false,
                ..ready
            },
            Visibility {
                parent_visible: false,
                ..ready
            },
            Visibility {
                parent_key: false,
                ..ready
            },
            Visibility {
                parent_minimized: true,
                ..ready
            },
            Visibility {
                parent_on_space: false,
                ..ready
            },
            Visibility {
                app_active: false,
                ..ready
            },
            Visibility {
                app_hidden: true,
                ..ready
            },
        ] {
            assert!(!blocked.should_show());
        }
    }

    #[test]
    fn deferred_work_reads_latest_intent_not_a_captured_show() {
        let intent = std::rc::Rc::new(VisibilityIntent::default());
        intent.requested.set(true);
        let queued = intent.clone();
        intent.requested.set(false);
        assert!(!Visibility {
            requested: queued.requested.get(),
            ..foreground()
        }
        .should_show());
    }
}
