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
        /// `Cmd-L`: open the field-anchored command overlay (location mode).
        OpenOverlay,
        /// `Cmd-T`: open the centered command palette in New Tab staging mode.
        OpenPalette,
        /// `Cmd-Shift-N`: stage a new space name in the active profile.
        NewSpace,
        /// `Cmd-Shift-P`: pin or unpin the active tab (region toggle).
        PinTab,
        /// `Cmd-Shift-↑`: move the active tab one step up within its region.
        ReorderTabUp,
        /// `Cmd-Shift-↓`: move the active tab one step down within its region.
        ReorderTabDown,
        /// `Cmd-Shift-M`: stage a destination space for the active tab;
        /// Enter commits the move, Escape cancels it.
        MoveTab,
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
        /// `Cmd-F`: open the find-in-page presentation over the active web
        /// view (task packet `find-in-page-portal-bar`).
        FindInPage,
        /// `Cmd-Plus` / `Cmd-=`: zoom the active view's site one contract
        /// step up (queue item 11, part A; task packet
        /// `site-settings-zoom-contract`).
        ZoomIn,
        /// `Cmd-Minus`: zoom the active view's site one contract step down.
        ZoomOut,
        /// `Cmd-0`: reset the active view's site to the default zoom level.
        ZoomReset,
    ]
);
