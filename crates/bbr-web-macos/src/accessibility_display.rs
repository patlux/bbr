//! macOS accessibility display options (task packet `v2-system-prefs`,
//! DESIGN.md §3.4/§6/§18.5).
//!
//! The shell follows two system settings live: **Increase Contrast**
//! (selects the §3.4 increased-contrast token set) and **Reduce Motion**
//! (selects the §6 reduced motion variants). Both are read through the
//! public `NSWorkspace` accessibility-display API, and changes arrive
//! through the public `NSWorkspaceAccessibilityDisplayOptionsDidChange`
//! notification on the workspace notification center — no polling, so an
//! idle shell never wakes for them.
//!
//! The adapter only reports plain values and a change signal; the theme
//! decision stays in the app's appearance authority.

use objc2::rc::Retained;
use objc2::{define_class, msg_send, sel, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSWorkspace, NSWorkspaceAccessibilityDisplayOptionsDidChangeNotification};
use objc2_foundation::{NSNotification, NSNotificationCenter, NSObject};

/// The two accessibility display options the shell honours.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AccessibilityDisplayOptions {
    /// System Settings → Accessibility → Display → Increase contrast.
    pub increase_contrast: bool,
    /// System Settings → Accessibility → Display → Reduce motion.
    pub reduce_motion: bool,
}

/// Read the current system values.
pub fn current_options() -> AccessibilityDisplayOptions {
    let workspace = NSWorkspace::sharedWorkspace();
    AccessibilityDisplayOptions {
        increase_contrast: workspace.accessibilityDisplayShouldIncreaseContrast(),
        reduce_motion: workspace.accessibilityDisplayShouldReduceMotion(),
    }
}

/// Ivars of the notification receiver: the app's change callback.
struct ObserverIvars {
    on_change: Box<dyn Fn()>,
}

define_class!(
    /// Receives `NSWorkspaceAccessibilityDisplayOptionsDidChangeNotification`
    /// on the main thread and forwards it to the app's callback.
    #[unsafe(super = NSObject)]
    #[ivars = ObserverIvars]
    #[thread_kind = MainThreadOnly]
    #[name = "BBRAccessibilityDisplayObserver"]
    struct DisplayOptionsReceiver;

    impl DisplayOptionsReceiver {
        #[unsafe(method(displayOptionsDidChange:))]
        fn display_options_did_change(&self, _notification: Option<&NSNotification>) {
            (self.ivars().on_change)();
        }
    }
);

impl DisplayOptionsReceiver {
    fn new(mtm: MainThreadMarker, on_change: Box<dyn Fn()>) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(ObserverIvars { on_change });
        // SAFETY: `init` on the main thread (the ChromeResponder pattern).
        unsafe { msg_send![super(this), init] }
    }
}

/// A live registration on the workspace notification center. Dropping it
/// removes the observer, so the callback can never fire after its owner is
/// gone.
pub struct AccessibilityDisplayObserver {
    receiver: Retained<DisplayOptionsReceiver>,
    center: Retained<NSNotificationCenter>,
}

impl AccessibilityDisplayObserver {
    /// Register `on_change` for every accessibility display option change.
    /// The callback runs on the main thread; it should re-read
    /// [`current_options`] rather than infer which option changed.
    pub fn install(mtm: MainThreadMarker, on_change: Box<dyn Fn()>) -> Self {
        let receiver = DisplayOptionsReceiver::new(mtm, on_change);
        let center = NSWorkspace::sharedWorkspace().notificationCenter();
        // SAFETY: main thread; `receiver` is retained by the returned value
        // and removed in `Drop` before release; the workspace center posts
        // this notification on the main thread; reading the extern
        // notification-name static is the only other unsafe part.
        unsafe {
            center.addObserver_selector_name_object(
                &receiver,
                sel!(displayOptionsDidChange:),
                Some(NSWorkspaceAccessibilityDisplayOptionsDidChangeNotification),
                None,
            );
        }
        Self { receiver, center }
    }
}

impl Drop for AccessibilityDisplayObserver {
    fn drop(&mut self) {
        // SAFETY: the receiver was registered with this center and is still
        // alive (we hold it).
        unsafe {
            self.center.removeObserver(&self.receiver);
        }
    }
}
