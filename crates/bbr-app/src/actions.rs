//! Keyboard actions for the browser chrome.
//!
//! These are GPUI [`Action`]s bound to keystrokes via `cx.bind_keys`. Because
//! the macOS platform routes `performKeyEquivalent:` (Cmd-…) through GPUI's
//! dispatch regardless of which view is the AppKit first responder, these
//! shortcuts fire even while the WKWebView has keyboard focus — the
//! browser-standard behaviour.

gpui::actions!(
    bbr,
    [
        /// `Cmd-L`: focus the command bar (location mode).
        FocusCommandBar,
        /// `Cmd-T`: open a new controller-backed tab.
        NewTab,
        /// `Cmd-Shift-N`: stage a new space name in the active profile.
        NewSpace,
        /// `Cmd-W`: close the active tab.
        CloseTab,
        /// `Cmd-Shift-T`: reopen the most recently closed tab.
        ReopenTab,
        /// `Cmd-B`: toggle the sidebar.
        ToggleSidebar,
        /// `Cmd-[`: go back.
        Back,
        /// `Cmd-]`: go forward.
        Forward,
        /// `Cmd-R`: reload.
        Reload,
        /// `Escape`: return focus to the web view.
        FocusWeb,
    ]
);
